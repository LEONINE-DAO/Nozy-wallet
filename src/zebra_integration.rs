use crate::error::{NozyError, NozyResult};
use serde::Deserialize;
use std::collections::HashMap;
use serde_json::Value;

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
}
