//! Zcash Name Service (ZNS) client helpers shared by CLI and companion surfaces.
//!
//! Indexer JSON-RPC: https://www.zcashnames.com/docs (resolve method).
//! Optional `.zcash` / `.zec` suffixes are stripped; they are never required.

use crate::error::{NozyError, NozyResult};
use serde::{Deserialize, Serialize};

const MAINNET_URL: &str = "https://light.zcash.me/zns-mainnet-test";
const TESTNET_URL: &str = "https://light.zcash.me/zns-testnet";

/// Default allowlisted indexer prefixes (production verify gate).
const DEFAULT_ALLOWED_PREFIXES: &[&str] =
    &["https://light.zcash.me/", "https://www.zcashnames.com/"];

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveResult {
    pub name: String,
    pub found: bool,
    pub registration: Option<ZnsRegistration>,
}

pub fn normalize_zns_name(raw: &str) -> String {
    let mut s = raw.trim().to_ascii_lowercase();
    if let Some(stripped) = s.strip_suffix(".zcash") {
        s = stripped.to_string();
    } else if let Some(stripped) = s.strip_suffix(".zec") {
        s = stripped.to_string();
    }
    s
}

pub fn is_valid_zns_name(name: &str) -> bool {
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

/// True when the recipient looks like a ZNS name (not a UA / t-addr / Sapling z-addr).
pub fn is_likely_zns_name(raw: &str) -> bool {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return false;
    }
    let compact: String = trimmed.chars().filter(|c| !c.is_whitespace()).collect();
    if compact.starts_with("u1")
        || compact.starts_with("utest1")
        || compact.starts_with("t1")
        || compact.starts_with("t2")
        || compact.starts_with("tm")
        || compact.starts_with("zs1")
        || compact.starts_with("ztestsapling")
        || compact.starts_with("zregtestsapling")
    {
        return false;
    }
    let name = normalize_zns_name(trimmed);
    is_valid_zns_name(&name)
}

pub fn indexer_url(network: Option<&str>) -> String {
    match network.map(|s| s.trim().to_ascii_lowercase()).as_deref() {
        Some("testnet") => {
            std::env::var("ZNS_TESTNET_URL").unwrap_or_else(|_| TESTNET_URL.to_string())
        }
        _ => std::env::var("ZNS_MAINNET_URL").unwrap_or_else(|_| MAINNET_URL.to_string()),
    }
}

/// Reject unknown indexer hosts unless `ZNS_ALLOW_UNTRUSTED_INDEXER=1`.
pub fn verify_indexer_url(url: &str) -> NozyResult<()> {
    if std::env::var("ZNS_ALLOW_UNTRUSTED_INDEXER")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        return Ok(());
    }
    let extra = std::env::var("ZNS_ALLOWED_INDEXER_PREFIXES").unwrap_or_default();
    let extras: Vec<&str> = extra
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    let allowed = DEFAULT_ALLOWED_PREFIXES
        .iter()
        .copied()
        .chain(extras)
        .collect::<Vec<_>>();
    if allowed.iter().any(|p| url.starts_with(p)) {
        return Ok(());
    }
    Err(NozyError::NetworkError(format!(
        "ZNS indexer URL is not allowlisted: {url}. Set ZNS_ALLOW_UNTRUSTED_INDEXER=1 or ZNS_ALLOWED_INDEXER_PREFIXES to override."
    )))
}

/// Resolve a Zcash name via the public indexer JSON-RPC `resolve` method.
pub async fn resolve_name(raw_name: &str, network: Option<&str>) -> NozyResult<ResolveResult> {
    let name = normalize_zns_name(raw_name);
    if !is_valid_zns_name(&name) {
        return Err(NozyError::InvalidInput(
            "Invalid Zcash name. Use lowercase letters, digits, and hyphens (e.g. alice).".into(),
        ));
    }

    let url = indexer_url(network);
    verify_indexer_url(&url)?;

    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "resolve",
        "params": [&name],
    });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| NozyError::NetworkError(format!("Failed to build HTTP client: {e}")))?;

    let res = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| NozyError::NetworkError(format!("ZNS indexer unreachable: {e}")))?;

    if !res.status().is_success() {
        return Err(NozyError::NetworkError(format!(
            "ZNS indexer HTTP {}",
            res.status()
        )));
    }

    let rpc: serde_json::Value = res
        .json()
        .await
        .map_err(|e| NozyError::NetworkError(format!("Invalid ZNS indexer response: {e}")))?;

    if let Some(err) = rpc.get("error") {
        let msg = err
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("ZNS resolve failed");
        return Err(NozyError::NetworkError(msg.to_string()));
    }

    let result = rpc.get("result");
    if result.is_none() || result == Some(&serde_json::Value::Null) {
        return Ok(ResolveResult {
            name,
            found: false,
            registration: None,
        });
    }

    let reg: ZnsRegistration = serde_json::from_value(result.cloned().unwrap_or_default())
        .map_err(|e| NozyError::NetworkError(format!("Unexpected ZNS registration shape: {e}")))?;

    if reg.address.is_empty() {
        return Ok(ResolveResult {
            name,
            found: false,
            registration: None,
        });
    }

    Ok(ResolveResult {
        name: if reg.name.is_empty() {
            name
        } else {
            reg.name.clone()
        },
        found: true,
        registration: Some(reg),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_strips_suffix() {
        assert_eq!(normalize_zns_name("Alice.zcash"), "alice");
        assert_eq!(normalize_zns_name("bob.zec"), "bob");
    }

    #[test]
    fn likely_name_vs_ua() {
        assert!(is_likely_zns_name("zenith"));
        assert!(is_likely_zns_name("taco-stand.zcash"));
        assert!(!is_likely_zns_name("u1abc"));
        assert!(!is_likely_zns_name("t1abc"));
    }

    #[test]
    fn allowlist_default_ok() {
        assert!(verify_indexer_url(MAINNET_URL).is_ok());
        assert!(verify_indexer_url("https://evil.example/zns").is_err());
    }
}
