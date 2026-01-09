// Monero RPC Client with Privacy Proxy Support
// Connects to monero-wallet-rpc through Tor/I2P

use crate::error::{NozyError, NozyResult};
use crate::privacy_network::proxy::ProxyConfig;
use serde_json::{json, Value};
use std::time::Duration;

pub struct MoneroRpcClient {
    url: String,
    username: Option<String>,
    password: Option<String>,
    client: reqwest::Client,
}

impl MoneroRpcClient {
    /// Create new Monero RPC client with privacy proxy
    /// Default URL: http://127.0.0.1:18082/json_rpc
    pub fn new(
        url: Option<String>,
        username: Option<String>,
        password: Option<String>,
        proxy: Option<ProxyConfig>,
    ) -> NozyResult<Self> {
        let url = url.unwrap_or_else(|| "http://127.0.0.1:18082/json_rpc".to_string());

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
            url,
            username,
            password,
            client,
        })
    }

    /// Make RPC call to Monero wallet
    pub async fn call(&self, method: &str, params: Value) -> NozyResult<Value> {
        let mut request = self.client.post(&self.url).json(&json!({
            "jsonrpc": "2.0",
            "id": "0",
            "method": method,
            "params": params
        }));

        // Add authentication if provided
        if let (Some(user), Some(pass)) = (&self.username, &self.password) {
            request = request.basic_auth(user, Some(pass));
        }

        let response = request
            .send()
            .await
            .map_err(|e| NozyError::NetworkError(format!("Monero RPC error: {}", e)))?;

        let json: Value = response
            .json()
            .await
            .map_err(|e| NozyError::NetworkError(format!("Failed to parse response: {}", e)))?;

        if let Some(error) = json.get("error") {
            return Err(NozyError::NetworkError(format!(
                "Monero RPC error: {}",
                error
            )));
        }

        json.get("result")
            .cloned()
            .ok_or_else(|| NozyError::NetworkError("No result in response".to_string()))
    }

    /// Get wallet balance (in atomic units - 1 XMR = 1e12 atomic units)
    pub async fn get_balance(&self) -> NozyResult<u64> {
        let result = self.call("get_balance", json!({})).await?;

        let balance = result
            .get("balance")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| NozyError::NetworkError("Invalid balance response".to_string()))?;

        Ok(balance)
    }

    /// Get wallet address (primary address)
    pub async fn get_address(&self) -> NozyResult<String> {
        let result = self.call("get_address", json!({})).await?;

        let address = result
            .get("address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| NozyError::NetworkError("Invalid address response".to_string()))?;

        Ok(address.to_string())
    }

    /// Create a new subaddress (for privacy - never reuse addresses!)
    pub async fn create_address(&self, account_index: u32) -> NozyResult<String> {
        let result = self
            .call(
                "create_address",
                json!({
                    "account_index": account_index
                }),
            )
            .await?;

        let address = result
            .get("address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| NozyError::NetworkError("Invalid address response".to_string()))?;

        Ok(address.to_string())
    }

    /// Send Monero transaction
    /// Returns (txid, tx_key, fee_atomic)
    pub async fn send(
        &self,
        destination: &str,
        amount: u64, // in atomic units
    ) -> NozyResult<(String, Option<String>, Option<u64>)> {
        let result = self
            .call(
                "transfer",
                json!({
                    "destinations": [{
                        "amount": amount,
                        "address": destination
                    }],
                    "priority": 1, // Normal priority
                    "ring_size": 11, // Standard for privacy
                    "get_tx_key": true
                }),
            )
            .await?;

        let txid = result
            .get("tx_hash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| NozyError::NetworkError("Invalid transaction response".to_string()))?
            .to_string();

        let tx_key = result
            .get("tx_key")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let fee_atomic = result.get("fee").and_then(|v| v.as_u64());

        Ok((txid, tx_key, fee_atomic))
    }

    /// Get current block height
    pub async fn get_height(&self) -> NozyResult<u64> {
        let result = self.call("get_height", json!({})).await?;

        let height = result
            .get("height")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| NozyError::NetworkError("Invalid height response".to_string()))?;

        Ok(height)
    }

    /// Get block hash for given height (for ZK verification)
    pub async fn get_block_hash(&self, height: u64) -> NozyResult<String> {
        let result = self
            .call(
                "get_block_hash",
                json!({
                    "height": height
                }),
            )
            .await?;

        let hash = result
            .get("hash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| NozyError::NetworkError("Invalid block hash response".to_string()))?;

        Ok(hash.to_string())
    }

    /// Get current block hash
    pub async fn get_current_block_hash(&self) -> NozyResult<String> {
        let height = self.get_height().await?;
        self.get_block_hash(height).await
    }

    /// Test connection to Monero wallet
    pub async fn test_connection(&self) -> NozyResult<bool> {
        match self.get_height().await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}
