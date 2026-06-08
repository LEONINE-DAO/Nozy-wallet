use prost::Message;
use tonic::Status;

use super::client::LwdClient;
use super::proto::{BlockId, BlockRange, ChainSpec};
use super::store::LwdCompactStore;
use crate::error::{ZeakingError, ZeakingResult};

pub(crate) fn map_grpc_status(op: &'static str, status: Status) -> ZeakingError {
    ZeakingError::Grpc(format!(
        "{}: {} (code: {:?})",
        op,
        status.message(),
        status.code()
    ))
}

fn map_grpc_transport(op: &'static str, display: impl std::fmt::Display) -> ZeakingError {
    ZeakingError::Grpc(format!("{op}: {display}"))
}

/// Options for [`sync_compact_range_with_options`] (resume + progress checkpoints).
#[derive(Clone, Debug)]
pub struct SyncCompactOptions {
    /// When `true`, only fetch heights after the highest row in `compact_blocks`:
    /// `effective_start = max(requested_start, max_compact_height + 1)` (still capped by `end`).
    /// Safe to combine with retries: already-stored heights are not re-downloaded.
    pub resume_from_store: bool,
    /// Persist `sync_meta.last_compact_progress` at least once every `N` successfully written blocks.
    /// Minimum effective value is `1` (every block).
    pub persist_progress_every: u64,
}

impl Default for SyncCompactOptions {
    fn default() -> Self {
        Self {
            resume_from_store: false,
            persist_progress_every: 32,
        }
    }
}

/// Result of a compact sync attempt (successful completion or partial write before error).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyncCompactStats {
    pub blocks_written: u64,
    pub range_start_requested: u64,
    pub range_start_effective: u64,
    pub range_end: u64,
}

/// Options for [`sync_compact_to_tip_with_options`].
#[derive(Clone, Debug)]
pub struct SyncCompactToTipOptions {
    /// Optional lower bound for the first height to fetch (e.g. Orchard activation or wallet birthday).
    /// Heights below this are never requested. Default is `1` when `None`.
    pub start_floor: Option<u64>,
    /// Checkpoint interval for [`SyncCompactOptions::persist_progress_every`].
    pub persist_progress_every: u64,
}

impl Default for SyncCompactToTipOptions {
    fn default() -> Self {
        Self {
            start_floor: None,
            persist_progress_every: SyncCompactOptions::default().persist_progress_every,
        }
    }
}

/// Result of [`sync_compact_to_tip_with_options`] (includes tip context for UI / callers).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyncCompactToTipStats {
    pub chain_tip: u64,
    pub blocks_written: u64,
    pub range_start_requested: u64,
    pub range_start_effective: u64,
    pub range_end: u64,
    /// `true` when the store is already caught up through `chain_tip` (no range to download).
    pub already_at_tip: bool,
}

/// First height to use for a “sync to tip” pass: `max(max_compact_height + 1, start_floor_or_1)`.
pub fn requested_start_height_for_tip_sync(
    store: &LwdCompactStore,
    start_floor: Option<u64>,
) -> ZeakingResult<u64> {
    let floor = start_floor.unwrap_or(1);
    Ok(match store.max_compact_height()? {
        Some(m) => m.saturating_add(1).max(floor),
        None => floor,
    })
}

/// Download compact blocks from the next missing height through the current lightwalletd tip.
///
/// Uses [`sync_compact_range_with_options`] with `resume_from_store: true` so retries stay idempotent.
/// Call [`requested_start_height_for_tip_sync`] if you only need the resolved start height.
/// Drop compact blocks above `tip` when the local cache is ahead of the current lightwalletd chain.
pub fn prune_stale_compact_cache(store: &LwdCompactStore, tip: u64) -> ZeakingResult<u64> {
    match store.max_compact_height()? {
        Some(max) if max > tip => store.prune_compact_blocks_above(tip),
        _ => Ok(0),
    }
}

