use crate::error::{NozyError, NozyResult};
use crate::notes::SerializableOrchardNote;
use hex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::PathBuf;

/// JSON object keys must be strings; encode byte/nullifier and height map keys for v2 persistence.
mod json_index_keys {
    use super::*;

    pub mod nullifier_index {
        use super::*;

        pub fn serialize<S>(map: &HashMap<Vec<u8>, usize>, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let string_map: HashMap<String, usize> = map
                .iter()
                .map(|(k, v)| (hex::encode(k), *v))
                .collect();
            string_map.serialize(serializer)
        }

        pub fn deserialize<'de, D>(deserializer: D) -> Result<HashMap<Vec<u8>, usize>, D::Error>
        where
            D: Deserializer<'de>,
        {
            let string_map: HashMap<String, usize> = HashMap::deserialize(deserializer)?;
            string_map
                .into_iter()
                .map(|(k, v)| {
                    hex::decode(&k)
                        .map(|bytes| (bytes, v))
                        .map_err(serde::de::Error::custom)
                })
                .collect()
        }
    }

    pub mod height_index {
        use super::*;

        pub fn serialize<S>(map: &BTreeMap<u32, Vec<usize>>, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let string_map: BTreeMap<String, Vec<usize>> = map
                .iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect();
            string_map.serialize(serializer)
        }

        pub fn deserialize<'de, D>(deserializer: D) -> Result<BTreeMap<u32, Vec<usize>>, D::Error>
        where
            D: Deserializer<'de>,
        {
            let string_map: BTreeMap<String, Vec<usize>> = BTreeMap::deserialize(deserializer)?;
            string_map
                .into_iter()
                .map(|(k, v)| {
                    k.parse::<u32>()
                        .map(|height| (height, v))
                        .map_err(serde::de::Error::custom)
                })
                .collect()
        }
    }

    pub mod address_index {
        use super::*;

        pub fn serialize<S>(
            map: &HashMap<Vec<u8>, Vec<usize>>,
            serializer: S,
        ) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let string_map: HashMap<String, Vec<usize>> = map
                .iter()
                .map(|(k, v)| (hex::encode(k), v.clone()))
                .collect();
            string_map.serialize(serializer)
        }

        pub fn deserialize<'de, D>(
            deserializer: D,
        ) -> Result<HashMap<Vec<u8>, Vec<usize>>, D::Error>
        where
            D: Deserializer<'de>,
        {
            let string_map: HashMap<String, Vec<usize>> = HashMap::deserialize(deserializer)?;
            string_map
                .into_iter()
                .map(|(k, v)| {
                    hex::decode(&k)
                        .map(|bytes| (bytes, v))
                        .map_err(serde::de::Error::custom)
                })
                .collect()
        }
    }
}

/// Complete index structure for fast note lookups.
///
/// # Persistence Format
/// - Version 2: Complete structure (notes + all indexes)
/// - Version 1: Notes only (legacy, automatically migrated)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteIndex {
    #[serde(default = "default_index_version")]
    version: u32,
    notes: Vec<SerializableOrchardNote>,
    #[serde(with = "json_index_keys::nullifier_index")]
    nullifier_index: HashMap<Vec<u8>, usize>,
    #[serde(with = "json_index_keys::height_index")]
    height_index: BTreeMap<u32, Vec<usize>>,
    #[serde(with = "json_index_keys::address_index")]
    address_index: HashMap<Vec<u8>, Vec<usize>>,
}

fn default_index_version() -> u32 {
    2
}

impl NoteIndex {
    pub fn new() -> Self {
        Self {
            version: 2,
            notes: Vec::new(),
            nullifier_index: HashMap::new(),
            height_index: BTreeMap::new(),
            address_index: HashMap::new(),
        }
    }

    pub fn from_notes(notes: Vec<SerializableOrchardNote>) -> Self {
        let mut index = Self::new();
        for note in notes {
            index.add_note(note);
        }
        index
    }

    pub fn add_note(&mut self, note: SerializableOrchardNote) {
        if self.nullifier_index.contains_key(&note.nullifier_bytes) {
            return;
        }

        let idx = self.notes.len();
        self.notes.push(note.clone());

        self.nullifier_index
            .insert(note.nullifier_bytes.clone(), idx);

        self.height_index
            .entry(note.block_height)
            .or_insert_with(Vec::new)
            .push(idx);

        self.address_index
            .entry(note.address_bytes.clone())
            .or_insert_with(Vec::new)
            .push(idx);
    }

