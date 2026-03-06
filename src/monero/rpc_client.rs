// Monero RPC Client with Privacy Proxy Support
// Connects to monero-wallet-rpc through Tor/I2P
// Includes retry with exponential backoff for fault tolerance.

use crate::error::{NozyError, NozyResult};
use crate::privacy_network::proxy::ProxyConfig;
use serde_json::{json, Value};
use std::time::Duration;

const MAX_RETRIES: u32 = 3;

fn is_retryable_network_error(msg: &str) -> bool {
    let lower = msg.to_lowercase();
    lower.contains("timeout")
        || lower.contains("connection")
        || lower.contains("connection reset")
        || lower.contains("connection refused")
        || lower.contains("failed to connect")
}

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
        let client = if let Some(proxy_config) = proxy {
            proxy_config.create_client()?
        } else {
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

    /// Make RPC call to Monero wallet (with retry and exponential backoff).
    pub async fn call(&self, method: &str, params: Value) -> NozyResult<Value> {
        let mut last_err = None;
        for attempt in 0..=MAX_RETRIES {
            match self.call_once(method, &params).await {
                Ok(v) => return Ok(v),
                Err(e) => {
                    let msg = e.to_string();
                    let retryable = is_retryable_network_error(&msg);
                    last_err = Some(e);
                    if attempt < MAX_RETRIES && retryable {
                        let delay_ms = 100 * (1 << attempt);
                        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                        continue;
                    }
                    return Err(last_err.unwrap());
                }
            }
        }
        Err(last_err
            .unwrap_or_else(|| NozyError::NetworkError("Monero RPC error: Unknown".to_string())))
    }

    async fn call_once(&self, method: &str, params: &Value) -> NozyResult<Value> {
        let mut request = self.client.post(&self.url).json(&json!({
            "jsonrpc": "2.0",
            "id": "0",
            "method": method,
            "params": params
        }));

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

    pub async fn get_balance(&self) -> NozyResult<u64> {
        let result = self.call("get_balance", json!({})).await?;

        let balance = result
            .get("balance")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| NozyError::NetworkError("Invalid balance response".to_string()))?;

        Ok(balance)
    }

    pub async fn get_address(&self) -> NozyResult<String> {
        let result = self.call("get_address", json!({})).await?;

        let address = result
            .get("address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| NozyError::NetworkError("Invalid address response".to_string()))?;

        Ok(address.to_string())
    }

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

    pub async fn send(
        &self,
        destination: &str,
        amount: u64,
    ) -> NozyResult<(String, Option<String>, Option<u64>)> {
        let result = self
            .call(
                "transfer",
                json!({
                    "destinations": [{
                        "amount": amount,
                        "address": destination
                    }],
                    "priority": 1,
                    "ring_size": 11,
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

    pub async fn get_height(&self) -> NozyResult<u64> {
        let result = self.call("get_height", json!({})).await?;

        let height = result
            .get("height")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| NozyError::NetworkError("Invalid height response".to_string()))?;

        Ok(height)
    }

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

    pub async fn get_current_block_hash(&self) -> NozyResult<String> {
        let height = self.get_height().await?;
        self.get_block_hash(height).await
    }

    pub async fn test_connection(&self) -> NozyResult<bool> {
        match self.get_height().await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}
