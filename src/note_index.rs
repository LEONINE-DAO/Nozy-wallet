use crate::error::{NozyError, NozyResult};
use crate::notes::SerializableOrchardNote;
use std::collections::{HashMap, BTreeMap};
use std::fs;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteIndex {
    notes: Vec<SerializableOrchardNote>,
    nullifier_index: HashMap<Vec<u8>, usize>,
    height_index: BTreeMap<u32, Vec<usize>>,
    address_index: HashMap<Vec<u8>, Vec<usize>>,
}

impl NoteIndex {
    pub fn new() -> Self {
        Self {
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

        self.nullifier_index.insert(note.nullifier_bytes.clone(), idx);

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

    pub fn get_notes_by_height_range(&self, start: u32, end: u32) -> Vec<&SerializableOrchardNote> {
        self.height_index
            .range(start..=end)
            .flat_map(|(_, indices)| indices.iter())
            .filter_map(|&idx| self.notes.get(idx))
            .collect()
    }

    pub fn get_notes_by_address(&self, address_bytes: &[u8]) -> Vec<&SerializableOrchardNote> {
        self.address_index
            .get(address_bytes)
            .map(|indices| {
                indices.iter()
                    .filter_map(|&idx| self.notes.get(idx))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_unspent_notes(&self) -> Vec<&SerializableOrchardNote> {
        self.notes.iter()
            .filter(|note| !note.spent)
            .collect()
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

    pub fn save_to_file(&self, path: &PathBuf) -> NozyResult<()> {
        let serialized = serde_json::to_string_pretty(&self.notes)
            .map_err(|e| NozyError::Storage(format!("Failed to serialize notes: {}", e)))?;
        
        fs::write(path, serialized)
            .map_err(|e| NozyError::Storage(format!("Failed to write notes: {}", e)))?;
        
        Ok(())
    }

    pub fn load_from_file(path: &PathBuf) -> NozyResult<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(path)
            .map_err(|e| NozyError::Storage(format!("Failed to read notes: {}", e)))?;

        let notes: Vec<SerializableOrchardNote> = serde_json::from_str(&content)
            .map_err(|e| NozyError::Storage(format!("Failed to parse notes: {}", e)))?;

        Ok(Self::from_notes(notes))
    }

    pub fn total_balance(&self) -> u64 {
        self.notes.iter()
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
}