    pub fn get_note_by_nullifier(&self, nullifier: &[u8]) -> Option<&SerializableOrchardNote> {
        self.nullifier_index
            .get(nullifier)
            .and_then(|&idx| self.notes.get(idx))
    }

    /// Get notes by height range in deterministic order.
    ///
    /// # Determinism Guarantees
    /// - Notes are returned in ascending height order
    /// - Within the same height, notes are ordered by their insertion index
    pub fn get_notes_by_height_range(&self, start: u32, end: u32) -> Vec<&SerializableOrchardNote> {
        self.height_index
            .range(start..=end)
            .flat_map(|(_, indices)| {
                // Sort indices within each height for deterministic ordering
                let mut sorted_indices = indices.clone();
                sorted_indices.sort();
                sorted_indices.into_iter()
            })
            .filter_map(|idx| self.notes.get(idx))
            .collect()
    }

    pub fn get_notes_by_address(&self, address_bytes: &[u8]) -> Vec<&SerializableOrchardNote> {
        self.address_index
            .get(address_bytes)
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|idx| self.notes.get(*idx))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get unspent notes as an iterator (more efficient than collecting to Vec).
    ///
    /// Returns notes in the order they were added to the index.
    pub fn get_unspent_notes(&self) -> impl Iterator<Item = &SerializableOrchardNote> {
        self.notes.iter().filter(|note| !note.spent)
    }

    /// Get unspent notes as a vector (for cases where you need a Vec).
    pub fn get_unspent_notes_vec(&self) -> Vec<&SerializableOrchardNote> {
        self.get_unspent_notes().collect()
    }

    pub fn get_all_notes(&self) -> &[SerializableOrchardNote] {
        &self.notes
    }

    pub fn mark_note_spent(&mut self, nullifier: &[u8]) -> bool {
        if let Some(&idx) = self.nullifier_index.get(nullifier) {
            if let Some(note) = self.notes.get_mut(idx) {
                note.spent = true;
                return true;
            }
        }
        false
    }

    /// Match an on-chain action nullifier against held notes (direct index or canonical recompute).
    pub fn mark_note_spent_on_chain(
        &mut self,
        chain_nullifier: &[u8],
        fvk: &orchard::keys::FullViewingKey,
    ) -> bool {
        if chain_nullifier.len() != 32 {
            return false;
        }
        if self.mark_note_spent(chain_nullifier) {
            return true;
        }

        let mut found_idx = None;
        for (idx, note) in self.notes.iter().enumerate() {
            if note.spent {
                continue;
            }
            if let Some(computed) = note.canonical_nullifier_bytes(fvk) {
                if computed.as_slice() == chain_nullifier {
                    found_idx = Some(idx);
                    break;
                }
            }
        }

        if let Some(idx) = found_idx {
            if let Some(note) = self.notes.get_mut(idx) {
                note.spent = true;
                if let Some(canonical) = note.canonical_nullifier_bytes(fvk) {
                    let old = note.nullifier_bytes.clone();
                    if old != canonical.to_vec() {
                        self.nullifier_index.remove(&old);
                        note.nullifier_bytes = canonical.to_vec();
                        self.nullifier_index
                            .insert(note.nullifier_bytes.clone(), idx);
                    }
                }
                return true;
            }
        }

        false
    }

    /// Refresh serialized Orchard incremental witnesses after sync (see [`crate::orchard_witness::OrchardWitnessTracker`]).
    pub fn apply_orchard_witnesses_from_tracker(
        &mut self,
        tracker: &crate::orchard_witness::OrchardWitnessTracker,
        tip_height: u32,
    ) -> crate::error::NozyResult<()> {
        for note in &mut self.notes {
            if note.nullifier_bytes.len() != 32 {
                continue;
            }
            let mut nf = [0u8; 32];
            nf.copy_from_slice(&note.nullifier_bytes);
            if let Some(bytes) = tracker.serialized_witness_for_nullifier(&nf)? {
                note.orchard_incremental_witness_hex = Some(hex::encode(bytes));
                note.orchard_witness_tip_height = Some(tip_height);
            }
        }
        Ok(())
    }