pub async fn sync_compact_to_tip_with_options(
    client: &mut LwdClient,
    store: &LwdCompactStore,
    options: SyncCompactToTipOptions,
) -> ZeakingResult<SyncCompactToTipStats> {
    let tip = chain_tip_height(client).await?;
    let _pruned = prune_stale_compact_cache(store, tip)?;
    let start = requested_start_height_for_tip_sync(store, options.start_floor)?;
    let compact = SyncCompactOptions {
        resume_from_store: true,
        persist_progress_every: options.persist_progress_every,
    };
    let stats = sync_compact_range_with_options(client, store, start, tip, compact).await?;
    let already_at_tip = stats.range_start_effective > stats.range_end;
    Ok(SyncCompactToTipStats {
        chain_tip: tip,
        blocks_written: stats.blocks_written,
        range_start_requested: stats.range_start_requested,
        range_start_effective: stats.range_start_effective,
        range_end: stats.range_end,
        already_at_tip,
    })
}

/// Same as [`sync_compact_to_tip_with_options`] with [`SyncCompactToTipOptions::default`].
pub async fn sync_compact_to_tip(
    client: &mut LwdClient,
    store: &LwdCompactStore,
) -> ZeakingResult<SyncCompactToTipStats> {
    sync_compact_to_tip_with_options(client, store, SyncCompactToTipOptions::default()).await
}

/// Download inclusive `[start, end]` compact blocks from lightwalletd into `store`.
/// Returns the number of blocks written.
///
/// Equivalent to [`sync_compact_range_with_options`] with [`SyncCompactOptions::default`] (no resume).
pub async fn sync_compact_range(
    client: &mut LwdClient,
    store: &LwdCompactStore,
    start: u64,
    end: u64,
) -> ZeakingResult<u64> {
    Ok(
        sync_compact_range_with_options(client, store, start, end, SyncCompactOptions::default())
            .await?
            .blocks_written,
    )
}

/// Same as [`sync_compact_range`] with explicit resume / SQLite progress checkpoints.
///
/// Writes metadata keys documented on [`crate::lwd::LwdCompactStore`].
pub async fn sync_compact_range_with_options(
    client: &mut LwdClient,
    store: &LwdCompactStore,
    start: u64,
    end: u64,
    options: SyncCompactOptions,
) -> ZeakingResult<SyncCompactStats> {
    if end < start {
        return Err(ZeakingError::InvalidOperation(format!(
            "sync_compact_range: end {end} < start {start}"
        )));
    }

    store.set_meta("last_sync_requested_start", &start.to_string())?;
    store.set_meta("last_sync_requested_end", &end.to_string())?;

    let mut eff_start = start;
    if options.resume_from_store {
        if let Some(m) = store.max_compact_height()? {
            eff_start = eff_start.max(m.saturating_add(1));
        }
    }

    if eff_start > end {
        store.set_meta("last_sync_end", &end.to_string())?;
        if let Some(h) = store.max_compact_height()? {
            store.set_meta("last_compact_progress", &h.to_string())?;
        }
        return Ok(SyncCompactStats {
            blocks_written: 0,
            range_start_requested: start,
            range_start_effective: eff_start,
            range_end: end,
        });
    }

    let range = BlockRange {
        start: Some(BlockId {
            height: eff_start,
            hash: vec![],
        }),
        end: Some(BlockId {
            height: end,
            hash: vec![],
        }),
    };

    let mut stream = client
        .get_block_range(range)
        .await
        .map_err(|e| map_grpc_transport("GetBlockRange", e))?
        .into_inner();

    let every = options.persist_progress_every.max(1);
    let mut count = 0u64;
    while let Some(cb) = stream
        .message()
        .await
        .map_err(|s| map_grpc_status("GetBlockRange stream", s))?
    {
        let height = cb.height;
        let mut buf = Vec::new();
        cb.encode(&mut buf)
            .map_err(|e| ZeakingError::Serialization(e.to_string()))?;
        let hash = if cb.hash.len() == 32 {
            Some(cb.hash.as_slice())
        } else {
            None
        };
        store.put_compact_block(height, hash, &buf)?;
        count += 1;
        if count.is_multiple_of(every) || height == end {
            store.set_meta("last_compact_progress", &height.to_string())?;
        }
    }

    store.set_meta("last_sync_end", &end.to_string())?;
    store.set_meta("last_compact_progress", &end.to_string())?;

    Ok(SyncCompactStats {
        blocks_written: count,
        range_start_requested: start,
        range_start_effective: eff_start,
        range_end: end,
    })
}

