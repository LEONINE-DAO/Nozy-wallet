use prost::Message;

use super::client::LwdClient;
use super::proto::{BlockId, BlockRange, ChainSpec};
use super::store::LwdCompactStore;
use crate::error::{ZeakingError, ZeakingResult};

/// Download inclusive `[start, end]` compact blocks from lightwalletd into `store`.
/// Returns the number of blocks written.
pub async fn sync_compact_range(
    client: &mut LwdClient,
    store: &LwdCompactStore,
    start: u64,
    end: u64,
) -> ZeakingResult<u64> {
    if end < start {
        return Err(ZeakingError::InvalidOperation(format!(
            "sync_compact_range: end {end} < start {start}"
        )));
    }

    let range = BlockRange {
        start: Some(BlockId {
            height: start,
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
        .map_err(|e| ZeakingError::Network(format!("GetBlockRange: {e}")))?
        .into_inner();

    let mut count = 0u64;
    while let Some(cb) = stream
        .message()
        .await
        .map_err(|e| ZeakingError::Network(format!("stream: {e}")))?
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
    }

    store.set_meta("last_sync_end", &end.to_string())?;
    Ok(count)
}

/// Chain tip height reported by lightwalletd (`GetLatestBlock`).
pub async fn chain_tip_height(client: &mut LwdClient) -> ZeakingResult<u64> {
    let id = client
        .get_latest_block(ChainSpec {})
        .await
        .map_err(|e| ZeakingError::Network(format!("GetLatestBlock: {e}")))?
        .into_inner();
    Ok(id.height)
}
