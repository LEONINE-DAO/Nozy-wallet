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

async fn resolve_name_inner(
    name: &str,
    network: Option<&str>,
) -> Result<ResolveResponse, (StatusCode, ResponseJson<serde_json::Value>)> {
    let url = indexer_url(network);
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
        return Ok(ResolveResponse {
            name: name.to_string(),
            found: false,
            registration: None,
        });
    }

    let reg: ZnsRegistration = serde_json::from_value(result.cloned().unwrap_or_default())
        .map_err(|e| {
            error_response(
                StatusCode::BAD_GATEWAY,
                format!("Unexpected ZNS registration shape: {e}"),
            )
        })?;

    if reg.address.is_empty() {
        return Ok(ResolveResponse {
            name: name.to_string(),
            found: false,
            registration: None,
        });
    }

    Ok(ResolveResponse {
        name: if reg.name.is_empty() {
            name.to_string()
        } else {
            reg.name.clone()
        },
        found: true,
        registration: Some(reg),
    })
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

    Ok(ResponseJson(
        resolve_name_inner(&name, payload.network.as_deref()).await?,
    ))
}

#[derive(Debug, Deserialize)]
pub struct LinkRequest {
    pub name: String,
    pub password: Option<String>,
    /// If set, must match resolved address and Business UA.
    #[serde(default)]
    pub expect_address: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LinkResponse {
    pub linked: bool,
    pub name: String,
    pub display: String,
    pub address: String,
    pub business_address: String,
}

/// GET `/api/zns/link` — currently linked name (local config).
pub async fn get_zns_link(
) -> Result<ResponseJson<serde_json::Value>, (StatusCode, ResponseJson<serde_json::Value>)> {
    let config = nozy::load_config();
    Ok(ResponseJson(serde_json::json!({
        "linked": config.linked_zns_name.is_some(),
        "name": config.linked_zns_name,
        "display": config.linked_zns_display(),
    })))
}

/// POST `/api/zns/link` — resolve name and link if it matches Business UA (account 1).
pub async fn link_zns_name(
    Json(payload): Json<LinkRequest>,
) -> Result<ResponseJson<LinkResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    let name = normalize_zns_name(&payload.name);
    if !is_valid_zns_name(&name) {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            "Invalid Zcash name. Use lowercase letters, digits, and hyphens (e.g. alice).",
        ));
    }

    let config = nozy::load_config();
    let network = if config.network == "testnet" {
        "testnet"
    } else {
        "mainnet"
    };

    let resolved = resolve_name_inner(&name, Some(network)).await?;
    if !resolved.found {
        return Err(error_response(
            StatusCode::NOT_FOUND,
            format!("No Zcash name registered for “{name}”."),
        ));
    }
    let reg = resolved
        .registration
        .ok_or_else(|| error_response(StatusCode::NOT_FOUND, "Name not found on indexer."))?;

    let (wallet, _storage) = crate::handlers::load_wallet_with_password(payload.password)
        .await
        .map_err(|e| error_response(StatusCode::UNAUTHORIZED, e))?;

    let net = if config.network == "testnet" {
        zcash_protocol::consensus::NetworkType::Test
    } else {
        zcash_protocol::consensus::NetworkType::Main
    };
    let business_address = wallet
        .generate_orchard_address(1, 0, net)
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let resolved_addr = reg.address.replace([' ', '\n', '\t'], "");
    if resolved_addr != business_address {
        return Err(error_response(
            StatusCode::CONFLICT,
            format!(
                "Name “{name}” resolves to a different address than your Business UA. Claim/update the name to point at your Business receive address first (zcashnames.com)."
            ),
        ));
    }

    if let Some(expect) = payload.expect_address.as_deref() {
        let expect = expect.replace([' ', '\n', '\t'], "");
        if !expect.is_empty() && expect != business_address {
            return Err(error_response(
                StatusCode::BAD_REQUEST,
                "expect_address does not match Business UA.",
            ));
        }
    }

    let mut config = nozy::load_config();
    config.linked_zns_name = Some(name.clone());
    if config.active_role != nozy::WalletRole::Business {
        config.active_role = nozy::WalletRole::Business;
    }
    nozy::save_config(&config).map_err(|e| {
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to save link: {e}"),
        )
    })?;

    Ok(ResponseJson(LinkResponse {
        linked: true,
        name: name.clone(),
        display: format!("{name}.zcash"),
        address: business_address.clone(),
        business_address,
    }))
}

/// DELETE `/api/zns/link` — clear local link (does not release on-chain name).
pub async fn unlink_zns_name(
) -> Result<ResponseJson<serde_json::Value>, (StatusCode, ResponseJson<serde_json::Value>)> {
    let mut config = nozy::load_config();
    config.linked_zns_name = None;
    nozy::save_config(&config).map_err(|e| {
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to clear link: {e}"),
        )
    })?;
    Ok(ResponseJson(serde_json::json!({
        "linked": false,
        "name": serde_json::Value::Null,
    })))
}

#[derive(Debug, Deserialize)]
pub struct ReverseQuery {
    pub address: String,
    #[serde(default)]
    pub network: Option<String>,
}

/// GET `/api/zns/reverse?address=` — names pointing at a UA (indexer list query).
pub async fn reverse_zns_lookup(
    axum::extract::Query(query): axum::extract::Query<ReverseQuery>,
) -> Result<ResponseJson<serde_json::Value>, (StatusCode, ResponseJson<serde_json::Value>)> {
    let address = query.address.replace([' ', '\n', '\t'], "");
    if address.is_empty() || (!address.starts_with("u1") && !address.starts_with("utest1")) {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            "address must be a unified address (u1… or utest1…).",
        ));
    }

    let url = indexer_url(query.network.as_deref());
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "resolve",
        "params": [address, 50, 0],
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
            .unwrap_or("ZNS reverse failed");
        return Err(error_response(StatusCode::BAD_GATEWAY, msg));
    }

    let names = match rpc.get("result") {
        Some(serde_json::Value::Array(arr)) => arr.clone(),
        Some(serde_json::Value::Null) | None => Vec::new(),
        Some(other) => vec![other.clone()],
    };

    Ok(ResponseJson(serde_json::json!({
        "address": address,
        "names": names,
    })))
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
