//! Zebra-family JSON-RPC client (Zebrad / Zakura) with cookie / basic auth.

use std::fs;
use std::path::PathBuf;

use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::error::{IndexerError, IndexerResult};

#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub chain: String,
    pub blocks: u64,
    pub subversion: String,
    pub build: String,
}

#[derive(Debug, Clone)]
pub struct RpcClient {
    url: String,
    http: Client,
    auth: Option<(String, String)>,
}

#[derive(Debug, Deserialize)]
struct RpcResponse<T> {
    result: Option<T>,
    error: Option<RpcErrorBody>,
}

#[derive(Debug, Deserialize)]
struct RpcErrorBody {
    code: i64,
    message: String,
}

impl RpcClient {
    pub fn new(url: &str) -> IndexerResult<Self> {
        let url = normalize_rpc_url(url);
        let http = Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| IndexerError::Rpc(format!("http client: {e}")))?;
        Ok(Self {
            url,
            http,
            auth: resolve_rpc_auth(),
        })
    }

    pub fn auth_configured(&self) -> bool {
        self.auth.is_some()
    }

    async fn call<T: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        params: Value,
    ) -> IndexerResult<T> {
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        });
        let mut req = self.http.post(&self.url).json(&body);
        if let Some((user, pass)) = &self.auth {
            req = req.basic_auth(user, Some(pass));
        }
        let resp = req
            .send()
            .await
            .map_err(|e| IndexerError::Rpc(format!("{method}: transport: {e}")))?;
        if !resp.status().is_success() {
            return Err(IndexerError::Rpc(format!(
                "{method}: HTTP {} — if Zakura cookie auth is on, set ZAKURA_RPC_COOKIE or ZAKURA_RPC_COOKIE_PATH",
                resp.status()
            )));
        }
        let parsed: RpcResponse<T> = resp
            .json()
            .await
            .map_err(|e| IndexerError::Rpc(format!("{method}: decode: {e}")))?;
        if let Some(err) = parsed.error {
            return Err(IndexerError::Rpc(format!(
                "{method}: RPC {} — {}",
                err.code, err.message
            )));
        }
        parsed
            .result
            .ok_or_else(|| IndexerError::Rpc(format!("{method}: missing result")))
    }

    pub async fn get_block_count(&self) -> IndexerResult<u64> {
        self.call("getblockcount", json!([])).await
    }

    pub async fn get_block_hash(&self, height: u64) -> IndexerResult<String> {
        self.call("getblockhash", json!([height])).await
    }

    /// Raw block hex (verbosity 0).
    pub async fn get_block_raw_hex(&self, hash: &str) -> IndexerResult<String> {
        self.call("getblock", json!([hash, 0])).await
    }

    /// JSON block (verbosity 1) — used to read `trees.*.size` when present.
    pub async fn get_block_verbose(&self, hash: &str) -> IndexerResult<Value> {
        self.call("getblock", json!([hash, 1])).await
    }

    pub async fn get_blockchain_info(&self) -> IndexerResult<Value> {
        self.call("getblockchaininfo", json!([])).await
    }

    pub async fn get_network_info(&self) -> IndexerResult<Value> {
        self.call("getnetworkinfo", json!([])).await
    }

    pub async fn probe_node(&self) -> IndexerResult<NodeInfo> {
        let chain_info = self.get_blockchain_info().await?;
        let net = self.get_network_info().await.unwrap_or(Value::Null);
        let chain = chain_info
            .get("chain")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let blocks = chain_info
            .get("blocks")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let subversion = net
            .get("subversion")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let build = net
            .get("build")
            .and_then(|v| v.as_str())
            .or_else(|| net.get("version").and_then(|v| v.as_str()))
            .unwrap_or("")
            .to_string();
        Ok(NodeInfo {
            chain,
            blocks,
            subversion,
            build,
        })
    }

    /// Fetch raw block bytes at height.
    pub async fn get_raw_block(&self, height: u64) -> IndexerResult<Vec<u8>> {
        let hash = self.get_block_hash(height).await?;
        let hex_str = self.get_block_raw_hex(&hash).await?;
        hex::decode(hex_str.trim()).map_err(|e| IndexerError::Rpc(format!("block hex: {e}")))
    }

    /// `z_gettreestate` JSON result. Zebrad expects height as a **string** (not a JSON number).
    pub async fn z_gettreestate(&self, height: u64) -> IndexerResult<Value> {
        self.call("z_gettreestate", json!([height.to_string()]))
            .await
    }

    /// `z_getsubtreesbyindex` JSON result.
    pub async fn z_getsubtreesbyindex(
        &self,
        pool: &str,
        start_index: u32,
        max_entries: u32,
    ) -> IndexerResult<Value> {
        if max_entries == 0 {
            self.call("z_getsubtreesbyindex", json!([pool, start_index]))
                .await
        } else {
            self.call(
                "z_getsubtreesbyindex",
                json!([pool, start_index, max_entries]),
            )
            .await
        }
    }

    /// Broadcast hex via `sendrawtransaction`. Returns txid string.
    pub async fn send_raw_transaction(&self, tx_hex: &str) -> IndexerResult<String> {
        self.call("sendrawtransaction", json!([tx_hex])).await
    }
}

pub fn normalize_rpc_url(url: &str) -> String {
    let u = url.trim().trim_end_matches('/');
    if u.starts_with("http://") || u.starts_with("https://") {
        u.to_string()
    } else {
        format!("http://{u}")
    }
}

pub fn detect_node_kind(subversion: &str) -> &'static str {
    let s = subversion.to_ascii_lowercase();
    if s.contains("zakura") {
        "Zakura"
    } else if s.contains("zebra") {
        "Zebrad"
    } else {
        "Zebra-family"
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

/// Same resolution order as Nozy `ZebraClient` (user/pass, inline cookie, cookie files).
pub fn resolve_rpc_auth() -> Option<(String, String)> {
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
            if let Some(pair) = parse_cookie_pair(&cookie_inline) {
                return Some(pair);
            }
        }
    }
    for path in candidate_cookie_paths() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Some(pair) = parse_cookie_pair(&content) {
                return Some(pair);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_adds_http() {
        assert_eq!(normalize_rpc_url("127.0.0.1:8232"), "http://127.0.0.1:8232");
    }

    #[test]
    fn detect_zakura() {
        assert_eq!(detect_node_kind("/Zakura:1.0.0/"), "Zakura");
        assert_eq!(detect_node_kind("/Zebra:2.0/"), "Zebrad");
    }
}
