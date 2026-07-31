//! Sapling compact-block scan (Phase 2+) and spend material for Phase 4.
//!
//! Decrypts LWD `CompactSaplingOutput`s with the wallet IVK, persists notes
//! (including ZIP-212 `rseed` for later spends), and marks spends from compact
//! nullifiers. Generated UAs include Sapling since Phase 3.

use crate::error::{NozyError, NozyResult};
use crate::paths::get_wallet_data_dir;
use crate::sapling_keys::{derive_sapling_account_keys, SaplingAccountKeys};
use sapling::note::ExtractedNoteCommitment;
use sapling::note_encryption::{
    try_sapling_compact_note_decryption, CompactOutputDescription, Zip212Enforcement,
};
use sapling::value::NoteValue;
use sapling::{Note, Nullifier, PaymentAddress, Rseed};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use zcash_note_encryption::EphemeralKeyBytes;
use zeaking::lwd::{sapling_slice_from_compact_block, LwdCompactStore, SaplingCompactOutputBytes};
use zip32::Scope;

pub const SAPLING_NOTES_FILE: &str = "sapling_notes.json";
pub const SAPLING_SCAN_PROGRESS_FILE: &str = "sapling_scan_progress.json";

/// Persisted Sapling note discovered via compact scan.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SerializableSaplingNote {
    pub value: u64,
    pub address_bytes: Vec<u8>,
    pub nullifier_bytes: Vec<u8>,
    pub cmu_bytes: Vec<u8>,
    pub block_height: u32,
    pub txid: String,
    pub position: u64,
    pub spent: bool,
    #[serde(default)]
    pub spent_in_txid: Option<String>,
    /// ZIP 212 `rseed` (32 bytes). Required to reconstruct the note for spending (Phase 4).
    /// Absent on notes scanned before this field existed — rescan to populate.
    #[serde(default)]
    pub rseed_bytes: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SaplingScanProgress {
    pub last_scanned_height: u64,
    pub next_note_position: u64,
}

#[derive(Debug, Clone, Default)]
pub struct SaplingScanStats {
    pub blocks_scanned: u64,
    pub outputs_seen: u64,
    pub notes_discovered: u64,
    pub notes_marked_spent: u64,
    pub range_start: u64,
    pub range_end: u64,
}

fn sapling_notes_path() -> PathBuf {
    get_wallet_data_dir().join(SAPLING_NOTES_FILE)
}

fn sapling_progress_path() -> PathBuf {
    get_wallet_data_dir().join(SAPLING_SCAN_PROGRESS_FILE)
}

pub fn load_sapling_notes() -> NozyResult<Vec<SerializableSaplingNote>> {
    let path = sapling_notes_path();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let data = fs::read_to_string(&path)
        .map_err(|e| NozyError::Storage(format!("read {}: {e}", path.display())))?;
    serde_json::from_str(&data)
        .map_err(|e| NozyError::Storage(format!("parse {}: {e}", path.display())))
}

pub fn save_sapling_notes(notes: &[SerializableSaplingNote]) -> NozyResult<()> {
    let dir = get_wallet_data_dir();
    fs::create_dir_all(&dir)
        .map_err(|e| NozyError::Storage(format!("create {}: {e}", dir.display())))?;
    let path = sapling_notes_path();
    let data = serde_json::to_string_pretty(notes)
        .map_err(|e| NozyError::Storage(format!("serialize sapling notes: {e}")))?;
    fs::write(&path, data).map_err(|e| NozyError::Storage(format!("write {}: {e}", path.display())))
}

pub fn load_sapling_scan_progress() -> NozyResult<SaplingScanProgress> {
    let path = sapling_progress_path();
    if !path.exists() {
        return Ok(SaplingScanProgress::default());
    }
    let data = fs::read_to_string(&path)
        .map_err(|e| NozyError::Storage(format!("read {}: {e}", path.display())))?;
    serde_json::from_str(&data)
        .map_err(|e| NozyError::Storage(format!("parse {}: {e}", path.display())))
}

