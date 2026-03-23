use std::sync::Arc;

use tokio::sync::Mutex;

use super::client::{connect_lightwalletd, LwdClient};
use super::proto::{BlockId, CompactBlock, CompactTx};
use super::sync::chain_tip_height;
use crate::error::{ZeakingError, ZeakingResult};
use crate::traits::BlockSource;
use crate::types::{BlockData, OrchardActionData, TransactionData};

fn arr32(slice: &[u8]) -> Option<[u8; 32]> {
    if slice.len() == 32 {
        let mut a = [0u8; 32];
        a.copy_from_slice(slice);
        Some(a)
    } else {
        None
    }
}

fn compact_tx_to_transaction_data(tx: CompactTx) -> TransactionData {
    let txid = hex::encode(&tx.hash);
    let orchard_actions: Vec<OrchardActionData> = tx
        .actions
        .into_iter()
        .filter_map(|a| {
            let cmx = arr32(&a.cmx)?;
            let nullifier = arr32(&a.nullifier);
            let cv = arr32(&a.ephemeral_key).unwrap_or([0u8; 32]);
            Some(OrchardActionData { nullifier, cmx, cv })
        })
        .collect();
    TransactionData {
        txid,
        raw_data: vec![],
        orchard_actions,
    }
}

fn compact_block_to_block_data(cb: CompactBlock) -> ZeakingResult<BlockData> {
    let height_u32 = u32::try_from(cb.height).map_err(|_| {
        ZeakingError::InvalidOperation(format!("block height {} does not fit u32", cb.height))
    })?;
    let hash = hex::encode(&cb.hash);
    let time = i64::from(cb.time);
    let tx_count = cb.vtx.len() as u32;
    let transactions: Vec<TransactionData> = cb
        .vtx
        .into_iter()
        .map(compact_tx_to_transaction_data)
        .collect();
    let size = tx_count.saturating_mul(2048).max(1);
    Ok(BlockData {
        height: height_u32,
        hash,
        time,
        size,
        transactions,
    })
}

/// [`BlockSource`] backed by lightwalletd compact blocks (Zebrad + lightwalletd).
pub struct LightwalletdBlockSource {
    client: Arc<Mutex<LwdClient>>,
}

impl LightwalletdBlockSource {
    pub async fn connect(grpc_uri: &str) -> ZeakingResult<Self> {
        let c = connect_lightwalletd(grpc_uri).await?;
        Ok(Self {
            client: Arc::new(Mutex::new(c)),
        })
    }
}

#[async_trait::async_trait]
impl BlockSource for LightwalletdBlockSource {
    async fn get_block(&self, height: u32) -> ZeakingResult<BlockData> {
        let mut c = self.client.lock().await;
        let cb = c
            .get_block(BlockId {
                height: u64::from(height),
                hash: vec![],
            })
            .await
            .map_err(|e| ZeakingError::Network(format!("GetBlock: {e}")))?
            .into_inner();
        compact_block_to_block_data(cb)
    }

    async fn get_block_hash(&self, height: u32) -> ZeakingResult<String> {
        Ok(self.get_block(height).await?.hash)
    }

    async fn get_block_count(&self) -> ZeakingResult<u32> {
        let mut c = self.client.lock().await;
        let tip = chain_tip_height(&mut c).await?;
        u32::try_from(tip).map_err(|_| {
            ZeakingError::InvalidOperation(format!("chain tip {tip} does not fit u32"))
        })
    }
}
