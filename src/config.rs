use crate::error::{NozyError, NozyResult};
use crate::paths::get_wallet_config_path;
use serde::{Deserialize, Serialize};
use std::env;
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

    /// Operator-controlled Zebra RPC endpoints allowed to connect directly (no Tor) when
    /// `privacy_network.require_privacy_network` is true. Public/community nodes should not
    /// be listed here; use Tor/onion for those.
    #[serde(default)]
    pub trusted_zebra_urls: Vec<String>,

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

    #[serde(default)]
    pub keystone: crate::keystone::KeystoneWalletConfig,
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

    /// When true (or `NOZY_BROADCAST_VIA_NYM_MIXNET=1`), remote `sendrawtransaction`
    /// goes through the Nym smolmix helper subprocess. Local/LAN URLs stay direct.
    #[serde(default = "default_false")]
    pub broadcast_via_nym_mixnet: bool,
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
            broadcast_via_nym_mixnet: false,
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
            trusted_zebra_urls: Vec::new(),
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
            keystone: crate::keystone::KeystoneWalletConfig::default(),
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

    let mut config = if config_path.exists() {
        match fs::read_to_string(&config_path) {
            Ok(content) => {
                // PowerShell often writes UTF-8 BOM; serde_json rejects it at column 1.
                let content = content.strip_prefix('\u{feff}').unwrap_or(&content);
                if content.trim().is_empty() {
                    WalletConfig::default()
                } else {
                    match serde_json::from_str::<WalletConfig>(content) {
                        Ok(config) => config,
                        Err(e) => {
                            let msg = e.to_string();
                            if msg.contains("EOF while parsing") {
                                WalletConfig::default()
                            } else {
                                eprintln!(
                                    "Warning: Failed to parse config.json: {}. Using defaults.",
                                    e
                                );
                                WalletConfig::default()
                            }
                        }
                    }
                }
            }
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
    };

    if let Ok(url) = env::var("ZEBRA_RPC_URL") {
        let trimmed = url.trim();
        if !trimmed.is_empty() {
            config.zebra_url = trimmed.to_string();
        }
    }

    config
}

/// Normalize Zebra RPC URLs for comparison (config trust list, overrides).
pub fn normalize_zebra_rpc_url(url: &str) -> String {
    let mut url = url.trim().to_string();
    url = url.replace("..", ".");
    url = url.replace(":///", "://");
    if url.starts_with("http://") {
        url = url.replace("http:///", "http://");
    } else if url.starts_with("https://") {
        url = url.replace("https:///", "https://");
    }

    if url.starts_with("http://") || url.starts_with("https://") {
        return url;
    }

    if let Some((host, port_str)) = url.split_once(':') {
        if let Ok(port) = port_str.parse::<u16>() {
            if port == 443 {
                return format!("https://{host}");
            }
            return format!("http://{url}");
        }
        let _ = host;
    }

    if url.starts_with("127.0.0.1")
        || url.starts_with("localhost")
        || url.starts_with("172.")
        || url.starts_with("10.")
        || url.starts_with("192.168.")
    {
        format!("http://{url}")
    } else {
        format!("https://{url}")
    }
}

impl WalletConfig {
    /// Config with an optional runtime Zebra URL override (API/CLI).
    pub fn with_zebra_url_override(mut self, zebra_url: Option<String>) -> Self {
        if let Some(url) = zebra_url {
            let trimmed = url.trim();
            if !trimmed.is_empty() {
                self.zebra_url = trimmed.to_string();
            }
        }
        self
    }

    /// True when `url` is the configured primary endpoint, listed in `trusted_zebra_urls`,
    /// or matches either on host:port (http/https scheme ignored for RPC ports).
    pub fn is_trusted_zebra_url(&self, url: &str) -> bool {
        if zebra_rpc_endpoints_match(url, &self.zebra_url) {
            return true;
        }
        self.trusted_zebra_urls
            .iter()
            .any(|trusted| zebra_rpc_endpoints_match(url, trusted))
    }

    /// Ensure a remote operator URL is allowlisted for direct RPC when privacy mode is on.
    pub fn ensure_trusted_zebra_url(&mut self, url: &str) {
        let normalized = normalize_zebra_rpc_url(url);
        if normalized.is_empty() {
            return;
        }
        if self
            .trusted_zebra_urls
            .iter()
            .any(|trusted| zebra_rpc_endpoints_match(trusted, &normalized))
        {
            return;
        }
        self.trusted_zebra_urls.push(normalized);
    }
}

/// Host:port identity for Zebra RPC URLs (scheme ignored so http/https forms match).
pub fn zebra_rpc_endpoint_key(url: &str) -> Option<String> {
    let normalized = normalize_zebra_rpc_url(url);
    let parsed = reqwest::Url::parse(&normalized).ok()?;
    let host = parsed.host_str()?.to_ascii_lowercase();
    let port = parsed.port_or_known_default()?;
    Some(format!("{host}:{port}"))
}

pub fn zebra_rpc_endpoints_match(a: &str, b: &str) -> bool {
    if normalize_zebra_rpc_url(a) == normalize_zebra_rpc_url(b) {
        return true;
    }
    match (zebra_rpc_endpoint_key(a), zebra_rpc_endpoint_key(b)) {
        (Some(ka), Some(kb)) => ka == kb,
        _ => false,
    }
}

#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    fn trusted_url_matches_normalized_forms() {
        let mut config = WalletConfig::default();
        config
            .trusted_zebra_urls
            .push("http://node.example.com:8232".to_string());
        assert!(config.is_trusted_zebra_url("http://node.example.com:8232"));
        assert!(config.is_trusted_zebra_url("node.example.com:8232"));
    }

    #[test]
    fn configured_zebra_url_is_trusted_without_allowlist_entry() {
        let mut config = WalletConfig::default();
        config.zebra_url = "http://vps.example.com:8232".to_string();
        config.privacy_network.require_privacy_network = true;
        assert!(config.is_trusted_zebra_url("http://vps.example.com:8232"));
        assert!(config.is_trusted_zebra_url("https://vps.example.com:8232"));
    }

    #[test]
    fn ensure_trusted_dedupes_by_host_port() {
        let mut config = WalletConfig::default();
        config.ensure_trusted_zebra_url("http://vps.example.com:8232");
        config.ensure_trusted_zebra_url("https://vps.example.com:8232");
        assert_eq!(config.trusted_zebra_urls.len(), 1);
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
