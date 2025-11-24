use crate::error::{NozyError, NozyResult};
use crate::paths::get_wallet_config_path;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfig {
    #[serde(default = "default_zebra_url")]
    pub zebra_url: String,
    #[serde(default = "default_network")]
    pub network: String,
    pub last_scan_height: Option<u32>,
    #[serde(default = "default_theme")]
    pub theme: String,
}

fn default_zebra_url() -> String {
    "http://127.0.0.1:8232".to_string()
}

fn default_network() -> String {
    "mainnet".to_string()
}

fn default_theme() -> String {
    "dark".to_string()
}

impl Default for WalletConfig {
    fn default() -> Self {
        Self {
            zebra_url: default_zebra_url(),
            network: default_network(),
            last_scan_height: None,
            theme: default_theme(),
        }
    }
}

pub fn load_config() -> WalletConfig {
    let config_path = get_wallet_config_path();
    
    if config_path.exists() {
        match fs::read_to_string(&config_path) {
            Ok(content) => {
                match serde_json::from_str::<WalletConfig>(&content) {
                    Ok(config) => config,
                    Err(e) => {
                        eprintln!("Warning: Failed to parse config.json: {}. Using defaults.", e);
                        WalletConfig::default()
                    }
                }
            },
            Err(e) => {
                eprintln!("Warning: Failed to read config.json: {}. Using defaults.", e);
                WalletConfig::default()
            }
        }
    } else {
        WalletConfig::default()
    }
}

pub fn save_config(config: &WalletConfig) -> NozyResult<()> {
    let config_path = get_wallet_config_path();
    
    // Ensure parent directory exists
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
