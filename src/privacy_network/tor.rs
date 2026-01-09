// Tor Proxy Support
// Handles Tor SOCKS5 proxy connections

use crate::error::{NozyError, NozyResult};
use crate::privacy_network::proxy::{PrivacyNetwork, PrivacyProxy};
use reqwest::Client;
use std::time::Duration;

pub struct TorProxy {
    proxy_url: String,
}

impl TorProxy {
    pub fn new(proxy_url: Option<String>) -> Self {
        Self {
            proxy_url: proxy_url.unwrap_or_else(|| "socks5://127.0.0.1:9050".to_string()),
        }
    }

    /// Test Tor connection by checking if we can reach Tor network
    pub async fn test_tor_connection(&self) -> NozyResult<bool> {
        let client = self.create_client()?;

        // Use Tor's check service
        match client
            .get("https://check.torproject.org/api/ip")
            .timeout(Duration::from_secs(15))
            .send()
            .await
        {
            Ok(response) => {
                if let Ok(text) = response.text().await {
                    // Check if response indicates we're using Tor
                    Ok(text.contains("IsTor") || text.contains("true"))
                } else {
                    Ok(false)
                }
            }
            Err(_) => Ok(false),
        }
    }

    /// Get current IP through Tor (for verification)
    pub async fn get_tor_ip(&self) -> NozyResult<String> {
        let client = self.create_client()?;

        let response = client
            .get("https://api.ipify.org?format=json")
            .timeout(Duration::from_secs(15))
            .send()
            .await
            .map_err(|e| NozyError::NetworkError(format!("Tor connection failed: {}", e)))?;

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| NozyError::NetworkError(format!("Failed to parse response: {}", e)))?;

        json.get("ip")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| NozyError::NetworkError("Invalid IP response".to_string()))
    }
}

impl PrivacyProxy for TorProxy {
    fn proxy_url(&self) -> String {
        self.proxy_url.clone()
    }

    async fn test_connection(&self) -> NozyResult<bool> {
        self.test_tor_connection().await
    }

    fn create_client(&self) -> NozyResult<Client> {
        Client::builder()
            .proxy(
                reqwest::Proxy::all(&self.proxy_url).map_err(|e| {
                    NozyError::NetworkError(format!("Invalid Tor proxy URL: {}", e))
                })?,
            )
            .timeout(Duration::from_secs(60))
            .build()
            .map_err(|e| NozyError::NetworkError(format!("Failed to create Tor client: {}", e)))
    }

    fn network_type(&self) -> PrivacyNetwork {
        PrivacyNetwork::Tor
    }
}
