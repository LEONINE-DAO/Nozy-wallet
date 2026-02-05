// I2P Proxy Support
// Handles I2P HTTP proxy connections

use crate::error::{NozyError, NozyResult};
use crate::privacy_network::proxy::{PrivacyNetwork, PrivacyProxy};
use reqwest::Client;
use std::time::Duration;

pub struct I2PProxy {
    proxy_url: String,
}

impl I2PProxy {
    pub fn new(proxy_url: Option<String>) -> Self {
        Self {
            proxy_url: proxy_url.unwrap_or_else(|| "http://127.0.0.1:4444".to_string()),
        }
    }

    /// Test I2P connection
    pub async fn test_i2p_connection(&self) -> NozyResult<bool> {
        let client = self.create_client()?;

        // Test I2P by trying to reach an I2P service
        // Using I2P's router console as test endpoint
        match client
            .get("http://127.0.0.1:7657/")
            .timeout(Duration::from_secs(10))
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

impl PrivacyProxy for I2PProxy {
    fn proxy_url(&self) -> String {
        self.proxy_url.clone()
    }

    fn test_connection(&self) -> impl std::future::Future<Output = NozyResult<bool>> + Send {
        self.test_i2p_connection()
    }

    fn create_client(&self) -> NozyResult<Client> {
        Client::builder()
            .proxy(
                reqwest::Proxy::http(&self.proxy_url).map_err(|e| {
                    NozyError::NetworkError(format!("Invalid I2P proxy URL: {}", e))
                })?,
            )
            .timeout(Duration::from_secs(60))
            .build()
            .map_err(|e| NozyError::NetworkError(format!("Failed to create I2P client: {}", e)))
    }

    fn network_type(&self) -> PrivacyNetwork {
        PrivacyNetwork::I2P
    }
}
