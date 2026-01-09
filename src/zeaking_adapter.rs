// Adapter to use Zeaking crate with Nozy's ZebraClient

use crate::block_parser::BlockParser;
use crate::zebra_integration::ZebraClient;
use async_trait::async_trait;
use zeaking::traits::{BlockParser as BlockParserTrait, BlockSource};
use zeaking::types::{BlockData, OrchardActionData, TransactionData};

pub struct ZebraBlockSource {
    client: ZebraClient,
}

impl ZebraBlockSource {
    pub fn new(client: ZebraClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl BlockSource for ZebraBlockSource {
    async fn get_block(&self, height: u32) -> zeaking::error::ZeakingResult<BlockData> {
        let block_data = self.client.get_block(height).await.map_err(|e| {
            zeaking::error::ZeakingError::Network(format!("Failed to get block: {}", e))
        })?;

        let block_hash = self.client.get_block_hash(height).await.map_err(|e| {
            zeaking::error::ZeakingError::Network(format!("Failed to get block hash: {}", e))
        })?;

        let time = block_data.get("time").and_then(|v| v.as_i64()).unwrap_or(0);
        let size = block_data
            .get("size")
            .and_then(|v| v.as_u64())
            .map(|s| s as u32)
            .unwrap_or(0);

        let block_parser = BlockParser::new(self.client.clone());
        let parsed_txs = block_parser.parse_block(height).await.map_err(|e| {
            zeaking::error::ZeakingError::InvalidOperation(format!("Failed to parse block: {}", e))
        })?;

        let transactions = parsed_txs
            .into_iter()
            .map(|tx| TransactionData {
                txid: tx.txid,
                raw_data: tx.raw_data,
                orchard_actions: tx
                    .orchard_actions
                    .into_iter()
                    .map(|action| OrchardActionData {
                        nullifier: if action.nullifier.iter().any(|&b| b != 0) {
                            Some(action.nullifier)
                        } else {
                            None
                        },
                        cmx: action.cmx,
                        cv: action.cv,
                    })
                    .collect(),
            })
            .collect();

        Ok(BlockData {
            height,
            hash: block_hash,
            time,
            size,
            transactions,
        })
    }

    async fn get_block_hash(&self, height: u32) -> zeaking::error::ZeakingResult<String> {
        self.client.get_block_hash(height).await.map_err(|e| {
            zeaking::error::ZeakingError::Network(format!("Failed to get block hash: {}", e))
        })
    }

    async fn get_block_count(&self) -> zeaking::error::ZeakingResult<u32> {
        self.client.get_block_count().await.map_err(|e| {
            zeaking::error::ZeakingError::Network(format!("Failed to get block count: {}", e))
        })
    }
}

pub struct ZebraBlockParser;

impl ZebraBlockParser {
    pub fn new(_client: ZebraClient) -> Self {
        Self
    }
}

#[async_trait]
impl BlockParserTrait for ZebraBlockParser {
    async fn parse_block(
        &self,
        block_data: BlockData,
    ) -> zeaking::error::ZeakingResult<Vec<TransactionData>> {
        Ok(block_data.transactions)
    }
}
