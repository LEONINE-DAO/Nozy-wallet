

use crate::error::{NozyError, NozyResult};
use std::collections::HashMap;
use serde_json::Value;
use tonic::transport::Channel;
use std::sync::Arc;

#[derive(Debug)]
pub struct ZebraGrpcClient {
    #[allow(dead_code)]
    channel: Arc<Channel>,
    #[allow(dead_code)]
    endpoint: String,
}

impl ZebraGrpcClient {
    pub async fn new(endpoint: String) -> NozyResult<Self> {
        let endpoint = Self::normalize_endpoint(endpoint);
        
        let channel = if endpoint.starts_with("https://") {
            tonic::transport::Channel::from_shared(endpoint.clone())
                .map_err(|e| NozyError::NetworkError(format!("Invalid gRPC endpoint: {}", e)))?
                .tls_config(tonic::transport::ClientTlsConfig::new())
                .map_err(|e| NozyError::NetworkError(format!("TLS config error: {}", e)))?
                .connect()
                .await
                .map_err(|e| NozyError::NetworkError(format!("Failed to connect to gRPC endpoint: {}", e)))?
        } else {
            tonic::transport::Channel::from_shared(endpoint.clone())
                .map_err(|e| NozyError::NetworkError(format!("Invalid gRPC endpoint: {}", e)))?
                .connect()
                .await
                .map_err(|e| NozyError::NetworkError(format!("Failed to connect to gRPC endpoint: {}", e)))?
        };

        Ok(Self {
            channel: Arc::new(channel),
            endpoint,
        })
    }

    fn normalize_endpoint(endpoint: String) -> String {
        let endpoint = endpoint.trim().to_string();
        
        
        if endpoint.starts_with("https://") {
            endpoint
        } else if endpoint.starts_with("http://") {
            endpoint
        } else {
            if endpoint.contains(":443") || endpoint.ends_with(":443/") {
                format!("https://{}", endpoint)
            } else {
                format!("http://{}", endpoint)
            }
        }
    }

    
    pub async fn get_block_count(&self) -> NozyResult<u32> {
        Err(NozyError::InvalidOperation(
            "gRPC implementation requires Zebra proto definitions. Please use JSON-RPC for now.".to_string()
        ))
    }

    /// Get block by height via gRPC
    /// TODO: Implement once Zebra gRPC proto definitions are available
    pub async fn get_block(&self, _height: u32) -> NozyResult<HashMap<String, Value>> {
        Err(NozyError::InvalidOperation(
            "gRPC implementation requires Zebra proto definitions. Please use JSON-RPC for now.".to_string()
        ))
    }

    /// Broadcast transaction via gRPC
    /// TODO: Implement once Zebra gRPC proto definitions are available
    pub async fn broadcast_transaction(&self, _tx_hex: &str) -> NozyResult<String> {
        Err(NozyError::InvalidOperation(
            "gRPC implementation requires Zebra proto definitions. Please use JSON-RPC for now.".to_string()
        ))
    }

    /// Test connection
    pub async fn test_connection(&self) -> NozyResult<()> {
        // For now, just verify the channel is ready
        // TODO: Implement actual gRPC health check once proto definitions are available
        Ok(())
    }
}