pub fn save_sapling_scan_progress(progress: &SaplingScanProgress) -> NozyResult<()> {
    let dir = get_wallet_data_dir();
    fs::create_dir_all(&dir)
        .map_err(|e| NozyError::Storage(format!("create {}: {e}", dir.display())))?;
    let path = sapling_progress_path();
    let data = serde_json::to_string_pretty(progress)
        .map_err(|e| NozyError::Storage(format!("serialize sapling scan progress: {e}")))?;
    fs::write(&path, data).map_err(|e| NozyError::Storage(format!("write {}: {e}", path.display())))
}

pub fn sapling_unspent_balance_zatoshis(notes: &[SerializableSaplingNote]) -> u64 {
    notes.iter().filter(|n| !n.spent).map(|n| n.value).sum()
}

/// ZIP 212 is enforced for compact scans (post-Canopy LWD traffic).
fn zip212_enforcement(_height: u32) -> Zip212Enforcement {
    Zip212Enforcement::On
}

fn compact_output_description(
    out: &SaplingCompactOutputBytes,
) -> NozyResult<CompactOutputDescription> {
    use zcash_note_encryption::COMPACT_NOTE_SIZE;
    if out.ciphertext.len() < COMPACT_NOTE_SIZE {
        return Err(NozyError::InvalidOperation(format!(
            "Sapling compact ciphertext too short: {} < {COMPACT_NOTE_SIZE}",
            out.ciphertext.len()
        )));
    }
    let mut enc = [0u8; COMPACT_NOTE_SIZE];
    enc.copy_from_slice(&out.ciphertext[..COMPACT_NOTE_SIZE]);
    let cmu = ExtractedNoteCommitment::from_bytes(&out.cmu)
        .into_option()
        .ok_or_else(|| NozyError::InvalidOperation("Invalid Sapling cmu".to_string()))?;
    Ok(CompactOutputDescription {
        ephemeral_key: EphemeralKeyBytes(out.ephemeral_key),
        cmu,
        enc_ciphertext: enc,
    })
}

fn txid_hex(txid_bytes: &[u8]) -> String {
    hex::encode(txid_bytes)
}

fn nullifier_bytes(nf: &Nullifier) -> Vec<u8> {
    nf.as_ref().to_vec()
}

fn rseed_bytes_from_note(note: &Note) -> Option<Vec<u8>> {
    match note.rseed() {
        Rseed::AfterZip212(bytes) => Some(bytes.to_vec()),
        Rseed::BeforeZip212(_) => None,
    }
}

/// True when the note has ZIP-212 rseed persisted (reconstructible). Witnesses are separate.
pub fn sapling_note_has_rseed(note: &SerializableSaplingNote) -> bool {
    note.rseed_bytes.as_ref().is_some_and(|b| b.len() == 32)
}

/// Reconstruct a Sapling `Note` from persisted fields (needs `rseed_bytes`).
pub fn reconstruct_sapling_note(note: &SerializableSaplingNote) -> NozyResult<Note> {
    let rseed_raw = note.rseed_bytes.as_ref().ok_or_else(|| {
        NozyError::InvalidOperation(
            "Sapling note missing rseed (rescan with Phase 4+ to persist spend material)".into(),
        )
    })?;
    let rseed_arr: [u8; 32] = rseed_raw
        .as_slice()
        .try_into()
        .map_err(|_| NozyError::InvalidOperation("Sapling rseed must be 32 bytes".into()))?;
    let addr_arr: [u8; 43] = note.address_bytes.as_slice().try_into().map_err(|_| {
        NozyError::InvalidOperation("Sapling address_bytes must be 43 bytes".into())
    })?;
    let address = PaymentAddress::from_bytes(&addr_arr).ok_or_else(|| {
        NozyError::InvalidOperation("Invalid Sapling payment address bytes".into())
    })?;
    Ok(Note::from_parts(
        address,
        NoteValue::from_raw(note.value),
        Rseed::AfterZip212(rseed_arr),
    ))
}

