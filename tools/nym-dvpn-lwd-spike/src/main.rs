//! Zcash compact-block sync over Nym 2-hop smoldvpn.
//!
//! Adapted from nymtech/nym `smoldvpn` example `zcash-sync` (Apache-2.0).
//! Tracking: https://github.com/LEONINE-DAO/Nozy-wallet/issues/146
//!
//! Build with `--release`. Requires `MNEMONIC`. Defaults to mainnet network
//! details when `NETWORK_NAME`/`NYM_API` are unset (see README.md).

mod common;

use std::process::ExitCode;
use std::time::{Duration, Instant};

use common::{BoxError, DirectConnector, TlsWrap};
use http::uri::PathAndQuery;
use smoldvpn::Tunnel;
use tonic::client::Grpc;
use tonic::transport::{Channel, Endpoint};
use tonic::Request;
use tonic_prost::ProstCodec;

const DEFAULT_BLOCKS: u64 = 10_000;
const GET_LATEST_BLOCK: &str = "/cash.z.wallet.sdk.rpc.CompactTxStreamer/GetLatestBlock";
const GET_BLOCK_RANGE: &str = "/cash.z.wallet.sdk.rpc.CompactTxStreamer/GetBlockRange";

#[derive(Clone, PartialEq, prost::Message)]
struct ChainSpec {}

#[derive(Clone, PartialEq, prost::Message)]
struct BlockId {
    #[prost(uint64, tag = "1")]
    height: u64,
    #[prost(bytes = "vec", tag = "2")]
    hash: Vec<u8>,
}

#[derive(Clone, PartialEq, prost::Message)]
struct BlockRange {
    #[prost(message, optional, tag = "1")]
    start: Option<BlockId>,
    #[prost(message, optional, tag = "2")]
    end: Option<BlockId>,
}

#[derive(Clone, PartialEq, prost::Message)]
struct CompactBlock {
    #[prost(uint64, tag = "2")]
    height: u64,
}

fn normalize_endpoint(url: &str) -> String {
    let u = url.trim().trim_end_matches('/');
    if u.starts_with("http://") || u.starts_with("https://") {
        u.to_string()
    } else {
        format!("https://{u}")
    }
}

async fn grpc_channel_direct(lwd: &str) -> Result<Channel, BoxError> {
    let endpoint = normalize_endpoint(lwd);
    if endpoint.starts_with("https://") {
        Ok(Endpoint::from_shared(endpoint)?
            .connect_with_connector(TlsWrap::h2(DirectConnector))
            .await?)
    } else {
        Ok(Endpoint::from_shared(endpoint)?.connect().await?)
    }
}

async fn grpc_channel_tunnel(lwd: &str, tunnel: &Tunnel) -> Result<Channel, BoxError> {
    let endpoint = normalize_endpoint(lwd);
    if endpoint.starts_with("https://") {
        Ok(Endpoint::from_shared(endpoint)?
            .connect_with_connector(TlsWrap::h2(tunnel.connector()))
            .await?)
    } else {
        Ok(Endpoint::from_shared(endpoint)?
            .connect_with_connector(tunnel.connector())
            .await?)
    }
}

async fn sync_last_blocks(channel: Channel, n_blocks: u64) -> Result<(u64, Duration), BoxError> {
    let mut grpc = Grpc::new(channel);
    grpc.ready().await?;

    let latest: BlockId = grpc
        .unary(
            Request::new(ChainSpec {}),
            PathAndQuery::from_static(GET_LATEST_BLOCK),
            ProstCodec::<ChainSpec, BlockId>::default(),
        )
        .await?
        .into_inner();
    let top = latest.height;
    let start = top.saturating_sub(n_blocks.saturating_sub(1));

    let range = BlockRange {
        start: Some(BlockId {
            height: start,
            hash: Vec::new(),
        }),
        end: Some(BlockId {
            height: top,
            hash: Vec::new(),
        }),
    };

    grpc.ready().await?;
    let t0 = Instant::now();
    let mut stream = grpc
        .server_streaming(
            Request::new(range),
            PathAndQuery::from_static(GET_BLOCK_RANGE),
            ProstCodec::<BlockRange, CompactBlock>::default(),
        )
        .await?
        .into_inner();

    let mut count = 0u64;
    while let Some(_block) = stream.message().await? {
        count += 1;
    }
    Ok((count, t0.elapsed()))
}

fn report(label: &str, blocks: u64, elapsed: Duration) {
    let secs = elapsed.as_secs_f64();
    let rate = if secs > 0.0 {
        blocks as f64 / secs
    } else {
        0.0
    };
    println!("  {label}: {blocks} blocks in {secs:.2}s ({rate:.0} blocks/s)");
}

async fn run() -> Result<(), BoxError> {
    common::init_crypto();
    let cli = common::parse_cli()?;
    let lwd = cli.lwd_url.clone();
    let n_blocks = cli.blocks.unwrap_or(DEFAULT_BLOCKS);

    println!("NozyWallet nym-dvpn-lwd-spike (issue #146)");
    println!("lightwalletd: {lwd}");

    println!(
        "real IP (no tunnel): {}",
        common::fmt_ipinfo(&common::ipinfo_direct().await?)
    );

    println!("\nsyncing last {n_blocks} blocks from {lwd} directly …");
    let (out_blocks, out_time) = sync_last_blocks(grpc_channel_direct(&lwd).await?, n_blocks).await?;
    report("direct", out_blocks, out_time);

    println!("\nprovisioning a {} tunnel …", common::describe(&cli));
    let session = common::new_session("nym-dvpn-lwd-spike-data").await;
    let reg = common::register(&session, &cli).await?;
    common::print_gateway("entry", &reg.entry.gateway);
    if let Some(exit) = reg.exit.as_ref() {
        common::print_gateway("exit", &exit.gateway);
    }
    let tunnel: Tunnel = common::build_tunnel(&reg, cli.quic).await?;

    let mut ip = None;
    for attempt in 1..=10 {
        match common::ipinfo_via_tunnel(&tunnel).await {
            Ok(v) => {
                ip = Some(v);
                break;
            }
            Err(e) => {
                println!("  tunnel warmup attempt {attempt} ({e}); retrying");
                tokio::time::sleep(Duration::from_secs(3)).await;
            }
        }
    }
    let ip = ip.ok_or("tunnel did not become usable after warmup")?;
    println!("IP through the tunnel: {}", common::fmt_ipinfo(&ip));

    println!("\nsyncing last {n_blocks} blocks from {lwd} through the tunnel …");
    let (in_blocks, in_time) =
        sync_last_blocks(grpc_channel_tunnel(&lwd, &tunnel).await?, n_blocks).await?;
    report("tunnel", in_blocks, in_time);

    let slowdown = in_time.as_secs_f64() / out_time.as_secs_f64().max(1e-9);
    println!("\ncomparison:");
    report("direct", out_blocks, out_time);
    report("tunnel", in_blocks, in_time);
    println!("  tunnel took {slowdown:.2}x the direct time");

    let _ = tokio::time::timeout(Duration::from_secs(5), tunnel.shutdown()).await;
    println!("PASS: synced {n_blocks} blocks inside and outside the tunnel");
    Ok(())
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> ExitCode {
    if let Err(e) = run().await {
        eprintln!("error: {e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
