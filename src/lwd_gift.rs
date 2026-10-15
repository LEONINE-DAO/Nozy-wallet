//! Build a shielded spend anchor from lightwalletd tree state plus the local compact cache.
//!
//! This is the Zodl path: the phone keeps the seed, the sync server supplies the tree,
//! and the finished transaction is submitted with `SendTransaction`.

use crate::error::{NozyError, NozyResult};
use crate::hd_wallet::HDWallet;
use crate::key_management::{zeroize_bytes, SecureSeed};
use crate::notes::{load_wallet_notes, SerializableOrchardNote, SpendableNote};
use crate::orchard_tree_codec::orchard_commitment_tree_from_final_state;
use crate::orchard_witness::{
    merkle_hash_from_cmx_bytes, merkle_path_from_witness, OrchardWitnessTracker,
};
use crate::shielded_pool::ShieldedPool;
use orchard::keys::SpendingKey;
use orchard::tree::{Anchor, MerklePath};
use std::collections::HashMap;
use std::path::Path;
use zeaking::lwd::{
    connect_lightwalletd, get_latest_tree_state, get_tree_state, orchard_slice_from_compact_block,
    LwdCompactStore, LwdTreeState,
};
use zip32::AccountId;

/// Phone proving is slower than a local node, so the expiry window is wider than the 5-block pilot.
pub const PHONE_GIFT_EXPIRY_DELTA_BLOCKS: u32 = 40;

pub struct LwdPreparedSpend {
    pub tip_height: u32,
    pub anchor: Anchor,
    pub merkle_path: MerklePath,
    pub note: SpendableNote,
    pub pool: ShieldedPool,
}

pub async fn prepare_gift_spend_from_lightwalletd(
    wallet: &HDWallet,
    lwd_url: &str,
    compact_db: &Path,
    needed_zatoshis: u64,
) -> NozyResult<LwdPreparedSpend> {
    let cached = load_wallet_notes()?;
    let unspent: Vec<&SerializableOrchardNote> = cached
        .iter()
        .filter(|n| !n.spent && n.to_orchard_note().is_some())
        .collect();
    if unspent.is_empty() {
        return Err(NozyError::InvalidOperation(
            "This wallet has no spendable shielded notes yet. Wait for sync, then try again."
                .into(),
        ));
    }

    let store = LwdCompactStore::open(compact_db)
        .map_err(|e| NozyError::InvalidOperation(format!("open compact store: {e}")))?;
    let min_h = store
        .min_compact_height()
        .map_err(|e| NozyError::Storage(format!("compact min: {e}")))?
        .ok_or_else(|| {
            NozyError::InvalidOperation(
                "This phone has not synced compact blocks yet. Open the wallet and wait for sync."
                    .into(),
            )
        })?;
    let max_h = store
        .max_compact_height()
        .map_err(|e| NozyError::Storage(format!("compact max: {e}")))?
        .ok_or_else(|| {
            NozyError::InvalidOperation(
                "This phone has not synced compact blocks yet. Open the wallet and wait for sync."
                    .into(),
            )
        })?;

    let mut client = connect_lightwalletd(lwd_url)
        .await
        .map_err(|e| NozyError::InvalidOperation(format!("Nozy server: {e}")))?;
    let latest = get_latest_tree_state(&mut client)
        .await
        .map_err(|e| NozyError::InvalidOperation(format!("Nozy server tree: {e}")))?;
    let anchor_height = u64::from(max_h).min(latest.height);
    let anchor_state = if anchor_height == latest.height {
        latest.clone()
    } else {
        get_tree_state(&mut client, anchor_height, None)
            .await
            .map_err(|e| {
                NozyError::InvalidOperation(format!("Nozy server tree at {anchor_height}: {e}"))
            })?
    };

    let pools = [ShieldedPool::Ironwood, ShieldedPool::Orchard];
    let mut last_err = None;
    for pool in pools {
        let pool_notes: Vec<_> = unspent.iter().copied().filter(|n| n.pool == pool).collect();
        if pool_notes.is_empty() {
            continue;
        }
        match witness_pool(
            &mut client,
            &store,
            &anchor_state,
            min_h,
            anchor_height,
            pool,
            &pool_notes,
        )
        .await
        {
            Ok(witnessed) => {
                if let Some(mut prepared) = pick_note(wallet, witnessed, needed_zatoshis, pool)? {
                    prepared.tip_height = u32::try_from(anchor_height).unwrap_or(u32::MAX);
                    return Ok(prepared);
                }
            }
            Err(e) => last_err = Some(e),
        }
    }
    Err(last_err.unwrap_or_else(|| {
        NozyError::InvalidOperation(
            "No shielded note on this phone can cover that amount yet.".into(),
        )
    }))
}