/// Try decrypting one compact Sapling output; returns a serializable note when it belongs to `keys`.
pub fn try_decrypt_sapling_compact_output(
    keys: &SaplingAccountKeys,
    out: &SaplingCompactOutputBytes,
    height: u32,
    txid: &str,
    position: u64,
) -> NozyResult<Option<SerializableSaplingNote>> {
    let prepared = keys.external_ivk.prepare();
    let description = compact_output_description(out)?;
    let Some((note, address)) =
        try_sapling_compact_note_decryption(&prepared, &description, zip212_enforcement(height))
    else {
        return Ok(None);
    };
    let nk = keys.dfvk.to_nk(Scope::External);
    let nf: Nullifier = note.nf(&nk, position);
    Ok(Some(SerializableSaplingNote {
        value: note.value().inner(),
        address_bytes: address.to_bytes().to_vec(),
        nullifier_bytes: nullifier_bytes(&nf),
        cmu_bytes: out.cmu.to_vec(),
        block_height: height,
        txid: txid.to_string(),
        position,
        spent: false,
        spent_in_txid: None,
        rseed_bytes: rseed_bytes_from_note(&note),
    }))
}

fn merge_discovered_note(
    notes: &mut Vec<SerializableSaplingNote>,
    new_note: SerializableSaplingNote,
) {
    if let Some(existing) = notes
        .iter_mut()
        .find(|n| n.nullifier_bytes == new_note.nullifier_bytes)
    {
        if !existing.spent {
            // Prefer newer discovery; keep rseed if the new record somehow lacks it.
            let keep_rseed = new_note.rseed_bytes.is_none() && existing.rseed_bytes.is_some();
            let rseed = if keep_rseed {
                existing.rseed_bytes.clone()
            } else {
                new_note.rseed_bytes.clone()
            };
            *existing = new_note;
            existing.rseed_bytes = rseed;
        }
        return;
    }
    notes.push(new_note);
}

fn mark_spent_by_nullifiers(
    notes: &mut [SerializableSaplingNote],
    nullifiers: &HashSet<[u8; 32]>,
    spent_in_txid: Option<&str>,
) -> u64 {
    let mut marked = 0u64;
    for note in notes.iter_mut() {
        if note.spent {
            continue;
        }
        let Ok(nf) = <[u8; 32]>::try_from(note.nullifier_bytes.as_slice()) else {
            continue;
        };
        if nullifiers.contains(&nf) {
            note.spent = true;
            if let Some(txid) = spent_in_txid {
                note.spent_in_txid = Some(txid.to_string());
            }
            marked += 1;
        }
    }
    marked
}

/// Scan one compact-block blob for Sapling notes/spends belonging to `keys`.
pub fn scan_sapling_compact_block_blob(
    keys: &SaplingAccountKeys,
    height: u64,
    data: &[u8],
    notes: &mut Vec<SerializableSaplingNote>,
    next_position: &mut u64,
) -> NozyResult<SaplingScanStats> {
    let slice = sapling_slice_from_compact_block(data)
        .map_err(|e| NozyError::InvalidOperation(format!("sapling compact decode: {e}")))?;
    let height_u32 = height as u32;
    let mut stats = SaplingScanStats {
        blocks_scanned: 1,
        range_start: height,
        range_end: height,
        ..Default::default()
    };

    // Prefer chain metadata: positions for this block are
    // [tree_size_after - outputs_in_block, tree_size_after).
    let outputs_in_block: u64 = slice.txs.iter().map(|t| t.outputs.len() as u64).sum();
    if let Some(tree_after) = slice.sapling_commitment_tree_size {
        let tree_after = u64::from(tree_after);
        if tree_after >= outputs_in_block {
            *next_position = tree_after - outputs_in_block;
        }
    }

    for tx in &slice.txs {
        let txid = txid_hex(&tx.txid_bytes);
        let spend_set: HashSet<[u8; 32]> = tx.spends_nf.iter().copied().collect();
        stats.notes_marked_spent +=
            mark_spent_by_nullifiers(notes, &spend_set, Some(txid.as_str()));

        for out in &tx.outputs {
            stats.outputs_seen += 1;
            let position = *next_position;
            *next_position = next_position.saturating_add(1);
            if let Some(discovered) =
                try_decrypt_sapling_compact_output(keys, out, height_u32, &txid, position)?
            {
                stats.notes_discovered += 1;
                merge_discovered_note(notes, discovered);
            }
        }
    }

    if let Some(tree_after) = slice.sapling_commitment_tree_size {
        *next_position = u64::from(tree_after);
    }

    Ok(stats)
}

