use crate::block_parser::ParsedTransaction;
use crate::cache::SimpleCache;
use crate::config::Protocol;
use crate::error::{NozyError, NozyResult};
use crate::hd_wallet::HDWallet;
use crate::key_management::{zeroize_bytes, SecureSeed};
use crate::note_index::NoteIndex;
use crate::orchard_tree_codec::orchard_commitment_tree_from_final_state;
use crate::orchard_tree_codec::OrchardCommitmentTree;
use crate::orchard_witness::{merkle_hash_from_cmx_bytes, OrchardWitnessTracker};
use crate::scan_log;
use crate::scan_verbose;
use crate::zebra_integration::ZebraClient;
use futures::future::join_all;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use orchard::{
    keys::{FullViewingKey, IncomingViewingKey, SpendingKey},
    note::{Note, Nullifier},
    Address as OrchardAddress,
};
use zip32::AccountId;

use zcash_note_encryption::try_compact_note_decryption;

#[derive(Debug, Clone)]
pub struct OrchardNote {
    pub note: Note,
    pub value: u64,
    pub address: OrchardAddress,
    pub nullifier: Nullifier,
    pub block_height: u32,
    pub txid: String,
    pub spent: bool,
    pub memo: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct SpendableNote {
    pub orchard_note: OrchardNote,
    pub spending_key: SpendingKey,
    pub derivation_path: String,
    pub orchard_incremental_witness_hex: Option<String>,
    pub orchard_witness_tip_height: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableOrchardNote {
    pub note_bytes: Vec<u8>,
    pub value: u64,
    pub address_bytes: Vec<u8>,
    pub nullifier_bytes: Vec<u8>,
    pub block_height: u32,
    pub txid: String,
    pub spent: bool,
    pub memo: Vec<u8>,
    /// Hex-encoded `zcash_primitives::merkle_tree::write_incremental_witness` blob (Orchard).
    #[serde(default)]
    pub orchard_incremental_witness_hex: Option<String>,
    /// Block height through which the witness was advanced (matches chain order used during scan).
    #[serde(default)]
    pub orchard_witness_tip_height: Option<u32>,
    /// Note rho (needed to derive the canonical Orchard nullifier; not present in compact actions).
    #[serde(default)]
    pub rho_bytes: Option<Vec<u8>>,
    /// Note rseed (needed to derive the canonical Orchard nullifier).
    #[serde(default)]
    pub rseed_bytes: Option<Vec<u8>>,
}

impl SerializableOrchardNote {
    /// Reconstruct the Orchard note when rho/rseed were persisted at discovery time.
    pub fn to_orchard_note(&self) -> Option<Note> {
        use orchard::note::{RandomSeed, Rho};
        let rho_bytes: [u8; 32] = self.rho_bytes.as_ref()?.as_slice().try_into().ok()?;
        let rho = Rho::from_bytes(&rho_bytes).into_option()?;
        let rseed_bytes: [u8; 32] = self.rseed_bytes.as_ref()?.as_slice().try_into().ok()?;
        let rseed = RandomSeed::from_bytes(rseed_bytes, &rho).into_option()?;
        let addr_bytes: [u8; 43] = self.address_bytes.as_slice().try_into().ok()?;
        let recipient = OrchardAddress::from_raw_address_bytes(&addr_bytes).into_option()?;
        let value = orchard::value::NoteValue::from_raw(self.value);
        Note::from_parts(recipient, value, rho, rseed).into_option()
    }

    /// Canonical nullifier for this note (matches spend builder / on-chain spends).
    pub fn canonical_nullifier_bytes(&self, fvk: &FullViewingKey) -> Option<[u8; 32]> {
        let note = self.to_orchard_note()?;
        Some(note.nullifier(fvk).to_bytes())
    }

    /// Reconstruct a wallet [`OrchardNote`] from persisted fields (history, display).
    pub fn to_wallet_orchard_note(&self) -> Option<OrchardNote> {
        use orchard::note::Nullifier;

        let note = self.to_orchard_note()?;
        let addr_bytes: [u8; 43] = self.address_bytes.as_slice().try_into().ok()?;
        let address = OrchardAddress::from_raw_address_bytes(&addr_bytes).into_option()?;
        let nullifier_bytes: [u8; 32] = self.nullifier_bytes.as_slice().try_into().ok()?;
        let nullifier = Nullifier::from_bytes(&nullifier_bytes).into_option()?;
        Some(OrchardNote {
            note,
            value: self.value,
            address,
            nullifier,
            block_height: self.block_height,
            txid: self.txid.clone(),
            spent: self.spent,
            memo: self.memo.clone(),
        })
    }
}

/// Build spendable notes from cached `notes.json` (no block scan).
///
/// Notes must have persisted Orchard witnesses; witness catch-up to chain tip happens at spend
/// build time via [`crate::orchard_tx::ZebraJsonRpcOrchardWitnessProvider`].
#[cfg(feature = "native")]
pub fn load_spendable_notes_from_wallet(wallet: &HDWallet) -> NozyResult<Vec<SpendableNote>> {
    use crate::key_management::{zeroize_bytes, SecureSeed};
    use zip32::AccountId;

    let cached = load_wallet_notes()?;
    let unspent: Vec<&SerializableOrchardNote> = cached
        .iter()
        .filter(|n| {
            !n.spent
                && n.orchard_incremental_witness_hex
                    .as_ref()
                    .is_some_and(|w| !w.is_empty())
        })
        .collect();
    if unspent.is_empty() {
        return Ok(Vec::new());
    }

    let account_id = AccountId::try_from(0)
        .map_err(|e| NozyError::KeyDerivation(format!("Invalid account ID: {:?}", e)))?;
    let mnemonic = wallet.get_mnemonic_object();
    let mut seed_bytes = mnemonic.to_seed("").to_vec();
    let secure_seed = SecureSeed::new(seed_bytes.clone());
    let orchard_sk = SpendingKey::from_zip32_seed(secure_seed.as_bytes(), 133, account_id)
        .map_err(|e| {
            NozyError::KeyDerivation(format!("Failed to derive Orchard spending key: {:?}", e))
        })?;
    zeroize_bytes(&mut seed_bytes);

    let mut spendable = Vec::with_capacity(unspent.len());
    for sn in unspent {
        let orchard_note = sn.to_wallet_orchard_note().ok_or_else(|| {
            NozyError::InvalidOperation(format!(
                "Cached note in tx {} is missing rho/rseed; run sync before sending",
                sn.txid
            ))
        })?;
        spendable.push(SpendableNote {
            orchard_note,
            spending_key: orchard_sk.clone(),
            derivation_path: "m/32'/133'/0'".to_string(),
            orchard_incremental_witness_hex: sn.orchard_incremental_witness_hex.clone(),
            orchard_witness_tip_height: sn.orchard_witness_tip_height,
        });
    }

    Ok(spendable)
}

impl From<&OrchardNote> for SerializableOrchardNote {
    fn from(note: &OrchardNote) -> Self {
        Self {
            note_bytes: {
                #[allow(unused_mut)]
                let mut bytes: Vec<u8> = Vec::new();
                #[allow(unused_unsafe)]
                {
                    bytes.extend_from_slice(&note.value.to_le_bytes());
                    bytes.extend_from_slice(&note.nullifier.to_bytes());
                    bytes
                }
            },
            value: note.value,
            address_bytes: note.address.to_raw_address_bytes().to_vec(),
            nullifier_bytes: note.nullifier.to_bytes().to_vec(),
            block_height: note.block_height,
            txid: note.txid.clone(),
            spent: note.spent,
            memo: note.memo.clone(),
            orchard_incremental_witness_hex: None,
            orchard_witness_tip_height: None,
            rho_bytes: Some(note.note.rho().to_bytes().to_vec()),
            rseed_bytes: Some(note.note.rseed().as_bytes().to_vec()),
        }
    }
}

/// Mark notes spent in `notes.json` immediately after broadcast.
///
/// Matches by canonical nullifier and, for legacy rows with a wrong stored nullifier,
/// by `(txid, block_height, value)` from the notes that were actually spent.
#[cfg(feature = "native")]
pub fn mark_wallet_notes_spent_from_spendables(spent: &[SpendableNote]) -> NozyResult<usize> {
    use crate::paths::get_wallet_data_dir;

    if spent.is_empty() {
        return Ok(0);
    }

    let notes_path = get_wallet_data_dir().join("notes.json");
    if !notes_path.exists() {
        return Ok(0);
    }

    let mut index = NoteIndex::load_from_file(&notes_path)?;
    let mut marked = 0usize;

    for sn in spent {
        let fvk = FullViewingKey::from(&sn.spending_key);
        let nf = sn.orchard_note.note.nullifier(&fvk).to_bytes();

        if index.mark_note_spent_on_chain(&nf, &fvk) {
            marked += 1;
            continue;
        }

        if index.mark_note_spent_by_spend_metadata(
            &sn.orchard_note.txid,
            sn.orchard_note.block_height,
            sn.orchard_note.value,
            &fvk,
            Some(sn.orchard_note.note.rho().to_bytes().as_slice()),
            Some(sn.orchard_note.note.rseed().as_bytes()),
        ) {
            marked += 1;
        }
    }

    if marked > 0 {
        index.save_to_file(&notes_path)?;
    }

    Ok(marked)
}

/// Load cached wallet notes from `notes.json`, supporting legacy array and v2 index formats.
#[cfg(feature = "native")]
pub fn load_wallet_notes() -> NozyResult<Vec<SerializableOrchardNote>> {
    use crate::note_index::NoteIndex;
    use crate::paths::get_wallet_data_dir;

    let notes_path = get_wallet_data_dir().join("notes.json");
    Ok(NoteIndex::load_from_file(&notes_path)?
        .get_all_notes()
        .to_vec())
}

/// Persist wallet notes to `notes.json` using the v2 index format (atomic write).
#[cfg(feature = "native")]
pub fn save_wallet_notes(notes: &[SerializableOrchardNote]) -> NozyResult<()> {
    use crate::note_index::NoteIndex;
    use crate::paths::get_wallet_data_dir;

    let notes_path = get_wallet_data_dir().join("notes.json");
    NoteIndex::from_notes(notes.to_vec()).save_to_file(&notes_path)
}

/// Sum unspent note values from an in-memory note list.
#[cfg(feature = "native")]
pub fn wallet_unspent_balance_zatoshis(notes: &[SerializableOrchardNote]) -> u64 {
    notes
        .iter()
        .filter(|note| !note.spent)
        .map(|note| note.value)
        .sum()
}

/// Release notes marked spent locally when a pilot tx expires unmined.
#[cfg(feature = "native")]
pub fn release_wallet_notes_by_nullifier_hex(nullifier_hexes: &[String]) -> NozyResult<usize> {
    use std::collections::HashSet;

    if nullifier_hexes.is_empty() {
        return Ok(0);
    }

    let targets: HashSet<Vec<u8>> = nullifier_hexes
        .iter()
        .filter_map(|h| hex::decode(h.trim()).ok())
        .collect();
    if targets.is_empty() {
        return Ok(0);
    }

    let mut notes = load_wallet_notes()?;
    let mut released = 0usize;
    for note in &mut notes {
        if note.spent && targets.contains(&note.nullifier_bytes) {
            note.spent = false;
            released += 1;
        }
    }

    if released > 0 {
        save_wallet_notes(&notes)?;
    }

    Ok(released)
}

/// Merge notes discovered during a scan into an existing cache.
#[cfg(feature = "native")]
pub fn merge_scanned_notes(
    existing: &mut Vec<SerializableOrchardNote>,
    new_notes: &[SerializableOrchardNote],
) {
    use std::collections::HashSet;

    let existing_nullifiers: HashSet<Vec<u8>> =
        existing.iter().map(|n| n.nullifier_bytes.clone()).collect();

    for new_note in new_notes {
        if let Some(existing) = existing.iter_mut().find(|n| {
            n.txid == new_note.txid
                && n.block_height == new_note.block_height
                && n.value == new_note.value
        }) {
            existing.spent = existing.spent || new_note.spent;
            if new_note.rho_bytes.is_some() {
                existing.nullifier_bytes = new_note.nullifier_bytes.clone();
                existing.rho_bytes = new_note.rho_bytes.clone();
                existing.rseed_bytes = new_note.rseed_bytes.clone();
            }
            continue;
        }
        if !existing_nullifiers.contains(&new_note.nullifier_bytes) {
            existing.push(new_note.clone());
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NoteScanResult {
    pub notes: Vec<SerializableOrchardNote>,
    pub total_balance: u64,
    pub unspent_count: usize,
    pub spendable_count: usize,
}

pub struct NoteScanner {
    wallet: HDWallet,
    zebra_client: ZebraClient,
    note_index: Option<NoteIndex>,
    block_cache: Option<Arc<SimpleCache<Vec<ParsedTransaction>>>>,
    parallel_blocks: usize,
}

impl NoteScanner {
    pub fn new(wallet: HDWallet, zebra_client: ZebraClient) -> Self {
        Self {
            wallet,
            zebra_client,
            note_index: None,
            block_cache: None,
            parallel_blocks: 5,
        }
    }

    pub fn set_parallel_blocks(&mut self, count: usize) {
        self.parallel_blocks = count.max(1).min(50);
    }

    pub fn enable_block_cache(&mut self) {
        if self.block_cache.is_none() {
            self.block_cache = Some(Arc::new(SimpleCache::new(3600))); // 1 hour TTL
        }
    }

    pub fn with_index(wallet: HDWallet, zebra_client: ZebraClient, index: NoteIndex) -> Self {
        Self {
            wallet,
            zebra_client,
            note_index: Some(index),
            block_cache: None,
            parallel_blocks: 5,
        }
    }

    pub fn with_index_file(
        wallet: HDWallet,
        zebra_client: ZebraClient,
        index_path: &std::path::PathBuf,
    ) -> NozyResult<Self> {
        let index = NoteIndex::load_from_file(index_path)?;
        Ok(Self {
            wallet,
            zebra_client,
            note_index: Some(index),
            block_cache: None,
            parallel_blocks: 5,
        })
    }

    pub fn get_index(&self) -> Option<&NoteIndex> {
        self.note_index.as_ref()
    }

    pub fn get_index_mut(&mut self) -> Option<&mut NoteIndex> {
        self.note_index.as_mut()
    }

    pub fn take_index(self) -> Option<NoteIndex> {
        self.note_index
    }

    pub fn save_index(&self, path: &std::path::PathBuf) -> NozyResult<()> {
        if let Some(ref index) = self.note_index {
            index.save_to_file(path)
        } else {
            Ok(())
        }
    }

    pub async fn scan_notes(
        &mut self,
        start_height: Option<u32>,
        end_height: Option<u32>,
    ) -> NozyResult<(NoteScanResult, Vec<SpendableNote>)> {
        let start_height = if let Some(start) = start_height {
            start
        } else {
            3050000
        };

        let chain_tip = self.zebra_client.get_block_count().await?;
        let requested_end = if let Some(end) = end_height {
            end
        } else {
            let max_scan = start_height + 1000;
            max_scan
        };
        let end_height = requested_end.min(chain_tip);

        let start_height = start_height.min(end_height);

        let total_blocks = (end_height - start_height + 1) as u64;
        if scan_log::scan_progress_enabled() {
            println!(
                "Scanning blocks {} to {} for Orchard notes... ({} blocks)",
                start_height, end_height, total_blocks
            );
        } else {
            tracing::info!(
                start_height,
                end_height,
                total_blocks,
                "Scanning Orchard notes"
            );
        }

        let pb = if scan_log::scan_progress_enabled() {
            let pb = ProgressBar::new(total_blocks);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} blocks ({percent}%) | {msg}")
                    .unwrap_or_else(|_| ProgressStyle::default_bar())
                    .progress_chars("#>-"),
            );
            pb.set_message("Scanning blocks...");
            pb
        } else {
            ProgressBar::hidden()
        };

        let account_id = AccountId::try_from(0)
            .map_err(|e| NozyError::KeyDerivation(format!("Invalid account ID: {:?}", e)))?;
        let mnemonic = self.wallet.get_mnemonic_object();

        let mut seed_bytes = mnemonic.to_seed("").to_vec();
        let secure_seed = SecureSeed::new(seed_bytes.clone());

        let orchard_sk = SpendingKey::from_zip32_seed(secure_seed.as_bytes(), 133, account_id)
            .map_err(|e| {
                NozyError::KeyDerivation(format!("Failed to derive Orchard spending key: {:?}", e))
            })?;

        let orchard_fvk = FullViewingKey::from(&orchard_sk);
        let orchard_ivk_external = orchard_fvk.to_ivk(orchard::keys::Scope::External);
        let orchard_ivk_internal = orchard_fvk.to_ivk(orchard::keys::Scope::Internal);

        zeroize_bytes(&mut seed_bytes);

        if scan_log::scan_progress_enabled() {
            pb.println(
                "Generated scanning keys from wallet mnemonic (Orchard only; External and Internal scopes)",
            );
        }

        let mut note_index = self.note_index.take().unwrap_or_else(NoteIndex::new);
        let mut all_notes = Vec::new();
        let mut spendable_notes = Vec::new();

        let block_cache = self
            .block_cache
            .clone()
            .unwrap_or_else(|| Arc::new(SimpleCache::new(3600)));

        let mut current_height = start_height;
        let batch_size = self.parallel_blocks;
        let zebra_client = &self.zebra_client;

        let mut witness_tracker: Option<OrchardWitnessTracker> =
            if matches!(zebra_client.protocol(), Protocol::JsonRpc) {
                let initial_tree: OrchardCommitmentTree = if start_height == 0 {
                    OrchardCommitmentTree::empty()
                } else {
                    let cp = start_height.saturating_sub(1);
                    let parsed = zebra_client.get_orchard_treestate_parsed(cp).await?;
                    if let Some(fs) = parsed.final_state {
                        orchard_commitment_tree_from_final_state(&fs)?
                    } else {
                        OrchardCommitmentTree::empty()
                    }
                };
                Some(OrchardWitnessTracker::new(initial_tree))
            } else {
                None
            };

        while current_height <= end_height {
            let batch_end = (current_height + batch_size as u32 - 1).min(end_height);

            let block_futures: Vec<_> = (current_height..=batch_end)
                .map(|height| {
                    let cache = block_cache.clone();
                    let client = zebra_client.clone();
                    async move {
                        let cache_key = format!("block_{}", height);
                        if let Some(cached) = cache.get(&cache_key) {
                            return (height, Ok(cached));
                        }

                        let block_hash = match client.get_block_hash(height).await {
                            Ok(hash) => hash,
                            Err(e) => {
                                return (
                                    height,
                                    Err(NozyError::NetworkError(format!(
                                        "Failed to get block hash: {}",
                                        e
                                    ))),
                                )
                            }
                        };

                        let block_data = match client.get_block_by_hash(&block_hash, 2).await {
                            Ok(data) => data,
                            Err(e) => {
                                return (
                                    height,
                                    Err(NozyError::NetworkError(format!(
                                        "Failed to get block: {}",
                                        e
                                    ))),
                                )
                            }
                        };

                        let block_value = match serde_json::to_value(&block_data) {
                            Ok(value) => value,
                            Err(e) => {
                                return (
                                    height,
                                    Err(NozyError::InvalidOperation(format!(
                                        "Failed to serialize block data: {}",
                                        e
                                    ))),
                                )
                            }
                        };
                        let result = Self::parse_block_data(&block_value, height);
                        if let Ok(ref txs) = result {
                            cache.set(cache_key, txs.clone());
                        }
                        (height, result)
                    }
                })
                .collect();

            let block_results = join_all(block_futures).await;

            for (height, block_result) in block_results {
                pb.set_message(format!("Block {}", height));

                match block_result {
                    Ok(transactions) => {
                        if let Err(e) = self.process_block_orchard_actions(
                            &transactions,
                            height,
                            &orchard_fvk,
                            &orchard_ivk_external,
                            &orchard_ivk_internal,
                            &orchard_sk,
                            witness_tracker.as_mut(),
                            &mut note_index,
                            &mut all_notes,
                            &mut spendable_notes,
                            &pb,
                        ) {
                            return Err(NozyError::ScanAtBlock {
                                height,
                                detail: e.to_string(),
                            });
                        }
                    }
                    Err(e) => {
                        return Err(NozyError::ScanAtBlock {
                            height,
                            detail: format!("Failed to fetch block while scanning notes: {}", e),
                        });
                    }
                }

                pb.inc(1);
            }

            current_height = batch_end + 1;
        }

        if let Some(ref tr) = witness_tracker {
            note_index.apply_orchard_witnesses_from_tracker(tr, end_height)?;
            for sn in &mut spendable_notes {
                let nf = sn.orchard_note.nullifier.to_bytes();
                if let Some(w) = tr.serialized_witness_for_nullifier(&nf)? {
                    sn.orchard_incremental_witness_hex = Some(hex::encode(w));
                    sn.orchard_witness_tip_height = Some(end_height);
                }
            }
        }

        spendable_notes.retain(|sn| {
            let nf = sn.orchard_note.nullifier.to_bytes();
            note_index
                .get_note_by_nullifier(&nf)
                .map(|n| !n.spent)
                .unwrap_or(!sn.orchard_note.spent)
        });

        if scan_log::scan_progress_enabled() {
            pb.finish_with_message("Scanning complete!");
        }

        let total_balance = note_index.total_balance();
        let unspent_count = note_index.unspent_count();
        let spendable_count = spendable_notes.len();

        let serializable_notes: Vec<SerializableOrchardNote> = note_index.get_all_notes().to_vec();

        self.note_index = Some(note_index);

        let result = NoteScanResult {
            notes: serializable_notes,
            total_balance,
            unspent_count,
            spendable_count,
        };

        if scan_log::scan_progress_enabled() {
            println!(
                "Scan complete: Orchard {} notes ({} spendable, {} ZAT)",
                result.notes.len(),
                spendable_count,
                total_balance
            );
        } else {
            tracing::info!(
                notes = result.notes.len(),
                spendable = spendable_count,
                balance_zat = total_balance,
                "Orchard scan complete"
            );
        }

        Ok((result, spendable_notes))
    }

    fn process_block_orchard_actions(
        &self,
        transactions: &[ParsedTransaction],
        block_height: u32,
        orchard_fvk: &FullViewingKey,
        orchard_ivk_external: &IncomingViewingKey,
        orchard_ivk_internal: &IncomingViewingKey,
        orchard_sk: &SpendingKey,
        mut witness_tracker: Option<&mut OrchardWitnessTracker>,
        note_index: &mut NoteIndex,
        all_notes: &mut Vec<OrchardNote>,
        spendable_notes: &mut Vec<SpendableNote>,
        pb: &ProgressBar,
    ) -> NozyResult<()> {
        for tx in transactions {
            for (action_idx, action) in tx.orchard_actions.iter().enumerate() {
                if action.nullifier.len() == 32 {
                    if note_index.mark_note_spent_on_chain(&action.nullifier, orchard_fvk) {
                        spendable_notes.retain(|sn| {
                            sn.orchard_note.nullifier.to_bytes().as_slice() != action.nullifier
                        });
                    }
                }

                let cmx_node = merkle_hash_from_cmx_bytes(&action.cmx)?;
                if let Some(tr) = witness_tracker.as_mut() {
                    tr.append_cmx(cmx_node)?;
                }

                let chosen = match self.decrypt_orchard_action(
                    action,
                    orchard_fvk,
                    orchard_ivk_external,
                    block_height,
                    &tx.txid,
                    orchard::keys::Scope::External,
                ) {
                    Ok(Some(n)) => Some((n, orchard::keys::Scope::External)),
                    Ok(None) => match self.decrypt_orchard_action(
                        action,
                        orchard_fvk,
                        orchard_ivk_internal,
                        block_height,
                        &tx.txid,
                        orchard::keys::Scope::Internal,
                    ) {
                        Ok(Some(n)) => Some((n, orchard::keys::Scope::Internal)),
                        Ok(None) => None,
                        Err(e) => return Err(e),
                    },
                    Err(e) => return Err(e),
                };

                if let Some((orchard_note, scope)) = chosen {
                    if let Some(tr) = witness_tracker.as_mut() {
                        tr.register_discovered_note(orchard_note.nullifier.to_bytes())?;
                    }

                    if scan_log::scan_progress_enabled() {
                        pb.println(format!(
                            "Found note in block {} (scope: {:?})",
                            block_height, scope
                        ));
                    } else {
                        tracing::info!(block_height, ?scope, "Found Orchard note");
                    }

                    let nf = orchard_note.nullifier.to_bytes();
                    let witness_hex = if let Some(t) = witness_tracker.as_ref() {
                        match t.serialized_witness_for_nullifier(&nf) {
                            Ok(Some(b)) => Some(hex::encode(b)),
                            Ok(None) => None,
                            Err(e) => return Err(e),
                        }
                    } else {
                        None
                    };

                    let mut serializable: SerializableOrchardNote = (&orchard_note).into();
                    serializable.orchard_incremental_witness_hex = witness_hex.clone();
                    serializable.orchard_witness_tip_height = Some(block_height);
                    note_index.add_note(serializable);

                    all_notes.push(orchard_note.clone());

                    spendable_notes.push(SpendableNote {
                        orchard_note,
                        spending_key: orchard_sk.clone(),
                        derivation_path: format!("m/32'/133'/0'/0/{}", action_idx),
                        orchard_incremental_witness_hex: witness_hex,
                        orchard_witness_tip_height: Some(block_height),
                    });
                }
            }
        }
        Ok(())
    }

    fn decrypt_orchard_action(
        &self,
        action: &OrchardActionData,
        orchard_fvk: &FullViewingKey,
        ivk: &IncomingViewingKey,
        block_height: u32,
        txid: &str,
        scope: orchard::keys::Scope,
    ) -> NozyResult<Option<OrchardNote>> {
        use orchard::{
            keys::PreparedIncomingViewingKey,
            note::{ExtractedNoteCommitment, Nullifier},
            note_encryption::{CompactAction, OrchardDomain},
        };
        use zcash_note_encryption::EphemeralKeyBytes;

        scan_verbose!(
            "Attempting Orchard action decryption in transaction {} (scope: {:?})",
            txid,
            scope
        );

        let nullifier_result = Nullifier::from_bytes(&action.nullifier);
        let nullifier = match nullifier_result.into_option() {
            Some(n) => n,
            None => {
                scan_verbose!("Invalid nullifier bytes");
                return Ok(None);
            }
        };

        let cmx_result = ExtractedNoteCommitment::from_bytes(&action.cmx);
        let cmx = match cmx_result.into_option() {
            Some(c) => c,
            None => {
                scan_verbose!("Invalid cmx bytes");
                return Ok(None);
            }
        };

        let ephemeral_key = EphemeralKeyBytes::from(action.ephemeral_key);

        let mut compact_enc_ciphertext = [0u8; 52];
        if action.encrypted_note.len() >= 52 {
            compact_enc_ciphertext.copy_from_slice(&action.encrypted_note[..52]);
        } else {
            scan_verbose!("Encrypted note too short for CompactAction");
            return Ok(None);
        }

        scan_verbose!("Parsed Orchard action components from blockchain");
        scan_verbose!("   Nullifier: {}", hex::encode(&action.nullifier));
        scan_verbose!("   CMX: {}", hex::encode(&action.cmx));
        scan_verbose!("   CV: {}", hex::encode(&action.cv));
        scan_verbose!("   RK: {}", hex::encode(&action.rk));
        scan_verbose!("   Ephemeral Key: {}", hex::encode(&action.ephemeral_key));
        scan_verbose!("   Encrypted Note: {} bytes", action.encrypted_note.len());
        scan_verbose!("   Out Ciphertext: {} bytes", action.enc_ciphertext.len());

        let compact_action =
            CompactAction::from_parts(nullifier, cmx, ephemeral_key, compact_enc_ciphertext);

        let domain = OrchardDomain::for_compact_action(&compact_action);

        let prepared_ivk = PreparedIncomingViewingKey::new(ivk);

        scan_verbose!("Orchard decryption framework ready (CompactAction + OrchardDomain + IVK)");

        match try_compact_note_decryption(&domain, &prepared_ivk, &compact_action) {
            Some((note, address)) => {
                scan_verbose!("Successfully decrypted note: {} ZAT", note.value().inner());

                let canonical_nullifier = note.nullifier(orchard_fvk);

                let orchard_note = OrchardNote {
                    note: note.clone(),
                    value: note.value().inner(),
                    address: address.clone(),
                    nullifier: canonical_nullifier,
                    block_height,
                    txid: txid.to_string(),
                    spent: false,
                    memo: Vec::new(),
                };

                Ok(Some(orchard_note))
            }
            None => Ok(None),
        }
    }

    pub(crate) fn parse_block_data(
        block_data: &serde_json::Value,
        height: u32,
    ) -> NozyResult<Vec<ParsedTransaction>> {
        let block_json: serde_json::Value = serde_json::to_value(block_data).map_err(|e| {
            NozyError::InvalidOperation(format!("Failed to serialize block data: {}", e))
        })?;

        let mut transactions = Vec::new();

        if let Some(tx_array) = block_json.get("tx").and_then(|v| v.as_array()) {
            for (i, tx) in tx_array.iter().enumerate() {
                if let Some(txid) = tx.get("txid").and_then(|v| v.as_str()) {
                    let has_orchard = tx
                        .get("orchard")
                        .and_then(|o| o.as_object())
                        .and_then(|o| o.get("actions"))
                        .and_then(|a| a.as_array())
                        .map(|a| !a.is_empty())
                        .unwrap_or(false);

                    if has_orchard {
                        let raw_hex = tx.get("hex").and_then(|v| v.as_str()).unwrap_or("");

                        let orchard_json = tx.get("orchard").ok_or_else(|| {
                            NozyError::InvalidOperation(
                                "No orchard data in transaction".to_string(),
                            )
                        })?;
                        let orchard_actions =
                            Self::parse_orchard_actions_from_json_static(orchard_json)?;

                        let parsed_tx = ParsedTransaction {
                            txid: txid.to_string(),
                            height,
                            index: i as u32,
                            raw_data: hex::decode(raw_hex).unwrap_or_default(),
                            orchard_actions,
                        };

                        transactions.push(parsed_tx);
                    }
                }
            }
        }

        Ok(transactions)
    }

    #[allow(dead_code)] // Reserved for future block transaction parsing functionality
    async fn get_block_transactions(&self, height: u32) -> NozyResult<Vec<ParsedTransaction>> {
        let block_hash = self.zebra_client.get_block_hash(height).await?;
        let block_data = self.zebra_client.get_block_by_hash(&block_hash, 2).await?;
        let block_value = serde_json::to_value(&block_data).map_err(|e| {
            NozyError::InvalidOperation(format!("Failed to serialize block data: {}", e))
        })?;
        Self::parse_block_data(&block_value, height)
    }

    #[allow(dead_code)] // Reserved for future Orchard action parsing functionality
    fn parse_orchard_actions_from_json(
        &self,
        orchard_json: &serde_json::Value,
    ) -> NozyResult<Vec<OrchardActionData>> {
        Self::parse_orchard_actions_from_json_static(orchard_json)
    }

    fn parse_orchard_actions_from_json_static(
        orchard_json: &serde_json::Value,
    ) -> NozyResult<Vec<OrchardActionData>> {
        let mut actions = Vec::new();

        if let Some(actions_array) = orchard_json["actions"].as_array() {
            scan_verbose!("Found {} Orchard actions to parse", actions_array.len());
            for (idx, action) in actions_array.iter().enumerate() {
                let nullifier_hex = action["nullifier"].as_str().unwrap_or("");
                let cmx_hex = action["cmx"].as_str().unwrap_or("");
                let ephemeral_key_hex = action["ephemeralKey"].as_str().unwrap_or("");
                let enc_ciphertext_hex = action["encCiphertext"].as_str().unwrap_or("");
                let out_ciphertext_hex = action["outCiphertext"].as_str().unwrap_or("");

                let cv_hex = action["cv"].as_str().unwrap_or("");
                let rk_hex = action["rk"].as_str().unwrap_or("");

                let nullifier = hex::decode(nullifier_hex).map_err(|e| {
                    NozyError::InvalidOperation(format!("Failed to decode nullifier hex: {}", e))
                })?;
                let cmx = hex::decode(cmx_hex).map_err(|e| {
                    NozyError::InvalidOperation(format!("Failed to decode cmx hex: {}", e))
                })?;
                let ephemeral_key = hex::decode(ephemeral_key_hex).map_err(|e| {
                    NozyError::InvalidOperation(format!(
                        "Failed to decode ephemeral_key hex: {}",
                        e
                    ))
                })?;
                let enc_ciphertext = hex::decode(enc_ciphertext_hex).map_err(|e| {
                    NozyError::InvalidOperation(format!(
                        "Failed to decode encCiphertext hex: {}",
                        e
                    ))
                })?;
                let out_ciphertext = hex::decode(out_ciphertext_hex).map_err(|e| {
                    NozyError::InvalidOperation(format!(
                        "Failed to decode outCiphertext hex: {}",
                        e
                    ))
                })?;
                let cv = hex::decode(cv_hex).map_err(|e| {
                    NozyError::InvalidOperation(format!("Failed to decode cv hex: {}", e))
                })?;
                let rk = hex::decode(rk_hex).map_err(|e| {
                    NozyError::InvalidOperation(format!("Failed to decode rk hex: {}", e))
                })?;

                if nullifier.len() == 32
                    && cmx.len() == 32
                    && ephemeral_key.len() == 32
                    && enc_ciphertext.len() >= 52
                    && out_ciphertext.len() == 80
                    && cv.len() == 32
                    && rk.len() == 32
                {
                    let mut nullifier_bytes = [0u8; 32];
                    let mut cmx_bytes = [0u8; 32];
                    let mut ephemeral_key_bytes = [0u8; 32];
                    let mut encrypted_note_bytes = [0u8; 580];
                    let mut enc_ciphertext_bytes = [0u8; 80];
                    let mut cv_bytes = [0u8; 32];
                    let mut rk_bytes = [0u8; 32];

                    nullifier_bytes.copy_from_slice(&nullifier);
                    cmx_bytes.copy_from_slice(&cmx);
                    ephemeral_key_bytes.copy_from_slice(&ephemeral_key);
                    let enc_len = enc_ciphertext.len().min(580);
                    encrypted_note_bytes[..enc_len].copy_from_slice(&enc_ciphertext[..enc_len]);
                    enc_ciphertext_bytes.copy_from_slice(&out_ciphertext);
                    cv_bytes.copy_from_slice(&cv);
                    rk_bytes.copy_from_slice(&rk);

                    let action_data = OrchardActionData {
                        nullifier: nullifier_bytes,
                        cmx: cmx_bytes,
                        ephemeral_key: ephemeral_key_bytes,
                        encrypted_note: encrypted_note_bytes,
                        enc_ciphertext: enc_ciphertext_bytes,
                        cv: cv_bytes,
                        rk: rk_bytes,
                    };

                    actions.push(action_data);
                    scan_verbose!("Parsed action {} successfully", idx);
                } else {
                    scan_verbose!("Skipping action {} with invalid field sizes", idx);
                    scan_verbose!(
                        "    nullifier: {} bytes (expected 32), cmx: {} bytes (expected 32), ephemeral_key: {} bytes (expected 32)",
                        nullifier.len(),
                        cmx.len(),
                        ephemeral_key.len()
                    );
                    scan_verbose!(
                        "    enc_ciphertext: {} bytes (expected >= 52), out_ciphertext: {} bytes (expected 80)",
                        enc_ciphertext.len(),
                        out_ciphertext.len()
                    );
                    scan_verbose!(
                        "    cv: {} bytes (expected 32), rk: {} bytes (expected 32)",
                        cv.len(),
                        rk.len()
                    );
                }
            }
        } else {
            scan_verbose!("No 'actions' array found in Orchard JSON");
        }

        Ok(actions)
    }
}

#[derive(Debug, Clone)]
pub struct OrchardActionData {
    pub nullifier: [u8; 32],
    pub cmx: [u8; 32],
    pub ephemeral_key: [u8; 32],
    pub encrypted_note: [u8; 580],
    pub enc_ciphertext: [u8; 80],
    pub cv: [u8; 32],
    pub rk: [u8; 32],
}

pub async fn scan_real_notes(
    zebra_client: &ZebraClient,
    wallet: &HDWallet,
    start_height: u32,
    end_height: u32,
) -> NozyResult<Vec<SpendableNote>> {
    if scan_log::scan_progress_enabled() {
        println!(
            "Scanning blockchain for Orchard notes from height {} to {}",
            start_height, end_height
        );
    } else {
        tracing::info!(start_height, end_height, "Scanning Orchard notes");
    }

    let mut scanner = NoteScanner::new(wallet.clone(), zebra_client.clone());
    let (_, spendable_notes) = scanner
        .scan_notes(Some(start_height), Some(end_height))
        .await?;

    if scan_log::scan_progress_enabled() {
        println!(
            "Note scanning completed. Found {} spendable notes",
            spendable_notes.len()
        );
    } else {
        tracing::info!(
            spendable = spendable_notes.len(),
            "Orchard note scan completed"
        );
    }
    Ok(spendable_notes)
}
