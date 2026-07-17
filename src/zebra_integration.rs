use crate::config::{normalize_zebra_rpc_url, BackendKind, Protocol, WalletConfig};
use crate::error::{NozyError, NozyResult};
use crate::grpc_client::ZebraGrpcClient;
use crate::shielded_pool::ShieldedPool;
use crate::zebra_tree_rpc::{
    parse_z_get_subtrees_by_index, parse_z_gettreestate_orchard, parse_z_gettreestate_pool,
    OrchardTreestateParsed, ShieldedTreestateParsed, ZebraSubtreesByIndex,
};
use hex;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Arc;

/// Transient Zebra JSON-RPC transport errors worth retrying with exponential backoff.
fn is_retryable_zebra_transport_error(msg: &str) -> bool {
    let m = msg.to_lowercase();
    m.contains("failed to connect")
        || m.contains("connection refused")
        || m.contains("connection failed")
        || m.contains("connection reset")
        || m.contains("connection closed")
        || m.contains("broken pipe")
        || m.contains("timeout")
        || m.contains("error sending request")
        // reqwest body read/decode failures under load (Gilmore: "error decoding response body")
        || m.contains("failed to read response")
        || m.contains("decoding response body")
        || m.contains("unexpected eof")
}

/// Detected full-node implementation behind the Zebra-family JSON-RPC surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChainNodeKind {
    Zebra,
    Zakura,
    Unknown,
}

