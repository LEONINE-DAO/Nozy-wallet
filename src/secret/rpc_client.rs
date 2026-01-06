// Secret Network RPC Client
// Connects to Secret Network nodes via LCD (Light Client Daemon) API

use crate::error::{NozyError, NozyResult};
use crate::privacy_network::proxy::ProxyConfig;
use serde_json::{json, Value};
use std::time::Duration;

#[derive(Clone)]
pub struct SecretRpcClient {
    lcd_url: String,
    client: reqwest::Client,
}

impl SecretRpcClient {
    /// Create new Secret Network RPC client with privacy proxy
    /// Default URL: https://api.secretapi.io (mainnet) or https://api.pulsar.scrttestnet.com (testnet)
    pub fn new(
        lcd_url: Option<String>,
        network: Option<&str>,
        proxy: Option<ProxyConfig>,
    ) -> NozyResult<Self> {
        let lcd_url = lcd_url.unwrap_or_else(|| {
            let network = network.unwrap_or("mainnet");
            match network {
                "mainnet" => "https://api.secretapi.io".to_string(),
                "testnet" => "https://api.pulsar.scrttestnet.com".to_string(),
                _ => "https://api.secretapi.io".to_string(),
            }
        });
        
        // Create client with privacy proxy
        let client = if let Some(proxy_config) = proxy {
            proxy_config.create_client()?
        } else {
            // Fallback to direct connection (privacy network auto-detect is async, skip for now)
            reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .map_err(|e| NozyError::NetworkError(format!("Failed to create client: {}", e)))?
        };
        
        Ok(Self {
            lcd_url,
            client,
        })
    }
    