    /// Save the complete index structure to file.
    ///
    /// Saves both notes and all indexes for fast loading without rebuild.
    /// Uses version 2 format (complete structure).
    pub fn save_to_file(&self, path: &PathBuf) -> NozyResult<()> {
        // Validate indexes before saving to catch any inconsistencies
        self.validate_indexes().map_err(|e| {
            NozyError::Storage(format!("Index validation failed before save: {}", e))
        })?;

        // Save complete structure (version 2 format)
        let serialized = serde_json::to_string_pretty(self)
            .map_err(|e| NozyError::Storage(format!("Failed to serialize index: {}", e)))?;

        // Atomic write: write to temp file first, then rename
        let temp_path = path.with_extension("tmp");
        fs::write(&temp_path, serialized)
            .map_err(|e| NozyError::Storage(format!("Failed to write index: {}", e)))?;

        fs::rename(&temp_path, path)
            .map_err(|e| NozyError::Storage(format!("Failed to rename temp file: {}", e)))?;

        Ok(())
    }

    /// Load index from file with automatic format detection and migration.
    ///
    /// # Format Support
    /// - Version 2: Complete structure (notes + indexes) - loads directly
    /// - Version 1/Legacy: Notes only - automatically migrates to version 2
    ///
    /// # Migration
    /// If a legacy format is detected, indexes are rebuilt and the file is
    /// automatically upgraded to version 2 format on next save.
    pub fn load_from_file(path: &PathBuf) -> NozyResult<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(path)
            .map_err(|e| NozyError::Storage(format!("Failed to read index: {}", e)))?;

