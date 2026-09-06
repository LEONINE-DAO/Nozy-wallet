//! Live probe: GetLightdInfo + GetLatestBlock + GetLatestTreeState against a running engine.
//!
//! ```text
//! cargo run -p nozy-sync-engine --example live_smoke_probe
//! ```

use nozy_sync_engine::proto::compact_tx_streamer_client::CompactTxStreamerClient;
use nozy_sync_engine::proto::{ChainSpec, Empty};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::var("LIGHTWALLETD_GRPC").unwrap_or_else(|_| "http://127.0.0.1:9067".into());
    let mut c = CompactTxStreamerClient::connect(url).await?;
    let info = c.get_lightd_info(Empty {}).await?.into_inner();
    println!(
        "PASS get_lightd_info vendor={} chain={} height={} est={} sub={}",
        info.vendor,
        info.chain_name,
        info.block_height,
        info.estimated_height,
        info.zcashd_subversion
    );
    let tip = c.get_latest_block(ChainSpec {}).await?.into_inner();
    println!(
        "PASS get_latest_block height={} hash_bytes={}",
        tip.height,
        tip.hash.len()
    );
    let ts = c.get_latest_tree_state(Empty {}).await?.into_inner();
    println!(
        "PASS get_latest_tree_state height={} hash={} sapling_chars={} orchard_chars={}",
        ts.height,
        &ts.hash.chars().take(16).collect::<String>(),
        ts.sapling_tree.len(),
        ts.orchard_tree.len()
    );
    Ok(())
}