/// Spend one shielded note to `recipient` and broadcast it with lightwalletd `SendTransaction`.
///
/// The seed stays on the phone. Tree state and compact blocks come from the sync server,
/// the same path gift-card payments already use.
pub async fn send_payment_via_lightwalletd(
    wallet: &HDWallet,
    recipient: &str,
    amount_zec: f64,
    memo: Option<&str>,
    lwd_url: &str,
    compact_db: &Path,
) -> NozyResult<String> {
    if !(amount_zec.is_finite() && amount_zec > 0.0) {
        return Err(NozyError::InvalidInput(
            "Amount must be a positive number of ZEC.".into(),
        ));
    }
    let amount_zatoshis = (amount_zec * 100_000_000.0).round() as u64;
    if amount_zatoshis == 0 {
        return Err(NozyError::InvalidInput(
            "Amount is too small (rounds to 0 zatoshis).".into(),
        ));
    }
    let recipient = crate::input_validation::normalize_unified_address(recipient);
    crate::input_validation::validate_zcash_address(&recipient)?;

    let memo_bytes = memo
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(|text| text.as_bytes().to_vec());
    if memo_bytes.as_ref().is_some_and(|bytes| bytes.len() > 512) {
        return Err(NozyError::InvalidInput(
            "Memo is longer than 512 bytes.".into(),
        ));
    }

    let mainnet = crate::config::load_config().network != "testnet";
    let block = crate::fee_policy::FeeBlock::next_block(mainnet, 3_500_000);
    let fee =
        crate::fee_policy::estimate_orchard_send_fee_zatoshis(memo_bytes.as_deref(), true, block);
    let needed = amount_zatoshis.saturating_add(fee);
    let notes = load_wallet_notes()?;
    let unspent_notes: Vec<_> = notes.iter().filter(|note| !note.spent).collect();
    let unspent: u64 = unspent_notes.iter().map(|note| note.value).sum();
    let largest = unspent_notes
        .iter()
        .map(|note| note.value)
        .max()
        .unwrap_or(0);
    let fee_zec = fee as f64 / 100_000_000.0;
    if unspent < needed {
        let have = unspent as f64 / 100_000_000.0;
        return Err(NozyError::InvalidOperation(format!(
            "Not enough spendable ZEC. This send needs the amount plus a network fee of {fee_zec:.8} ZEC (spendable {have:.8} ZEC)."
        )));
    }
    // One shielded action spends one note. A balance split across notes cannot be combined.
    if largest < needed {
        let largest_zec = largest as f64 / 100_000_000.0;
        let max_send_zec = largest.saturating_sub(fee) as f64 / 100_000_000.0;
        return Err(NozyError::InvalidOperation(format!(
            "This phone spends one shielded note at a time. You have {} unspent notes; the largest is {largest_zec:.8} ZEC. After the {fee_zec:.8} ZEC network fee, the most you can send is {max_send_zec:.8} ZEC.",
            unspent_notes.len()
        )));
    }

    let prepared =
        prepare_gift_spend_from_lightwalletd(wallet, lwd_url, compact_db, needed).await?;
    let zebra = crate::zebra_integration::ZebraClient::new("http://127.0.0.1:1".to_string());
    let memo_ref = memo_bytes.as_deref();
    let built = match prepared.pool {
        ShieldedPool::Ironwood => {
            crate::ironwood_tx::build_single_ironwood_spend_prepared(
                &zebra,
                &prepared.note,
                &recipient,
                amount_zatoshis,
                fee,
                memo_ref,
                PHONE_GIFT_EXPIRY_DELTA_BLOCKS,
                prepared.tip_height,
                prepared.anchor,
                prepared.merkle_path,
            )
            .await?
        }
        ShieldedPool::Orchard => {
            let builder = crate::orchard_tx::OrchardTransactionBuilder::new(true);
            builder
                .build_single_spend(
                    &zebra,
                    &crate::orchard_tx::ZebraJsonRpcOrchardWitnessProvider,
                    std::slice::from_ref(&prepared.note),
                    &recipient,
                    amount_zatoshis,
                    fee,
                    memo_ref,
                    PHONE_GIFT_EXPIRY_DELTA_BLOCKS,
                    Some((prepared.tip_height, prepared.anchor, prepared.merkle_path)),
                )
                .await?
        }
    };

    let txid = built.txid.clone();
    let mut client = connect_lightwalletd(lwd_url)
        .await
        .map_err(|e| NozyError::InvalidOperation(format!("Nozy server submit: {e}")))?;
    zeaking::lwd::send_transaction(&mut client, built.raw_transaction)
        .await
        .map_err(|e| {
            NozyError::InvalidOperation(format!("The network rejected this payment: {e}"))
        })?;
    let _ = crate::notes::mark_wallet_notes_spent_from_spendables(
        std::slice::from_ref(&prepared.note),
        Some(&txid),
    );
    Ok(txid)
}

