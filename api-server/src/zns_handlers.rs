//! Zcash Name Service (ZNS) resolve proxy for companion / extension clients.
//!
//! Indexer JSON-RPC: https://www.zcashnames.com/docs (resolve method).
//! Optional `.zcash` / `.zec` suffixes are stripped; they are never required.

use axum::{extract::Json, http::StatusCode, response::Json as ResponseJson};
use serde::{Deserialize, Serialize};

use crate::handlers::error_response;

const MAINNET_URL: &str = "https://light.zcash.me/zns-mainnet-test";
const TESTNET_URL: &str = "https://light.zcash.me/zns-testnet";

#[derive(Debug, Deserialize)]
pub struct ResolveRequest {
    /// Name as typed in a Zcash address field (`alice`, optionally `alice.zcash` / `alice.zec`).
    pub name: String,
    /// `"mainnet"` (default) or `"testnet"`.
    #[serde(default)]
    pub network: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ZnsRegistration {
    pub name: String,
    pub address: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub txid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nonce: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_action: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ResolveResponse {
    pub name: String,
    pub found: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registration: Option<ZnsRegistration>,
}

fn normalize_zns_name(raw: &str) -> String {
    let mut s = raw.trim().to_ascii_lowercase();
    if let Some(stripped) = s.strip_suffix(".zcash") {
        s = stripped.to_string();
    } else if let Some(stripped) = s.strip_suffix(".zec") {
        s = stripped.to_string();
    }
    s
}

fn is_valid_zns_name(name: &str) -> bool {
    let len = name.len();
    if !(1..=63).contains(&len) {
        return false;
    }
    let bytes = name.as_bytes();
    if !bytes[0].is_ascii_alphanumeric() || !bytes[len - 1].is_ascii_alphanumeric() {
        return false;
    }
    name.bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

fn indexer_url(network: Option<&str>) -> String {
    match network.map(|s| s.trim().to_ascii_lowercase()).as_deref() {
        Some("testnet") => {
            std::env::var("ZNS_TESTNET_URL").unwrap_or_else(|_| TESTNET_URL.to_string())
        }
        _ => std::env::var("ZNS_MAINNET_URL").unwrap_or_else(|_| MAINNET_URL.to_string()),
    }
}

/// POST `/api/zns/resolve` — proxy to the public ZNS indexer `resolve` RPC.
pub async fn resolve_zns_name(
    Json(payload): Json<ResolveRequest>,
) -> Result<ResponseJson<ResolveResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    let name = normalize_zns_name(&payload.name);
    if !is_valid_zns_name(&name) {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            "Invalid Zcash name. Use lowercase letters, digits, and hyphens (e.g. alice).",
        ));
    }

    let url = indexer_url(payload.network.as_deref());
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "resolve",
        "params": [name],
    });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to build HTTP client: {e}"),
            )
        })?;

    let res = client.post(url).json(&body).send().await.map_err(|e| {
        error_response(
            StatusCode::BAD_GATEWAY,
            format!("ZNS indexer unreachable: {e}"),
        )
    })?;

    if !res.status().is_success() {
        return Err(error_response(
            StatusCode::BAD_GATEWAY,
            format!("ZNS indexer HTTP {}", res.status()),
        ));
    }

    let rpc: serde_json::Value = res.json().await.map_err(|e| {
        error_response(
            StatusCode::BAD_GATEWAY,
            format!("Invalid ZNS indexer response: {e}"),
        )
    })?;

    if let Some(err) = rpc.get("error") {
        let msg = err
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("ZNS resolve failed");
        return Err(error_response(StatusCode::BAD_GATEWAY, msg));
    }

    let result = rpc.get("result");
    if result.is_none() || result == Some(&serde_json::Value::Null) {
        return Ok(ResponseJson(ResolveResponse {
            name,
            found: false,
            registration: None,
        }));
    }

    let reg: ZnsRegistration = serde_json::from_value(result.cloned().unwrap_or_default())
        .map_err(|e| {
            error_response(
                StatusCode::BAD_GATEWAY,
                format!("Unexpected ZNS registration shape: {e}"),
            )
        })?;

    if reg.address.is_empty() {
        return Ok(ResponseJson(ResolveResponse {
            name,
            found: false,
            registration: None,
        }));
    }

    Ok(ResponseJson(ResolveResponse {
        name: if reg.name.is_empty() {
            name
        } else {
            reg.name.clone()
        },
        found: true,
        registration: Some(reg),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_optional_suffixes() {
        assert_eq!(normalize_zns_name("Alice.zcash"), "alice");
        assert_eq!(normalize_zns_name("bob.zec"), "bob");
        assert_eq!(normalize_zns_name("carol"), "carol");
    }

    #[test]
    fn validates_names() {
        assert!(is_valid_zns_name("alice"));
        assert!(is_valid_zns_name("a"));
        assert!(is_valid_zns_name("my-name"));
        assert!(!is_valid_zns_name("-bad"));
        assert!(!is_valid_zns_name("Bad"));
        assert!(!is_valid_zns_name(""));
    }
}
