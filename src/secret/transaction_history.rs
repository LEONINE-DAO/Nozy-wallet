// Secret Network Transaction History
// Tracks and manages Secret Network transaction history

use crate::error::{NozyError, NozyResult};
use crate::paths::get_wallet_data_dir;
use crate::secret::rpc_client::SecretRpcClient;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretTransactionRecord {
    pub txid: String,
    pub contract_address: String,
    pub recipient_address: String,
    pub amount: u128,
    pub token_symbol: String,
    pub fee_uscrt: u64,
    pub gas_limit: u64,
    pub memo: Option<String>,
    pub created_at: DateTime<Utc>,
    pub broadcast_at: Option<DateTime<Utc>>,
    pub status: SecretTransactionStatus,
    pub block_height: Option<u64>,
    pub block_time: Option<DateTime<Utc>>,
    pub confirmations: u32,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SecretTransactionStatus {
    Pending,
    Confirmed,
    Failed,
}

pub struct SecretTransactionStorage {
    transactions: Arc<Mutex<HashMap<String, SecretTransactionRecord>>>,
    storage_path: std::path::PathBuf,
}

impl SecretTransactionStorage {
    pub fn new() -> NozyResult<Self> {
        let data_dir = get_wallet_data_dir();
        let storage_path = data_dir.join("secret_transactions.json");
        
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
    
    pub fn add_transaction(&self, tx: SecretTransactionRecord) -> NozyResult<()> {
        let mut transactions = self.transactions.lock()
            .map_err(|e| NozyError::Storage(format!("Mutex poisoned: {}", e)))?;
        transactions.insert(tx.txid.clone(), tx);
        self.save_transactions()?;
        Ok(())
    }
    
    pub fn get_transaction(&self, txid: &str) -> Option<SecretTransactionRecord> {
        let transactions = self.transactions.lock()
            .ok()?; // Return None if mutex is poisoned
        transactions.get(txid).cloned()
    }
    
    pub fn get_all_transactions(&self) -> Vec<SecretTransactionRecord> {
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
    
    pub fn get_pending_transactions(&self) -> Vec<SecretTransactionRecord> {
        let transactions_guard = self.transactions.lock()
            .map_err(|e| {
                eprintln!("⚠️  Warning: Transaction history mutex poisoned, returning empty list: {}", e);
            })
            .ok();
        
        if let Some(transactions) = transactions_guard {
            transactions.values()
                .filter(|tx| tx.status == SecretTransactionStatus::Pending)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }
    
    pub fn update_transaction_status(
        &self,
        txid: &str,
        status: SecretTransactionStatus,
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
        rpc: &SecretRpcClient,
        txid: &str,
    ) -> NozyResult<bool> {
        // Query transaction from Secret Network
        match rpc.get_transaction(txid).await {
            Ok(response) => {
                let tx_response = response.get("tx_response")
                    .ok_or_else(|| NozyError::NetworkError("No tx_response in result".to_string()))?;
                
                let code = tx_response.get("code")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                
                if code != 0 {
                    // Transaction failed
                    let error_msg = tx_response.get("raw_log")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    
                    self.update_transaction_status(
                        txid,
                        SecretTransactionStatus::Failed,
                        None,
                        None,
                        error_msg,
                    )?;
                    return Ok(true);
                }
                
                // Transaction succeeded
                let height = tx_response.get("height")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<u64>().ok());
                
                let timestamp = tx_response.get("timestamp")
                    .and_then(|v| v.as_str())
                    .and_then(|s| {
                        chrono::DateTime::parse_from_rfc3339(s)
                            .ok()
                            .map(|dt| dt.with_timezone(&Utc))
                    });
                
                // Get current height for confirmations
                let current_height = rpc.get_height().await.unwrap_or(0);
                let confirmations = if let Some(h) = height {
                    current_height.saturating_sub(h) + 1
                } else {
                    0
                };
                
                // Update confirmations
                if let Ok(mut transactions) = self.transactions.lock() {
                    if let Some(tx) = transactions.get_mut(txid) {
                        tx.confirmations = confirmations as u32;
                    }
                }
                
                self.update_transaction_status(
                    txid,
                    SecretTransactionStatus::Confirmed,
                    height,
                    timestamp,
                    None,
                )?;
                
                Ok(true)
            },
            Err(e) => {
                // Transaction might not be found yet (still in mempool)
                // Don't update status, just return false
                if e.to_string().contains("not found") || e.to_string().contains("404") {
                    Ok(false)
                } else {
                    Err(e)
                }
            }
        }
    }
    
    pub async fn update_confirmations(
        &self,
        rpc: &SecretRpcClient,
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
                let new_confirmations = current_height.saturating_sub(block_height) + 1;
                if new_confirmations != tx.confirmations as u64 {
                    tx.confirmations = new_confirmations as u32;
                    updated_count += 1;
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