async fn witness_pool(
    client: &mut zeaking::lwd::LwdClient,
    store: &LwdCompactStore,
    anchor_state: &LwdTreeState,
    min_h: u64,
    anchor_height: u64,
    pool: ShieldedPool,
    notes: &[&SerializableOrchardNote],
) -> NozyResult<Vec<(SerializableOrchardNote, Vec<u8>)>> {
    let anchor_tree_hex = tree_hex_for_pool(anchor_state, pool);
    if anchor_tree_hex.is_empty() {
        return Err(NozyError::InvalidOperation(format!(
            "The Nozy server did not return a {} tree, so this phone cannot build the payment yet.",
            pool.as_str()
        )));
    }
    let anchor_tree = decode_tree(&anchor_tree_hex)?;
    let checkpoint = if min_h == 0 {
        crate::orchard_tree_codec::OrchardCommitmentTree::empty()
    } else {
        let prior = get_tree_state(client, min_h.saturating_sub(1), None)
            .await
            .map_err(|e| {
                NozyError::InvalidOperation(format!(
                    "Nozy server tree at {}: {e}",
                    min_h.saturating_sub(1)
                ))
            })?;
        let hex = tree_hex_for_pool(&prior, pool);
        if hex.is_empty() {
            return Err(NozyError::InvalidOperation(format!(
                "The Nozy server has no {} tree at the start of this wallet's sync.",
                pool.as_str()
            )));
        }
        decode_tree(&hex)?
    };

    let mut wanted: HashMap<[u8; 32], [u8; 32]> = HashMap::new();
    for note in notes {
        if note.note_bytes.len() == 32 && note.nullifier_bytes.len() == 32 {
            let mut cmx = [0u8; 32];
            let mut nf = [0u8; 32];
            cmx.copy_from_slice(&note.note_bytes);
            nf.copy_from_slice(&note.nullifier_bytes);
            wanted.insert(cmx, nf);
        }
    }
    if wanted.is_empty() {
        return Err(NozyError::InvalidOperation(
            "Saved notes are missing the commitment needed to spend them. Sync again, then retry."
                .into(),
        ));
    }

    let mut tracker = OrchardWitnessTracker::new(checkpoint);
    for h in min_h..=anchor_height {
        let Some(blob) = store
            .get_compact_block(h)
            .map_err(|e| NozyError::Storage(format!("compact read {h}: {e}")))?
        else {
            return Err(NozyError::InvalidOperation(format!(
                "Compact block {h} is missing. Leave the wallet open until sync finishes, then retry."
            )));
        };
        let slice = orchard_slice_from_compact_block(&blob)
            .map_err(|e| NozyError::InvalidOperation(format!("compact block {h}: {e}")))?;
        for tx in slice.txs {
            let actions = match pool {
                ShieldedPool::Orchard => &tx.orchard_actions,
                ShieldedPool::Ironwood => &tx.ironwood_actions,
            };
            for action in actions {
                let node = merkle_hash_from_cmx_bytes(&action.cmx)?;
                tracker.append_cmx(node)?;
                if let Some(nf) = wanted.get(&action.cmx) {
                    tracker.register_discovered_note(*nf)?;
                }
            }
        }
    }

    if tracker.root_at_tip() != anchor_tree.root() {
        return Err(NozyError::InvalidOperation(format!(
            "The {} tree on this phone does not match the Nozy server at block {anchor_height}. Sync again, then retry.",
            pool.as_str()
        )));
    }

    let mut out = Vec::new();
    for note in notes {
        if note.nullifier_bytes.len() != 32 {
            continue;
        }
        let mut nf = [0u8; 32];
        nf.copy_from_slice(&note.nullifier_bytes);
        if let Some(bytes) = tracker.serialized_witness_for_nullifier(&nf)? {
            out.push(((*note).clone(), bytes));
        }
    }
    if out.is_empty() {
        return Err(NozyError::InvalidOperation(format!(
            "Sync has not found a {} note inside the downloaded blocks yet.",
            pool.as_str()
        )));
    }
    Ok(out)
}

fn tree_hex_for_pool(state: &LwdTreeState, pool: ShieldedPool) -> String {
    match pool {
        ShieldedPool::Ironwood => state.ironwood_tree.trim().to_string(),
        ShieldedPool::Orchard => state.orchard_tree.trim().to_string(),
    }
}

