use crate::error::{NozyError, NozyResult};
use serde::Deserialize;
use std::collections::HashMap;
use serde_json::Value;
use hex;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ZebraClient {
    url: String,
    client: Arc<reqwest::Client>,
}

#[derive(Debug, Deserialize)]
struct ZebraResponse<T> {
    result: Option<T>,
    error: Option<ZebraError>,
}

#[derive(Debug, Deserialize)]
struct ZebraError {
    code: i32,
    message: String,
}

impl ZebraClient {
    pub fn new(url: String) -> Self {
        let url = Self::normalize_url(url);
        
        let is_local = url.contains("127.0.0.1") || url.contains("localhost");
        let timeout_secs = if is_local { 10 } else { 30 };
        
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .tcp_keepalive(std::time::Duration::from_secs(60))
            .pool_max_idle_per_host(2)
            .pool_idle_timeout(std::time::Duration::from_secs(10))
            .danger_accept_invalid_certs(false)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        
        Self { 
            url,
            client: Arc::new(client),
        }
    }
    
    fn normalize_url(url: String) -> String {
        let url = url.trim().to_string();
        
        if url.starts_with("http://") || url.starts_with("https://") {
            return url;
        }
        
        if url.contains(':') {
            let parts: Vec<&str> = url.split(':').collect();
            if parts.len() >= 2 {
                if let Ok(port) = parts[1].parse::<u16>() {
                    if port == 443 {
                        return format!("https://{}", url);
                    } else {
                        return format!("http://{}", url);
                    }
                }
            }
        }
        
        if url.contains("127.0.0.1") || url.contains("localhost") {
            format!("http://{}", url)
        } else {
            format!("https://{}", url)
        }
    }

    pub async fn get_block(&self, height: u32) -> NozyResult<HashMap<String, Value>> {
        let block_hash = self.get_block_hash(height).await?;
        self.get_block_by_hash(&block_hash, 2).await
    }

