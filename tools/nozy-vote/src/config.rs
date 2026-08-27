//! Hash-pinned static + dynamic voting config fetch (ZIP draft 1244 discovery).

use anyhow::{anyhow, bail, Context, Result};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, ValueEnum)]
#[value(rename_all = "lower")]
pub enum Environment {
    Stage,
    Prod,
}

impl Environment {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stage => "stage",
            Self::Prod => "prod",
        }
    }
}

#[derive(Debug, Deserialize)]
struct StaticConfig {
    static_config_version: u32,
    dynamic_config_url: String,
    trusted_keys: Vec<TrustedKey>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct TrustedKey {
    key_id: String,
    alg: String,
    pubkey: String,
}

#[derive(Debug, Deserialize)]
struct DynamicConfig {
    config_version: u32,
    vote_servers: Vec<LabeledUrl>,
    pir_endpoints: Vec<LabeledUrl>,
    #[serde(default)]
    pir_layout: Option<serde_json::Value>,
    #[serde(default)]
    rounds: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct LabeledUrl {
    url: String,
    label: String,
}

#[derive(Debug)]
pub struct ResolvedConfig {
    pub dynamic_config_url: String,
    pub(crate) vote_servers: Vec<LabeledUrl>,
    pub(crate) pir_endpoints: Vec<LabeledUrl>,
    pub round_ids: Vec<String>,
    pub trusted_key_ids: Vec<String>,
    pub pir_layout: Option<serde_json::Value>,
    pub static_config_version: u32,
    pub dynamic_config_version: u32,
}

impl ResolvedConfig {
    pub fn summary_value(&self) -> serde_json::Value {
        serde_json::json!({
            "static_config_version": self.static_config_version,
            "dynamic_config_version": self.dynamic_config_version,
            "dynamic_config_url": self.dynamic_config_url,
            "trusted_key_ids": self.trusted_key_ids,
            "vote_servers": self.vote_servers,
            "pir_endpoints": self.pir_endpoints,
            "pir_layout": self.pir_layout,
            "round_ids": self.round_ids,
            "round_count": self.round_ids.len(),
        })
    }
}

/// Parse `url?checksum=sha256:HEX` (cosmovisor-style pin used by Valar wallets).
pub fn parse_pinned_source(source: &str) -> Result<(String, String)> {
    let (url, query) = match source.split_once('?') {
        Some((u, q)) => (u.to_string(), q),
        None => bail!("static source must include ?checksum=sha256:HEX pin"),
    };
    let mut checksum: Option<String> = None;
    for part in query.split('&') {
        if let Some(rest) = part.strip_prefix("checksum=") {
            checksum = Some(rest.to_string());
        }
    }
    let checksum = checksum.ok_or_else(|| anyhow!("missing checksum= query parameter"))?;
    let hex = checksum
        .strip_prefix("sha256:")
        .ok_or_else(|| anyhow!("checksum must be sha256:HEX, got {checksum}"))?
        .to_ascii_lowercase();
    if hex.len() != 64 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        bail!("invalid sha256 hex in pin");
    }
    Ok((url, hex))
}

fn http_get_bytes(url: &str) -> Result<Vec<u8>> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(format!("nozy-vote/{}", env!("CARGO_PKG_VERSION")))
        .build()?;
    let resp = client
        .get(url)
        .header("Cache-Control", "no-cache")
        .header("Pragma", "no-cache")
        .send()
        .with_context(|| format!("GET {url}"))?;
    if !resp.status().is_success() {
        bail!("GET {url} returned HTTP {}", resp.status());
    }
    Ok(resp.bytes()?.to_vec())
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

/// Fetch static (hash-pinned) then dynamic config. Does **not** yet verify
/// per-round Ed25519 signatures — that lands with `--features voting-sdk`
/// via `zcash_voting::config`.
pub fn fetch_and_resolve(source: &str) -> Result<ResolvedConfig> {
    let (static_url, expected_hash) = parse_pinned_source(source)?;
    let static_bytes = http_get_bytes(&static_url)?;
    let actual = sha256_hex(&static_bytes);
    if actual != expected_hash {
        bail!(
            "static config hash mismatch: expected {expected_hash}, got {actual} \
             (recompute pin after Valar rotates static-voting-config.json)"
        );
    }

    let static_cfg: StaticConfig =
        serde_json::from_slice(&static_bytes).context("decode static voting config")?;
    if static_cfg.static_config_version != 1 {
        bail!(
            "unsupported static_config_version {}",
            static_cfg.static_config_version
        );
    }

    let dynamic_bytes = http_get_bytes(&static_cfg.dynamic_config_url)?;
    let dynamic: DynamicConfig =
        serde_json::from_slice(&dynamic_bytes).context("decode dynamic voting config")?;
    if dynamic.config_version != 1 {
        bail!("unsupported dynamic config_version {}", dynamic.config_version);
    }
    if dynamic.vote_servers.is_empty() {
        bail!("dynamic config has no vote_servers");
    }
    if dynamic.pir_endpoints.is_empty() {
        bail!("dynamic config has no pir_endpoints");
    }
    if dynamic.pir_layout.is_none() {
        bail!("dynamic config missing pir_layout (required by current wallets)");
    }

    let mut round_ids: Vec<String> = dynamic.rounds.keys().cloned().collect();
    round_ids.sort();

    Ok(ResolvedConfig {
        dynamic_config_url: static_cfg.dynamic_config_url,
        vote_servers: dynamic.vote_servers,
        pir_endpoints: dynamic.pir_endpoints,
        round_ids,
        trusted_key_ids: static_cfg
            .trusted_keys
            .into_iter()
            .map(|k| k.key_id)
            .collect(),
        pir_layout: dynamic.pir_layout,
        static_config_version: static_cfg.static_config_version,
        dynamic_config_version: dynamic.config_version,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_short_hash() {
        assert!(parse_pinned_source(
            "https://voting.valargroup.org/stage/static-voting-config.json?checksum=sha256:aabbcc",
        )
        .is_err());
    }

    #[test]
    fn parses_valid_pin() {
        let hex = "a".repeat(64);
        let (url, hash) = parse_pinned_source(&format!(
            "https://example.com/static.json?checksum=sha256:{hex}"
        ))
        .unwrap();
        assert_eq!(url, "https://example.com/static.json");
        assert_eq!(hash, hex);
    }
}
