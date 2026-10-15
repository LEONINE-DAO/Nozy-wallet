//! Print Nozy lightwalletd tree-state sizes. No wallet keys.
//!
//! `cargo run -p zeaking --features lightwalletd --example tree_state_probe`

use zeaking::lwd::{connect_lightwalletd, get_latest_tree_state};

#[tokio::main]
async fn main() {
    let url = std::env::var("LIGHTWALLETD_GRPC")
        .unwrap_or_else(|_| "https://lwd.nozywallet.org:443".to_string());
    let mut client = connect_lightwalletd(&url)
        .await
        .unwrap_or_else(|e| panic!("connect {url}: {e}"));
    let ts = get_latest_tree_state(&mut client)
        .await
        .unwrap_or_else(|e| panic!("GetLatestTreeState: {e}"));
    println!(
        "height={} network={} sapling_len={} orchard_len={} ironwood_len={}",
        ts.height,
        ts.network,
        ts.sapling_tree.len(),
        ts.orchard_tree.len(),
        ts.ironwood_tree.len()
    );
}
