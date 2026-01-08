use crate::error::{NozyError, NozyResult};
use crate::notes::{OrchardNote, SpendableNote};
use crate::block_parser::{ParsedTransaction, OrchardAction};
use crate::zcash_keys::{ZcashSpendingKey, ZcashAddressType};
use crate::note_parser::{ParsedNoteData, TransactionOutput};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use sha2::{Sha256, Digest};

#[derive(Debug, Clone)]
pub struct NoteStorage {
    storage_path: String,
    notes: Arc<Mutex<HashMap<String, StoredNote>>>,
    spending_keys: Arc<Mutex<HashMap<String, StoredSpendingKey>>>,
    transactions: Arc<Mutex<HashMap<String, StoredTransaction>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredNote {
    pub id: String,
    pub note: OrchardNote,
    pub spending_key_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_confirmed: bool,
    pub confirmation_height: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredSpendingKey {
    pub id: String,
    pub spending_key: ZcashSpendingKey,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub note_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredTransaction {
    pub id: String,
    pub transaction: ParsedTransaction,
    pub block_height: u32,
    pub block_time: DateTime<Utc>,
    pub stored_at: DateTime<Utc>,
    pub is_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub total_notes: usize,
    pub unspent_notes: usize,
    pub total_spending_keys: usize,
    pub total_transactions: usize,
    pub total_storage_size: u64,
}

impl NoteStorage {
    pub fn new(storage_path: String) -> NozyResult<Self> {
        let storage = Self {
            storage_path: storage_path.clone(),
            notes: Arc::new(Mutex::new(HashMap::new())),
            spending_keys: Arc::new(Mutex::new(HashMap::new())),
            transactions: Arc::new(Mutex::new(HashMap::new())),
        };
        
        storage.ensure_storage_directory()?;
        storage.load_stored_data()?;
        
        Ok(storage)
    }

    fn ensure_storage_directory(&self) -> NozyResult<()> {
        let path = Path::new(&self.storage_path);
        if !path.exists() {
            fs::create_dir_all(path)
                .map_err(|e| NozyError::Storage(format!("Failed to create storage directory: {}", e)))?;
        }
        Ok(())
    }

    fn load_stored_data(&self) -> NozyResult<()> {
        self.load_notes()?;
        self.load_spending_keys()?;
        self.load_transactions()?;
        Ok(())
    }

    fn load_notes(&self) -> NozyResult<()> {
        let notes_path = format!("{}/notes.json", self.storage_path);
        let path = Path::new(&notes_path);
        
        if path.exists() {
            let content = fs::read_to_string(path)
                .map_err(|e| NozyError::Storage(format!("Failed to read notes: {}", e)))?;
            
            let stored_notes: HashMap<String, StoredNote> = serde_json::from_str(&content)
                .map_err(|e| NozyError::Storage(format!("Failed to parse notes: {}", e)))?;
            
            *self.notes.lock()
                .map_err(|e| NozyError::Storage(format!("Mutex poisoned: {}", e)))? = stored_notes;
        }
        
        Ok(())
    }

    fn load_spending_keys(&self) -> NozyResult<()> {
        let keys_path = format!("{}/spending_keys.json", self.storage_path);
        let path = Path::new(&keys_path);
        
        if path.exists() {
            let content = fs::read_to_string(path)
                .map_err(|e| NozyError::Storage(format!("Failed to read spending keys: {}", e)))?;
            
            let stored_keys: HashMap<String, StoredSpendingKey> = serde_json::from_str(&content)
                .map_err(|e| NozyError::Storage(format!("Failed to parse spending keys: {}", e)))?;
            
            *self.spending_keys.lock()
                .map_err(|e| NozyError::Storage(format!("Mutex poisoned: {}", e)))? = stored_keys;
        }
        
        Ok(())
    }

    fn load_transactions(&self) -> NozyResult<()> {
        let txs_path = format!("{}/transactions.json", self.storage_path);
        let path = Path::new(&txs_path);
        
        if path.exists() {
            let content = fs::read_to_string(path)
                .map_err(|e| NozyError::Storage(format!("Failed to read transactions: {}", e)))?;
            
            let stored_txs: HashMap<String, StoredTransaction> = serde_json::from_str(&content)
                .map_err(|e| NozyError::Storage(format!("Failed to parse transactions: {}", e)))?;
            
            *self.transactions.lock()
                .map_err(|e| NozyError::Storage(format!("Mutex poisoned: {}", e)))? = stored_txs;
        }
        
        Ok(())
    }

    pub fn save_all(&self) -> NozyResult<()> {
        self.save_notes()?;
        self.save_spending_keys()?;
        self.save_transactions()?;
        Ok(())
    }

    fn save_notes(&self) -> NozyResult<()> {
        let notes_path = format!("{}/notes.json", self.storage_path);
        let notes = self.notes.lock()
            .map_err(|e| NozyError::Storage(format!("Mutex poisoned: {}", e)))?;
        let content = serde_json::to_string_pretty(&*notes)
            .map_err(|e| NozyError::Storage(format!("Failed to serialize notes: {}", e)))?;
        
        fs::write(&notes_path, content)
            .map_err(|e| NozyError::Storage(format!("Failed to write notes: {}", e)))?;
        
        Ok(())
    }

    fn save_spending_keys(&self) -> NozyResult<()> {
        let keys_path = format!("{}/spending_keys.json", self.storage_path);
        let keys = self.spending_keys.lock()
            .map_err(|e| NozyError::Storage(format!("Mutex poisoned: {}", e)))?;
        let content = serde_json::to_string_pretty(&*keys)
            .map_err(|e| NozyError::Storage(format!("Failed to serialize spending keys: {}", e)))?;
        
        fs::write(&keys_path, content)
            .map_err(|e| NozyError::Storage(format!("Failed to write spending keys: {}", e)))?;
        
        Ok(())
    }

    fn save_transactions(&self) -> NozyResult<()> {
        let txs_path = format!("{}/transactions.json", self.storage_path);
        let txs = self.transactions.lock()
            .map_err(|e| NozyError::Storage(format!("Mutex poisoned: {}", e)))?;
        let content = serde_json::to_string_pretty(&*txs)
            .map_err(|e| NozyError::Storage(format!("Failed to serialize transactions: {}", e)))?;
        
        fs::write(&txs_path, content)
            .map_err(|e| NozyError::Storage(format!("Failed to write transactions: {}", e)))?;
        
        Ok(())
    }

    pub fn store_note(&self, note: OrchardNote, spending_key_id: &str) -> NozyResult<()> {
        let note_id = self.generate_note_id(&note);
        let now = Utc::now();
        
        let stored_note = StoredNote {
            id: note_id.clone(),
            note,
            spending_key_id: spending_key_id.to_string(),
            created_at: now,
            updated_at: now,
            is_confirmed: false,
            confirmation_height: None,
        };
        
        self.notes.lock()
            .map_err(|e| NozyError::Storage(format!("Mutex poisoned: {}", e)))?
            .insert(note_id, stored_note);
        self.update_spending_key_note_count(spending_key_id)?;
        self.save_notes()?;
        
        Ok(())
    }

    pub fn store_spending_key(&self, spending_key: ZcashSpendingKey) -> NozyResult<()> {
        let key_id = self.generate_spending_key_id(&spending_key);
        let now = Utc::now();
        
        let stored_key = StoredSpendingKey {
            id: key_id.clone(),
            spending_key,
            created_at: now,
            last_used: None,
            note_count: 0,
        };
        
        self.spending_keys.lock()
            .map_err(|e| NozyError::Storage(format!("Mutex poisoned: {}", e)))?
            .insert(key_id, stored_key);
        self.save_spending_keys()?;
        
        Ok(())
    }

    pub fn store_transaction(&self, transaction: ParsedTransaction, block_height: u32, block_time: DateTime<Utc>) -> NozyResult<()> {
        let tx_id = transaction.txid.clone();
        let now = Utc::now();
        
        let stored_tx = StoredTransaction {
            id: tx_id.clone(),
            transaction,
            block_height,
            block_time,
            stored_at: now,
            is_confirmed: true,
        };
        
        self.transactions.lock()
            .map_err(|e| NozyError::Storage(format!("Mutex poisoned: {}", e)))?
            .insert(tx_id, stored_tx);
        self.save_transactions()?;
        
        Ok(())
    }

    pub fn get_all_notes(&self) -> Vec<StoredNote> {
        self.notes.lock()
            .map_err(|e| {
                eprintln!("Warning: Mutex poisoned in get_all_notes: {}", e);
            })
            .ok()
            .map(|notes| notes.values().cloned().collect())
            .unwrap_or_default()
    }

    pub fn get_notes_for_address(&self, address: &str) -> Vec<StoredNote> {
        self.notes.lock()
            .map_err(|e| {
                eprintln!("Warning: Mutex poisoned in get_notes_for_address: {}", e);
            })
            .ok()
            .map(|notes| notes
                .values()
                .filter(|note| note.note.recipient == address)
                .cloned()
                .collect())
            .unwrap_or_default()
    }

    pub fn get_unspent_notes(&self) -> Vec<StoredNote> {
        self.notes.lock()
            .map_err(|e| {
                eprintln!("Warning: Mutex poisoned in get_unspent_notes: {}", e);
            })
            .ok()
            .map(|notes| notes
                .values()
                .filter(|note| !note.note.spent)
                .cloned()
                .collect())
            .unwrap_or_default()
    }

    pub fn get_spending_keys_for_address(&self, address: &str) -> Vec<StoredSpendingKey> {
        self.spending_keys.lock()
            .map_err(|e| {
                eprintln!("Warning: Mutex poisoned in get_spending_keys_for_address: {}", e);
            })
            .ok()
            .map(|keys| keys
                .values()
                .filter(|key| key.spending_key.address == address)
                .cloned()
                .collect())
            .unwrap_or_default()
    }

    pub fn mark_note_spent(&self, note_id: &str) -> NozyResult<()> {
        let mut notes = self.notes.lock()
            .map_err(|e| NozyError::Storage(format!("Mutex poisoned: {}", e)))?;
        if let Some(note) = notes.get_mut(note_id) {
            note.note.spent = true;
            note.updated_at = Utc::now();
            drop(notes); // Release lock before saving
            self.save_notes()?;
        }
        Ok(())
    }

    pub fn update_note_confirmation(&self, note_id: &str, is_confirmed: bool, confirmation_height: Option<u32>) -> NozyResult<()> {
        let mut notes = self.notes.lock()
            .map_err(|e| NozyError::Storage(format!("Mutex poisoned: {}", e)))?;
        if let Some(note) = notes.get_mut(note_id) {
            note.is_confirmed = is_confirmed;
            note.confirmation_height = confirmation_height;
            note.updated_at = Utc::now();
            drop(notes); // Release lock before saving
            self.save_notes()?;
        }
        Ok(())
    }

    pub fn get_stats(&self) -> StorageStats {
        let notes = self.notes.lock()
            .map_err(|e| {
                eprintln!("Warning: Mutex poisoned in get_stats (notes): {}", e);
            })
            .ok();
        let spending_keys = self.spending_keys.lock()
            .map_err(|e| {
                eprintln!("Warning: Mutex poisoned in get_stats (spending_keys): {}", e);
            })
            .ok();
        let transactions = self.transactions.lock()
            .map_err(|e| {
                eprintln!("Warning: Mutex poisoned in get_stats (transactions): {}", e);
            })
            .ok();
        
        let unspent_notes = notes.as_ref()
            .map(|n| n.values().filter(|note| !note.note.spent).count())
            .unwrap_or(0);
        
        StorageStats {
            total_notes: notes.as_ref().map(|n| n.len()).unwrap_or(0),
            unspent_notes,
            total_spending_keys: spending_keys.as_ref().map(|k| k.len()).unwrap_or(0),
            total_transactions: transactions.as_ref().map(|t| t.len()).unwrap_or(0),
            total_storage_size: 0, // Would calculate actual file sizes
        }
    }

    fn generate_note_id(&self, note: &OrchardNote) -> String {
        let mut hasher = Sha256::new();
        hasher.update(&note.commitment);
        hasher.update(&note.txid);
        hasher.update(&note.value.to_le_bytes());
        hex::encode(&hasher.finalize()[..8])
    }

    fn generate_spending_key_id(&self, key: &ZcashSpendingKey) -> String {
        let mut hasher = Sha256::new();
        hasher.update(&key.private_key);
        hasher.update(&key.address);
        hex::encode(&hasher.finalize()[..8])
    }

    fn update_spending_key_note_count(&self, key_id: &str) -> NozyResult<()> {
        let mut keys = self.spending_keys.lock()
            .map_err(|e| NozyError::Storage(format!("Mutex poisoned: {}", e)))?;
        if let Some(key) = keys.get_mut(key_id) {
            key.note_count += 1;
            key.last_used = Some(Utc::now());
        }
        Ok(())
    }

    pub fn clear_all(&self) -> NozyResult<()> {
        self.notes.lock()
            .map_err(|e| NozyError::Storage(format!("Mutex poisoned: {}", e)))?
            .clear();
        self.spending_keys.lock()
            .map_err(|e| NozyError::Storage(format!("Mutex poisoned: {}", e)))?
            .clear();
        self.transactions.lock()
            .map_err(|e| NozyError::Storage(format!("Mutex poisoned: {}", e)))?
            .clear();
        
        self.save_all()?;
        Ok(())
    }
} 