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
use orchard::keys::{FullViewingKey, SpendingKey};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
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

fn discovered_from_compact(
    wallet: &HDWallet,
    address: &str,
    fvk: &FullViewingKey,
    action: &OrchardActionCompactData,
    height: u32,
    txid: &str,
    pool: ShieldedPool,
) -> NozyResult<Option<SerializableOrchardNote>> {
    let Some(decrypted) =
        wallet.decrypt_orchard_action_compact(action, address, height, txid, pool)?
    else {
        return Ok(None);
    };
    let mut note = SerializableOrchardNote {
        note_bytes: decrypted.cmx.to_vec(),
        value: decrypted.value,
        address_bytes: decrypted.orchard_address_raw,
        nullifier_bytes: decrypted.nullifier.to_vec(),
        block_height: height,
        txid: txid.to_string(),
        spent: false,
        memo: Vec::new(),
        orchard_incremental_witness_hex: None,
        orchard_witness_tip_height: None,
        ironwood_incremental_witness_hex: None,
        ironwood_witness_tip_height: None,
        rho_bytes: Some(decrypted.rho.to_vec()),
        rseed_bytes: Some(decrypted.rseed.to_vec()),
        spent_in_txid: None,
        pool,
    };
    if let Some(canonical) = note.canonical_nullifier_bytes(fvk) {
        note.nullifier_bytes = canonical.to_vec();
    }
    Ok(Some(note))
}

fn other_pool(pool: ShieldedPool) -> ShieldedPool {
    match pool {
        ShieldedPool::Orchard => ShieldedPool::Ironwood,
        ShieldedPool::Ironwood => ShieldedPool::Orchard,
    }
}

fn decrypt_either_pool(
    wallet: &HDWallet,
    address: &str,
    fvk: &FullViewingKey,
    compact: &OrchardActionCompactData,
    height: u32,
    txid: &str,
    preferred: ShieldedPool,
) -> Option<SerializableOrchardNote> {
    for pool in [preferred, other_pool(preferred)] {
        match discovered_from_compact(wallet, address, fvk, compact, height, txid, pool) {
            Ok(Some(note)) => return Some(note),
            Ok(None) => {}
            Err(_) => {}
        }
    }
    None
}

fn scan_action_list(
    wallet: &HDWallet,
    address: &str,
    fvk: &FullViewingKey,
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
        if let Some(discovered) =
            decrypt_either_pool(wallet, address, fvk, &compact, height, txid, preferred)
        {
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
        let address = wallet.generate_orchard_address(
            account,
            0,
            zcash_protocol::consensus::NetworkType::Main,
        )?;
        let fvk = fvk_for_account(&wallet, account)?;
        prepared.push((address, fvk));
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
                for (address, fvk) in &prepared {
                    scan_action_list(
                        &wallet,
                        address,
                        fvk,
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
                        &wallet,
                        address,
                        fvk,
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
