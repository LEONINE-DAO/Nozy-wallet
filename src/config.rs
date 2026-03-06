use crate::error::{NozyError, NozyResult};
use crate::paths::get_wallet_config_path;
use serde::{Deserialize, Serialize};
use std::fs;

/// Nozy talking to Zebra & Crosslink

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BackendKind {
    Zebra,
    Crosslink,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    JsonRpc,
    Grpc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfig {
    #[serde(default = "default_zebra_url")]
    pub zebra_url: String,

    #[serde(default)]
    pub zebra_fallback_urls: Vec<String>,

    #[serde(default = "default_crosslink_url")]
    pub crosslink_url: String,

    #[serde(default = "default_network")]
    pub network: String,

    pub last_scan_height: Option<u32>,

    #[serde(default = "default_theme")]
    pub theme: String,

    #[serde(default = "default_backend")]
    pub backend: BackendKind,

    #[serde(default = "default_protocol")]
    pub protocol: Protocol,

    #[serde(default)]
    pub privacy_network: PrivacyNetworkConfig,

    #[serde(default)]
    pub zk_verification: crate::monero_zk_verifier::types::ZkVerificationConfig,

    #[serde(default)]
    pub secret_network: SecretNetworkConfig,

    #[serde(default)]
    pub swap: SwapConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretNetworkConfig {
    #[serde(default = "default_secret_lcd_url")]
    pub lcd_url: String,

    #[serde(default)]
    pub address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapConfig {
    #[serde(default = "default_false")]
    pub auto_churn: bool,

    #[serde(default = "default_swap_api_url")]
    pub api_url: String,

    #[serde(default)]
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyNetworkConfig {
    #[serde(default = "default_true")]
    pub tor_enabled: bool,

    #[serde(default = "default_tor_proxy")]
    pub tor_proxy: String,

    #[serde(default = "default_false")]
    pub i2p_enabled: bool,

    #[serde(default = "default_i2p_proxy")]
    pub i2p_proxy: String,

    #[serde(default = "default_preferred_network")]
    pub preferred_network: String,

    #[serde(default = "default_true")]
    pub require_privacy_network: bool,
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_tor_proxy() -> String {
    "socks5://127.0.0.1:9050".to_string()
}

fn default_i2p_proxy() -> String {
    "http://127.0.0.1:4444".to_string()
}

fn default_preferred_network() -> String {
    "tor".to_string()
}

impl Default for PrivacyNetworkConfig {
    fn default() -> Self {
        Self {
            tor_enabled: true,
            tor_proxy: default_tor_proxy(),
            i2p_enabled: false,
            i2p_proxy: default_i2p_proxy(),
            preferred_network: default_preferred_network(),
            require_privacy_network: true,
        }
    }
}

fn default_zebra_url() -> String {
    "http://127.0.0.1:8232".to_string()
}

fn default_crosslink_url() -> String {
    String::new()
}

fn default_network() -> String {
    "mainnet".to_string()
}

fn default_theme() -> String {
    "dark".to_string()
}

fn default_backend() -> BackendKind {
    BackendKind::Zebra
}

fn default_protocol() -> Protocol {
    Protocol::JsonRpc
}

fn default_secret_lcd_url() -> String {
    "https://api.secretapi.io".to_string()
}

fn default_swap_api_url() -> String {
    "https://api.swap-service.example.com".to_string()
}

impl Default for SecretNetworkConfig {
    fn default() -> Self {
        Self {
            lcd_url: default_secret_lcd_url(),
            address: None,
        }
    }
}

impl Default for WalletConfig {
    fn default() -> Self {
        Self {
            zebra_url: default_zebra_url(),
            zebra_fallback_urls: Vec::new(),
            crosslink_url: default_crosslink_url(),
            network: default_network(),
            last_scan_height: None,
            theme: default_theme(),
            backend: default_backend(),
            protocol: default_protocol(),
            privacy_network: PrivacyNetworkConfig::default(),
            zk_verification: crate::monero_zk_verifier::types::ZkVerificationConfig::default(),
            secret_network: SecretNetworkConfig::default(),
            swap: SwapConfig::default(),
        }
    }
}

impl Default for SwapConfig {
    fn default() -> Self {
        Self {
            auto_churn: false,
            api_url: default_swap_api_url(),
            api_key: None,
        }
    }
}

pub fn load_config() -> WalletConfig {
    let config_path = get_wallet_config_path();

    if config_path.exists() {
        match fs::read_to_string(&config_path) {
            Ok(content) => match serde_json::from_str::<WalletConfig>(&content) {
                Ok(config) => config,
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to parse config.json: {}. Using defaults.",
                        e
                    );
                    WalletConfig::default()
                }
            },
            Err(e) => {
                eprintln!(
                    "Warning: Failed to read config.json: {}. Using defaults.",
                    e
                );
                WalletConfig::default()
            }
        }
    } else {
        WalletConfig::default()
    }
}

pub fn save_config(config: &WalletConfig) -> NozyResult<()> {
    let config_path = get_wallet_config_path();

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| NozyError::Storage(format!("Failed to create config directory: {}", e)))?;
    }

    let serialized = serde_json::to_string_pretty(config)
        .map_err(|e| NozyError::Storage(format!("Failed to serialize config: {}", e)))?;

    fs::write(&config_path, serialized)
        .map_err(|e| NozyError::Storage(format!("Failed to write config: {}", e)))?;

    Ok(())
}

pub fn update_last_scan_height(height: u32) -> NozyResult<()> {
    let mut config = load_config();
    config.last_scan_height = Some(height);
    save_config(&config)
}

pub fn ensure_local_zebra_node() -> NozyResult<()> {
    let mut config = load_config();
    if config.zebra_url != "http://127.0.0.1:8232"
        && !config.zebra_url.contains("127.0.0.1")
        && !config.zebra_url.contains("localhost")
    {
        config.zebra_url = "http://127.0.0.1:8232".to_string();
        save_config(&config)?;
    }
    Ok(())
}
