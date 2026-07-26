//! Shared smolmix JSON-RPC egress helpers (issue #147).
//!
//! Used by the spike binary and optionally by `nozy` via `--features nym-mixnet`.

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Instant;

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::Request;
use hyper_util::rt::TokioIo;
use rustls::pki_types::ServerName;
use serde_json::json;
use smolmix::Tunnel;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_rustls::TlsConnector;
use url::Url;

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, Clone)]
pub struct RpcTarget {
    pub use_tls: bool,
    pub host: String,
    pub sni: String,
    pub addr: SocketAddr,
    pub path: String,
}

/// How a zebra URL is classified for mixnet submit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MixnetUrlClass {
    /// Loopback / RFC1918 / ULA — wallet must use Case A1 direct path.
    LocalOrPrivate,
    /// Exit-reachable candidate (public DNS/IP). Still may fail auth/firewall.
    ExitReachableCandidate,
}

/// True when a Nym exit cannot reasonably reach this host (loopback / RFC1918 / ULA / link-local).
pub fn host_unreachable_from_mixnet_exit(host: &str) -> bool {
    let host_lc = host.trim().trim_matches(['[', ']']).to_ascii_lowercase();
    if host_lc == "localhost" {
        return true;
    }
    if let Ok(ip) = host_lc.parse::<IpAddr>() {
        return match ip {
            IpAddr::V4(v4) => {
                let o = v4.octets();
                v4.is_loopback()
                    || v4.is_private()
                    || (o[0] == 169 && o[1] == 254)
                    || v4.is_link_local()
            }
            IpAddr::V6(v6) => {
                v6.is_loopback() || v6.is_unique_local() || v6.is_unicast_link_local()
            }
        };
    }
    false
}

pub fn classify_zebra_url_for_mixnet(raw: &str) -> Result<MixnetUrlClass, BoxError> {
    let url = Url::parse(raw).or_else(|_| Url::parse(&format!("http://{raw}")))?;
    let host = url
        .host_str()
        .ok_or("zebra URL missing host")?
        .to_string();
    if host_unreachable_from_mixnet_exit(&host) {
        return Ok(MixnetUrlClass::LocalOrPrivate);
    }
    if let Ok(ip) = host.parse::<IpAddr>() {
        if host_unreachable_from_mixnet_exit(&ip.to_string()) {
            return Ok(MixnetUrlClass::LocalOrPrivate);
        }
    }
    Ok(MixnetUrlClass::ExitReachableCandidate)
}

pub fn parse_rpc_target(raw: &str) -> Result<RpcTarget, BoxError> {
    let url = Url::parse(raw).or_else(|_| Url::parse(&format!("http://{raw}")))?;
    let host = url
        .host_str()
        .ok_or("zebra URL missing host")?
        .to_string();
    let use_tls = url.scheme() == "https";
    let port = url.port().unwrap_or(if use_tls { 443 } else { 80 });
    let addr: SocketAddr = if let Ok(ip) = host.parse::<IpAddr>() {
        SocketAddr::new(ip, port)
    } else {
        use std::net::ToSocketAddrs;
        (host.as_str(), port)
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| format!("could not resolve {host}"))?
    };
    if host_unreachable_from_mixnet_exit(&host)
        || host_unreachable_from_mixnet_exit(&addr.ip().to_string())
    {
        return Err(format!(
            "refusing mixnet RPC to {host} ({}): Nym exit cannot reach loopback/private LAN. \
             Use an exit-reachable public/testnet Zebrad URL (see NYM_IP_PRIVACY_CASE_BREAKDOWN.md D2b).",
            addr.ip()
        )
        .into());
    }
    let path = if url.path().is_empty() {
        "/".to_string()
    } else {
        url.path().to_string()
    };
    Ok(RpcTarget {
        use_tls,
        host: host.clone(),
        sni: host,
        addr,
        path,
    })
}

/// Structured evidence row for case-breakdown logs (D2a–D2c).
#[derive(Debug, Clone)]
pub struct EvidenceStep {
    pub id: String,
    pub result: String,
    pub detail: String,
}

impl EvidenceStep {
    pub fn pass(id: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            result: "PASS".into(),
            detail: detail.into(),
        }
    }

    pub fn fail(id: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            result: "FAIL".into(),
            detail: detail.into(),
        }
    }

    pub fn na(id: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            result: "N/A".into(),
            detail: detail.into(),
        }
    }

    pub fn blocked(id: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            result: "BLOCKED".into(),
            detail: detail.into(),
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        json!({
            "id": self.id,
            "result": self.result,
            "detail": self.detail,
        })
    }
}

pub fn install_crypto() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}

fn tls_connector() -> TlsConnector {
    let mut roots = rustls::RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let cfg = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    TlsConnector::from(Arc::new(cfg))
}

pub async fn build_tunnel(ipr: Option<&str>) -> Result<Tunnel, BoxError> {
    let mut builder = Tunnel::builder();
    if let Some(addr) = ipr {
        builder = builder.ipr_address(addr.parse()?);
    }
    Ok(builder.build().await?)
}

