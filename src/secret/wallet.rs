// Secret Network Wallet Integration
// High-level interface for Secret Network operations with SNIP-20 token support

use crate::error::{NozyError, NozyResult};
use crate::privacy_network::proxy::ProxyConfig;
use crate::secret::rpc_client::SecretRpcClient;
use crate::secret::snip20::{Snip20Token, TokenInfo};
use crate::secret::transaction_history::{
    SecretTransactionRecord, SecretTransactionStatus, SecretTransactionStorage,
};
use crate::secret_keys::SecretKeyPair;
use chrono::Utc;

pub struct SecretWallet {
    rpc: SecretRpcClient,
    address: String,
    key_pair: Option<SecretKeyPair>, // Optional for query-only operations
}

impl SecretWallet {
    /// Create new Secret Network wallet (query-only, no signing)
    pub fn new(
        address: String,
        lcd_url: Option<String>,
        network: Option<&str>,
        proxy: Option<ProxyConfig>,
    ) -> NozyResult<Self> {
        let rpc = SecretRpcClient::new(lcd_url, network, proxy)?;

        Ok(Self {
            rpc,
            address,
            key_pair: None,
        })
    }

    /// Create new Secret Network wallet with key pair (for signing)
    pub fn new_with_key_pair(
        address: String,
        key_pair: SecretKeyPair,
        lcd_url: Option<String>,
        network: Option<&str>,
        proxy: Option<ProxyConfig>,
    ) -> NozyResult<Self> {
        let rpc = SecretRpcClient::new(lcd_url, network, proxy)?;

        Ok(Self {
            rpc,
            address,
            key_pair: Some(key_pair),
        })
    }

    /// Get wallet address
    pub fn address(&self) -> &str {
        &self.address
    }

    /// Get SCRT balance (native token)
    pub async fn get_scrt_balance(&self) -> NozyResult<f64> {
        let balance_uscrt = self.rpc.get_balance(&self.address).await?;
        Ok(balance_uscrt as f64 / 1_000_000.0) // 1 SCRT = 1e6 uscrt
    }

    /// Get SNIP-20 token balance
    pub async fn get_token_balance(&self, token_contract: &str) -> NozyResult<(f64, TokenInfo)> {
        let token = Snip20Token::new(token_contract.to_string(), self.rpc.clone());
        let info = token.get_token_info().await?;
        let balance = token.get_balance(&self.address).await?;

        let decimals = info.decimals as u32;
        let divisor = 10_u128.pow(decimals);
        let balance_float = balance as f64 / divisor as f64;

        Ok((balance_float, info))
    }

    /// Send SCRT (native token)
    /// Note: This requires proper transaction signing with private keys
    pub async fn send_scrt(&self, recipient: &str, amount: f64) -> NozyResult<String> {
        let amount_uscrt = (amount * 1_000_000.0) as u64;
        println!("🔒 Sending {:.6} SCRT to {}", amount, recipient);

        if self.key_pair.is_none() {
            return Err(NozyError::InvalidOperation(
                "Wallet does not have signing keys. Use new_with_key_pair() to create a wallet with signing capability.".to_string()
            ));
        }

        // TODO: Implement transaction construction and signing
        // This requires cosmrs for building Cosmos SDK transactions
        Err(NozyError::InvalidOperation(
            "SCRT transfers require transaction construction. Transaction building not yet implemented.".to_string()
        ))
    }