fn decode_tree(hex_str: &str) -> NozyResult<crate::orchard_tree_codec::OrchardCommitmentTree> {
    let raw = hex_str.trim().trim_start_matches("0x");
    let bytes = hex::decode(raw)
        .map_err(|e| NozyError::InvalidOperation(format!("Nozy server tree is not hex: {e}")))?;
    orchard_commitment_tree_from_final_state(&bytes)
}

fn pick_note(
    wallet: &HDWallet,
    witnessed: Vec<(SerializableOrchardNote, Vec<u8>)>,
    needed: u64,
    pool: ShieldedPool,
) -> NozyResult<Option<LwdPreparedSpend>> {
    let keys = account_spending_keys(wallet);
    let mut best: Option<(SerializableOrchardNote, Vec<u8>, SpendingKey)> = None;
    for (note, witness) in witnessed {
        if note.value < needed {
            continue;
        }
        let Some(sk) = keys.iter().find(|sk| {
            let fvk = orchard::keys::FullViewingKey::from(*sk);
            note.canonical_nullifier_bytes(&fvk)
                .is_some_and(|nf| nf.as_slice() == note.nullifier_bytes.as_slice())
        }) else {
            continue;
        };
        let sk = sk.clone();
        let replace = best
            .as_ref()
            .map(|(cur, _, _)| note.value < cur.value)
            .unwrap_or(true);
        if replace {
            best = Some((note, witness, sk));
        }
    }
    let Some((note, witness_bytes, spending_key)) = best else {
        return Ok(None);
    };
    let orchard_note = note.to_wallet_orchard_note().ok_or_else(|| {
        NozyError::InvalidOperation("Saved note could not be rebuilt for spending.".into())
    })?;
    let mut witness =
        crate::orchard_tree_codec::orchard_incremental_witness_from_bytes(&witness_bytes)?;
    let (anchor, merkle_path) = merkle_path_from_witness(&witness)?;
    let _ = &mut witness;
    let mut spendable = SpendableNote {
        orchard_note,
        spending_key,
        derivation_path: "m/32'/133'/0'".to_string(),
        pool,
        orchard_incremental_witness_hex: None,
        orchard_witness_tip_height: None,
        ironwood_incremental_witness_hex: None,
        ironwood_witness_tip_height: None,
    };
    let hex_wit = hex::encode(&witness_bytes);
    match pool {
        ShieldedPool::Orchard => {
            spendable.orchard_incremental_witness_hex = Some(hex_wit);
        }
        ShieldedPool::Ironwood => {
            spendable.ironwood_incremental_witness_hex = Some(hex_wit);
        }
    }
    Ok(Some(LwdPreparedSpend {
        tip_height: note.block_height,
        anchor,
        merkle_path,
        note: spendable,
        pool,
    }))
}

pub struct ExternalCompactScan {
    pub note: Option<SerializableOrchardNote>,
    pub spent: bool,
}

/// Trial-decrypt compact blocks for a gift-card key that is not part of the wallet seed.
pub fn scan_compact_store_for_key(
    store: &LwdCompactStore,
    spending_key: &SpendingKey,
    start_height: u32,
    end_height: u64,
) -> NozyResult<ExternalCompactScan> {
    let start = u64::from(start_height);
    if start > end_height {
        return Ok(ExternalCompactScan {
            note: None,
            spent: false,
        });
    }
    let fvk = orchard::keys::FullViewingKey::from(spending_key);
    let ivk_external = fvk.to_ivk(orchard::keys::Scope::External);
    let ivk_internal = fvk.to_ivk(orchard::keys::Scope::Internal);
    let mut found: Vec<SerializableOrchardNote> = Vec::new();
    for height in start..=end_height {
        let Some(blob) = store
            .get_compact_block(height)
            .map_err(|e| NozyError::Storage(format!("compact read {height}: {e}")))?
        else {
            return Err(NozyError::InvalidOperation(format!(
                "Compact block {height} is missing. Leave the wallet open until sync finishes, then redeem again."
            )));
        };
        let slice = orchard_slice_from_compact_block(&blob)
            .map_err(|e| NozyError::InvalidOperation(format!("compact block {height}: {e}")))?;
        let block_height = u32::try_from(height).unwrap_or(u32::MAX);
        for tx in slice.txs {
            let txid = compact_txid_hex(&tx.txid_bytes);
            for (pool, actions) in [
                (ShieldedPool::Orchard, &tx.orchard_actions),
                (ShieldedPool::Ironwood, &tx.ironwood_actions),
            ] {
                for action in actions {
                    for note in found.iter_mut() {
                        if !note.spent && note.nullifier_bytes.as_slice() == action.nullifier {
                            note.spent = true;
                        }
                    }
                    let data = crate::hd_wallet::OrchardActionCompactData {
                        nullifier: action.nullifier,
                        cmx: action.cmx,
                        ephemeral_key: action.ephemeral_key,
                        encrypted_note: action.ciphertext.clone(),
                    };
                    if let Some(note) = crate::orchard_compact_scan::try_decrypt_compact(
                        &fvk,
                        &ivk_external,
                        &ivk_internal,
                        &data,
                        block_height,
                        &txid,
                        pool,
                    ) {
                        found.push(note);
                    }
                }
            }
        }
    }
    let spent = found.iter().any(|note| note.spent);
    let note = found
        .into_iter()
        .filter(|note| !note.spent)
        .max_by_key(|note| note.value);
    Ok(ExternalCompactScan { note, spent })
}