        // Try to load as complete structure (version 2)
        match serde_json::from_str::<NoteIndex>(&content) {
            Ok(mut index) => {
                // Validate loaded indexes
                if let Err(e) = index.validate_indexes() {
                    // If validation fails, rebuild indexes from notes
                    eprintln!(
                        "Warning: Index validation failed, rebuilding indexes: {}",
                        e
                    );
                    let notes = index.notes.clone();
                    index = Self::from_notes(notes);
                }
                Ok(index)
            }
            Err(_) => {
                // Fall back to legacy format (notes only)
                match serde_json::from_str::<Vec<SerializableOrchardNote>>(&content) {
                    Ok(notes) => {
                        println!("Migrating legacy index format to version 2...");
                        let index = Self::from_notes(notes);
                        // Save in new format for next time
                        if let Err(e) = index.save_to_file(path) {
                            eprintln!("Warning: Failed to save migrated index: {}", e);
                        }
                        Ok(index)
                    }
                    Err(e) => Err(NozyError::Storage(format!(
                        "Failed to parse index (tried both v2 and legacy formats): {}",
                        e
                    ))),
                }
            }
        }
    }

    pub fn total_balance(&self) -> u64 {
        self.notes
            .iter()
            .filter(|note| !note.spent)
            .map(|note| note.value)
            .sum()
    }

    pub fn count(&self) -> usize {
        self.notes.len()
    }

    pub fn unspent_count(&self) -> usize {
        self.notes.iter().filter(|note| !note.spent).count()
    }

    /// Validate that all indexes are consistent with the notes vector.
    ///
    /// # Returns
    /// - `Ok(())` if all indexes are consistent
    /// - `Err` with description of first inconsistency found
    ///
    /// # Checks Performed
    /// - All nullifier_index entries point to valid note indices
    /// - All notes have corresponding nullifier_index entries
    /// - All height_index entries point to valid note indices
    /// - All address_index entries point to valid note indices
    /// - No duplicate nullifiers in notes
    pub fn validate_indexes(&self) -> NozyResult<()> {
        // Check nullifier_index consistency
        for (nullifier, &idx) in &self.nullifier_index {
            if idx >= self.notes.len() {
                return Err(NozyError::Storage(format!(
                    "Nullifier index points to invalid note index: {} >= {}",
                    idx,
                    self.notes.len()
                )));
            }
            let note = &self.notes[idx];
            if note.nullifier_bytes != *nullifier {
                return Err(NozyError::Storage(format!(
                    "Nullifier index mismatch: expected {:?}, found {:?}",
                    hex::encode(nullifier),
                    hex::encode(&note.nullifier_bytes)
                )));
            }
        }

        // Check that all notes have nullifier_index entries
        for (idx, note) in self.notes.iter().enumerate() {
            if self.nullifier_index.get(&note.nullifier_bytes) != Some(&idx) {
                return Err(NozyError::Storage(format!(
                    "Note at index {} missing from nullifier_index",
                    idx
                )));
            }
        }

        // Check height_index consistency
        for (height, indices) in &self.height_index {
            for &idx in indices {
                if idx >= self.notes.len() {
                    return Err(NozyError::Storage(format!(
                        "Height index at height {} points to invalid note index: {} >= {}",
                        height,
                        idx,
                        self.notes.len()
                    )));
                }
                if self.notes[idx].block_height != *height {
                    return Err(NozyError::Storage(format!(
                        "Height index mismatch: expected height {}, found {}",
                        height, self.notes[idx].block_height
                    )));
                }
            }
        }

        // Check address_index consistency
        for (address, indices) in &self.address_index {
            for &idx in indices {
                if idx >= self.notes.len() {
                    return Err(NozyError::Storage(format!(
                        "Address index points to invalid note index: {} >= {}",
                        idx,
                        self.notes.len()
                    )));
                }
                if self.notes[idx].address_bytes != *address {
                    return Err(NozyError::Storage(format!(
                        "Address index mismatch: expected {:?}, found {:?}",
                        hex::encode(address),
                        hex::encode(&self.notes[idx].address_bytes)
                    )));
                }
            }
        }

        // Check for duplicate nullifiers (shouldn't happen due to deduplication)
        let mut seen_nullifiers = std::collections::HashSet::new();
        for note in &self.notes {
            if !seen_nullifiers.insert(&note.nullifier_bytes) {
                return Err(NozyError::Storage(format!(
                    "Duplicate nullifier found: {:?}",
                    hex::encode(&note.nullifier_bytes)
                )));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_note(value: u64, height: u32, nullifier_byte: u8) -> SerializableOrchardNote {
        SerializableOrchardNote {
            note_bytes: vec![0u8; 180],
            value,
            address_bytes: vec![2u8; 43],
            nullifier_bytes: vec![nullifier_byte; 32],
            block_height: height,
            txid: "abc".into(),
            spent: false,
            memo: vec![],
            orchard_incremental_witness_hex: None,
            orchard_witness_tip_height: None,
            rho_bytes: None,
            rseed_bytes: None,
        }
    }

    #[test]
    fn note_index_v2_json_roundtrip_with_notes() {
        let index = NoteIndex::from_notes(vec![
            sample_note(250_000, 3_379_045, 1),
            sample_note(100_000, 3_379_050, 2),
        ]);

        let json = serde_json::to_string_pretty(&index).expect("serialize v2 index");
        assert!(
            json.contains("\"3379045\""),
            "height keys must serialize as JSON strings"
        );

        let loaded: NoteIndex =
            serde_json::from_str(&json).expect("deserialize v2 index with string map keys");
        loaded.validate_indexes().expect("loaded indexes valid");
        assert_eq!(loaded.count(), 2);
        assert_eq!(loaded.total_balance(), 350_000);
    }

    #[test]
    fn note_index_save_and_load_file_roundtrip() {
        let path = std::env::temp_dir().join(format!(
            "nozy_note_index_test_{}.json",
            std::process::id()
        ));
        let index = NoteIndex::from_notes(vec![sample_note(250_000, 3_379_045, 9)]);

        index
            .save_to_file(&path.to_path_buf())
            .expect("save index");

        let loaded = NoteIndex::load_from_file(&path.to_path_buf()).expect("load index");
        assert_eq!(loaded.unspent_count(), 1);
        assert_eq!(loaded.total_balance(), 250_000);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn note_index_migrates_legacy_array_format() {
        let path = std::env::temp_dir().join(format!(
            "nozy_note_index_legacy_test_{}.json",
            std::process::id()
        ));
        let notes = vec![sample_note(250_000, 3_379_045, 7)];
        let legacy = serde_json::to_string_pretty(&notes).expect("legacy array json");
        std::fs::write(&path, legacy).expect("write legacy file");

        let loaded = NoteIndex::load_from_file(&path.to_path_buf()).expect("load legacy");
        assert_eq!(loaded.total_balance(), 250_000);
        let migrated = std::fs::read_to_string(&path).expect("read migrated file");
        assert!(
            migrated.contains("nullifier_index"),
            "migration should rewrite as v2 index"
        );

        let _ = std::fs::remove_file(&path);
    }
}
