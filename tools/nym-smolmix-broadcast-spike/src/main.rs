//! Nym smolmix broadcast / egress spike (issue #147).
//!
//! Modes: IP relocate, JSON-RPC probe, dry reachability, and `--sendraw` (wallet helper).
//!
//! Adapted from nymtech/nym `smolmix` example `tcp` (Apache-2.0 patterns).

use std::io::{self, Read};
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

use bytes::Bytes;
use http_body_util::BodyExt;
use hyper::Request;
use hyper_util::rt::TokioIo;
use nym_smolmix_broadcast_spike::{
    build_tunnel, dry_reachability_evidence, install_crypto, json_rpc_post_over_tunnel,
    parse_rpc_target, sendrawtransaction_over_smolmix, BoxError, EvidenceStep,
};
use rustls::pki_types::ServerName;
use rustls::RootCertStore;
use serde_json::json;
use smolmix::Tunnel;
use std::sync::Arc;
use tokio_rustls::TlsConnector;
use tracing::info;

const USAGE: &str = "\
NozyWallet nym-smolmix-broadcast-spike (issue #147)

Modes:
  --ip-relocate         Compare clearnet vs mixnet exit IP (default)
  --rpc-probe           POST getblockcount through mixnet
  --both                ip-relocate then rpc-probe
  --dry-reachability    Classify zebra URL (no tunnel) — LAN refuse vs candidate
  --sendraw <hex>       sendrawtransaction over mixnet; prints txid on stdout
  --sendraw-stdin       same, raw hex from stdin (preferred for long txs)

Options:
  --ipr <addr>          Optional IPR exit Recipient (default: auto-discover)
  --zebra <url>         Exit-reachable JSON-RPC base (ZEBRA_URL or default LAN URL)
  --evidence-json <p>   Append/write structured evidence JSON for case breakdowns
  -h, --help

Examples:
  cargo run --release -- --dry-reachability --zebra http://127.0.0.1:8232
  cargo run --release -- --ip-relocate --evidence-json docs/reference/evidence/nym-d2.json
  cargo run --release -- --rpc-probe --zebra https://EXIT_REACHABLE_HOST:18232
  cargo run --release -- --sendraw-stdin --zebra https://EXIT_REACHABLE_HOST:18232 < tx.hex

Wallet integration: set NOZY_BROADCAST_VIA_NYM_MIXNET=1 and NOZY_NYM_SMOLMIX_BIN to this binary.
Private/LAN zebra URLs are refused for mixnet RPC/sendraw (Case A1 stays direct).
";

#[derive(Debug, Clone, PartialEq, Eq)]
enum Mode {
    IpRelocate,
    RpcProbe,
    Both,
    DryReachability,
    SendRaw(String),
}

struct Cli {
    mode: Mode,
    ipr: Option<String>,
    zebra: String,
    evidence_json: Option<PathBuf>,
}

fn default_zebra_url() -> String {
    std::env::var("ZEBRA_URL").unwrap_or_else(|_| "http://172.20.199.206:18232".to_string())
}

fn parse_cli() -> Result<Cli, BoxError> {
    let mut mode = Mode::IpRelocate;
    let mut ipr = None;
    let mut zebra = default_zebra_url();
    let mut evidence_json = None;
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--ip-relocate" => mode = Mode::IpRelocate,
            "--rpc-probe" => mode = Mode::RpcProbe,
            "--both" => mode = Mode::Both,
            "--dry-reachability" => mode = Mode::DryReachability,
            "--sendraw" => {
                i += 1;
                let hex = args
                    .get(i)
                    .ok_or("--sendraw requires a hex value")?
                    .trim()
                    .trim_start_matches("0x")
                    .to_string();
                if hex.is_empty() {
                    return Err("--sendraw hex is empty".into());
                }
                mode = Mode::SendRaw(hex);
            }
            "--sendraw-stdin" => {
                let mut buf = String::new();
                io::stdin().read_to_string(&mut buf)?;
                let hex = buf
                    .chars()
                    .filter(|c| !c.is_whitespace())
                    .collect::<String>()
                    .trim_start_matches("0x")
                    .to_string();
                if hex.is_empty() {
                    return Err("--sendraw-stdin: empty stdin".into());
                }
                mode = Mode::SendRaw(hex);
            }
            "--ipr" => {
                i += 1;
                ipr = Some(args.get(i).ok_or("--ipr requires a value")?.clone());
            }
            "--zebra" => {
                i += 1;
                zebra = args.get(i).ok_or("--zebra requires a value")?.clone();
            }
            "--evidence-json" => {
                i += 1;
                evidence_json = Some(PathBuf::from(
                    args.get(i).ok_or("--evidence-json requires a path")?,
                ));
            }
            "-h" | "--help" => {
                print!("{USAGE}");
                std::process::exit(0);
            }
            other => return Err(format!("unknown argument {other:?}\n\n{USAGE}").into()),
        }
        i += 1;
    }
    Ok(Cli {
        mode,
        ipr,
        zebra,
        evidence_json,
    })
}

