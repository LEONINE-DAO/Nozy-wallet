// Monero Wallet Integration
// High-level interface for Monero operations with privacy

use crate::error::{NozyError, NozyResult};
use crate::monero::rpc_client::MoneroRpcClient;
use crate::monero::transaction_history::{MoneroTransactionStorage, MoneroTransactionRecord, MoneroTransactionStatus};
use crate::privacy_network::proxy::ProxyConfig;
use chrono::Utc;

pub struct MoneroWallet {
    rpc: MoneroRpcClient,
    tx_storage: MoneroTransactionStorage,
}

impl MoneroWallet {
    /// Create new Monero wallet with privacy proxy
    pub fn new(
        rpc_url: Option<String>,
        username: Option<String>,
        password: Option<String>,
        proxy: Option<ProxyConfig>,
    ) -> NozyResult<Self> {
        Ok(Self {
            rpc: MoneroRpcClient::new(rpc_url, username, password, proxy)?,
            tx_storage: MoneroTransactionStorage::new()?,
        })
    }
    
    /// Get balance in XMR (converts from atomic units)
    pub async fn get_balance_xmr(&self) -> NozyResult<f64> {
        let balance_atomic = self.rpc.get_balance().await?;
        Ok(balance_atomic as f64 / 1_000_000_000_000.0) // 1 XMR = 1e12 atomic units
    }
    
    /// Get primary address
    pub async fn get_address(&self) -> NozyResult<String> {
        self.rpc.get_address().await
    }
    
    /// Create new subaddress for privacy (NEVER reuse addresses!)
    pub async fn create_subaddress(&self, account_index: u32) -> NozyResult<String> {
        println!("🛡️  Generating new Monero subaddress (privacy: no reuse)");
        self.rpc.create_address(account_index).await
    }
    
    /// Validate Monero address format
    pub fn validate_address(&self, address: &str) -> NozyResult<()> {
        // Monero addresses are 95 characters (standard) or 106 characters (integrated)
        // They start with '4' (mainnet) or '9' (testnet) for standard addresses
        if address.len() != 95 && address.len() != 106 {
            return Err(NozyError::InvalidInput(
                format!("Invalid Monero address length: expected 95 or 106 characters, got {}", address.len())
            ));
        }
        
        if !address.starts_with('4') && !address.starts_with('9') && !address.starts_with('8') {
            return Err(NozyError::InvalidInput(
                "Invalid Monero address format: must start with 4 (mainnet), 8 (subaddress), or 9 (testnet)".to_string()
            ));
        }
        
        // Basic character validation (Base58)
        if !address.chars().all(|c| c.is_alphanumeric()) {
            return Err(NozyError::InvalidInput(
                "Invalid Monero address: contains invalid characters".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Validate amount (must be positive and reasonable)
    pub fn validate_amount(&self, amount_xmr: f64) -> NozyResult<()> {
        if amount_xmr <= 0.0 {
            return Err(NozyError::InvalidInput(
                "Amount must be greater than zero".to_string()
            ));
        }
        
        if amount_xmr > 1_000_000.0 {
            return Err(NozyError::InvalidInput(
                "Amount exceeds maximum (1,000,000 XMR)".to_string()
            ));
        }
        
        // Check minimum amount (0.000000000001 XMR = 1 atomic unit)
        if amount_xmr < 0.000000000001 {
            return Err(NozyError::InvalidInput(
                "Amount is too small (minimum: 0.000000000001 XMR)".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Send XMR (converts to atomic units) with transaction tracking
    pub async fn send_xmr(&self, destination: &str, amount_xmr: f64) -> NozyResult<String> {
        // Validate inputs
        self.validate_address(destination)?;
        self.validate_amount(amount_xmr)?;
        
        // Check balance
        let balance = self.get_balance_xmr().await?;
        if amount_xmr > balance {
            return Err(NozyError::InsufficientFunds(
                format!("Insufficient balance: {:.8} XMR available, {:.8} XMR requested", balance, amount_xmr)
            ));
        }
        
        let amount_atomic = (amount_xmr * 1_000_000_000_000.0) as u64;
        println!("🔒 Sending {:.8} XMR through privacy network", amount_xmr);
        
        // Create transaction record before sending
        let created_at = Utc::now();
        
        // Send transaction
        let result = self.rpc.send(destination, amount_atomic).await;
        
        match result {
            Ok((txid, tx_key, fee_atomic)) => {
                let fee_xmr = fee_atomic.map(|f| f as f64 / 1_000_000_000_000.0);
                
                // Store transaction record
                let tx_record = MoneroTransactionRecord {
                    txid: txid.clone(),
                    recipient_address: destination.to_string(),
                    amount_xmr,
                    amount_atomic,
                    fee_xmr,
                    fee_atomic,
                    created_at,
                    broadcast_at: Some(Utc::now()),
                    status: MoneroTransactionStatus::Pending,
                    block_height: None,
                    block_time: None,
                    confirmations: 0,
                    error: None,
                    tx_key,
                };
                
                self.tx_storage.add_transaction(tx_record)?;
                
                println!("✅ Transaction sent: {}", txid);
                if let Some(fee) = fee_xmr {
                    println!("   Fee: {:.8} XMR", fee);
                }
                Ok(txid)
            },
            Err(e) => {
                // Store failed transaction
                let tx_record = MoneroTransactionRecord {
                    txid: format!("failed_{}", created_at.timestamp()),
                    recipient_address: destination.to_string(),
                    amount_xmr,
                    amount_atomic,
                    fee_xmr: None,
                    fee_atomic: None,
                    created_at,
                    broadcast_at: None,
                    status: MoneroTransactionStatus::Failed,
                    block_height: None,
                    block_time: None,
                    confirmations: 0,
                    error: Some(e.to_string()),
                    tx_key: None,
                };
                
                let _ = self.tx_storage.add_transaction(tx_record);
                
                Err(e)
            }
        }
    }
    
    /// Get transaction history
    pub fn get_transaction_history(&self) -> NozyResult<Vec<MoneroTransactionRecord>> {
        Ok(self.tx_storage.get_all_transactions())
    }
    
    /// Get transaction by ID
    pub fn get_transaction(&self, txid: &str) -> Option<MoneroTransactionRecord> {
        self.tx_storage.get_transaction(txid)
    }
    
    /// Check transaction status
    pub async fn check_transaction_status(&self, txid: &str) -> NozyResult<bool> {
        self.tx_storage.check_transaction_status(&self.rpc, txid).await
    }
    
    /// Update confirmations for all transactions
    pub async fn update_confirmations(&self) -> NozyResult<usize> {
        self.tx_storage.update_confirmations(&self.rpc).await
    }
    
    /// Get current block height
    pub async fn get_block_height(&self) -> NozyResult<u64> {
        self.rpc.get_height().await
    }
    
    /// Get block hash for given height (for ZK verification)
    pub async fn get_block_hash(&self, height: u64) -> NozyResult<String> {
        self.rpc.get_block_hash(height).await
    }
    
    /// Get current block hash
    pub async fn get_current_block_hash(&self) -> NozyResult<String> {
        self.rpc.get_current_block_hash().await
    }
    
    /// Test if Monero wallet is connected
    pub async fn is_connected(&self) -> bool {
        self.rpc.test_connection().await.unwrap_or(false)
    }
}
