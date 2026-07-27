//! Shared helpers adapted from Nym `smoldvpn` examples/common (Apache-2.0).
//! Kept local so this spike does not vendor the whole Nym examples tree.

#![allow(dead_code)]

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Once};
use std::task::{Context, Poll};

use http::Uri;
use hyper_util::rt::TokioIo;
use nym_crypto::asymmetric::{ed25519, x25519};
use nym_network_defaults::NymNetworkDetails;
use nym_sdk_session::{
    GatewayInfo, GatewaySpec, HopConfig, QuicBridge, Registration, Session, SessionConfig,
};
use smoldvpn::{BridgeParams, PeerConfig, Tunnel, TunnelBuilder};
use rustls::pki_types::ServerName;
use serde_json::Value;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio_rustls::TlsConnector;
use tower::Service;

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

static INSTALL_PROVIDER: Once = Once::new();

pub fn init_crypto() {
    INSTALL_PROVIDER.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

pub fn mnemonic() -> bip39::Mnemonic {
    std::env::var("MNEMONIC")
        .or_else(|_| std::env::var("NYX_ACCOUNT_MNEMONIC"))
        .expect("set MNEMONIC or NYX_ACCOUNT_MNEMONIC")
        .parse()
        .expect("valid bip39 mnemonic")
}

pub const DEFAULT_DVPN_DIRECTORY: &str =
    "https://nymvpn.com/api/public/v1/directory/gateways?show_vpn_only=true";

/// Sandbox directory (only if you funded sandbox NYM, not mainnet Keplr).
pub const SANDBOX_DVPN_DIRECTORY: &str =
    "https://sandbox-node-status-api.nymte.ch/dvpn/v1/directory/gateways";

pub fn dvpn_directory_url() -> String {
    std::env::var("DVPN_DIRECTORY_URL").unwrap_or_else(|_| DEFAULT_DVPN_DIRECTORY.to_string())
}

/// Prefer mainnet defaults (funded Keplr `$NYM`). Only call `new_from_env()` when
/// `NETWORK_NAME` + `NYM_API` are already set (e.g. sourced `envs/sandbox.env`).
pub fn network_details() -> NymNetworkDetails {
    let from_env = std::env::var("NETWORK_NAME").is_ok() && std::env::var("NYM_API").is_ok();
    if from_env {
        NymNetworkDetails::new_from_env()
    } else {
        NymNetworkDetails::new_mainnet()
    }
}

pub async fn new_session(data_dir: &str) -> Session {
    let network = network_details();
    println!(
        "nym network: {} (set NETWORK_NAME+NYM_API to override, e.g. sandbox.env)",
        network.network_name
    );
    Session::new(
        SessionConfig {
            mnemonic: mnemonic(),
            network,
            credential_store_path: Some(format!("{data_dir}/creds.db").into()),
            data_path: data_dir.into(),
            dvpn_directory_url: Some(dvpn_directory_url()),
            automatic_topups: None,
            bandwidth_provider: None,
            reuse_registrations: true,
        },
        tokio_util::sync::CancellationToken::new(),
    )
    .await
    .expect("session init")
}

pub fn bridge_params(qb: &QuicBridge) -> BridgeParams {
    BridgeParams {
        addresses: qb.addresses.clone(),
        sni_host: qb.sni_host.clone(),
        id_pubkey_base64: qb.id_pubkey_base64.clone(),
    }
}

pub async fn build_two_hop_tunnel(reg: &Registration, use_quic: bool) -> Result<Tunnel, BoxError> {
    let entry = peer_from_hop(&reg.entry);
    let exit = peer_from_hop(
        reg.exit
            .as_ref()
            .ok_or("two-hop registration has no exit hop")?,
    );
    let mut builder = TunnelBuilder::two_hop(entry, exit);
    if use_quic {
        let qb = reg
            .entry
            .bridge
            .as_ref()
            .ok_or("QUIC requested but the entry hop carries no bridge params")?;
        builder = builder.quic_bridge(bridge_params(qb));
    }
    Ok(builder.connect().await?)
}

pub async fn build_tunnel(reg: &Registration, use_quic: bool) -> Result<Tunnel, BoxError> {
    if reg.exit.is_some() {
        build_two_hop_tunnel(reg, use_quic).await
    } else {
        let entry = peer_from_hop(&reg.entry);
        Ok(TunnelBuilder::single_hop(entry).connect().await?)
    }
}

pub const USAGE: &str = "\
options:
  --two-hop            entry + exit gateways (default)
  --one-hop            a single gateway (entry == exit); cannot be combined with --quic
  --entry <spec>       entry gateway selector (default: random)
  --exit <spec>        exit gateway selector (default: random)
  --gateway <spec>     set both entry and exit (handy for --one-hop)
  --quic               require a QUIC-bridge-capable entry gateway (two-hop only)
  --blocks <n>         number of compact blocks to sync (default 10000)
  --lwd <url>          lightwalletd base URL (default https://zec.rocks:443 or LIGHTWALLETD_GRPC)
  -h, --help           print this help

<spec> is one of:
  random               any WireGuard-capable gateway (default)
  <CC>                 two-letter ISO country code, e.g. DE, CH
  <base58>             exact gateway ed25519 identity";

pub struct Cli {
    pub two_hop: bool,
    pub entry: GatewaySpec,
    pub exit: GatewaySpec,
    pub quic: bool,
    pub blocks: Option<u64>,
    pub lwd_url: String,
}

fn parse_spec(s: &str) -> Result<GatewaySpec, BoxError> {
    if s.eq_ignore_ascii_case("random") {
        Ok(GatewaySpec::Random)
    } else if s.len() == 2 && s.chars().all(|c| c.is_ascii_alphabetic()) {
        Ok(GatewaySpec::Country(s.to_ascii_uppercase()))
    } else {
        let key = ed25519::PublicKey::from_base58_string(s)
            .map_err(|e| format!("invalid gateway spec {s:?}: {e}"))?;
        Ok(GatewaySpec::Identity(key))
    }
}

pub fn default_lwd_url() -> String {
    std::env::var("LIGHTWALLETD_GRPC").unwrap_or_else(|_| "https://zec.rocks:443".to_string())
}

pub fn parse_cli() -> Result<Cli, BoxError> {
    let mut two_hop = true;
    let (mut entry, mut exit) = (GatewaySpec::Random, GatewaySpec::Random);
    let mut quic = false;
    let mut blocks = None;
    let mut lwd_url = default_lwd_url();

    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--quic" => quic = true,
            "--one-hop" | "--single-hop" => two_hop = false,
            "--two-hop" => two_hop = true,
            "--entry" => {
                i += 1;
                entry = parse_spec(args.get(i).ok_or("--entry requires a value")?)?;
            }
            "--exit" => {
                i += 1;
                exit = parse_spec(args.get(i).ok_or("--exit requires a value")?)?;
            }
            "--gateway" => {
                i += 1;
                let s = parse_spec(args.get(i).ok_or("--gateway requires a value")?)?;
                entry = s.clone();
                exit = s;
            }
            "--blocks" => {
                i += 1;
                let v = args.get(i).ok_or("--blocks requires a value")?;
                blocks = Some(
                    v.parse()
                        .map_err(|_| format!("--blocks expects a number, got {v:?}"))?,
                );
            }
            "--lwd" => {
                i += 1;
                lwd_url = args.get(i).ok_or("--lwd requires a value")?.clone();
            }
            "-h" | "--help" => {
                println!("{USAGE}");
                std::process::exit(0);
            }
            other => return Err(format!("unknown argument {other:?}\n\n{USAGE}").into()),
        }
        i += 1;
    }

    if quic && !two_hop {
        return Err("--quic requires two-hop mode (QUIC fronts the entry leg only)".into());
    }
    Ok(Cli {
        two_hop,
        entry,
        exit,
        quic,
        blocks,
        lwd_url,
    })
}