fn tls_connector() -> TlsConnector {
    let mut roots = RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let cfg = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    TlsConnector::from(Arc::new(cfg))
}

fn extract_cf_ip(body: &str) -> Option<&str> {
    body.lines().find_map(|l| l.strip_prefix("ip="))
}

fn print_step(step: &EvidenceStep) {
    println!("  [{}] {}: {}", step.result, step.id, step.detail);
}

fn write_evidence(path: &PathBuf, steps: &[EvidenceStep]) -> Result<(), BoxError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let doc = json!({
        "generated_at_unix": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
        "tool": "nym-smolmix-broadcast-spike",
        "issue": 147,
        "steps": steps.iter().map(EvidenceStep::to_json).collect::<Vec<_>>(),
    });
    std::fs::write(path, serde_json::to_string_pretty(&doc)?)?;
    eprintln!("wrote evidence → {}", path.display());
    Ok(())
}

async fn clearnet_cf_trace() -> Result<(String, std::time::Duration), BoxError> {
    let t0 = Instant::now();
    let resp = reqwest::get("https://cloudflare.com/cdn-cgi/trace").await?;
    let body = resp.text().await?;
    Ok((body, t0.elapsed()))
}

async fn mixnet_cf_trace(tunnel: &Tunnel) -> Result<(String, std::time::Duration), BoxError> {
    const HOST: &str = "cloudflare.com";
    const PATH: &str = "/cdn-cgi/trace";
    let t0 = Instant::now();

    let tcp = tunnel.tcp_connect("1.1.1.1:443".parse()?).await?;
    let domain = ServerName::try_from(HOST)?.to_owned();
    let tls = tls_connector().connect(domain, tcp).await?;

    let (mut sender, conn) = hyper::client::conn::http1::handshake(TokioIo::new(tls)).await?;
    tokio::spawn(async move {
        let _ = conn.await;
    });

    let req = Request::get(PATH)
        .header("Host", HOST)
        .body(http_body_util::Empty::<Bytes>::new())?;
    let resp = sender.send_request(req).await?;
    let body = resp.into_body().collect().await?.to_bytes();
    Ok((String::from_utf8_lossy(&body).into_owned(), t0.elapsed()))
}

async fn run_ip_relocate(tunnel: &Tunnel) -> Result<EvidenceStep, BoxError> {
    info!("Fetching Cloudflare /cdn-cgi/trace via clearnet…");
    let (clear_body, clear_dt) = clearnet_cf_trace().await?;
    let clear_ip = extract_cf_ip(&clear_body).unwrap_or("?");

    info!("Fetching via smolmix mixnet…");
    let (mix_body, mix_dt) = mixnet_cf_trace(tunnel).await?;
    let mix_ip = extract_cf_ip(&mix_body).unwrap_or("?");

    println!("\n=== IP relocate (D2a — biggest-win prerequisite) ===");
    println!("  clearnet IP : {clear_ip}  ({clear_dt:?})");
    println!("  mixnet IP   : {mix_ip}  ({mix_dt:?})");

    let step = if clear_ip != "?" && mix_ip != "?" && clear_ip != mix_ip {
        println!("  PASS: exit IP differs from host — mixnet egress works");
        EvidenceStep::pass(
            "D2a",
            format!(
                "clearnet={clear_ip} ({clear_dt:?}); mixnet={mix_ip} ({mix_dt:?}); exit differs"
            ),
        )
    } else if clear_ip == mix_ip {
        println!("  WARN: IPs match — check IPR / network; not safe to claim IP hide yet");
        EvidenceStep::fail(
            "D2a",
            format!("clearnet and mixnet both reported {clear_ip}"),
        )
    } else {
        println!("  WARN: could not parse one or both IPs");
        EvidenceStep::fail(
            "D2a",
            format!("unparsed IPs clearnet={clear_ip:?} mixnet={mix_ip:?}"),
        )
    };
    print_step(&step);
    Ok(step)
}

