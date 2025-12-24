// Privacy Proxy Interface
// Unified interface for Tor and I2P proxies

use crate::error::{NozyError, NozyResult};
use reqwest::Client;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum PrivacyNetwork {
    Tor,
    I2P,
    None, // Direct connection (not recommended)
}

#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub network: PrivacyNetwork,
    pub proxy_url: String,
    pub enabled: bool,
}

pub trait PrivacyProxy: Send + Sync {
    /// Get proxy URL for HTTP client
    fn proxy_url(&self) -> String;
    
    /// Test if proxy is working
    async fn test_connection(&self) -> NozyResult<bool>;
    
    /// Create HTTP client with proxy configured
    fn create_client(&self) -> NozyResult<Client>;
    
    /// Get network type
    fn network_type(&self) -> PrivacyNetwork;
}

impl ProxyConfig {
    pub fn new_tor(proxy_url: Option<String>) -> Self {
        Self {
            network: PrivacyNetwork::Tor,
            proxy_url: proxy_url.unwrap_or_else(|| "socks5://127.0.0.1:9050".to_string()),
            enabled: true,
        }
    }
    
    pub fn new_i2p(proxy_url: Option<String>) -> Self {
        Self {
            network: PrivacyNetwork::I2P,
            proxy_url: proxy_url.unwrap_or_else(|| "http://127.0.0.1:4444".to_string()),
            enabled: true,
        }
    }
    
    pub fn disabled() -> Self {
        Self {
            network: PrivacyNetwork::None,
            proxy_url: String::new(),
            enabled: false,
        }
    }
    
    /// Create HTTP client with proxy
    pub fn create_client(&self) -> NozyResult<Client> {
        let mut builder = Client::builder()
            .timeout(Duration::from_secs(60));
        
        if self.enabled && !self.proxy_url.is_empty() {
            builder = builder.proxy(
                reqwest::Proxy::all(&self.proxy_url)
                    .map_err(|e| NozyError::NetworkError(format!("Invalid proxy URL: {}", e)))?
            );
        }
        
        builder.build()
            .map_err(|e| NozyError::NetworkError(format!("Failed to create HTTP client: {}", e)))
    }
    
    /// Test proxy connection
    pub async fn test_connection(&self) -> NozyResult<bool> {
        if !self.enabled {
            return Ok(false);
        }
        
        let client = self.create_client()?;
        
        // Test by checking external IP (should be different from real IP)
        // Using a privacy-friendly IP check service
        match client
            .get("https://check.torproject.org/api/ip")
            .timeout(Duration::from_secs(10))
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
    
    /// Auto-detect available privacy network
    pub async fn auto_detect() -> Self {
        // Try Tor first
        let tor_config = Self::new_tor(None);
        if tor_config.test_connection().await.unwrap_or(false) {
            println!("✅ Tor detected and working");
            return tor_config;
        }
        
        // Try I2P
        let i2p_config = Self::new_i2p(None);
        if i2p_config.test_connection().await.unwrap_or(false) {
            println!("✅ I2P detected and working");
            return i2p_config;
        }
        
        println!("⚠️  No privacy network detected (Tor/I2P)");
        Self::disabled()
    }
}
