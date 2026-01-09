// Monero Transaction History
// Tracks and manages Monero transaction history

use crate::error::{NozyError, NozyResult};
use crate::paths::get_wallet_data_dir;
use crate::monero::rpc_client::MoneroRpcClient;
use serde::{Serialize, Deserialize};
use serde_json;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoneroTransactionRecord {
    pub txid: String,
    pub recipient_address: String,
    pub amount_xmr: f64,
    pub amount_atomic: u64, // 1 XMR = 1e12 atomic units
    pub fee_xmr: Option<f64>,
    pub fee_atomic: Option<u64>,
    pub created_at: DateTime<Utc>,
    pub broadcast_at: Option<DateTime<Utc>>,
    pub status: MoneroTransactionStatus,
    pub block_height: Option<u64>,
    pub block_time: Option<DateTime<Utc>>,
    pub confirmations: u32,
    pub error: Option<String>,
    pub tx_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MoneroTransactionStatus {
    Pending,
    Confirmed,
    Failed,
}

pub struct MoneroTransactionStorage {
    transactions: Arc<Mutex<HashMap<String, MoneroTransactionRecord>>>,
    storage_path: std::path::PathBuf,
}

impl MoneroTransactionStorage {
    pub fn new() -> NozyResult<Self> {
        let data_dir = get_wallet_data_dir();
        let storage_path = data_dir.join("monero_transactions.json");
        
        let transactions = if storage_path.exists() {
            let content = fs::read_to_string(&storage_path)
                .map_err(|e| NozyError::Storage(format!("Failed to read transaction history: {}", e)))?;
            serde_json::from_str(&content)
                .unwrap_or_else(|_| HashMap::new())
        } else {
            HashMap::new()
        };
        
        Ok(Self {
            transactions: Arc::new(Mutex::new(transactions)),
            storage_path,
        })
    }
    
    pub fn add_transaction(&self, tx: MoneroTransactionRecord) -> NozyResult<()> {
        let mut transactions = self.transactions.lock()
            .map_err(|e| NozyError::Storage(format!("Mutex poisoned: {}", e)))?;
        transactions.insert(tx.txid.clone(), tx);
        self.save_transactions()?;
        Ok(())
    }
    
    pub fn get_transaction(&self, txid: &str) -> Option<MoneroTransactionRecord> {
        let transactions = self.transactions.lock()
            .ok()?; 
        transactions.get(txid).cloned()
    }
    
    pub fn get_all_transactions(&self) -> Vec<MoneroTransactionRecord> {
        let transactions_guard = self.transactions.lock()
            .map_err(|e| {
                eprintln!("⚠️  Warning: Transaction history mutex poisoned, returning empty list: {}", e);
            })
            .ok();
        
        if let Some(transactions) = transactions_guard {
            transactions.values().cloned().collect()
        } else {
            Vec::new()
        }
    }
    
    pub fn get_pending_transactions(&self) -> Vec<MoneroTransactionRecord> {
        let transactions_guard = self.transactions.lock()
            .map_err(|e| {
                eprintln!("⚠️  Warning: Transaction history mutex poisoned, returning empty list: {}", e);
            })
            .ok();
        
        if let Some(transactions) = transactions_guard {
            transactions.values()
                .filter(|tx| tx.status == MoneroTransactionStatus::Pending)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }
    
    pub fn update_transaction_status(
        &self,
        txid: &str,
        status: MoneroTransactionStatus,
        block_height: Option<u64>,
        block_time: Option<DateTime<Utc>>,
        error: Option<String>,
    ) -> NozyResult<bool> {
        let mut transactions = self.transactions.lock()
            .map_err(|e| NozyError::Storage(format!("Mutex poisoned: {}", e)))?;
        
        if let Some(tx) = transactions.get_mut(txid) {
            tx.status = status;
            if let Some(height) = block_height {
                tx.block_height = Some(height);
            }
            if let Some(time) = block_time {
                tx.block_time = Some(time);
            }
            if let Some(err) = error {
                tx.error = Some(err);
            }
            self.save_transactions()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    pub async fn check_transaction_status(
        &self,
        rpc: &MoneroRpcClient,
        txid: &str,
    ) -> NozyResult<bool> {
       
        let result = rpc.call("get_transfer_by_txid", serde_json::json!({
            "txid": txid
        })).await;
        
        match result {
            Ok(transfer_info) => {
                let height = transfer_info.get("height")
                    .and_then(|v| v.as_u64());
                
                let timestamp = transfer_info.get("timestamp")
                    .and_then(|v| v.as_u64())
                    .map(|ts| {
                        DateTime::from_timestamp(ts as i64, 0)
                            .unwrap_or_else(|| Utc::now())
                    });
                
                let current_height = rpc.get_height().await.unwrap_or(0);
                let confirmations = if let Some(h) = height {
                    if h > 0 {
                        current_height.saturating_sub(h) + 1
                    } else {
                        0 
                    }
                } else {
                    0
                };
                
                
                if let Ok(mut transactions) = self.transactions.lock() {
                    if let Some(tx) = transactions.get_mut(txid) {
                        tx.confirmations = confirmations as u32;
                    }
                }
                
                if let Some(h) = height {
                    if h > 0 {
                        self.update_transaction_status(
                            txid,
                            MoneroTransactionStatus::Confirmed,
                            height,
                            timestamp,
                            None,
                        )?;
                    }
                }
                
                Ok(true)
            },
            Err(e) => {
                
                let error_str = e.to_string();
                if error_str.contains("not found") || error_str.contains("Invalid") {
                    Ok(false)
                } else {
                    Err(e)
                }
            }
        }
    }
    
    pub async fn update_confirmations(
        &self,
        rpc: &MoneroRpcClient,
    ) -> NozyResult<usize> {
        let current_height = rpc.get_height().await.unwrap_or(0);
        let mut updated_count = 0;
        
        let mut transactions = match self.transactions.lock() {
            Ok(tx) => tx,
            Err(e) => {
                eprintln!("⚠️  Warning: Transaction history mutex poisoned: {}", e);
                return Ok(0);
            }
        };
        for tx in transactions.values_mut() {
            if let Some(block_height) = tx.block_height {
                if block_height > 0 {
                    let new_confirmations = current_height.saturating_sub(block_height) + 1;
                    if new_confirmations != tx.confirmations as u64 {
                        tx.confirmations = new_confirmations as u32;
                        updated_count += 1;
                    }
                }
            }
        }
        
        if updated_count > 0 {
            self.save_transactions()?;
        }
        
        Ok(updated_count)
    }
    
    fn save_transactions(&self) -> NozyResult<()> {
        let transactions = self.transactions.lock()
            .map_err(|e| NozyError::Storage(format!("Mutex poisoned: {}", e)))?;
        let serialized = serde_json::to_string_pretty(&*transactions)
            .map_err(|e| NozyError::Storage(format!("Failed to serialize transactions: {}", e)))?;
        
        fs::write(&self.storage_path, serialized)
            .map_err(|e| NozyError::Storage(format!("Failed to write transactions: {}", e)))?;
        
        Ok(())
    }
}