    /// Send SNIP-20 tokens (e.g., Shade tokens)
    pub async fn send_token(
        &self,
        token_contract: &str,
        recipient: &str,
        amount: f64,
        memo: Option<String>,
    ) -> NozyResult<String> {
        let token = Snip20Token::new(token_contract.to_string(), self.rpc.clone());
        let info = token.get_token_info().await?;

        let decimals = info.decimals as u32;
        let amount_raw = (amount * 10_f64.powi(decimals as i32)) as u128;

        println!("🔒 Sending {:.6} {} to {}", amount, info.symbol, recipient);

        let key_pair = self.key_pair.as_ref()
            .ok_or_else(|| NozyError::InvalidOperation(
                "Wallet does not have signing keys. Use new_with_key_pair() to create a wallet with signing capability.".to_string()
            ))?;

        // Validate inputs
        self.validate_address(recipient)?;
        self.validate_address(token_contract)?;
        self.validate_amount(amount)?;

        // Build and sign transaction
        use crate::secret::transaction::SecretTransactionBuilder;
        let tx_builder = SecretTransactionBuilder::new(self.rpc.clone(), None);

        // Get dynamic fee and gas estimates
        let fee_amount = self.rpc.get_gas_prices().await?;
        let gas_limit = self.rpc.estimate_gas(vec![]).await?;

        let tx_bytes = tx_builder
            .build_and_sign_transaction(
                key_pair,
                token_contract,
                recipient,
                amount_raw,
                memo.clone(),
                fee_amount,
                gas_limit,
            )
            .await?;

        // Store transaction record before broadcasting
        let tx_storage = SecretTransactionStorage::new()?;
        let tx_record = SecretTransactionRecord {
            txid: String::new(), // Will be set after broadcast
            contract_address: token_contract.to_string(),
            recipient_address: recipient.to_string(),
            amount: amount_raw,
            token_symbol: info.symbol.clone(),
            fee_uscrt: fee_amount,
            gas_limit,
            memo: memo.clone(),
            created_at: Utc::now(),
            broadcast_at: None,
            status: SecretTransactionStatus::Pending,
            block_height: None,
            block_time: None,
            confirmations: 0,
            error: None,
        };

        // Broadcast transaction
        let response = match self.rpc.broadcast_tx(tx_bytes, "BROADCAST_MODE_SYNC").await {
            Ok(res) => res,
            Err(e) => {
                // Store failed transaction
                let mut failed_tx = tx_record.clone();
                failed_tx.status = SecretTransactionStatus::Failed;
                failed_tx.error = Some(e.to_string());
                let _ = tx_storage.add_transaction(failed_tx);
                return Err(e);
            }
        };

        // Extract transaction hash
        let tx_hash = response
            .get("tx_response")
            .and_then(|r| r.get("txhash"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                NozyError::NetworkError("No transaction hash in response".to_string())
            })?;

        // Update transaction record with hash and broadcast time
        let mut final_tx = tx_record;
        final_tx.txid = tx_hash.to_string();
        final_tx.broadcast_at = Some(Utc::now());
        tx_storage.add_transaction(final_tx)?;

        Ok(tx_hash.to_string())
    }

    /// Validate Secret Network address
    fn validate_address(&self, address: &str) -> NozyResult<()> {
        if !address.starts_with("secret1") {
            return Err(NozyError::AddressParsing(
                "Invalid Secret Network address. Must start with 'secret1'".to_string(),
            ));
        }

        // Basic bech32 validation
        if address.len() < 45 || address.len() > 50 {
            return Err(NozyError::AddressParsing(
                "Invalid Secret Network address length".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate amount
    fn validate_amount(&self, amount: f64) -> NozyResult<()> {
        if amount <= 0.0 {
            return Err(NozyError::InvalidOperation(
                "Amount must be greater than zero".to_string(),
            ));
        }

        if amount > 1_000_000_000.0 {
            return Err(NozyError::InvalidOperation(
                "Amount too large. Maximum is 1,000,000,000".to_string(),
            ));
        }

        Ok(())
    }

    /// Get key pair (if available)
    pub fn key_pair(&self) -> Option<&SecretKeyPair> {
        self.key_pair.as_ref()
    }

    /// Get current block height
    pub async fn get_block_height(&self) -> NozyResult<u64> {
        self.rpc.get_height().await
    }

    /// Test connection
    pub async fn test_connection(&self) -> NozyResult<bool> {
        self.rpc.test_connection().await
    }

    /// Create SNIP-20 token interface
    pub fn get_token(&self, contract_address: &str) -> Snip20Token {
        Snip20Token::new(contract_address.to_string(), self.rpc.clone())
    }
}

// Note: For production use, you'll need to integrate with a proper Secret Network wallet library
// that handles:
// 1. Private key management and signing
// 2. Transaction construction and broadcasting
// 3. Viewing key management for viewing encrypted balances
// 4. Proper encryption/decryption of contract queries