    /// Make GET request to Secret Network LCD API
    async fn get(&self, endpoint: &str) -> NozyResult<Value> {
        let url = format!("{}{}", self.lcd_url, endpoint);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| NozyError::NetworkError(format!("Secret Network API error: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(NozyError::NetworkError(format!(
                "Secret Network API error: HTTP {}",
                response.status()
            )));
        }
        
        let json: Value = response.json().await
            .map_err(|e| NozyError::NetworkError(format!("Failed to parse response: {}", e)))?;
        
        Ok(json)
    }
    
    /// Make POST request to Secret Network LCD API
    async fn post(&self, endpoint: &str, body: Value) -> NozyResult<Value> {
        let url = format!("{}{}", self.lcd_url, endpoint);
        
        let response = self.client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| NozyError::NetworkError(format!("Secret Network API error: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(NozyError::NetworkError(format!(
                "Secret Network API error: HTTP {}",
                response.status()
            )));
        }
        
        let json: Value = response.json().await
            .map_err(|e| NozyError::NetworkError(format!("Failed to parse response: {}", e)))?;
        
        Ok(json)
    }
    
    /// Get account information
    pub async fn get_account(&self, address: &str) -> NozyResult<Value> {
        let endpoint = format!("/cosmos/auth/v1beta1/accounts/{}", address);
        self.get(&endpoint).await
    }
    
    /// Get account balance (SCRT)
    pub async fn get_balance(&self, address: &str) -> NozyResult<u64> {
        let endpoint = format!("/cosmos/bank/v1beta1/balances/{}", address);
        let response = self.get(&endpoint).await?;
        
        // Parse balance from response
        if let Some(balances) = response.get("balances").and_then(|v| v.as_array()) {
            for balance in balances {
                if let Some(denom) = balance.get("denom").and_then(|v| v.as_str()) {
                    if denom == "uscrt" {
                        if let Some(amount) = balance.get("amount").and_then(|v| v.as_str()) {
                            return Ok(amount.parse::<u64>()
                                .map_err(|_| NozyError::NetworkError("Invalid balance format".to_string()))?);
                        }
                    }
                }
            }
        }
        
        Ok(0)
    }
    
    /// Query contract (for SNIP-20 tokens)
    pub async fn query_contract(&self, contract_address: &str, query: Value) -> NozyResult<Value> {
        let endpoint = format!("/compute/v1beta1/query/{}", contract_address);
        let body = json!({
            "query": query
        });
        self.post(&endpoint, body).await
    }
    
    /// Execute contract (for SNIP-20 transfers, etc.)
    /// Note: This endpoint requires proper transaction signing
    pub async fn execute_contract(
        &self,
        contract_address: &str,
        sender: &str,
        msg: Value,
        funds: Option<Vec<Value>>,
    ) -> NozyResult<Value> {
        // The LCD API execute endpoint requires a signed transaction
        // We need to:
        // 1. Get account info (account_number, sequence)
        // 2. Build transaction
        // 3. Sign transaction
        // 4. Encode as protobuf
        // 5. Broadcast
        
        // For now, return an error indicating this needs proper implementation
        Err(NozyError::InvalidOperation(
            "Contract execution requires signed transactions. Transaction signing and broadcasting not yet fully implemented.".to_string()
        ))
    }
    
    /// Broadcast a signed transaction
    pub async fn broadcast_tx(&self, tx_bytes: Vec<u8>, mode: &str) -> NozyResult<Value> {
        let endpoint = "/cosmos/tx/v1beta1/txs";
        let body = json!({
            "tx_bytes": hex::encode(tx_bytes),
            "mode": mode // "BROADCAST_MODE_SYNC", "BROADCAST_MODE_ASYNC", or "BROADCAST_MODE_BLOCK"
        });
        
        self.post(&endpoint, body).await
    }
    
    /// Get transaction by hash
    pub async fn get_transaction(&self, txid: &str) -> NozyResult<Value> {
        let endpoint = format!("/cosmos/tx/v1beta1/txs/{}", txid);
        self.get(&endpoint).await
    }
    
    /// Get gas prices from network
    pub async fn get_gas_prices(&self) -> NozyResult<u64> {
        // Try to query network for current gas prices
        // Secret Network typically uses fixed fees, but we can query if available
        // For now, return a reasonable default based on network conditions
        // In production, could query /cosmos/tx/v1beta1/fees endpoint if available
        
        // Conservative estimate: 0.1 SCRT (100,000 uscrt)
        // This covers most contract executions
        Ok(100_000)
    }
    
    /// Estimate gas for transaction
    pub async fn estimate_gas(&self, _tx_bytes: Vec<u8>) -> NozyResult<u64> {
        // Try to simulate transaction to get gas estimate
        // For now, return conservative estimate based on operation type
        // In production, use /cosmos/tx/v1beta1/simulate endpoint
        
        // Standard gas limits for Secret Network:
        // - Simple transfer: ~100,000 gas
        // - Contract execution: ~200,000-500,000 gas depending on complexity
        // - Complex operations: up to 1,000,000 gas
        
        // Return conservative estimate for SNIP-20 transfers
        Ok(200_000)
    }
    
    /// Get network info (chain ID, etc.)
    pub async fn get_network_info(&self) -> NozyResult<Value> {
        let response = self.get("/cosmos/base/tendermint/v1beta1/node_info").await?;
        Ok(response)
    }
    
    /// Get chain ID
    pub async fn get_chain_id(&self) -> NozyResult<String> {
        let info = self.get_network_info().await?;
        let chain_id = info.get("default_node_info")
            .and_then(|n| n.get("network"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "secret-4".to_string()); // Default to mainnet
        
        Ok(chain_id)
    }
    
    /// Get account information
    pub async fn get_account_info(&self, address: &str) -> NozyResult<(u64, u64)> {
        let endpoint = format!("/cosmos/auth/v1beta1/accounts/{}", address);
        let response = self.get(&endpoint).await?;
        
        let account = response.get("account")
            .ok_or_else(|| NozyError::NetworkError("No account in response".to_string()))?;
        
        let account_number = account.get("account_number")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);
        
        let sequence = account.get("sequence")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);
        
        Ok((account_number, sequence))
    }
    
    /// Get current block height
    pub async fn get_height(&self) -> NozyResult<u64> {
        let response = self.get("/cosmos/base/tendermint/v1beta1/blocks/latest").await?;
        
        let height = response
            .get("block")
            .and_then(|b| b.get("header"))
            .and_then(|h| h.get("height"))
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
            .ok_or_else(|| NozyError::NetworkError("Invalid block height response".to_string()))?;
        
        Ok(height)
    }
    
    /// Test connection to Secret Network
    pub async fn test_connection(&self) -> NozyResult<bool> {
        match self.get_height().await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}