/// Returns the persisted “last compact height written” hint from [`LwdCompactStore`] metadata if set.
pub fn compact_sync_progress_height(store: &LwdCompactStore) -> ZeakingResult<Option<u64>> {
    let Some(s) = store.get_meta("last_compact_progress")? else {
        return Ok(None);
    };
    Ok(s.trim().parse().ok())
}

/// Chain tip height reported by lightwalletd (`GetLatestBlock`).
pub async fn chain_tip_height(client: &mut LwdClient) -> ZeakingResult<u64> {
    let id = client
        .get_latest_block(ChainSpec {})
        .await
        .map_err(|e| map_grpc_transport("GetLatestBlock", e))?
        .into_inner();
    Ok(id.height)
}

#[cfg(all(test, feature = "lightwalletd"))]
mod tests {
    use super::*;

    #[test]
    fn map_grpc_status_message_contains_op_and_status() {
        let s = Status::cancelled("unit test cancel");
        let ZeakingError::Grpc(msg) = map_grpc_status("GetBlockRange stream", s) else {
            panic!("expected Grpc");
        };
        assert!(msg.contains("GetBlockRange stream"));
        assert!(msg.contains("unit test cancel"));
    }

    #[test]
    fn compact_sync_progress_height_reads_meta() {
        let path = std::env::temp_dir().join(format!(
            "zeaking_compact_progress_test_{}.sqlite",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        let store = LwdCompactStore::open(&path).unwrap();
        assert_eq!(compact_sync_progress_height(&store).unwrap(), None);
        store.set_meta("last_compact_progress", " 100500 ").unwrap();
        assert_eq!(compact_sync_progress_height(&store).unwrap(), Some(100500));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn requested_start_height_for_tip_empty_store() {
        let path = std::env::temp_dir().join(format!(
            "zeaking_tip_start_empty_{}.sqlite",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        let store = LwdCompactStore::open(&path).unwrap();
        assert_eq!(
            requested_start_height_for_tip_sync(&store, None).unwrap(),
            1
        );
        assert_eq!(
            requested_start_height_for_tip_sync(&store, Some(500)).unwrap(),
            500
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn prune_compact_blocks_above_drops_stale_rows() {
        let path = std::env::temp_dir().join(format!(
            "zeaking_prune_stale_{}.sqlite",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        let store = LwdCompactStore::open(&path).unwrap();
        store.put_compact_block(100, None, &[1]).unwrap();
        store.put_compact_block(200, None, &[2]).unwrap();
        store
            .set_meta("last_compact_progress", "200")
            .unwrap();
        let deleted = store.prune_compact_blocks_above(150).unwrap();
        assert_eq!(deleted, 1);
        assert_eq!(store.max_compact_height().unwrap(), Some(100));
        assert_eq!(
            store.get_meta("last_compact_progress").unwrap().as_deref(),
            Some("100")
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn requested_start_height_for_tip_after_max() {
        let path = std::env::temp_dir().join(format!(
            "zeaking_tip_start_max_{}.sqlite",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        let store = LwdCompactStore::open(&path).unwrap();
        store.put_compact_block(100, None, &[1, 2, 3]).unwrap();
        assert_eq!(
            requested_start_height_for_tip_sync(&store, None).unwrap(),
            101
        );
        assert_eq!(
            requested_start_height_for_tip_sync(&store, Some(200)).unwrap(),
            200
        );
        let _ = std::fs::remove_file(&path);
    }
}
