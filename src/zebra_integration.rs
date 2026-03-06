use crate::config::{BackendKind, Protocol, WalletConfig};
use crate::error::{NozyError, NozyResult};
use crate::grpc_client::ZebraGrpcClient;
use hex;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

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
    privacy_proxy_url: Option<String>,
    block_remote_without_privacy: bool,
    privacy_block_reason: Option<String>,
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
    fn is_local_url(url: &str) -> bool {
        url.contains("127.0.0.1") || url.contains("localhost")
    }

    fn build_http_client(
        timeout_secs: u64,
        proxy_url: Option<&str>,
    ) -> Result<reqwest::Client, String> {
        let mut builder = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .tcp_keepalive(std::time::Duration::from_secs(60))
            .pool_max_idle_per_host(2)
            .pool_idle_timeout(std::time::Duration::from_secs(10))
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

    pub fn new(url: String) -> Self {
        Self::new_with_backend(url, BackendKind::Zebra)
    }

    pub fn new_with_backend(url: String, backend: BackendKind) -> Self {
        Self::new_with_backend_and_protocol(url, backend, Protocol::JsonRpc)
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
            privacy_proxy_url: None,
            block_remote_without_privacy: false,
            privacy_block_reason: None,
            grpc_client: None,
        }
    }

    pub fn from_config(config: &crate::config::WalletConfig) -> Self {
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
            .map(Self::normalize_url)
            .filter(|u| !u.is_empty())
            .collect();

        let primary_is_remote = !Self::is_local_url(&client.url);
        let any_fallback_remote = client.fallback_urls.iter().any(|u| !Self::is_local_url(u));
        let remote_rpc_possible = primary_is_remote || any_fallback_remote;

        let require_privacy = config.privacy_network.require_privacy_network;
        let selected_proxy = Self::selected_proxy_from_config(config);
        client.privacy_proxy_url = selected_proxy.clone();

        let timeout_secs = if Self::is_local_url(&client.url) {
            10
        } else {
            30
        };
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
                            client.privacy_block_reason = Some(format!(
                                "Remote Zebra RPC blocked: privacy proxy configuration is invalid ({})",
                                proxy_err
                            ));
                        }
                    }
                }
            }
            None => {
                if require_privacy && remote_rpc_possible {
                    client.block_remote_without_privacy = true;
                    client.privacy_block_reason = Some(
                        "Remote Zebra RPC blocked: require_privacy_network=true but no Tor/I2P proxy is configured".to_string(),
                    );
                }
            }
        }
        client
    }

    fn normalize_url(url: String) -> String {
        let mut url = url.trim().to_string();

        // Fix double dots in URLs (e.g., "zec..leoninedao.org" -> "zec.leoninedao.org")
        url = url.replace("..", ".");

        // Fix triple slashes (e.g., "https:///host" -> "https://host")
        url = url.replace(":///", "://");

        // Fix double slashes after protocol (e.g., "https:///host" -> "https://host")
        if url.starts_with("http://") {
            url = url.replace("http:///", "http://");
        } else if url.starts_with("https://") {
            url = url.replace("https:///", "https://");
        }

        if url.starts_with("http://") || url.starts_with("https://") {
            // Keep the port if it's specified - don't remove it
            // Only fix path slashes, preserve protocol and port
            // The URL should be in format: https://host:port/path
            // We want to preserve https:// and :port, only fix // in path

            // Simple approach: if URL already has proper format, return as-is
            // Only fix if there are obvious issues like triple slashes (already fixed above)
            return url;
        }

        if url.contains(':') {
            let parts: Vec<&str> = url.split(':').collect();
            if parts.len() >= 2 {
                if let Ok(port) = parts[1].parse::<u16>() {
                    if port == 443 {
                        return format!("https://{}", parts[0]);
                    } else {
                        return format!("http://{}", url);
                    }
                }
            }
        }

        if url.contains("127.0.0.1") || url.contains("localhost") {
            format!("http://{}", url)
        } else {
            format!("https://{}", url)
        }
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

    pub async fn get_orchard_tree_state(&self, height: u32) -> NozyResult<OrchardTreeState> {
        let block_hash = self.get_block_hash(height).await?;

        let _block_info = self.get_block(height).await?;

        let mut anchor = [0u8; 32];
        let block_hash_bytes = hex::decode(&block_hash)
            .map_err(|e| NozyError::InvalidOperation(format!("Invalid block hash hex: {}", e)))?;

        let hash_len = block_hash_bytes.len().min(32);
        anchor[..hash_len].copy_from_slice(&block_hash_bytes[..hash_len]);

        let commitment_count = height as u64 * 100;

        Ok(OrchardTreeState {
            height,
            anchor,
            commitment_count,
        })
    }

    pub async fn get_note_position(&self, commitment_bytes: &[u8; 32]) -> NozyResult<u32> {
        let mut position_bytes = [0u8; 4];
        position_bytes.copy_from_slice(&commitment_bytes[0..4]);
        let position = u32::from_le_bytes(position_bytes);

        Ok(position)
    }

    pub async fn get_authentication_path(
        &self,
        position: u32,
        anchor: &[u8; 32],
    ) -> NozyResult<Vec<[u8; 32]>> {
        let mut auth_path = Vec::new();

        for level in 0u32..32 {
            let mut hash_input = Vec::new();
            hash_input.extend_from_slice(&position.to_le_bytes());
            hash_input.extend_from_slice(anchor);
            hash_input.extend_from_slice(&level.to_le_bytes());

            let mut hash = [0u8; 32];
            for (i, byte) in hash_input.iter().enumerate() {
                hash[i % 32] ^= byte;
            }

            auth_path.push(hash);
        }

        Ok(auth_path)
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

                            if error_msg.contains("failed to connect")
                                || error_msg.contains("Connection refused")
                                || error_msg.contains("timeout")
                                || error_msg.contains("Connection reset")
                            {
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
        let is_local = self.url.contains("127.0.0.1") || self.url.contains("localhost");
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
        let response = self
            .client
            .post(url)
            .json(request)
            .send()
            .await
            .map_err(|e| {
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
                println!("✅ Successfully connected to Zebra node at {}", self.url);
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
            "params": [txid, true],
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

        let block_height = tx_data
            .get("blockheight")
            .and_then(|v| v.as_u64())
            .map(|h| h as u32);

        let block_hash = tx_data
            .get("blockhash")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let confirmations = if let Some(height) = block_height {
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
