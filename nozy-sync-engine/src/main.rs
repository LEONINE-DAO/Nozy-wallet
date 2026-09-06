//! `nozy-sync-engine` — Nozy Sync Engine binary (Zebra/Zakura → CompactTxStreamer).
//!
//! Issue: https://github.com/LEONINE-DAO/Nozy-wallet/issues/274

use std::sync::Arc;

use clap::Parser;
use tokio::sync::RwLock;
use tracing::info;

use nozy_sync_engine::banner::print_startup_banner;
use nozy_sync_engine::config::IndexerConfig;
use nozy_sync_engine::ingest::{bootstrap, run_ingest_loop};
use nozy_sync_engine::rpc::RpcClient;
use nozy_sync_engine::serve::ServeState;
use nozy_sync_engine::store::IndexerStore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = IndexerConfig::parse();
    cfg.validate()?;

    print_startup_banner(
        cfg.no_banner,
        cfg.print_banner,
        env!("CARGO_PKG_VERSION"),
        cfg.effective_rpc_url(),
        &cfg.bind.to_string(),
    );
    if cfg.print_banner {
        return Ok(());
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&cfg.rust_log)),
        )
        .init();

    let rpc_url = cfg.effective_rpc_url().to_string();
    info!(%rpc_url, bind = %cfg.bind, db = %cfg.db_path.display(), "starting Nozy Sync Engine");

    let rpc = RpcClient::new(&rpc_url)?;
    let store = Arc::new(IndexerStore::open(&cfg.db_path)?);
    let handle = bootstrap(&rpc, &store, &cfg.network).await?;

    let node = Arc::new(RwLock::new(handle.node.clone()));
    let rpc_serve = Arc::new(RpcClient::new(&rpc_url)?);
    let serve = ServeState {
        store: Arc::clone(&store),
        rpc: rpc_serve,
        node: Arc::clone(&node),
        node_kind: handle.node_kind,
        vendor: "NozyWallet/nozy-sync-engine".into(),
        version: env!("CARGO_PKG_VERSION").into(),
    };

    let poll_ms = cfg.poll_ms;
    let start_height = cfg.start_height;
    let backfill = cfg.backfill;
    let rpc_ingest = RpcClient::new(&rpc_url)?;
    let store_ingest = Arc::clone(&store);
    tokio::spawn(async move {
        run_ingest_loop(rpc_ingest, store_ingest, poll_ms, start_height, backfill).await;
    });

    let rpc_meta = RpcClient::new(&rpc_url)?;
    let node_meta = Arc::clone(&node);
    tokio::spawn(async move {
        loop {
            if let Ok(info) = rpc_meta.probe_node().await {
                *node_meta.write().await = info;
            }
            tokio::time::sleep(std::time::Duration::from_secs(15)).await;
        }
    });

    info!(
        "CompactTxStreamer listening on {} (point LIGHTWALLETD_GRPC here)",
        cfg.bind
    );
    tonic::transport::Server::builder()
        .add_service(serve.into_service())
        .serve(cfg.bind)
        .await?;

    Ok(())
}
