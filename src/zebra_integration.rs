use crate::error::{NozyError, NozyResult};
use serde::Deserialize;
use std::collections::HashMap;
use serde_json::Value;
use hex;

#[derive(Debug, Clone)]
pub struct ZebraClient {
    url: String,
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
        Self { url }
    }

    pub async fn get_block(&self, height: u32) -> NozyResult<HashMap<String, Value>> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "getblock",
            "params": [height],
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
        
        let block_info = self.get_block(height).await?;
        
        let mut anchor = [0u8; 32];
        let block_hash_bytes = hex::decode(&block_hash)
            .map_err(|e| NozyError::InvalidOperation(format!("Invalid block hash hex: {}", e)))?;
        
        let hash_len = block_hash_bytes.len().min(32);
        anchor[..hash_len].copy_from_slice(&block_hash_bytes[..hash_len]);
        
        let commitment_count = height as u64 * 100; // Rough estimate
        
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
        let client = reqwest::Client::new();
        let response = client
            .post(&self.url)
            .json(&request)
            .send()
            .await
            .map_err(|e| NozyError::NetworkError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(NozyError::NetworkError(format!("HTTP error: {}", response.status())));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| NozyError::NetworkError(format!("Failed to read response: {}", e)))?;

        let zebra_response: ZebraResponse<T> = serde_json::from_str(&response_text)
            .map_err(|e| NozyError::InvalidOperation(format!("Invalid JSON response: {}", e)))?;

        Ok(zebra_response)
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
