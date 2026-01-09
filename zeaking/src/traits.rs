use crate::error::ZeakingResult;
use crate::types::{BlockData, TransactionData};

#[async_trait::async_trait]
pub trait BlockSource: Send + Sync {
    async fn get_block(&self, height: u32) -> ZeakingResult<BlockData>;

    async fn get_block_hash(&self, height: u32) -> ZeakingResult<String>;

    async fn get_block_count(&self) -> ZeakingResult<u32>;
}

#[async_trait::async_trait]
pub trait BlockParser: Send + Sync {
    async fn parse_block(&self, block_data: BlockData) -> ZeakingResult<Vec<TransactionData>> {
        Ok(block_data.transactions)
    }
}