pub async fn register(session: &Session, cli: &Cli) -> Result<Registration, BoxError> {
    session.ensure_ticketbooks(cli.two_hop).await?;
    let reg = if !cli.two_hop {
        session.register_single_hop(&cli.entry).await?
    } else if cli.quic {
        session.register_two_hop_quic(&cli.entry, &cli.exit).await?
    } else {
        session.register_two_hop(&cli.entry, &cli.exit).await?
    };
    Ok(reg)
}

pub fn describe(cli: &Cli) -> String {
    let mode = if cli.two_hop { "two-hop" } else { "single-hop" };
    let quic = if cli.quic { " (QUIC entry)" } else { "" };
    format!("{mode}{quic}")
}

pub fn peer_from_hop(hop: &HopConfig) -> PeerConfig {
    PeerConfig {
        gateway_public_key: hop.wg_config.public_key,
        // x25519::PrivateKey is !Clone; reconstruct from bytes so registration outlives connect().
        client_private_key: x25519::PrivateKey::from_secret(hop.client_private_key.to_bytes()),
        preshared_key: hop.wg_config.psk.as_ref().map(|p| *p.as_bytes()),
        endpoint: hop.wg_config.endpoint,
        assigned_ipv4: hop.wg_config.private_ipv4,
        assigned_ipv6: Some(hop.wg_config.private_ipv6),
    }
}