pub async fn prepare_external_spend_from_lightwalletd(
    client: &mut zeaking::lwd::LwdClient,
    store: &LwdCompactStore,
    anchor_state: &LwdTreeState,
    anchor_height: u64,
    note: SerializableOrchardNote,
    spending_key: SpendingKey,
) -> NozyResult<LwdPreparedSpend> {
    let min_h = u64::from(note.block_height);
    if min_h > anchor_height {
        return Err(NozyError::InvalidOperation(
            "This gift card is not confirmed yet. Redeem it again after the payment is mined."
                .into(),
        ));
    }
    let pool = note.pool;
    let witnessed = witness_pool(
        client,
        store,
        anchor_state,
        min_h,
        anchor_height,
        pool,
        &[&note],
    )
    .await?;
    let Some((found, witness_bytes)) = witnessed.into_iter().next() else {
        return Err(NozyError::InvalidOperation(
            "The gift card note was found, but its witness is not ready. Sync again, then redeem."
                .into(),
        ));
    };
    let mut prepared = spend_from_witness(found, witness_bytes, spending_key, pool)?;
    prepared.tip_height = u32::try_from(anchor_height).unwrap_or(u32::MAX);
    Ok(prepared)
}

fn spend_from_witness(
    note: SerializableOrchardNote,
    witness_bytes: Vec<u8>,
    spending_key: SpendingKey,
    pool: ShieldedPool,
) -> NozyResult<LwdPreparedSpend> {
    let orchard_note = note.to_wallet_orchard_note().ok_or_else(|| {
        NozyError::InvalidOperation("Saved note could not be rebuilt for spending.".into())
    })?;
    let witness =
        crate::orchard_tree_codec::orchard_incremental_witness_from_bytes(&witness_bytes)?;
    let (anchor, merkle_path) = merkle_path_from_witness(&witness)?;
    let hex_wit = hex::encode(&witness_bytes);
    let mut spendable = SpendableNote {
        orchard_note,
        spending_key,
        derivation_path: "m/32'/133'/0'".to_string(),
        pool,
        orchard_incremental_witness_hex: None,
        orchard_witness_tip_height: None,
        ironwood_incremental_witness_hex: None,
        ironwood_witness_tip_height: None,
    };
    match pool {
        ShieldedPool::Orchard => {
            spendable.orchard_incremental_witness_hex = Some(hex_wit);
        }
        ShieldedPool::Ironwood => {
            spendable.ironwood_incremental_witness_hex = Some(hex_wit);
        }
    }
    Ok(LwdPreparedSpend {
        tip_height: note.block_height,
        anchor,
        merkle_path,
        note: spendable,
        pool,
    })
}

fn compact_txid_hex(bytes: &[u8]) -> String {
    if bytes.len() == 32 {
        let mut rev = bytes.to_vec();
        rev.reverse();
        hex::encode(rev)
    } else {
        hex::encode(bytes)
    }
}

fn account_spending_keys(wallet: &HDWallet) -> Vec<SpendingKey> {
    let mnemonic = wallet.get_mnemonic_object();
    let mut seed_bytes = mnemonic.to_seed("").to_vec();
    let secure_seed = SecureSeed::new(seed_bytes.clone());
    let mut keys = Vec::new();
    for account in 0u32..8 {
        let Ok(account_id) = AccountId::try_from(account) else {
            continue;
        };
        if let Ok(sk) = SpendingKey::from_zip32_seed(secure_seed.as_bytes(), 133, account_id) {
            keys.push(sk);
        }
    }
    zeroize_bytes(&mut seed_bytes);
    keys
}