impl ChainNodeKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Zebra => "Zebra",
            Self::Zakura => "Zakura",
            Self::Unknown => "Zcash node",
        }
    }

    /// Classify from `getnetworkinfo` → `subversion` (or similar) string.
    pub fn from_subversion(subversion: &str) -> Self {
        let s = subversion.to_ascii_lowercase();
        if s.contains("zakura") {
            Self::Zakura
        } else if s.contains("zebra") || s.contains("zebrad") {
            Self::Zebra
        } else {
            Self::Unknown
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZebraConnectionMode {
    DirectLocal,
    DirectTrusted,
    DirectRemote,
    TorProxy,
    I2pProxy,
    /// Opt-in smolmix path for remote `sendrawtransaction` (`NOZY_BROADCAST_VIA_NYM_MIXNET=1`).
    NymMixnet,
    Blocked,
}

impl ZebraConnectionMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DirectLocal => "direct_local",
            Self::DirectTrusted => "direct_trusted",
            Self::DirectRemote => "direct_remote",
            Self::TorProxy => "tor_proxy",
            Self::I2pProxy => "i2p_proxy",
            Self::NymMixnet => "nym_mixnet",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ZebraClient {
    url: String,
    /// Fallback Zebra RPC URLs tried in order when primary fails after retries (fault tolerance).
    fallback_urls: Vec<String>,
    /// Nozy is talking to both `Zebra` and `Crosslink` use the same JSON-RPC
    /// interface from Nozy's perspective, but this flag lets us adapt
    /// behaviour later (for example, staking-specific calls) without
    /// changing call sites.
    #[allow(dead_code)]
    backend: BackendKind,
    protocol: Protocol,
    client: Arc<reqwest::Client>,
    rpc_auth: Option<(String, String)>,
    privacy_proxy_url: Option<String>,
    block_remote_without_privacy: bool,
    privacy_block_reason: Option<String>,
    connection_mode: ZebraConnectionMode,
    /// Remote sendraw over Nym smolmix helper (env and/or config).
    broadcast_via_nym_mixnet: bool,
    #[allow(dead_code)]
    grpc_client: Option<Arc<ZebraGrpcClient>>,
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
    fn is_local_host(host: &str) -> bool {
        let host_lc = host.trim().trim_matches(['[', ']']).to_ascii_lowercase();
        if host_lc == "localhost" {
            return true;
        }

        if let Ok(ip) = host_lc.parse::<IpAddr>() {
            match ip {
                IpAddr::V4(v4) => {
                    let octets = v4.octets();
                    v4.is_loopback() || v4.is_private() || (octets[0] == 169 && octets[1] == 254)
                    // link-local
                }
                IpAddr::V6(v6) => {
                    v6.is_loopback() || v6.is_unique_local() || v6.is_unicast_link_local()
                }
            }
        } else {
            false
        }
    }

    /// True when `url` targets loopback / private / link-local Zebrad (desktop preferred path).
    pub fn url_is_local(url: &str) -> bool {
        Self::is_local_url(url)
    }

    fn is_local_url(url: &str) -> bool {
        if let Ok(parsed) = reqwest::Url::parse(url) {
            if let Some(host) = parsed.host_str() {
                return Self::is_local_host(host);
            }
        }

        // Fallback for raw host[:port] values.
        let host = url
            .split("://")
            .nth(1)
            .unwrap_or(url)
            .split('/')
            .next()
            .unwrap_or(url)
            .split(':')
            .next()
            .unwrap_or(url);
        Self::is_local_host(host)
    }

    fn build_http_client(
        timeout_secs: u64,
        proxy_url: Option<&str>,
    ) -> Result<reqwest::Client, String> {
        let mut builder = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .tcp_keepalive(std::time::Duration::from_secs(60))
            // Sync opens many concurrent JSON-RPC calls; a tiny idle pool caused extra
            // connect/teardown under load (localhost included).
            .pool_max_idle_per_host(32)
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .danger_accept_invalid_certs(false);

        if let Some(url) = proxy_url {
            let proxy = reqwest::Proxy::all(url)
                .map_err(|e| format!("Invalid privacy proxy URL '{}': {}", url, e))?;
            builder = builder.proxy(proxy);
        }

        builder
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))
    }

    fn selected_proxy_from_config(config: &WalletConfig) -> Option<String> {
        let privacy = &config.privacy_network;
        let preferred = privacy.preferred_network.to_lowercase();

        match preferred.as_str() {
            "i2p" if privacy.i2p_enabled && !privacy.i2p_proxy.is_empty() => {
                Some(privacy.i2p_proxy.clone())
            }
            "tor" if privacy.tor_enabled && !privacy.tor_proxy.is_empty() => {
                Some(privacy.tor_proxy.clone())
            }
            _ if privacy.tor_enabled && !privacy.tor_proxy.is_empty() => {
                Some(privacy.tor_proxy.clone())
            }
            _ if privacy.i2p_enabled && !privacy.i2p_proxy.is_empty() => {
                Some(privacy.i2p_proxy.clone())
            }
            _ => None,
        }
    }

    fn parse_cookie_pair(cookie: &str) -> Option<(String, String)> {
        let trimmed = cookie.trim();
        let (user, pass) = trimmed.split_once(':')?;
        if user.is_empty() || pass.is_empty() {
            return None;
        }
        Some((user.to_string(), pass.to_string()))
    }

    fn candidate_cookie_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        for env_key in ["ZEBRA_RPC_COOKIE_PATH", "ZAKURA_RPC_COOKIE_PATH"] {
            if let Ok(path) = std::env::var(env_key) {
                if !path.trim().is_empty() {
                    paths.push(PathBuf::from(path));
                }
            }
        }

        for env_key in ["ZEBRA_RPC_COOKIE", "ZAKURA_RPC_COOKIE"] {
            if let Ok(cookie_inline) = std::env::var(env_key) {
                if let Some(pair) = Self::parse_cookie_pair(&cookie_inline) {
                    // Inline cookie env is handled in `resolve_rpc_auth`; no file path to add.
                    let _ = pair;
                }
            }
        }

        for cache_name in ["zebra", "zakura"] {
            if let Ok(home) = std::env::var("HOME") {
                paths.push(
                    PathBuf::from(&home)
                        .join(".cache")
                        .join(cache_name)
                        .join(".cookie"),
                );
            }
            if let Ok(profile) = std::env::var("USERPROFILE") {
                paths.push(
                    PathBuf::from(&profile)
                        .join(".cache")
                        .join(cache_name)
                        .join(".cookie"),
                );
            }
        }

        paths
    }

    fn resolve_rpc_auth() -> Option<(String, String)> {
        for (user_key, pass_key) in [
            ("ZEBRA_RPC_USER", "ZEBRA_RPC_PASS"),
            ("ZAKURA_RPC_USER", "ZAKURA_RPC_PASS"),
        ] {
            if let (Ok(user), Ok(pass)) = (std::env::var(user_key), std::env::var(pass_key)) {
                if !user.trim().is_empty() && !pass.trim().is_empty() {
                    return Some((user, pass));
                }
            }
        }

        for env_key in ["ZEBRA_RPC_COOKIE", "ZAKURA_RPC_COOKIE"] {
            if let Ok(cookie_inline) = std::env::var(env_key) {
                if let Some(pair) = Self::parse_cookie_pair(&cookie_inline) {
                    return Some(pair);
                }
            }
        }

        for path in Self::candidate_cookie_paths() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Some(pair) = Self::parse_cookie_pair(&content) {
                    return Some(pair);
                }
            }
        }

        None
    }

    pub fn new(url: String) -> Self {
        Self::new_with_backend(url, BackendKind::Zebra)
    }

    pub fn new_with_backend(url: String, backend: BackendKind) -> Self {
        Self::new_with_backend_and_protocol(url, backend, Protocol::JsonRpc)
    }

    pub fn protocol(&self) -> Protocol {
        self.protocol.clone()
    }

    pub fn new_with_backend_and_protocol(
        url: String,
        backend: BackendKind,
        protocol: Protocol,
    ) -> Self {
        let url = Self::normalize_url(url.clone());

        let is_local = Self::is_local_url(&url);
        let timeout_secs = if is_local { 10 } else { 30 };

        let client =
            Self::build_http_client(timeout_secs, None).unwrap_or_else(|_| reqwest::Client::new());

        Self {
            url,
            fallback_urls: Vec::new(),
            backend,
            protocol,
            client: Arc::new(client),
            rpc_auth: Self::resolve_rpc_auth(),
            privacy_proxy_url: None,
            block_remote_without_privacy: false,
            privacy_block_reason: None,
            connection_mode: if is_local {
                ZebraConnectionMode::DirectLocal
            } else {
                ZebraConnectionMode::DirectRemote
            },
            broadcast_via_nym_mixnet: false,
            grpc_client: None,
        }
    }

    pub fn connection_mode(&self) -> ZebraConnectionMode {
        self.connection_mode
    }

    pub fn from_config(config: &WalletConfig) -> Self {
        Self::from_config_with_url(config, None)
    }

    /// Unified wallet Zebra client: applies privacy policy, trusted operator URLs, and Tor/I2P.
    pub fn from_config_with_url(config: &WalletConfig, zebra_url_override: Option<&str>) -> Self {
        let effective = config
            .clone()
            .with_zebra_url_override(zebra_url_override.map(|s| s.to_string()));
        Self::build_from_config(&effective)
    }

    fn build_from_config(config: &WalletConfig) -> Self {
        let (backend, url, fallback_urls) = match &config.backend {
            BackendKind::Zebra => (
                BackendKind::Zebra,
                config.zebra_url.clone(),
                config.zebra_fallback_urls.clone(),
            ),
            BackendKind::Crosslink => {
                let url = if !config.crosslink_url.is_empty() {
                    config.crosslink_url.clone()
                } else {
                    config.zebra_url.clone()
                };
                (BackendKind::Crosslink, url, Vec::new())
            }
        };

        let mut client = Self::new_with_backend_and_protocol(url, backend, config.protocol.clone());
        client.fallback_urls = fallback_urls
            .into_iter()
            .map(|u| Self::normalize_url(u))
            .filter(|u| !u.is_empty())
            .collect();

        let primary_local = Self::is_local_url(&client.url);
        let primary_trusted = config.is_trusted_zebra_url(&client.url);

        client.broadcast_via_nym_mixnet = crate::nym_mixnet_broadcast::mixnet_broadcast_requested(
            config.privacy_network.broadcast_via_nym_mixnet,
        );

        if primary_local || primary_trusted {
            client.connection_mode = if primary_local {
                ZebraConnectionMode::DirectLocal
            } else if client.broadcast_via_nym_mixnet {
                ZebraConnectionMode::NymMixnet
            } else {
                ZebraConnectionMode::DirectTrusted
            };
            return client;
        }

        let primary_is_remote = !primary_local;
        let any_fallback_remote = client.fallback_urls.iter().any(|u| !Self::is_local_url(u));
        let remote_rpc_possible = primary_is_remote || any_fallback_remote;

        let require_privacy = config.privacy_network.require_privacy_network;
        let selected_proxy = if remote_rpc_possible {
            Self::selected_proxy_from_config(config)
        } else {
            None
        };
        client.privacy_proxy_url = selected_proxy.clone();

        let timeout_secs = if primary_local { 10 } else { 30 };
        match selected_proxy.as_deref() {
            Some(proxy_url) if proxy_url.to_ascii_lowercase().contains("socks") => {
                client.connection_mode = ZebraConnectionMode::TorProxy;
            }
            Some(_) => {
                client.connection_mode = ZebraConnectionMode::I2pProxy;
            }
            None if remote_rpc_possible && !require_privacy => {
                client.connection_mode = ZebraConnectionMode::DirectRemote;
            }
            None if remote_rpc_possible => {
                client.connection_mode = ZebraConnectionMode::Blocked;
            }
            _ => {}
        }

        match selected_proxy {
            Some(proxy_url) => {
                match Self::build_http_client(timeout_secs, Some(proxy_url.as_str())) {
                    Ok(http_client) => {
                        client.client = Arc::new(http_client);
                        client.block_remote_without_privacy = false;
                        client.privacy_block_reason = None;
                    }
                    Err(proxy_err) => {
                        if require_privacy && remote_rpc_possible {
                            client.block_remote_without_privacy = true;
                            client.connection_mode = ZebraConnectionMode::Blocked;
                            client.privacy_block_reason = Some(format!(
                                "Remote Zebra RPC blocked: privacy proxy configuration is invalid ({proxy_err})"
                            ));
                        }
                    }
                }
            }
            None => {
                if require_privacy && remote_rpc_possible {
                    client.block_remote_without_privacy = true;
                    client.connection_mode = ZebraConnectionMode::Blocked;
                    client.privacy_block_reason = Some(
                        "Remote Zebra RPC blocked: require_privacy_network=true but no Tor/I2P proxy is configured. \
Add the node to trusted_zebra_urls for operator infrastructure, or enable Tor."
                            .to_string(),
                    );
                }
            }
        }

        client.broadcast_via_nym_mixnet = crate::nym_mixnet_broadcast::mixnet_broadcast_requested(
            config.privacy_network.broadcast_via_nym_mixnet,
        );
        if client.broadcast_via_nym_mixnet && !Self::is_local_url(&client.url) {
            // Display egress intent for remote submit; sync still uses Tor/direct rules above.
            client.connection_mode = ZebraConnectionMode::NymMixnet;
        }

        client
    }

    fn normalize_url(url: String) -> String {
        normalize_zebra_rpc_url(&url)
    }

    pub async fn get_block(&self, height: u32) -> NozyResult<HashMap<String, Value>> {
        let block_hash = self.get_block_hash(height).await?;
        self.get_block_by_hash(&block_hash, 2).await
    }

    pub async fn get_block_by_hash(
        &self,
        block_hash: &str,
        verbosity: u32,
    ) -> NozyResult<HashMap<String, Value>> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "getblock",
            "params": [block_hash, verbosity],
            "id": 1
        });

        let response: ZebraResponse<HashMap<String, Value>> = self.make_request(request).await?;

        if let Some(error) = response.error {
            return Err(NozyError::InvalidOperation(format!(
                "Zebra RPC error: {} (code: {})",
                error.message, error.code
            )));
        }

        response
            .result
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
            return Err(NozyError::InvalidOperation(format!(
                "Zebra RPC error: {} (code: {})",
                error.message, error.code
            )));
        }

        response
            .result
            .ok_or_else(|| NozyError::InvalidOperation("No block hash in response".to_string()))
    }

    pub async fn get_block_count(&self) -> NozyResult<u32> {
        match self.protocol {
            Protocol::Grpc => {
                // Initialize gRPC client if needed
                let grpc_client = self.get_grpc_client().await?;
                grpc_client.get_block_count().await
            }
            Protocol::JsonRpc => {
                let request = serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "getblockcount",
                    "params": [],
                    "id": 1
                });

                let response: ZebraResponse<u32> = self.make_request(request).await?;

                if let Some(error) = response.error {
                    return Err(NozyError::InvalidOperation(format!(
                        "Zebra RPC error: {} (code: {})",
                        error.message, error.code
                    )));
                }

                response.result.ok_or_else(|| {
                    NozyError::InvalidOperation("Invalid block height response".to_string())
                })
            }
        }
    }

    async fn get_grpc_client(&self) -> NozyResult<Arc<ZebraGrpcClient>> {
        let grpc_client = ZebraGrpcClient::new(self.url.clone()).await?;
        Ok(Arc::new(grpc_client))
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
            return Err(NozyError::InvalidOperation(format!(
                "Zebra RPC error: {} (code: {})",
                error.message, error.code
            )));
        }

        response
            .result
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
            return Err(NozyError::InvalidOperation(format!(
                "Zebra RPC error: {} (code: {})",
                error.message, error.code
            )));
        }

        response
            .result
            .ok_or_else(|| NozyError::InvalidOperation("No fee estimate in response".to_string()))
    }

    pub async fn broadcast_transaction(&self, raw_tx: &str) -> NozyResult<String> {
        if let Some(txid) = crate::nym_mixnet_broadcast::maybe_broadcast_via_nym_mixnet(
            &self.url,
            raw_tx,
            self.broadcast_via_nym_mixnet,
        )
        .await?
        {
            return Ok(txid);
        }

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "sendrawtransaction",
            "params": [raw_tx],
            "id": 1
        });

        let response: ZebraResponse<String> = self.make_request(request).await?;

        if let Some(error) = response.error {
            return Err(NozyError::InvalidOperation(format!(
                "Zebra RPC error: {} (code: {})",
                error.message, error.code
            )));
        }

        response.result.ok_or_else(|| {
            NozyError::InvalidOperation("No transaction hash in response".to_string())
        })
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
            return Err(NozyError::InvalidOperation(format!(
                "Zebra RPC error: {} (code: {})",
                error.message, error.code
            )));
        }

        response
            .result
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
            return Err(NozyError::InvalidOperation(format!(
                "Zebra RPC error: {} (code: {})",
                error.message, error.code
            )));
        }

        response
            .result
            .ok_or_else(|| NozyError::InvalidOperation("Invalid network info response".to_string()))
    }

    /// Total Orchard shielded pool from `getblockchaininfo` → `valuePools` (JSON-RPC).
    pub async fn get_orchard_pool_stats(&self) -> NozyResult<OrchardPoolStats> {
        let info = self.get_blockchain_info().await?;
        parse_orchard_pool_from_blockchain_info(&info)
    }

    /// Ironwood pool chain balance (NU6.3+). Returns error if node has no ironwood pool yet.
    pub async fn get_ironwood_pool_stats(&self) -> NozyResult<ShieldedPoolStats> {
        let info = self.get_blockchain_info().await?;
        parse_pool_from_blockchain_info(&info, "ironwood")
    }

    pub async fn get_blockchain_info(&self) -> NozyResult<HashMap<String, Value>> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "getblockchaininfo",
            "params": [],
            "id": 1
        });

        let response: ZebraResponse<HashMap<String, Value>> = self.make_request(request).await?;

        if let Some(error) = response.error {
            return Err(NozyError::InvalidOperation(format!(
                "Zebra RPC error: {} (code: {})",
                error.message, error.code
            )));
        }

        response.result.ok_or_else(|| {
            NozyError::InvalidOperation("Invalid getblockchaininfo response".to_string())
        })
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
            return Err(NozyError::InvalidOperation(format!(
                "Zebra RPC error: {} (code: {})",
                error.message, error.code
            )));
        }

        response.result.ok_or_else(|| {
            NozyError::InvalidOperation("No transaction data in response".to_string())
        })
    }

    /// True when `getrawtransaction` verbose reports the tx is confirmed in the active chain.
    pub async fn transaction_in_active_chain(&self, txid: &str) -> NozyResult<bool> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "getrawtransaction",
            "params": [txid, 1],
            "id": 1
        });

        let response: ZebraResponse<HashMap<String, Value>> = self.make_request(request).await?;

        if let Some(error) = response.error {
            if error.code == -5 {
                return Ok(false);
            }
            return Err(NozyError::InvalidOperation(format!(
                "Zebra RPC error: {} (code: {})",
                error.message, error.code
            )));
        }

        Ok(response
            .result
            .and_then(|result| {
                result
                    .get("in_active_chain")
                    .and_then(|value| value.as_bool())
            })
            .unwrap_or(false))
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
            return Err(NozyError::InvalidOperation(format!(
                "Zebra RPC error: {} (code: {})",
                error.message, error.code
            )));
        }

        response.result.ok_or_else(|| {
            NozyError::InvalidOperation("No decoded transaction data in response".to_string())
        })
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
            return Err(NozyError::InvalidOperation(format!(
                "Zebra RPC error: {} (code: {})",
                error.message, error.code
            )));
        }

        response
            .result
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
            return Err(NozyError::InvalidOperation(format!(
                "Zebra RPC error: {} (code: {})",
                error.message, error.code
            )));
        }

        response
            .result
            .ok_or_else(|| NozyError::InvalidOperation("No block template in response".to_string()))
    }

    pub async fn get_best_block_height(&self) -> NozyResult<u32> {
        self.get_block_count().await
    }

    /// Full Orchard treestate from `z_gettreestate` (JSON-RPC only).
    pub async fn get_orchard_treestate_parsed(
        &self,
        height: u32,
    ) -> NozyResult<OrchardTreestateParsed> {
        self.get_shielded_treestate_parsed(ShieldedPool::Orchard, height)
            .await
    }

    /// Full Ironwood treestate from `z_gettreestate` (JSON-RPC only).
    pub async fn get_ironwood_treestate_parsed(
        &self,
        height: u32,
    ) -> NozyResult<ShieldedTreestateParsed> {
        self.get_shielded_treestate_parsed(ShieldedPool::Ironwood, height)
            .await
    }

    /// Full shielded-pool treestate from `z_gettreestate` (JSON-RPC only).
    pub async fn get_shielded_treestate_parsed(
        &self,
        pool: ShieldedPool,
        height: u32,
    ) -> NozyResult<ShieldedTreestateParsed> {
        match self.protocol {
            Protocol::JsonRpc => {
                let request = serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "z_gettreestate",
                    "params": [height.to_string()],
                    "id": 1
                });

                let response: ZebraResponse<Value> = self.make_request(request).await?;

                if let Some(error) = response.error {
                    return Err(NozyError::InvalidOperation(format!(
                        "Zebra RPC error: {} (code: {})",
                        error.message, error.code
                    )));
                }

                let result = response.result.ok_or_else(|| {
                    NozyError::InvalidOperation("No z_gettreestate result".to_string())
                })?;

                if pool == ShieldedPool::Orchard {
                    parse_z_gettreestate_orchard(&result)
                } else {
                    parse_z_gettreestate_pool(&result, pool)
                }
            }
            Protocol::Grpc => Err(NozyError::InvalidOperation(
                "z_gettreestate requires JSON-RPC (shielded treestate)".to_string(),
            )),
        }
    }

    /// `z_getsubtreesbyindex` (JSON-RPC only). Pool is typically `"orchard"`.
    pub async fn z_get_subtrees_by_index(
        &self,
        pool: &str,
        start_index: u32,
        limit: Option<u32>,
    ) -> NozyResult<ZebraSubtreesByIndex> {
        match self.protocol {
            Protocol::JsonRpc => {
                let request = if let Some(lim) = limit {
                    serde_json::json!({
                        "jsonrpc": "2.0",
                        "method": "z_getsubtreesbyindex",
                        "params": [pool, start_index, lim],
                        "id": 1
                    })
                } else {
                    serde_json::json!({
                        "jsonrpc": "2.0",
                        "method": "z_getsubtreesbyindex",
                        "params": [pool, start_index],
                        "id": 1
                    })
                };

                let response: ZebraResponse<Value> = self.make_request(request).await?;

                if let Some(error) = response.error {
                    return Err(NozyError::InvalidOperation(format!(
                        "Zebra RPC error: {} (code: {})",
                        error.message, error.code
                    )));
                }

                let result = response.result.ok_or_else(|| {
                    NozyError::InvalidOperation("No z_getsubtreesbyindex result".to_string())
                })?;

                parse_z_get_subtrees_by_index(&result)
            }
            Protocol::Grpc => Err(NozyError::InvalidOperation(
                "z_getsubtreesbyindex requires JSON-RPC".to_string(),
            )),
        }
    }

    pub async fn get_orchard_tree_state(&self, height: u32) -> NozyResult<OrchardTreeState> {
        self.get_shielded_tree_state(ShieldedPool::Orchard, height)
            .await
    }

    pub async fn get_ironwood_tree_state(&self, height: u32) -> NozyResult<IronwoodTreeState> {
        self.get_shielded_tree_state(ShieldedPool::Ironwood, height)
            .await
    }

    pub async fn get_shielded_tree_state(
        &self,
        pool: ShieldedPool,
        height: u32,
    ) -> NozyResult<OrchardTreeState> {
        match self.protocol {
            Protocol::JsonRpc => {
                let parsed = self.get_shielded_treestate_parsed(pool, height).await?;
                Ok(OrchardTreeState {
                    height: parsed.height,
                    anchor: parsed.anchor,
                    commitment_count: parsed.commitment_count,
                })
            }
            Protocol::Grpc => self.get_orchard_tree_state_grpc_placeholder(height).await,
        }
    }

    async fn get_orchard_tree_state_grpc_placeholder(
        &self,
        height: u32,
    ) -> NozyResult<OrchardTreeState> {
        let block_hash = self.get_block_hash(height).await?;
        let mut anchor = [0u8; 32];
        let block_hash_bytes = hex::decode(&block_hash)
            .map_err(|e| NozyError::InvalidOperation(format!("Invalid block hash hex: {}", e)))?;
        let hash_len = block_hash_bytes.len().min(32);
        anchor[..hash_len].copy_from_slice(&block_hash_bytes[..hash_len]);
        Ok(OrchardTreeState {
            height,
            anchor,
            commitment_count: height as u64 * 100,
        })
    }

    async fn make_request<T>(&self, request: serde_json::Value) -> NozyResult<ZebraResponse<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        const MAX_RETRIES: u32 = 3;
        let urls_to_try: Vec<&str> = std::iter::once(self.url.as_str())
            .chain(self.fallback_urls.iter().map(String::as_str))
            .collect();
        let mut last_error = None;
        let mut privacy_notice_printed = false;

        for url in &urls_to_try {
            if self.block_remote_without_privacy && !Self::is_local_url(url) {
                let reason = self
                    .privacy_block_reason
                    .clone()
                    .unwrap_or_else(|| "Remote Zebra RPC blocked by privacy policy".to_string());
                if !privacy_notice_printed {
                    eprintln!(
                        "🛡️ Privacy policy active: blocking remote Zebra RPC to {}. {}. \
This blocks remote RPC only; localhost RPC remains allowed.",
                        url, reason
                    );
                    privacy_notice_printed = true;
                }
                last_error = Some(NozyError::NetworkError(reason));
                continue;
            }

            for attempt in 0..=MAX_RETRIES {
                match self.try_request(url, &request).await {
                    Ok(response) => return Ok(response),
                    Err(e) => {
                        last_error = Some(e);

                        if attempt < MAX_RETRIES {
                            let error_msg = match &last_error {
                                Some(NozyError::NetworkError(msg)) => msg,
                                _ => {
                                    return Err(last_error
                                        .expect("last_error should be Some at this point"));
                                }
                            };

                            if is_retryable_zebra_transport_error(error_msg) {
                                let delay_ms = 100 * (1 << attempt);
                                tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms))
                                    .await;
                                continue;
                            } else {
                                return Err(
                                    last_error.expect("last_error should be Some at this point")
                                );
                            }
                        }
                    }
                }
            }
        }

        let tried = urls_to_try.join(", ");
        let is_local = Self::is_local_url(&self.url);
        let error_msg = if is_local && urls_to_try.len() <= 1 {
            format!(
                "Failed to connect to local Zebra node at {} after {} attempts. \
                Make sure Zebra is running and RPC is enabled. \
                Check your ~/.config/zebrad.toml for: [rpc] listen_addr = \"127.0.0.1:8232\"",
                self.url,
                MAX_RETRIES + 1
            )
        } else {
            format!(
                "Failed to connect to Zebra node(s) [{}] after {} attempts each: {}",
                tried,
                MAX_RETRIES + 1,
                last_error
                    .as_ref()
                    .map(|e| format!("{}", e))
                    .unwrap_or_else(|| "Unknown error".to_string())
            )
        };

        Err(NozyError::NetworkError(error_msg))
    }

    async fn try_request<T>(
        &self,
        url: &str,
        request: &serde_json::Value,
    ) -> NozyResult<ZebraResponse<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        let mut req = self.client.post(url).json(request);
        if let Some((user, pass)) = &self.rpc_auth {
            req = req.basic_auth(user, Some(pass));
        }

        let response = req.send().await.map_err(|e| {
            let error_msg = if e.is_connect() {
                format!("Connection failed to {}: {}. Is Zebra running?", url, e)
            } else if e.is_timeout() {
                format!(
                    "Request timeout to {}. The node may be slow or overloaded.",
                    url
                )
            } else {
                format!("HTTP request failed: {}", e)
            };
            NozyError::NetworkError(error_msg)
        })?;

        if !response.status().is_success() {
            return Err(NozyError::NetworkError(format!(
                "HTTP error {} from {}. The Zebra RPC endpoint may not be configured correctly.",
                response.status(),
                url
            )));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| NozyError::NetworkError(format!("Failed to read response: {}", e)))?;

        let zebra_response: ZebraResponse<T> =
            serde_json::from_str(&response_text).map_err(|e| {
                NozyError::InvalidOperation(format!(
                    "Invalid JSON response from {}: {}. Response: {}",
                    url,
                    e,
                    &response_text[..response_text.len().min(200)]
                ))
            })?;

        Ok(zebra_response)
    }

    /// Detect node implementation from `getnetworkinfo` when available.
    pub async fn detect_chain_node_kind(&self) -> ChainNodeKind {
        match self.get_network_info().await {
            Ok(info) => {
                if let Some(Value::String(subver)) = info.get("subversion") {
                    return ChainNodeKind::from_subversion(subver);
                }
            }
            Err(_) => {}
        }
        ChainNodeKind::Unknown
    }

    /// NozyWallet-required RPC: treestate at chain tip (Orchard witness path).
    pub async fn probe_wallet_treestate(&self) -> NozyResult<()> {
        let height = self.get_block_count().await?;
        self.get_orchard_treestate_parsed(height).await?;
        Ok(())
    }

    pub async fn test_connection(&self) -> NozyResult<()> {
        match self.protocol {
            Protocol::Grpc => {
                let grpc_client = self.get_grpc_client().await?;
                grpc_client.test_connection().await?;
                println!(
                    "✅ Successfully connected to Zebra node via gRPC at {}",
                    self.url
                );
            }
            Protocol::JsonRpc => {
                let block_count = self.get_block_count().await?;
                let kind = self.detect_chain_node_kind().await;
                println!(
                    "✅ Successfully connected to {} node at {}",
                    kind.label(),
                    self.url
                );
                println!("   Current block height: {}", block_count);
            }
        }
        Ok(())
    }

    pub async fn broadcast_transaction_bytes(&self, raw_transaction: &[u8]) -> NozyResult<String> {
        let tx_hex = hex::encode(raw_transaction);

        self.broadcast_transaction(&tx_hex).await
    }

    pub async fn get_transaction_details(&self, txid: &str) -> NozyResult<serde_json::Value> {
        let raw_tx = self.get_raw_transaction(txid).await?;

        Ok(serde_json::json!({"raw": raw_tx}))
    }

    pub async fn get_transaction_info(&self, txid: &str) -> NozyResult<TransactionInfo> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "getrawtransaction",
            "params": [txid, 1],
            "id": 1
        });

        let response: ZebraResponse<serde_json::Value> = self.make_request(request).await?;

        if let Some(error) = response.error {
            return Err(NozyError::InvalidOperation(format!(
                "Zebra RPC error: {} (code: {})",
                error.message, error.code
            )));
        }

        let tx_data = response.result.ok_or_else(|| {
            NozyError::InvalidOperation("No transaction data in response".to_string())
        })?;

        let block_height = parse_tx_block_height(&tx_data);
        let block_hash = parse_tx_block_hash(&tx_data);

        let confirmations =
            if let Some(conf) = tx_data.get("confirmations").and_then(|v| v.as_u64()) {
                conf as u32
            } else if let Some(height) = block_height {
                let current_height = self.get_block_count().await.unwrap_or(0);
                current_height.saturating_sub(height) + 1
            } else {
                0
            };

        Ok(TransactionInfo {
            txid: txid.to_string(),
            block_height,
            block_hash,
            confirmations,
            in_mempool: block_height.is_none(),
        })
    }

    pub async fn check_transaction_exists(&self, txid: &str) -> NozyResult<bool> {
        match self.get_transaction_info(txid).await {
            Ok(_) => Ok(true),
            Err(NozyError::InvalidOperation(msg)) if msg.contains("No transaction") => Ok(false),
            Err(e) => Err(e),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchardPoolStats {
    pub chain_value_zec: f64,
    pub chain_value_zat: u64,
    pub monitored: bool,
    pub block_height: u32,
}

/// Chain-reported shielded pool balance (`getblockchaininfo` → `valuePools`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShieldedPoolStats {
    pub pool_id: String,
    pub chain_value_zec: f64,
    pub chain_value_zat: u64,
    pub monitored: bool,
    pub block_height: u32,
}

fn parse_pool_from_blockchain_info(
    info: &HashMap<String, Value>,
    pool_id: &str,
) -> NozyResult<ShieldedPoolStats> {
    let block_height = info.get("blocks").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let pools = info
        .get("valuePools")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            NozyError::InvalidOperation("getblockchaininfo: missing valuePools".to_string())
        })?;

    for pool in pools {
        if pool.get("id").and_then(|v| v.as_str()) == Some(pool_id) {
            let chain_value_zec = pool
                .get("chainValue")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let chain_value_zat = pool
                .get("chainValueZat")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let monitored = pool
                .get("monitored")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            return Ok(ShieldedPoolStats {
                pool_id: pool_id.to_string(),
                chain_value_zec,
                chain_value_zat,
                monitored,
                block_height,
            });
        }
    }

    Err(NozyError::InvalidOperation(format!(
        "getblockchaininfo: {pool_id} pool not found in valuePools"
    )))
}

