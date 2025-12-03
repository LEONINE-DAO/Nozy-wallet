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
pub struct WalletConfig {
    #[serde(default = "default_zebra_url")]
    pub zebra_url: String,

   
    /// If empty, `zebra_url` will be used as a fallback when backend == Crosslink.
    #[serde(default = "default_crosslink_url")]
    pub crosslink_url: String,

    #[serde(default = "default_network")]
    pub network: String,

    pub last_scan_height: Option<u32>,

    #[serde(default = "default_theme")]
    pub theme: String,

    
    /// Defaults to `zebra` to preserve existing behavior.
    #[serde(default = "default_backend")]
    pub backend: BackendKind,
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

impl Default for WalletConfig {
    fn default() -> Self {
        Self {
            zebra_url: default_zebra_url(),
            crosslink_url: default_crosslink_url(),
            network: default_network(),
            last_scan_height: None,
            theme: default_theme(),
            backend: default_backend(),
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
    if config.zebra_url != "http://127.0.0.1:8232" && !config.zebra_url.contains("127.0.0.1") && !config.zebra_url.contains("localhost") {
        config.zebra_url = "http://127.0.0.1:8232".to_string();
        save_config(&config)?;
    }
    Ok(())
}