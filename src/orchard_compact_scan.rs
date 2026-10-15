//! Orchard / Ironwood trial-decrypt from the local lightwalletd compact cache.
//!
//! Mobile receive addresses are Orchard UAs. Compact sync used to only scan Sapling,
//! so incoming shielded pays never appeared in balance or history.

use crate::error::{NozyError, NozyResult};
use crate::hd_wallet::{HDWallet, OrchardActionCompactData};
use crate::notes::{
    load_wallet_notes, merge_scanned_notes, save_wallet_notes, wallet_unspent_balance_zatoshis,
    SerializableOrchardNote,
};
use crate::paths::get_wallet_data_dir;
use crate::shielded_pool::ShieldedPool;
use orchard::keys::{
    FullViewingKey, IncomingViewingKey, PreparedIncomingViewingKey, Scope, SpendingKey,
};
use orchard::note::ExtractedNoteCommitment;
use orchard::note_encryption::{CompactAction, IronwoodDomain, OrchardDomain};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use zcash_note_encryption::{try_compact_note_decryption, EphemeralKeyBytes};
use zeaking::lwd::{orchard_slice_from_compact_block, LwdCompactStore};
use zip32::AccountId;

pub const ORCHARD_SCAN_PROGRESS_FILE: &str = "orchard_compact_scan_progress.json";

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrchardCompactScanProgress {
    pub last_scanned_height: u64,
}

#[derive(Debug, Clone, Default)]
pub struct OrchardCompactScanStats {
    pub blocks_scanned: u64,
    pub actions_seen: u64,
    pub orchard_actions_seen: u64,
    pub ironwood_actions_seen: u64,
    pub notes_discovered: u64,
    pub orchard_notes_discovered: u64,
    pub ironwood_notes_discovered: u64,
    pub notes_marked_spent: u64,
    pub range_start: u64,
    pub range_end: u64,
}

fn progress_path() -> PathBuf {
    get_wallet_data_dir().join(ORCHARD_SCAN_PROGRESS_FILE)
}