fn parse_orchard_pool_from_blockchain_info(
    info: &HashMap<String, Value>,
) -> NozyResult<OrchardPoolStats> {
    let stats = parse_pool_from_blockchain_info(info, "orchard")?;
    Ok(OrchardPoolStats {
        chain_value_zec: stats.chain_value_zec,
        chain_value_zat: stats.chain_value_zat,
        monitored: stats.monitored,
        block_height: stats.block_height,
    })
}

/// Zebra `getrawtransaction` uses `height`; some stacks use `blockheight`.
fn parse_tx_block_height(tx_data: &serde_json::Value) -> Option<u32> {
    tx_data
        .get("height")
        .or_else(|| tx_data.get("blockheight"))
        .and_then(|v| v.as_u64())
        .map(|h| h as u32)
}

fn parse_tx_block_hash(tx_data: &serde_json::Value) -> Option<String> {
    tx_data
        .get("blockhash")
        .or_else(|| tx_data.get("block_hash"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

#[derive(Debug, Clone)]
pub struct TransactionInfo {
    pub txid: String,
    pub block_height: Option<u32>,
    pub block_hash: Option<String>,
    pub confirmations: u32,
    pub in_mempool: bool,
}

#[derive(Debug, Clone)]
pub struct OrchardTreeState {
    pub height: u32,
    pub anchor: [u8; 32],
    pub commitment_count: u64,
}

/// Ironwood uses the Orchard note-commitment tree primitives with a separate pool/tree.
pub type IronwoodTreeState = OrchardTreeState;

#[cfg(test)]
mod chain_node_kind_tests {
    use super::ChainNodeKind;

    #[test]
    fn detects_zakura_subversion() {
        assert_eq!(
            ChainNodeKind::from_subversion("/Zakura:1.0.0/"),
            ChainNodeKind::Zakura
        );
    }

    #[test]
    fn detects_zebra_subversion() {
        assert_eq!(
            ChainNodeKind::from_subversion("/Zebra:6.0.0/"),
            ChainNodeKind::Zebra
        );
    }
}

#[cfg(test)]
mod connection_policy_tests {
    use super::*;
    use crate::config::{PrivacyNetworkConfig, WalletConfig};

    #[test]
    fn configured_zebra_url_connects_direct_without_trusted_list() {
        let mut config = WalletConfig::default();
        config.zebra_url = "http://vps.example.com:8232".to_string();
        config.privacy_network = PrivacyNetworkConfig {
            require_privacy_network: true,
            tor_enabled: true,
            ..PrivacyNetworkConfig::default()
        };
        let client = ZebraClient::from_config(&config);
        assert_eq!(client.connection_mode(), ZebraConnectionMode::DirectTrusted);
    }

    #[test]
    fn trusted_remote_url_bypasses_privacy_block() {
        let mut config = WalletConfig::default();
        config.zebra_url = "http://vps.example.com:8232".to_string();
        config.trusted_zebra_urls = vec!["http://vps.example.com:8232".to_string()];
        config.privacy_network = PrivacyNetworkConfig {
            require_privacy_network: true,
            tor_enabled: true,
            ..PrivacyNetworkConfig::default()
        };
        let client = ZebraClient::from_config(&config);
        assert_eq!(client.connection_mode(), ZebraConnectionMode::DirectTrusted);
    }

    #[test]
    fn unrelated_remote_url_is_not_auto_trusted() {
        let mut config = WalletConfig::default();
        config.zebra_url = "http://vps.example.com:8232".to_string();
        assert!(!config.is_trusted_zebra_url("http://public.example.com:8232"));
    }
}

#[cfg(test)]
mod tx_field_parsing_tests {
    use super::{parse_tx_block_hash, parse_tx_block_height};

    #[test]
    fn parses_zebra_height_field() {
        let tx = serde_json::json!({
            "height": 3381141,
            "blockhash": "abc123",
            "confirmations": 11
        });
        assert_eq!(parse_tx_block_height(&tx), Some(3381141));
        assert_eq!(parse_tx_block_hash(&tx).as_deref(), Some("abc123"));
    }

    #[test]
    fn falls_back_to_blockheight_field() {
        let tx = serde_json::json!({ "blockheight": 100, "block_hash": "deadbeef" });
        assert_eq!(parse_tx_block_height(&tx), Some(100));
        assert_eq!(parse_tx_block_hash(&tx).as_deref(), Some("deadbeef"));
    }
}

#[cfg(test)]
mod transport_retry_tests {
    use super::is_retryable_zebra_transport_error;

    #[test]
    fn retries_response_body_decode_errors() {
        assert!(is_retryable_zebra_transport_error(
            "Failed to read response: error decoding response body"
        ));
    }

    #[test]
    fn retries_connection_failures() {
        assert!(is_retryable_zebra_transport_error(
            "Connection failed to http://127.0.0.1:8232: connection refused"
        ));
    }

    #[test]
    fn does_not_retry_invalid_rpc_payload() {
        assert!(!is_retryable_zebra_transport_error(
            "Zebra RPC error: block not found (code: -5)"
        ));
    }
}