/// Scan `[start_height, end_height]` from an LWD compact SQLite cache.
pub fn scan_sapling_from_compact_store(
    store: &LwdCompactStore,
    keys: &SaplingAccountKeys,
    start_height: u64,
    end_height: u64,
    start_position: u64,
) -> NozyResult<(Vec<SerializableSaplingNote>, SaplingScanStats)> {
    let mut notes = load_sapling_notes().unwrap_or_default();
    let mut next_position = start_position;
    let mut totals = SaplingScanStats {
        range_start: start_height,
        range_end: end_height,
        ..Default::default()
    };

    store
        .for_each_compact_block_range(start_height, end_height, |height, data| {
            let stats =
                scan_sapling_compact_block_blob(keys, height, data, &mut notes, &mut next_position)
                    .map_err(|e| zeaking::error::ZeakingError::InvalidOperation(e.to_string()))?;
            totals.blocks_scanned += stats.blocks_scanned;
            totals.outputs_seen += stats.outputs_seen;
            totals.notes_discovered += stats.notes_discovered;
            totals.notes_marked_spent += stats.notes_marked_spent;
            Ok(())
        })
        .map_err(|e| NozyError::InvalidOperation(format!("compact store iterate: {e}")))?;

    save_sapling_notes(&notes)?;
    save_sapling_scan_progress(&SaplingScanProgress {
        last_scanned_height: end_height,
        next_note_position: next_position,
    })?;
    Ok((notes, totals))
}

/// Convenience: derive account-0 keys from seed and scan the compact store.
pub fn scan_sapling_from_seed_and_store(
    seed: &[u8],
    store: &LwdCompactStore,
    start_height: u64,
    end_height: u64,
    start_position: u64,
) -> NozyResult<(Vec<SerializableSaplingNote>, SaplingScanStats)> {
    let keys = derive_sapling_account_keys(seed, 0, 0)?;
    scan_sapling_from_compact_store(store, &keys, start_height, end_height, start_position)
}