async fn http1_json_post<S>(
    stream: S,
    host: &str,
    path: &str,
    body: &str,
) -> Result<String, BoxError>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let (mut sender, conn) = hyper::client::conn::http1::handshake(TokioIo::new(stream)).await?;
    tokio::spawn(async move {
        let _ = conn.await;
    });
    let req = Request::post(path)
        .header("Host", host)
        .header("Content-Type", "application/json")
        .header("Content-Length", body.len())
        .body(Full::new(Bytes::from(body.to_string())))?;
    let resp = sender.send_request(req).await?;
    let status = resp.status();
    let bytes = resp.into_body().collect().await?.to_bytes();
    let text = String::from_utf8_lossy(&bytes).into_owned();
    if !status.is_success() {
        return Err(format!("HTTP {status}: {text}").into());
    }
    Ok(text)
}

/// POST JSON-RPC body to `zebra_url` over an existing smolmix tunnel.
pub async fn json_rpc_post_over_tunnel(
    tunnel: &Tunnel,
    zebra_url: &str,
    body: &str,
) -> Result<(String, std::time::Duration), BoxError> {
    let target = parse_rpc_target(zebra_url)?;
    let t0 = Instant::now();
    let tcp = tunnel.tcp_connect(target.addr).await?;
    let text = if target.use_tls {
        let domain = ServerName::try_from(target.sni.clone())?;
        let tls = tls_connector().connect(domain, tcp).await?;
        http1_json_post(tls, &target.host, &target.path, body).await?
    } else {
        http1_json_post(tcp, &target.host, &target.path, body).await?
    };
    Ok((text, t0.elapsed()))
}

/// One-shot: build tunnel, POST JSON-RPC, shut down.
pub async fn json_rpc_post_over_smolmix(
    zebra_url: &str,
    body: &str,
    ipr: Option<&str>,
) -> Result<String, BoxError> {
    install_crypto();
    let tunnel = build_tunnel(ipr).await?;
    let (text, _) = json_rpc_post_over_tunnel(&tunnel, zebra_url, body).await?;
    tunnel.shutdown().await;
    Ok(text)
}

/// `sendrawtransaction` over smolmix; returns txid string from JSON-RPC `result`.
pub async fn sendrawtransaction_over_smolmix(
    zebra_url: &str,
    raw_tx_hex: &str,
    ipr: Option<&str>,
) -> Result<String, BoxError> {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "sendrawtransaction",
        "params": [raw_tx_hex],
    })
    .to_string();
    let text = json_rpc_post_over_smolmix(zebra_url, &body, ipr).await?;
    let v: serde_json::Value = serde_json::from_str(&text)?;
    if let Some(err) = v.get("error").filter(|e| !e.is_null()) {
        return Err(format!("Zebra RPC error: {err}").into());
    }
    v.get("result")
        .and_then(|r| r.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| format!("no txid in sendrawtransaction response: {text}").into())
}

/// Dry reachability gate used by `--dry-reachability` (no tunnel).
pub fn dry_reachability_evidence(zebra_url: &str) -> EvidenceStep {
    match classify_zebra_url_for_mixnet(zebra_url) {
        Ok(MixnetUrlClass::LocalOrPrivate) => EvidenceStep::na(
            "D2b-reachability",
            format!(
                "refused local/private target {zebra_url} (Case A1 / E5 - not a mixnet bug)"
            ),
        ),
        Ok(MixnetUrlClass::ExitReachableCandidate) => match parse_rpc_target(zebra_url) {
            Ok(t) => EvidenceStep::pass(
                "D2b-reachability",
                format!(
                    "candidate exit-reachable target {} → {}:{}",
                    zebra_url,
                    t.addr.ip(),
                    t.addr.port()
                ),
            ),
            Err(e) => EvidenceStep::fail("D2b-reachability", e.to_string()),
        },
        Err(e) => EvidenceStep::fail("D2b-reachability", e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_loopback_and_rfc1918() {
        assert!(host_unreachable_from_mixnet_exit("127.0.0.1"));
        assert!(host_unreachable_from_mixnet_exit("localhost"));
        assert!(host_unreachable_from_mixnet_exit("172.20.199.206"));
        assert!(host_unreachable_from_mixnet_exit("192.168.1.10"));
        assert!(host_unreachable_from_mixnet_exit("10.0.0.2"));
        assert!(!host_unreachable_from_mixnet_exit("1.1.1.1"));
        assert!(!host_unreachable_from_mixnet_exit("example.com"));
    }

    #[test]
    fn parse_rpc_refuses_lan() {
        let err = parse_rpc_target("http://172.20.199.206:18232").unwrap_err();
        assert!(err.to_string().contains("refusing mixnet RPC"));
    }

    #[test]
    fn classify_local_vs_candidate() {
        assert_eq!(
            classify_zebra_url_for_mixnet("http://127.0.0.1:8232").unwrap(),
            MixnetUrlClass::LocalOrPrivate
        );
        assert_eq!(
            classify_zebra_url_for_mixnet("https://example.com:18232").unwrap(),
            MixnetUrlClass::ExitReachableCandidate
        );
    }

    #[test]
    fn dry_reachability_marks_lan_na() {
        let step = dry_reachability_evidence("http://192.168.1.5:8232");
        assert_eq!(step.result, "N/A");
        assert_eq!(step.id, "D2b-reachability");
    }
}