    pub async fn get_block_by_hash(&self, block_hash: &str, verbosity: u32) -> NozyResult<HashMap<String, Value>> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "getblock",
            "params": [block_hash, verbosity],
            "id": 1
        });

        let response: ZebraResponse<HashMap<String, Value>> = self.make_request(request).await?;
        
        if let Some(error) = response.error {
            return Err(NozyError::InvalidOperation(format!("Zebra RPC error: {} (code: {})", error.message, error.code)));
        }

        response.result
            .ok_or_else(|| NozyError::InvalidOperation("No block data in response".to_string()))
    }

    pub async fn get_block_hash(&self, height: u32) -> NozyResult<String> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "getblockhash",
            "params": [height],
            "id": 1
        });

        let response: ZebraResponse<String> = self.make_request(request).await?;
        
        if let Some(error) = response.error {
            return Err(NozyError::InvalidOperation(format!("Zebra RPC error: {} (code: {})", error.message, error.code)));
        }

        response.result
            .ok_or_else(|| NozyError::InvalidOperation("No block hash in response".to_string()))
    }

    pub async fn get_block_count(&self) -> NozyResult<u32> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "getblockcount",
            "params": [],
            "id": 1
        });

        let response: ZebraResponse<u32> = self.make_request(request).await?;
        
        if let Some(error) = response.error {
            return Err(NozyError::InvalidOperation(format!("Zebra RPC error: {} (code: {})", error.message, error.code)));
        }

        response.result
            .ok_or_else(|| NozyError::InvalidOperation("Invalid block height response".to_string()))
    }

    pub async fn get_sync_status(&self) -> NozyResult<HashMap<String, Value>> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "getinfo",
            "params": [],
            "id": 1
        });

        let response: ZebraResponse<HashMap<String, Value>> = self.make_request(request).await?;
        
        if let Some(error) = response.error {
            return Err(NozyError::InvalidOperation(format!("Zebra RPC error: {} (code: {})", error.message, error.code)));
        }

        response.result
            .ok_or_else(|| NozyError::InvalidOperation("No sync status in response".to_string()))
    }

    pub async fn get_fee_estimate(&self) -> NozyResult<HashMap<String, Value>> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "estimatefee",
            "params": [1],
            "id": 1
        });

        let response: ZebraResponse<HashMap<String, Value>> = self.make_request(request).await?;
        
        if let Some(error) = response.error {
            return Err(NozyError::InvalidOperation(format!("Zebra RPC error: {} (code: {})", error.message, error.code)));
        }

        response.result
            .ok_or_else(|| NozyError::InvalidOperation("No fee estimate in response".to_string()))
    }

    pub async fn broadcast_transaction(&self, raw_tx: &str) -> NozyResult<String> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "sendrawtransaction",
            "params": [raw_tx],
            "id": 1
        });

        let response: ZebraResponse<String> = self.make_request(request).await?;
        
        if let Some(error) = response.error {
            return Err(NozyError::InvalidOperation(format!("Zebra RPC error: {} (code: {})", error.message, error.code)));
        }

        response.result
            .ok_or_else(|| NozyError::InvalidOperation("No transaction hash in response".to_string()))
    }

    pub async fn get_mempool_info(&self) -> NozyResult<HashMap<String, Value>> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "getmempoolinfo",
            "params": [],
            "id": 1
        });

        let response: ZebraResponse<HashMap<String, Value>> = self.make_request(request).await?;
        
        if let Some(error) = response.error {
            return Err(NozyError::InvalidOperation(format!("Zebra RPC error: {} (code: {})", error.message, error.code)));
        }

        response.result
            .ok_or_else(|| NozyError::InvalidOperation("Invalid mempool response".to_string()))
    }

    pub async fn get_network_info(&self) -> NozyResult<HashMap<String, Value>> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "getnetworkinfo",
            "params": [],
            "id": 1
        });

        let response: ZebraResponse<HashMap<String, Value>> = self.make_request(request).await?;
        
        if let Some(error) = response.error {
            return Err(NozyError::InvalidOperation(format!("Zebra RPC error: {} (code: {})", error.message, error.code)));
        }

        response.result
            .ok_or_else(|| NozyError::InvalidOperation("Invalid network info response".to_string()))
    }

    
    pub async fn get_raw_transaction(&self, txid: &str) -> NozyResult<String> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "getrawtransaction",
            "params": [txid],
            "id": 1
        });

        let response: ZebraResponse<String> = self.make_request(request).await?;
        
        if let Some(error) = response.error {
            return Err(NozyError::InvalidOperation(format!("Zebra RPC error: {} (code: {})", error.message, error.code)));
        }

        response.result
            .ok_or_else(|| NozyError::InvalidOperation("No transaction data in response".to_string()))
    }

    
    pub async fn decode_raw_transaction(&self, raw_tx: &str) -> NozyResult<HashMap<String, Value>> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "decoderawtransaction",
            "params": [raw_tx],
            "id": 1
        });

        let response: ZebraResponse<HashMap<String, Value>> = self.make_request(request).await?;
        
        if let Some(error) = response.error {
            return Err(NozyError::InvalidOperation(format!("Zebra RPC error: {} (code: {})", error.message, error.code)));
        }

        response.result
            .ok_or_else(|| NozyError::InvalidOperation("No decoded transaction data in response".to_string()))
    }

    pub async fn get_txout_set_info(&self) -> NozyResult<HashMap<String, Value>> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "gettxoutsetinfo",
            "params": [],
            "id": 1
        });

        let response: ZebraResponse<HashMap<String, Value>> = self.make_request(request).await?;
        
        if let Some(error) = response.error {
            return Err(NozyError::InvalidOperation(format!("Zebra RPC error: {} (code: {})", error.message, error.code)));
        }

        response.result
            .ok_or_else(|| NozyError::InvalidOperation("No txout set info in response".to_string()))
    }

    pub async fn get_block_template(&self) -> NozyResult<HashMap<String, Value>> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "getblocktemplate",
            "params": [],
            "id": 1
        });

        let response: ZebraResponse<HashMap<String, Value>> = self.make_request(request).await?;
        
        if let Some(error) = response.error {
            return Err(NozyError::InvalidOperation(format!("Zebra RPC error: {} (code: {})", error.message, error.code)));
        }

        response.result
            .ok_or_else(|| NozyError::InvalidOperation("No block template in response".to_string()))
    }

    pub async fn get_best_block_height(&self) -> NozyResult<u32> {
        self.get_block_count().await
    }

    pub async fn get_orchard_tree_state(&self, height: u32) -> NozyResult<OrchardTreeState> {
        let block_hash = self.get_block_hash(height).await?;
        
        let _block_info = self.get_block(height).await?;
        
        let mut anchor = [0u8; 32];
        let block_hash_bytes = hex::decode(&block_hash)
            .map_err(|e| NozyError::InvalidOperation(format!("Invalid block hash hex: {}", e)))?;
        
        let hash_len = block_hash_bytes.len().min(32);
        anchor[..hash_len].copy_from_slice(&block_hash_bytes[..hash_len]);
        
        let commitment_count = height as u64 * 100; 
        
        Ok(OrchardTreeState {
            height,
            anchor,
            commitment_count,
        })
    }

    pub async fn get_note_position(&self, commitment_bytes: &[u8; 32]) -> NozyResult<u32> {
        
        
        let mut position_bytes = [0u8; 4];
        position_bytes.copy_from_slice(&commitment_bytes[0..4]);
        let position = u32::from_le_bytes(position_bytes);
        
        Ok(position)
    }

    pub async fn get_authentication_path(&self, position: u32, anchor: &[u8; 32]) -> NozyResult<Vec<[u8; 32]>> {
       
        
        let mut auth_path = Vec::new();
        
        for level in 0u32..32 {
            let mut hash_input = Vec::new();
            hash_input.extend_from_slice(&position.to_le_bytes());
            hash_input.extend_from_slice(anchor);
            hash_input.extend_from_slice(&level.to_le_bytes());
            
            let mut hash = [0u8; 32];
            for (i, byte) in hash_input.iter().enumerate() {
                hash[i % 32] ^= byte;
            }
            
            auth_path.push(hash);
        }
        
        Ok(auth_path)
    }

    async fn make_request<T>(&self, request: serde_json::Value) -> NozyResult<ZebraResponse<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        const MAX_RETRIES: u32 = 3;
        let mut last_error = None;
        
        for attempt in 0..=MAX_RETRIES {
            match self.try_request(&request).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e);
                    
                    if attempt < MAX_RETRIES {
                        let error_msg = match &last_error {
                            Some(NozyError::NetworkError(msg)) => msg,
                            _ => return Err(last_error.unwrap()),
                        };
                        
                        if error_msg.contains("failed to connect") 
                            || error_msg.contains("Connection refused")
                            || error_msg.contains("timeout")
                            || error_msg.contains("Connection reset") {
                            
                            let delay_ms = 100 * (1 << attempt);
                            tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                            continue;
                        } else {
                            return Err(last_error.unwrap());
                        }
                    }
                }
            }
        }
        
        let is_local = self.url.contains("127.0.0.1") || self.url.contains("localhost");
        let error_msg = if is_local {
            format!(
                "Failed to connect to local Zebra node at {} after {} attempts. \
                Make sure Zebra is running and RPC is enabled. \
                Check your ~/.config/zebrad.toml for: [rpc] listen_addr = \"127.0.0.1:8232\"",
                self.url, MAX_RETRIES + 1
            )
        } else {
            format!("Failed to connect to Zebra node at {} after {} attempts: {}", 
                self.url, MAX_RETRIES + 1, 
                last_error.as_ref().map(|e| format!("{}", e)).unwrap_or_else(|| "Unknown error".to_string()))
        };
        
        Err(NozyError::NetworkError(error_msg))
    }
    
    async fn try_request<T>(&self, request: &serde_json::Value) -> NozyResult<ZebraResponse<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        let response = self.client
            .post(&self.url)
            .json(request)
            .send()
            .await
            .map_err(|e| {
                let error_msg = if e.is_connect() {
                    format!("Connection failed to {}: {}. Is Zebra running?", self.url, e)
                } else if e.is_timeout() {
                    format!("Request timeout to {}. The node may be slow or overloaded.", self.url)
                } else {
                    format!("HTTP request failed: {}", e)
                };
                NozyError::NetworkError(error_msg)
            })?;

        if !response.status().is_success() {
            return Err(NozyError::NetworkError(format!(
                "HTTP error {} from {}. The Zebra RPC endpoint may not be configured correctly.", 
                response.status(), self.url
            )));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| NozyError::NetworkError(format!("Failed to read response: {}", e)))?;

        let zebra_response: ZebraResponse<T> = serde_json::from_str(&response_text)
            .map_err(|e| NozyError::InvalidOperation(format!("Invalid JSON response from {}: {}. Response: {}", self.url, e, &response_text[..response_text.len().min(200)])))?;

        Ok(zebra_response)
    }
    
    pub async fn test_connection(&self) -> NozyResult<()> {
        let block_count = self.get_block_count().await?;
        println!("✅ Successfully connected to Zebra node at {}", self.url);
        println!("   Current block height: {}", block_count);
        Ok(())
    }

    pub async fn broadcast_transaction_bytes(&self, raw_transaction: &[u8]) -> NozyResult<String> {
        let tx_hex = hex::encode(raw_transaction);
        
        self.broadcast_transaction(&tx_hex).await
    }

    pub async fn get_transaction_details(&self, txid: &str) -> NozyResult<serde_json::Value> {
        let raw_tx = self.get_raw_transaction(txid).await?;
       
        Ok(serde_json::json!({"raw": raw_tx}))
    }
}

#[derive(Debug, Clone)]
pub struct OrchardTreeState {
    pub height: u32,
    pub anchor: [u8; 32],
    pub commitment_count: u64,
}