/// Incremental (or full) Sapling scan over whatever compact blocks are cached locally.
pub fn scan_sapling_wallet_from_compact_store(
    seed: &[u8],
    store: &LwdCompactStore,
    start_floor: Option<u64>,
    full_rescan: bool,
) -> NozyResult<(Vec<SerializableSaplingNote>, SaplingScanStats)> {
    let Some(end_height) = store
        .max_compact_height()
        .map_err(|e| NozyError::Storage(format!("compact max height: {e}")))?
    else {
        return Ok((
            load_sapling_notes().unwrap_or_default(),
            SaplingScanStats::default(),
        ));
    };
    let min_height = store
        .min_compact_height()
        .map_err(|e| NozyError::Storage(format!("compact min height: {e}")))?
        .unwrap_or(end_height);

    let progress = if full_rescan {
        SaplingScanProgress::default()
    } else {
        load_sapling_scan_progress().unwrap_or_default()
    };

    let mut start_height = start_floor.unwrap_or_else(|| {
        if progress.last_scanned_height > 0 {
            progress.last_scanned_height.saturating_add(1)
        } else {
            min_height
        }
    });
    if full_rescan {
        start_height = start_floor.unwrap_or(min_height);
    }
    start_height = start_height.max(min_height);

    if start_height > end_height {
        let notes = load_sapling_notes().unwrap_or_default();
        return Ok((
            notes,
            SaplingScanStats {
                range_start: start_height,
                range_end: end_height,
                ..Default::default()
            },
        ));
    }

    let start_position = if full_rescan || progress.last_scanned_height == 0 {
        0
    } else {
        progress.next_note_position
    };

    scan_sapling_from_seed_and_store(seed, store, start_height, end_height, start_position)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bip39::Mnemonic;
    use rand::rngs::OsRng;
    use sapling::note_encryption::{sapling_note_encryption, SaplingDomain};
    use sapling::util::generate_random_rseed;
    use sapling::value::NoteValue;
    use zcash_note_encryption::{Domain, COMPACT_NOTE_SIZE};

    const TEST_MNEMONIC: &str =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    fn test_keys() -> SaplingAccountKeys {
        let seed = Mnemonic::parse(TEST_MNEMONIC).unwrap().to_seed("");
        derive_sapling_account_keys(&seed, 0, 0).unwrap()
    }

    fn encrypt_compact(keys: &SaplingAccountKeys, value: u64) -> (SaplingCompactOutputBytes, u64) {
        let mut rng = OsRng;
        let note_value = NoteValue::from_raw(value);
        let rseed = generate_random_rseed(Zip212Enforcement::On, &mut rng);
        let note = keys.payment_address.create_note(note_value, rseed);
        let enc = sapling_note_encryption(None, note.clone(), [0u8; 512], &mut rng);
        let full_ct = enc.encrypt_note_plaintext();
        let mut compact = [0u8; COMPACT_NOTE_SIZE];
        compact.copy_from_slice(&full_ct[..COMPACT_NOTE_SIZE]);
        let epk_bytes = SaplingDomain::epk_bytes(enc.epk());
        (
            SaplingCompactOutputBytes {
                cmu: note.cmu().to_bytes(),
                ephemeral_key: epk_bytes.0,
                ciphertext: compact.to_vec(),
            },
            value,
        )
    }

    #[test]
    fn decrypts_compact_output_encrypted_to_wallet_address() {
        let keys = test_keys();
        let (out, value) = encrypt_compact(&keys, 12_345);
        let discovered = try_decrypt_sapling_compact_output(&keys, &out, 2_000_000, "deadbeef", 7)
            .unwrap()
            .expect("discovered");
        assert_eq!(discovered.value, value);
        assert_eq!(discovered.position, 7);
        assert!(!discovered.spent);
        assert_eq!(discovered.nullifier_bytes.len(), 32);
        assert!(sapling_note_has_rseed(&discovered));
        let rebuilt = reconstruct_sapling_note(&discovered).unwrap();
        assert_eq!(rebuilt.value().inner(), value);
        assert_eq!(rebuilt.cmu().to_bytes().as_slice(), out.cmu.as_slice());
    }

    #[test]
    fn foreign_output_does_not_decrypt() {
        let keys = test_keys();
        let other_seed = Mnemonic::parse(
            "legal winner thank year wave sausage worth useful legal winner thank yellow",
        )
        .unwrap()
        .to_seed("");
        let other = derive_sapling_account_keys(&other_seed, 0, 0).unwrap();
        let (out, _) = encrypt_compact(&other, 99);
        let discovered =
            try_decrypt_sapling_compact_output(&keys, &out, 2_000_000, "abcd", 1).unwrap();
        assert!(discovered.is_none());
    }

    #[test]
    fn marks_spent_by_nullifier() {
        let mut notes = vec![SerializableSaplingNote {
            value: 1,
            address_bytes: vec![1],
            nullifier_bytes: vec![9u8; 32],
            cmu_bytes: vec![2u8; 32],
            block_height: 1,
            txid: "a".into(),
            position: 0,
            spent: false,
            spent_in_txid: None,
            rseed_bytes: None,
        }];
        let mut set = HashSet::new();
        set.insert([9u8; 32]);
        assert_eq!(mark_spent_by_nullifiers(&mut notes, &set, Some("b")), 1);
        assert!(notes[0].spent);
        assert_eq!(notes[0].spent_in_txid.as_deref(), Some("b"));
    }
}
