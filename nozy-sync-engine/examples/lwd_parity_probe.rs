//! Spot-check compact blocks against a reference CompactTxStreamer (stock lightwalletd).
//!
//! Encode path (Zebrad RPC): `ZEBRA_RPC_URL` set → parse `getblock` and compare to LWD.
//! Serve path: `NOZY_PARITY_ENGINE_GRPC` set → GetBlock from Nozy Sync Engine vs LWD.
//!
//! ```text
//! set NOZY_PARITY_LWD_GRPC=https://zec.rocks:443
//! cargo run -p nozy-sync-engine --example lwd_parity_probe
//! ```

use nozy_sync_engine::compact::raw_block_to_compact;
use nozy_sync_engine::parity::compare_compact;
use nozy_sync_engine::proto::compact_tx_streamer_client::CompactTxStreamerClient;
use nozy_sync_engine::proto::{BlockId, ChainSpec, CompactBlock, Empty};
use nozy_sync_engine::rpc::RpcClient;
use nozy_sync_engine::tree_sizes::{
    tree_sizes_from_getblock_json, tree_sizes_from_zebra_json, TreeSizes,
};

fn env_url(key: &str, default: &str) -> String {
    std::env::var(key)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| default.to_string())
}

async fn fetch_block(
    client: &mut CompactTxStreamerClient<tonic::transport::Channel>,
    height: u64,
) -> Result<CompactBlock, Box<dyn std::error::Error>> {
    let b = client
        .get_block(BlockId {
            height,
            hash: vec![],
        })
        .await?
        .into_inner();
    Ok(b)
}

async fn encode_from_rpc(
    rpc: &RpcClient,
    height: u64,
) -> Result<CompactBlock, Box<dyn std::error::Error>> {
    let raw = rpc.get_raw_block(height).await?;
    let mut compact = raw_block_to_compact(&raw, 0, 0, 0)?;
    let mut sizes = TreeSizes::default();
    if let Ok(ts) = rpc.z_gettreestate(height).await {
        sizes = tree_sizes_from_zebra_json(&ts);
    }
    if sizes.is_zero() {
        if let Ok(hash) = rpc.get_block_hash(height).await {
            if let Ok(block) = rpc.get_block_verbose(&hash).await {
                sizes = tree_sizes_from_getblock_json(&block);
            }
        }
    }
    compact.chain_metadata = Some(sizes.into_chain_metadata());
    Ok(compact)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lwd = env_url("NOZY_PARITY_LWD_GRPC", "https://zec.rocks:443");
    let engine = std::env::var("NOZY_PARITY_ENGINE_GRPC")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let rpc_url = std::env::var("ZEBRA_RPC_URL")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let n: u64 = std::env::var("NOZY_PARITY_BLOCKS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5);

    println!("reference_lwd: {lwd}");
    if let Some(e) = &engine {
        println!("engine: {e}");
    }
    if let Some(r) = &rpc_url {
        println!("zebra_rpc: {r}");
    }

    let mut lwd_client = CompactTxStreamerClient::connect(lwd.clone()).await?;
    let info = lwd_client.get_lightd_info(Empty {}).await?.into_inner();
    let tip = lwd_client
        .get_latest_block(ChainSpec {})
        .await?
        .into_inner()
        .height;
    println!(
        "lwd_info vendor={} chain={} tip={}",
        info.vendor, info.chain_name, tip
    );

    let start = tip.saturating_sub(n.saturating_sub(1));
    let rpc = match &rpc_url {
        Some(url) => Some(RpcClient::new(url)?),
        None => None,
    };

    let mut engine_client = match &engine {
        Some(url) => Some(CompactTxStreamerClient::connect(url.clone()).await?),
        None => None,
    };

    let mut fail = 0u32;
    let mut pass = 0u32;
    for height in start..=tip {
        let reference = fetch_block(&mut lwd_client, height).await?;
        if let Some(rpc) = &rpc {
            let ours = encode_from_rpc(rpc, height).await?;
            let report = compare_compact(&ours, &reference);
            if report.ok() {
                println!("PASS encode height={height} vtx={}", ours.vtx.len());
                pass += 1;
            } else {
                println!("FAIL encode height={height}");
                for i in &report.issues {
                    println!("  {} ours={} ref={}", i.field, i.ours, i.reference);
                }
                fail += 1;
            }
        }
        if let Some(eng_client) = engine_client.as_mut() {
            let ours = fetch_block(eng_client, height).await?;
            let report = compare_compact(&ours, &reference);
            if report.ok() {
                println!("PASS engine height={height} vtx={}", ours.vtx.len());
                pass += 1;
            } else {
                println!("FAIL engine height={height}");
                for i in &report.issues {
                    println!("  {} ours={} ref={}", i.field, i.ours, i.reference);
                }
                fail += 1;
            }
        }
        if rpc.is_none() && engine.is_none() {
            println!(
                "LWD_ONLY height={height} hash={} vtx={} sapling_tree={} orchard_tree={}",
                hex::encode(&reference.hash),
                reference.vtx.len(),
                reference
                    .chain_metadata
                    .as_ref()
                    .map(|m| m.sapling_commitment_tree_size)
                    .unwrap_or(0),
                reference
                    .chain_metadata
                    .as_ref()
                    .map(|m| m.orchard_commitment_tree_size)
                    .unwrap_or(0),
            );
        }
    }

    if rpc.is_none() && engine.is_none() {
        eprintln!(
            "set ZEBRA_RPC_URL and/or NOZY_PARITY_ENGINE_GRPC to compare against the reference LWD"
        );
        std::process::exit(2);
    }
    println!("summary pass={pass} fail={fail} heights={start}..={tip}");
    if fail > 0 {
        std::process::exit(1);
    }
    Ok(())
}
