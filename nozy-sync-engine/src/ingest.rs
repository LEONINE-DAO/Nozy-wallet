//! Tip-following ingest: Zebra/Zakura → compact SQLite.

use std::sync::Arc;
use std::time::Duration;

use tracing::{info, warn};

use crate::compact::{encode_compact_block, raw_block_to_compact};
use crate::error::IndexerResult;
use crate::rpc::{detect_node_kind, NodeInfo, RpcClient};
use crate::store::IndexerStore;
use crate::tree_sizes::tree_sizes_after_block;

pub struct IngestHandle {
    pub node: NodeInfo,
    pub node_kind: &'static str,
}

/// Probe node, persist chain labels, return handle for serve layer.
pub async fn bootstrap(
    rpc: &RpcClient,
    store: &IndexerStore,
    expected_network: &str,
) -> IndexerResult<IngestHandle> {
    let node = rpc.probe_node().await?;
    let node_kind = detect_node_kind(&node.subversion);
    if !expected_network.is_empty() && !node.chain.eq_ignore_ascii_case(expected_network) {
        return Err(crate::error::IndexerError::Config(format!(
            "node chain '{}' does not match NOZY_SYNC_ENGINE_NETWORK '{}'",
            node.chain, expected_network
        )));
    }
    store.set_meta("chain", &node.chain)?;
    store.set_meta("node_kind", node_kind)?;
    store.set_meta("subversion", &node.subversion)?;
    info!(
        %node_kind,
        chain = %node.chain,
        tip = node.blocks,
        auth = rpc.auth_configured(),
        "connected to Zebra-family RPC"
    );
    Ok(IngestHandle { node, node_kind })
}

/// Background loop: extend compact store to node tip; prune on reorg tip drop.
pub async fn run_ingest_loop(
    rpc: RpcClient,
    store: Arc<IndexerStore>,
    poll_ms: u64,
    start_height: u64,
    backfill: u64,
) {
    let interval = Duration::from_millis(poll_ms);
    loop {
        if let Err(e) = ingest_once(&rpc, &store, start_height, backfill).await {
            warn!(error = %e, "ingest pass failed");
        }
        tokio::time::sleep(interval).await;
    }
}

async fn ingest_once(
    rpc: &RpcClient,
    store: &IndexerStore,
    start_height: u64,
    backfill: u64,
) -> IndexerResult<()> {
    let tip = rpc.get_block_count().await?;
    store.set_meta("node_tip", &tip.to_string())?;

    if let Some(max) = store.max_height()? {
        if max > tip {
            let pruned = store.prune_above(tip)?;
            info!(pruned, tip, "pruned compact rows above node tip (reorg)");
        }
    }

    let next = match store.max_height()? {
        Some(h) => h.saturating_add(1),
        None => {
            let from_backfill = tip.saturating_sub(backfill);
            if start_height > 0 {
                start_height.max(from_backfill.min(start_height))
            } else {
                from_backfill
            }
        }
    };

    if next > tip {
        return Ok(());
    }

    let mut height = next;
    while height <= tip {
        match ingest_height(rpc, store, height).await {
            Ok(()) => {
                if height % 100 == 0 || height == tip {
                    info!(height, tip, "ingest progress");
                }
                height += 1;
            }
            Err(e) => {
                warn!(height, error = %e, "failed to ingest height; will retry");
                break;
            }
        }
    }
    Ok(())
}

async fn ingest_height(rpc: &RpcClient, store: &IndexerStore, height: u64) -> IndexerResult<()> {
    let raw = rpc.get_raw_block(height).await?;
    let mut compact = raw_block_to_compact(&raw, 0, 0, 0)?;
    if compact.height != height {
        return Err(crate::error::IndexerError::Compact(format!(
            "coinbase height {} != requested {}",
            compact.height, height
        )));
    }
    let sizes = tree_sizes_after_block(rpc, store, height, &compact).await?;
    compact.chain_metadata = Some(sizes.into_chain_metadata());
    let hash = compact.hash.clone();
    let bytes = encode_compact_block(&compact)?;
    store.put_compact_block(height, &hash, &bytes)?;
    store.set_meta("indexed_tip", &height.to_string())?;
    Ok(())
}