async fn run_rpc_probe(tunnel: &Tunnel, zebra: &str) -> Result<EvidenceStep, BoxError> {
    let reach = dry_reachability_evidence(zebra);
    if reach.result == "N/A" {
        println!("\n=== JSON-RPC probe (D2b) ===");
        print_step(&reach);
        return Ok(EvidenceStep::blocked(
            "D2b",
            format!("skipped: {}", reach.detail),
        ));
    }

    let target = parse_rpc_target(zebra)?;
    let payload = r#"{"jsonrpc":"2.0","id":1,"method":"getblockcount","params":[]}"#;

    info!(
        "JSON-RPC probe via mixnet → {} ({}) tls={}",
        target.addr, target.host, target.use_tls
    );
    let (text, elapsed) = json_rpc_post_over_tunnel(tunnel, zebra, payload).await?;

    println!("\n=== JSON-RPC probe (D2b — sendrawtransaction path shape) ===");
    println!("  target  : {zebra}");
    println!(
        "  via     : smolmix TCP → {}:{}",
        target.addr.ip(),
        target.addr.port()
    );
    println!("  elapsed : {elapsed:?}");
    println!("  body    : {text}");

    let step = if text.contains("\"result\"") || text.contains("\"error\"") {
        // JSON-RPC error body still proves the mixnet TCP/HTTP path (auth may fail).
        let ok_result = text.contains("\"result\"") && !text.contains("\"error\":{");
        if ok_result {
            println!("  PASS: received JSON-RPC result over mixnet");
            EvidenceStep::pass(
                "D2b",
                format!("getblockcount over mixnet to {zebra} in {elapsed:?}: {text}"),
            )
        } else {
            println!("  PASS(path): JSON-RPC response over mixnet (check auth/method)");
            EvidenceStep::pass(
                "D2b-path",
                format!("JSON-RPC body over mixnet to {zebra} in {elapsed:?}: {text}"),
            )
        }
    } else {
        println!("  WARN: unexpected body — check auth / path / node");
        EvidenceStep::fail("D2b", format!("unexpected body from {zebra}: {text}"))
    };
    print_step(&step);
    Ok(step)
}

async fn run_sendraw(zebra: &str, hex: &str, ipr: Option<&str>) -> Result<EvidenceStep, BoxError> {
    let reach = dry_reachability_evidence(zebra);
    if reach.result == "N/A" {
        return Err(format!("sendraw blocked: {}", reach.detail).into());
    }
    let _ = parse_rpc_target(zebra)?;
    info!(
        "sendrawtransaction via smolmix → {zebra} ({} hex chars)",
        hex.len()
    );
    let txid = sendrawtransaction_over_smolmix(zebra, hex, ipr).await?;
    // Machine-readable: sole stdout line for wallet subprocess parser.
    println!("{txid}");
    Ok(EvidenceStep::pass(
        "D2c",
        format!("sendrawtransaction over mixnet → txid {txid}"),
    ))
}

async fn run() -> Result<(), BoxError> {
    let cli = parse_cli()?;
    let sendraw = matches!(cli.mode, Mode::SendRaw(_));
    let mut steps: Vec<EvidenceStep> = Vec::new();

    // Logs on stderr so --sendraw stdout stays a single txid line.
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                tracing_subscriber::EnvFilter::new(if sendraw { "warn" } else { "info" })
            }),
        )
        .init();
    install_crypto();

    if !sendraw {
        println!("NozyWallet nym-smolmix-broadcast-spike (issue #147)");
        println!("mode={:?} zebra={}", cli.mode, cli.zebra);
    } else {
        eprintln!(
            "nym-smolmix-broadcast-spike sendraw zebra={} ipr={:?}",
            cli.zebra, cli.ipr
        );
    }

    match &cli.mode {
        Mode::DryReachability => {
            let step = dry_reachability_evidence(&cli.zebra);
            println!("\n=== Dry reachability (no tunnel) ===");
            print_step(&step);
            steps.push(step);
            if let Some(path) = &cli.evidence_json {
                write_evidence(path, &steps)?;
            }
            return Ok(());
        }
        Mode::SendRaw(hex) => {
            let step = run_sendraw(&cli.zebra, hex, cli.ipr.as_deref()).await?;
            steps.push(step);
            if let Some(path) = &cli.evidence_json {
                // Evidence goes to stderr-side file; stdout remains txid-only.
                write_evidence(path, &steps)?;
            }
            return Ok(());
        }
        Mode::RpcProbe | Mode::Both => {
            let reach = dry_reachability_evidence(&cli.zebra);
            steps.push(reach.clone());
            if reach.result == "N/A" {
                println!("\n=== Pre-check ===");
                print_step(&reach);
                steps.push(EvidenceStep::blocked(
                    "D2b",
                    "not run — provide exit-reachable --zebra URL",
                ));
                if let Some(path) = &cli.evidence_json {
                    write_evidence(path, &steps)?;
                }
                return Ok(());
            }
            let _ = parse_rpc_target(&cli.zebra)?;
        }
        Mode::IpRelocate => {}
    }

    info!("Building smolmix tunnel (may take a while on first connect)…");
    let tunnel = build_tunnel(cli.ipr.as_deref()).await?;
    info!("Tunnel ready");

    match &cli.mode {
        Mode::IpRelocate => steps.push(run_ip_relocate(&tunnel).await?),
        Mode::RpcProbe => steps.push(run_rpc_probe(&tunnel, &cli.zebra).await?),
        Mode::Both => {
            steps.push(run_ip_relocate(&tunnel).await?);
            steps.push(run_rpc_probe(&tunnel, &cli.zebra).await?);
        }
        Mode::SendRaw(_) | Mode::DryReachability => unreachable!(),
    }

    tunnel.shutdown().await;
    if let Some(path) = &cli.evidence_json {
        write_evidence(path, &steps)?;
    }
    Ok(())
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> ExitCode {
    if let Err(e) = run().await {
        eprintln!("error: {e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
