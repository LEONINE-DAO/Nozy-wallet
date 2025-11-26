use crate::error::{NozyError, NozyResult};
use crate::paths::get_wallet_data_dir;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use crate::notes::OrchardNote;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};


#[derive(Debug, Clone)]
pub struct NoteForHistory {
    pub id: String,
    pub note: OrchardNote,
    pub created_at: DateTime<Utc>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentTransactionRecord {
   
    pub txid: String,
    
   
    pub recipient_address: String,
    
   
    pub amount_zatoshis: u64,
    
    
    pub fee_zatoshis: u64,
    
  
    pub memo: Option<Vec<u8>>,
    
    
    pub created_at: DateTime<Utc>,
    
    
    pub broadcast_at: Option<DateTime<Utc>>,
    
    
    pub status: TransactionStatus,
    
    pub block_height: Option<u32>,
    
    
    pub block_time: Option<DateTime<Utc>>,
    
    
    pub confirmations: u32,
    
    
    pub spent_note_ids: Vec<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionView {
    pub txid: String,
    
    pub transaction_type: TransactionType,
    
    pub net_amount_zatoshis: i64,
    
    pub fee_zatoshis: Option<u64>,
    
    pub recipient_address: Option<String>,
    
    pub my_addresses: Vec<String>,
    
    pub block_height: Option<u32>,
    
    pub block_time: Option<DateTime<Utc>>,
    
    pub confirmations: u32,
    
    pub status: TransactionStatus,
    
    pub memo: Option<String>,
    
    pub notes_involved: Vec<String>,
    
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransactionType {
    Sent,
    
    Received,
    
    Change,
    
    Mixed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransactionStatus {
    Pending,
    
    Confirmed,
    
    Failed,
}

impl TransactionStatus {
    pub fn is_confirmed(&self) -> bool {
        matches!(self, TransactionStatus::Confirmed)
    }
    
    pub fn is_pending(&self) -> bool {
        matches!(self, TransactionStatus::Pending)
    }
}

impl TransactionType {
    pub fn label(&self) -> &'static str {
        match self {
            TransactionType::Sent => "Sent",
            TransactionType::Received => "Received",
            TransactionType::Change => "Change",
            TransactionType::Mixed => "Mixed",
        }
    }
}

impl SentTransactionRecord {
    pub fn new(
        txid: String,
        recipient_address: String,
        amount_zatoshis: u64,
        fee_zatoshis: u64,
        memo: Option<Vec<u8>>,
        spent_note_ids: Vec<String>,
    ) -> Self {
        Self {
            txid,
            recipient_address,
            amount_zatoshis,
            fee_zatoshis,
            memo,
            created_at: Utc::now(),
            broadcast_at: None,
            status: TransactionStatus::Pending,
            block_height: None,
            block_time: None,
            confirmations: 0,
            spent_note_ids,
        }
    }
    
    pub fn mark_broadcast(&mut self) {
        self.broadcast_at = Some(Utc::now());
    }
    
    pub fn mark_confirmed(&mut self, block_height: u32, block_time: DateTime<Utc>, current_height: u32) {
        self.status = TransactionStatus::Confirmed;
        self.block_height = Some(block_height);
        self.block_time = Some(block_time);
        self.confirmations = current_height.saturating_sub(block_height) + 1;
    }
    
    pub fn mark_failed(&mut self) {
        self.status = TransactionStatus::Failed;
    }
}

impl TransactionView {
    pub fn from_received_notes(notes: &[&NoteForHistory], current_height: u32) -> Option<Self> {
        if notes.is_empty() {
            return None;
        }
        
        let first_note = notes[0];
        let txid = first_note.note.txid.clone();
        
        let total_received: u64 = notes.iter()
            .map(|n| n.note.value)
            .sum();
        
        let block_height = Some(first_note.note.block_height);
        let block_time = first_note.created_at;
        
        let memo = notes.iter()
            .find_map(|n| {
                if !n.note.memo.is_empty() {
                    String::from_utf8(n.note.memo.clone()).ok()
                } else {
                    None
                }
            });
        
        
        let my_addresses: Vec<String> = notes.iter()
            .map(|n| {
                format!("{:?}", n.note.address)
            })
            .collect();
        
        let confirmations = if let Some(height) = block_height {
            current_height.saturating_sub(height) + 1
        } else {
            0
        };
        
        Some(Self {
            txid,
            transaction_type: TransactionType::Received,
            net_amount_zatoshis: total_received as i64,
            fee_zatoshis: None,
            recipient_address: None,
            my_addresses,
            block_height,
            block_time: Some(block_time),
            confirmations,
            status: TransactionStatus::Confirmed,
            memo,
            notes_involved: notes.iter().map(|n| n.id.clone()).collect(),
            created_at: first_note.created_at,
        })
    }
    
    pub fn from_sent_record(record: &SentTransactionRecord) -> Self {
        Self {
            txid: record.txid.clone(),
            transaction_type: TransactionType::Sent,
            net_amount_zatoshis: -(record.amount_zatoshis as i64),
            fee_zatoshis: Some(record.fee_zatoshis),
            recipient_address: Some(record.recipient_address.clone()),
            my_addresses: vec![], 
            block_height: record.block_height,
            block_time: record.block_time,
            confirmations: record.confirmations,
            status: record.status.clone(),
            memo: record.memo.as_ref()
                .and_then(|m| String::from_utf8(m.clone()).ok()),
            notes_involved: record.spent_note_ids.clone(),
            created_at: record.created_at,
        }
    }
    
    pub fn merge_with_received(&mut self, received_amount: u64) {
        self.transaction_type = TransactionType::Mixed;
        self.net_amount_zatoshis += received_amount as i64;
    }
    
    pub fn amount_zec(&self) -> f64 {
        (self.net_amount_zatoshis.abs() as f64) / 100_000_000.0
    }
    
    pub fn fee_zec(&self) -> Option<f64> {
        self.fee_zatoshis.map(|f| (f as f64) / 100_000_000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_transaction_status() {
        let status = TransactionStatus::Pending;
        assert!(status.is_pending());
        assert!(!status.is_confirmed());
        
        let status = TransactionStatus::Confirmed;
        assert!(!status.is_pending());
        assert!(status.is_confirmed());
    }
    
    #[test]
    fn test_transaction_type_label() {
        assert_eq!(TransactionType::Sent.label(), "Sent");
        assert_eq!(TransactionType::Received.label(), "Received");
        assert_eq!(TransactionType::Change.label(), "Change");
        assert_eq!(TransactionType::Mixed.label(), "Mixed");
    }
    
    #[test]
    fn test_sent_transaction_record() {
        let mut record = SentTransactionRecord::new(
            "abc123".to_string(),
            "u1test".to_string(),
            100_000_000, 
            10_000,      
            None,
            vec!["note1".to_string()],
        );
        
        assert_eq!(record.status, TransactionStatus::Pending);
        assert_eq!(record.confirmations, 0);
        
        record.mark_broadcast();
        assert!(record.broadcast_at.is_some());
        
        let block_time = Utc::now();
        record.mark_confirmed(1000, block_time, 1010);
        assert_eq!(record.status, TransactionStatus::Confirmed);
        assert_eq!(record.confirmations, 11);
    }
    
    #[test]
    fn test_transaction_view_amounts() {
        let record = SentTransactionRecord::new(
            "abc123".to_string(),
            "u1test".to_string(),
            100_000_000, 
            10_000,     
            None,
            vec![],
        );
        
        let view = TransactionView::from_sent_record(&record);
        assert_eq!(view.amount_zec(), 1.0);
        assert_eq!(view.fee_zec(), Some(0.0001));
        assert_eq!(view.net_amount_zatoshis, -100_000_000);
    }
}

/// Storage manager for sent transactions
pub struct SentTransactionStorage {
    storage_path: std::path::PathBuf,
    transactions: Arc<Mutex<HashMap<String, SentTransactionRecord>>>,
}

impl SentTransactionStorage {
    /// Create a new storage instance using the wallet data directory
    pub fn new() -> NozyResult<Self> {
        let data_dir = get_wallet_data_dir();
        let storage = Self {
            storage_path: data_dir.clone(),
            transactions: Arc::new(Mutex::new(HashMap::new())),
        };
        
        storage.ensure_storage_directory()?;
        storage.load_transactions()?;
        
        Ok(storage)
    }
    
    /// Create with custom storage path (for testing)
    pub fn with_path(storage_path: std::path::PathBuf) -> NozyResult<Self> {
        let storage = Self {
            storage_path: storage_path.clone(),
            transactions: Arc::new(Mutex::new(HashMap::new())),
        };
        
        storage.ensure_storage_directory()?;
        storage.load_transactions()?;
        
        Ok(storage)
    }
    
    fn ensure_storage_directory(&self) -> NozyResult<()> {
        if !self.storage_path.exists() {
            fs::create_dir_all(&self.storage_path)
                .map_err(|e| NozyError::Storage(format!("Failed to create storage directory: {}", e)))?;
        }
        Ok(())
    }
    
    fn get_transactions_path(&self) -> std::path::PathBuf {
        self.storage_path.join("sent_transactions.json")
    }
    
    /// Load sent transactions from disk
    fn load_transactions(&self) -> NozyResult<()> {
        let transactions_path = self.get_transactions_path();
        let path = Path::new(&transactions_path);
        
        if path.exists() {
            let content = fs::read_to_string(path)
                .map_err(|e| NozyError::Storage(format!("Failed to read sent transactions: {}", e)))?;
            
            let stored_transactions: HashMap<String, SentTransactionRecord> = 
                serde_json::from_str(&content)
                    .map_err(|e| NozyError::Storage(format!("Failed to parse sent transactions: {}", e)))?;
            
            *self.transactions.lock().unwrap() = stored_transactions;
        }
        
        Ok(())
    }
    
    /// Save sent transactions to disk
    fn save_transactions(&self) -> NozyResult<()> {
        let transactions_path = self.get_transactions_path();
        let transactions = self.transactions.lock().unwrap();
        let content = serde_json::to_string_pretty(&*transactions)
            .map_err(|e| NozyError::Storage(format!("Failed to serialize sent transactions: {}", e)))?;
        
        fs::write(&transactions_path, content)
            .map_err(|e| NozyError::Storage(format!("Failed to write sent transactions: {}", e)))?;
        
        Ok(())
    }
    
    /// Add or update a sent transaction record
    pub fn save_transaction(&self, transaction: SentTransactionRecord) -> NozyResult<()> {
        let txid = transaction.txid.clone();
        {
            let mut transactions = self.transactions.lock().unwrap();
            transactions.insert(txid, transaction);
        }
        self.save_transactions()?;
        Ok(())
    }
    
    /// Get a sent transaction by TXID
    pub fn get_transaction(&self, txid: &str) -> Option<SentTransactionRecord> {
        let transactions = self.transactions.lock().unwrap();
        transactions.get(txid).cloned()
    }
    
    /// Get all sent transactions
    pub fn get_all_transactions(&self) -> Vec<SentTransactionRecord> {
        let transactions = self.transactions.lock().unwrap();
        transactions.values().cloned().collect()
    }
    
    /// Get pending transactions (for status updates)
    pub fn get_pending_transactions(&self) -> Vec<SentTransactionRecord> {
        let transactions = self.transactions.lock().unwrap();
        transactions.values()
            .filter(|tx| tx.status == TransactionStatus::Pending)
            .cloned()
            .collect()
    }
    
    /// Update transaction status (for confirmed transactions)
    pub fn update_transaction_status(
        &self,
        txid: &str,
        block_height: u32,
        block_time: DateTime<Utc>,
        current_height: u32,
    ) -> NozyResult<bool> {
        let mut updated = false;
        {
            let mut transactions = self.transactions.lock().unwrap();
            if let Some(tx) = transactions.get_mut(txid) {
                tx.mark_confirmed(block_height, block_time, current_height);
                updated = true;
            }
        }
        
        if updated {
            self.save_transactions()?;
        }
        
        Ok(updated)
    }
    
    /// Mark transaction as failed
    pub fn mark_transaction_failed(&self, txid: &str) -> NozyResult<bool> {
        let mut updated = false;
        {
            let mut transactions = self.transactions.lock().unwrap();
            if let Some(tx) = transactions.get_mut(txid) {
                tx.mark_failed();
                updated = true;
            }
        }
        
        if updated {
            self.save_transactions()?;
        }
        
        Ok(updated)
    }
    
    /// Remove a transaction (for cleanup)
    pub fn remove_transaction(&self, txid: &str) -> NozyResult<bool> {
        let removed = {
            let mut transactions = self.transactions.lock().unwrap();
            transactions.remove(txid).is_some()
        };
        
        if removed {
            self.save_transactions()?;
        }
        
        Ok(removed)
    }
    
    /// Get count of stored transactions
    pub fn count(&self) -> usize {
        let transactions = self.transactions.lock().unwrap();
        transactions.len()
    }
}