fn load_progress() -> OrchardCompactScanProgress {
    let path = progress_path();
    if !path.exists() {
        return OrchardCompactScanProgress::default();
    }
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_progress(progress: &OrchardCompactScanProgress) -> NozyResult<()> {
    let path = progress_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(
        &path,
        serde_json::to_string_pretty(progress)
            .map_err(|e| NozyError::Storage(format!("serialize orchard scan progress: {e}")))?,
    )
    .map_err(|e| NozyError::Storage(format!("write orchard scan progress: {e}")))
}

fn txid_hex(txid_bytes: &[u8]) -> String {
    hex::encode(txid_bytes)
}

fn fvk_for_account(wallet: &HDWallet, account: u32) -> NozyResult<FullViewingKey> {
    let account_id = AccountId::try_from(account)
        .map_err(|e| NozyError::KeyDerivation(format!("Invalid account ID: {e:?}")))?;
    let seed = wallet.get_mnemonic_object().to_seed("");
    let sk = SpendingKey::from_zip32_seed(&seed, 133, account_id).map_err(|e| {
        NozyError::KeyDerivation(format!("Failed to derive Orchard spending key: {e:?}"))
    })?;
    Ok(FullViewingKey::from(&sk))
}

fn mark_spent(
    notes: &mut [SerializableOrchardNote],
    nfs: &HashSet<[u8; 32]>,
    spent_in_txid: &str,
) -> u64 {
    let mut marked = 0u64;
    for note in notes.iter_mut() {
        if note.spent {
            continue;
        }
        let Ok(nf) = <[u8; 32]>::try_from(note.nullifier_bytes.as_slice()) else {
            continue;
        };
        if nfs.contains(&nf) {
            note.spent = true;
            note.spent_in_txid = Some(spent_in_txid.to_string());
            marked += 1;
        }
    }
    marked
}

pub(crate) fn try_decrypt_compact(
    fvk: &FullViewingKey,
    ivk_external: &IncomingViewingKey,
    ivk_internal: &IncomingViewingKey,
    action: &OrchardActionCompactData,
    height: u32,
    txid: &str,
    pool: ShieldedPool,
) -> Option<SerializableOrchardNote> {
    if action.encrypted_note.len() < 52 {
        return None;
    }
    let nullifier = orchard::note::Nullifier::from_bytes(&action.nullifier).into_option()?;
    let cmx = ExtractedNoteCommitment::from_bytes(&action.cmx).into_option()?;
    let ephemeral_key = EphemeralKeyBytes::from(action.ephemeral_key);
    let mut compact_enc_ciphertext = [0u8; 52];
    compact_enc_ciphertext.copy_from_slice(&action.encrypted_note[..52]);
    let compact_action =
        CompactAction::from_parts(nullifier, cmx, ephemeral_key, compact_enc_ciphertext);
    for ivk in [ivk_external, ivk_internal] {
        let prepared = PreparedIncomingViewingKey::new(ivk);
        let decrypted = match pool {
            ShieldedPool::Orchard => {
                let domain = OrchardDomain::for_compact_action(&compact_action);
                try_compact_note_decryption(&domain, &prepared, &compact_action)
            }
            ShieldedPool::Ironwood => {
                let domain = IronwoodDomain::for_compact_action(&compact_action);
                try_compact_note_decryption(&domain, &prepared, &compact_action)
            }
        };
        let Some((note, note_address)) = decrypted else {
            continue;
        };
        let decrypted_value = note.value().inner();
        let rho = note.rho().to_bytes();
        let rseed = *note.rseed().as_bytes();
        let address_bytes = note_address.to_raw_address_bytes().to_vec();
        let decrypted = SerializableOrchardNote {
            note_bytes: action.cmx.to_vec(),
            value: decrypted_value,
            address_bytes,
            nullifier_bytes: action.nullifier.to_vec(),
            block_height: height,
            txid: txid.to_string(),
            spent: false,
            // Compact blocks omit the memo half of encCiphertext. `received_memo`
            // fills this from the full transaction before history is served.
            memo: Vec::new(),
            orchard_incremental_witness_hex: None,
            orchard_witness_tip_height: None,
            ironwood_incremental_witness_hex: None,
            ironwood_witness_tip_height: None,
            rho_bytes: Some(rho.to_vec()),
            rseed_bytes: Some(rseed.to_vec()),
            spent_in_txid: None,
            pool,
        };
        let mut note = decrypted;
        if let Some(canonical) = note.canonical_nullifier_bytes(fvk) {
            note.nullifier_bytes = canonical.to_vec();
        }
        return Some(note);
    }
    None
}

fn other_pool(pool: ShieldedPool) -> ShieldedPool {
    match pool {
        ShieldedPool::Orchard => ShieldedPool::Ironwood,
        ShieldedPool::Ironwood => ShieldedPool::Orchard,
    }
}

fn decrypt_either_pool(
    fvk: &FullViewingKey,
    ivk_external: &IncomingViewingKey,
    ivk_internal: &IncomingViewingKey,
    compact: &OrchardActionCompactData,
    height: u32,
    txid: &str,
    preferred: ShieldedPool,
) -> Option<SerializableOrchardNote> {
    for pool in [preferred, other_pool(preferred)] {
        if let Some(note) =
            try_decrypt_compact(fvk, ivk_external, ivk_internal, compact, height, txid, pool)
        {
            return Some(note);
        }
    }
    None
}

fn scan_action_list(
    fvk: &FullViewingKey,
    ivk_external: &IncomingViewingKey,
    ivk_internal: &IncomingViewingKey,
    actions: &[zeaking::lwd::OrchardCompactActionBytes],
    height: u32,
    txid: &str,
    preferred: ShieldedPool,
    ironwood_field: bool,
    notes: &mut Vec<SerializableOrchardNote>,
    stats: &mut OrchardCompactScanStats,
) -> NozyResult<()> {
    let mut spend_nfs: HashSet<[u8; 32]> = HashSet::new();
    for action in actions {
        spend_nfs.insert(action.nullifier);
        stats.actions_seen += 1;
        if ironwood_field {
            stats.ironwood_actions_seen += 1;
        } else {
            stats.orchard_actions_seen += 1;
        }
        if action.ciphertext.len() < 52 {
            continue;
        }
        let compact = OrchardActionCompactData {
            nullifier: action.nullifier,
            cmx: action.cmx,
            ephemeral_key: action.ephemeral_key,
            encrypted_note: action.ciphertext.clone(),
        };
        if let Some(discovered) = decrypt_either_pool(
            fvk,
            ivk_external,
            ivk_internal,
            &compact,
            height,
            txid,
            preferred,
        ) {
            stats.notes_discovered += 1;
            match discovered.pool {
                ShieldedPool::Ironwood => stats.ironwood_notes_discovered += 1,
                ShieldedPool::Orchard => stats.orchard_notes_discovered += 1,
            }
            merge_scanned_notes(notes, std::slice::from_ref(&discovered));
        }
    }
    stats.notes_marked_spent += mark_spent(notes, &spend_nfs, txid);
    Ok(())
}

/// Incremental (or full) Orchard/Ironwood scan over cached compact blocks.
///
/// `max_account` is inclusive (ZIP-32 account index). Account 0 is always scanned.
pub fn scan_orchard_wallet_from_compact_store(
    mnemonic: &str,
    store: &LwdCompactStore,
    start_floor: Option<u64>,
    max_account: u32,
    full_rescan: bool,
) -> NozyResult<(Vec<SerializableOrchardNote>, OrchardCompactScanStats)> {
    let wallet = HDWallet::from_mnemonic(mnemonic)?;
    let Some(end_height) = store
        .max_compact_height()
        .map_err(|e| NozyError::Storage(format!("compact max height: {e}")))?
    else {
        return Ok((
            load_wallet_notes().unwrap_or_default(),
            OrchardCompactScanStats::default(),
        ));
    };
    let min_height = store
        .min_compact_height()
        .map_err(|e| NozyError::Storage(format!("compact min height: {e}")))?
        .unwrap_or(end_height);

    let mut notes = load_wallet_notes().unwrap_or_default();
    let rescan = full_rescan || notes.is_empty();
    let progress = if rescan {
        OrchardCompactScanProgress::default()
    } else {
        load_progress()
    };

    let mut start_height = if rescan {
        start_floor.unwrap_or(min_height)
    } else if progress.last_scanned_height > 0 {
        progress.last_scanned_height.saturating_add(1)
    } else {
        start_floor.unwrap_or(min_height)
    };
    start_height = start_height.max(min_height);
    let mut totals = OrchardCompactScanStats {
        range_start: start_height,
        range_end: end_height,
        ..Default::default()
    };
    if start_height > end_height {
        return Ok((notes, totals));
    }

    let mut accounts: Vec<u32> = (0..=max_account).collect();
    if !accounts.contains(&0) {
        accounts.insert(0, 0);
    }
    accounts.sort_unstable();
    accounts.dedup();

    let mut prepared = Vec::new();
    for account in accounts {
        let fvk = fvk_for_account(&wallet, account)?;
        let ivk_external = fvk.to_ivk(Scope::External);
        let ivk_internal = fvk.to_ivk(Scope::Internal);
        prepared.push((fvk, ivk_external, ivk_internal));
    }

    store
        .for_each_compact_block_range(start_height, end_height, |height, data| {
            let slice = orchard_slice_from_compact_block(data)
                .map_err(|e| zeaking::error::ZeakingError::InvalidOperation(e.to_string()))?;
            totals.blocks_scanned += 1;
            let height_u32 = height as u32;
            for tx in &slice.txs {
                let txid = txid_hex(&tx.txid_bytes);
                let preferred = if crate::ironwood::is_ironwood_active(height_u32, false) {
                    ShieldedPool::Ironwood
                } else {
                    ShieldedPool::Orchard
                };
                for (fvk, ivk_external, ivk_internal) in &prepared {
                    scan_action_list(
                        fvk,
                        ivk_external,
                        ivk_internal,
                        &tx.orchard_actions,
                        height_u32,
                        &txid,
                        preferred,
                        false,
                        &mut notes,
                        &mut totals,
                    )
                    .map_err(|e| zeaking::error::ZeakingError::InvalidOperation(e.to_string()))?;
                    scan_action_list(
                        fvk,
                        ivk_external,
                        ivk_internal,
                        &tx.ironwood_actions,
                        height_u32,
                        &txid,
                        ShieldedPool::Ironwood,
                        true,
                        &mut notes,
                        &mut totals,
                    )
                    .map_err(|e| zeaking::error::ZeakingError::InvalidOperation(e.to_string()))?;
                }
            }
            if totals.blocks_scanned % 32 == 0 {
                let _ = save_progress(&OrchardCompactScanProgress {
                    last_scanned_height: height,
                });
            }
            Ok(())
        })
        .map_err(|e| NozyError::InvalidOperation(format!("compact store iterate: {e}")))?;

    save_wallet_notes(&notes)?;
    save_progress(&OrchardCompactScanProgress {
        last_scanned_height: end_height,
    })?;
    let _ = wallet_unspent_balance_zatoshis(&notes);
    Ok((notes, totals))
}