pub fn print_gateway(label: &str, gw: &GatewayInfo) {
    println!("  {label} gateway:");
    println!("    identity : {}", gw.identity.to_base58_string());
    println!(
        "    moniker  : {}",
        gw.name
            .as_deref()
            .unwrap_or("(none — Nym nodes have no moniker)")
    );
    println!("    node id  : {}", gw.node_id);
    println!(
        "    country  : {}",
        gw.country.as_deref().unwrap_or("unknown")
    );
    println!("    ip       : {}", gw.ip);
}

const IPINFO_HOST: &str = "ipinfo.io";

pub fn tls_config(alpn: &[&[u8]]) -> Arc<rustls::ClientConfig> {
    init_crypto();
    let mut roots = rustls::RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let mut cfg = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    cfg.alpn_protocols = alpn.iter().map(|a| a.to_vec()).collect();
    Arc::new(cfg)
}

pub async fn https_get_json<S>(stream: S, host: &str, path: &str) -> Result<Value, BoxError>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let connector = TlsConnector::from(tls_config(&[&b"http/1.1"[..]]));
    let sni = ServerName::try_from(host.to_string())?;
    let mut tls = connector.connect(sni, stream).await?;

    let req = format!(
        "GET {path} HTTP/1.1\r\nHost: {host}\r\nUser-Agent: nym-dvpn-lwd-spike\r\n\
         Accept: application/json\r\nConnection: close\r\n\r\n"
    );
    tls.write_all(req.as_bytes()).await?;

    let mut buf = Vec::new();
    let mut chunk = [0u8; 8192];
    loop {
        match tls.read(&mut chunk).await {
            Ok(0) => break,
            Ok(n) => buf.extend_from_slice(&chunk[..n]),
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        }
    }
    let text = String::from_utf8_lossy(&buf);
    let start = text.find('{').ok_or("no JSON body in response")?;
    let end = text.rfind('}').ok_or("truncated JSON body")?;
    Ok(serde_json::from_str(&text[start..=end])?)
}

pub async fn ipinfo_direct() -> Result<Value, BoxError> {
    let tcp = tokio::net::TcpStream::connect((IPINFO_HOST, 443)).await?;
    https_get_json(tcp, IPINFO_HOST, "/json").await
}

pub async fn ipinfo_via_tunnel(tunnel: &Tunnel) -> Result<Value, BoxError> {
    let stream = tunnel.tcp_connect_host(IPINFO_HOST, 443).await?;
    https_get_json(stream, IPINFO_HOST, "/json").await
}

pub fn fmt_ipinfo(v: &Value) -> String {
    let s = |k: &str| v.get(k).and_then(Value::as_str).unwrap_or("?").to_string();
    format!(
        "{} ({}, {}) — {}",
        s("ip"),
        s("city"),
        s("country"),
        s("org")
    )
}

#[derive(Clone, Default)]
pub struct DirectConnector;

impl Service<Uri> for DirectConnector {
    type Response = TokioIo<tokio::net::TcpStream>;
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, uri: Uri) -> Self::Future {
        Box::pin(async move {
            let host = uri.host().ok_or("uri missing host")?.to_string();
            let port = uri.port_u16().unwrap_or(443);
            let tcp = tokio::net::TcpStream::connect((host.as_str(), port)).await?;
            Ok(TokioIo::new(tcp))
        })
    }
}

#[derive(Clone)]
pub struct TlsWrap<C> {
    inner: C,
    config: Arc<rustls::ClientConfig>,
}

impl<C> TlsWrap<C> {
    pub fn h2(inner: C) -> Self {
        Self {
            inner,
            config: tls_config(&[&b"h2"[..]]),
        }
    }
}

impl<C, S> Service<Uri> for TlsWrap<C>
where
    C: Service<Uri, Response = TokioIo<S>> + Clone + Send + 'static,
    C::Future: Send + 'static,
    C::Error: Into<BoxError>,
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    type Response = TokioIo<tokio_rustls::client::TlsStream<S>>;
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, uri: Uri) -> Self::Future {
        let mut inner = self.inner.clone();
        let config = self.config.clone();
        Box::pin(async move {
            let host = uri.host().ok_or("uri missing host")?.to_string();
            let io = inner.call(uri).await.map_err(Into::into)?;
            let stream = io.into_inner();
            let sni = ServerName::try_from(host)?;
            let tls = TlsConnector::from(config).connect(sni, stream).await?;
            Ok(TokioIo::new(tls))
        })
    }
}
