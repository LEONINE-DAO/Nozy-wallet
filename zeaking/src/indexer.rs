use crate::error::{ZeakingError, ZeakingResult};
use crate::traits::{BlockParser as BlockParserTrait, BlockSource};
use crate::types::{
    BlockData, IndexStats, IndexedBlock, IndexedTransaction, OrchardActionIndex, TransactionData,
};
use chrono::{DateTime, Utc};
use hex;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Main Zeaking indexer
#[derive(Debug)]
pub struct Zeaking<BS, BP = DefaultBlockParser>
where
    BS: BlockSource,
    BP: BlockParserTrait,
{
    index_path: PathBuf,
    block_index: Arc<Mutex<BlockIndex>>,
    transaction_index: Arc<Mutex<TransactionIndex>>,
    metadata: Arc<Mutex<IndexMetadata>>,
    block_source: Arc<BS>,
    block_parser: Arc<BP>,
}

#[derive(Debug, Clone)]
pub struct DefaultBlockParser;

#[async_trait::async_trait]
impl BlockParserTrait for DefaultBlockParser {
    async fn parse_block(&self, block_data: BlockData) -> ZeakingResult<Vec<TransactionData>> {
        Ok(block_data.transactions)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct BlockIndex {
    blocks: BTreeMap<u32, IndexedBlock>,
    hash_to_height: HashMap<String, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct TransactionIndex {
    transactions: HashMap<String, IndexedTransaction>,
    block_to_txs: BTreeMap<u32, Vec<String>>,
    nullifier_to_tx: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IndexMetadata {
    last_indexed_height: u32,
    last_indexed_time: DateTime<Utc>,
    total_blocks_indexed: u64,
    total_transactions_indexed: u64,
    index_version: u32,
    last_saved_height: u32,
    last_saved_time: DateTime<Utc>,
}

impl<BS, BP> Zeaking<BS, BP>
where
    BS: BlockSource,
    BP: BlockParserTrait,
{
    pub async fn new(
        index_path: PathBuf,
        block_source: BS,
        block_parser: BP,
    ) -> ZeakingResult<Self> {
        let block_index = Arc::new(Mutex::new(BlockIndex::default()));
        let transaction_index = Arc::new(Mutex::new(TransactionIndex::default()));
        let metadata = Arc::new(Mutex::new(IndexMetadata {
            last_indexed_height: 0,
            last_indexed_time: Utc::now(),
            total_blocks_indexed: 0,
            total_transactions_indexed: 0,
            index_version: 1,
            last_saved_height: 0,
            last_saved_time: Utc::now(),
        }));

        let zeaking = Self {
            index_path: index_path.clone(),
            block_index,
            transaction_index,
            metadata,
            block_source: Arc::new(block_source),
            block_parser: Arc::new(block_parser),
        };

        zeaking.load_index()?;

        Ok(zeaking)
    }

    fn get_index_dir(&self) -> PathBuf {
        self.index_path.clone()
    }

    fn load_index(&self) -> ZeakingResult<()> {
        let index_dir = self.get_index_dir();

        if !index_dir.exists() {
            fs::create_dir_all(&index_dir).map_err(|e| {
                ZeakingError::Storage(format!("Failed to create index directory: {}", e))
            })?;
            return Ok(());
        }

        let blocks_path = index_dir.join("blocks.json");
        if blocks_path.exists() {
            let content = fs::read_to_string(&blocks_path).map_err(|e| {
                ZeakingError::Storage(format!("Failed to read blocks index: {}", e))
            })?;
            let block_idx: BlockIndex = serde_json::from_str(&content).map_err(|e| {
                ZeakingError::Storage(format!("Failed to parse blocks index: {}", e))
            })?;
            *self.block_index.lock().unwrap() = block_idx;
        }

        let txs_path = index_dir.join("transactions.json");
        if txs_path.exists() {
            let content = fs::read_to_string(&txs_path).map_err(|e| {
                ZeakingError::Storage(format!("Failed to read transactions index: {}", e))
            })?;
            let tx_idx: TransactionIndex = serde_json::from_str(&content).map_err(|e| {
                ZeakingError::Storage(format!("Failed to parse transactions index: {}", e))
            })?;
            *self.transaction_index.lock().unwrap() = tx_idx;
        }

        let metadata_path = index_dir.join("metadata.json");
        if metadata_path.exists() {
            let content = fs::read_to_string(&metadata_path)
                .map_err(|e| ZeakingError::Storage(format!("Failed to read metadata: {}", e)))?;
            let mut meta: IndexMetadata = serde_json::from_str(&content)
                .map_err(|e| ZeakingError::Storage(format!("Failed to parse metadata: {}", e)))?;

            if meta.last_saved_height == 0 && meta.last_indexed_height > 0 {
                meta.last_saved_height = meta.last_indexed_height;
                meta.last_saved_time = meta.last_indexed_time;
            }
            *self.metadata.lock().unwrap() = meta;
        }

        Ok(())
    }

    pub fn save_index(&self) -> ZeakingResult<()> {
        self.save_index_incremental(false)
    }

    fn save_index_incremental(&self, incremental: bool) -> ZeakingResult<()> {
        let index_dir = self.get_index_dir();
        fs::create_dir_all(&index_dir).map_err(|e| {
            ZeakingError::Storage(format!("Failed to create index directory: {}", e))
        })?;

        let meta = self.metadata.lock().unwrap();
        let last_saved = if incremental {
            meta.last_saved_height
        } else {
            0
        };
        let current_height = meta.last_indexed_height;
        drop(meta);

        if incremental && last_saved >= current_height {
            return Ok(());
        }

        let atomic_write = |path: &PathBuf, content: String| -> ZeakingResult<()> {
            let temp_path = path.with_extension("tmp");
            fs::write(&temp_path, content)
                .map_err(|e| ZeakingError::Storage(format!("Failed to write temp file: {}", e)))?;
            fs::rename(&temp_path, path)
                .map_err(|e| ZeakingError::Storage(format!("Failed to rename temp file: {}", e)))?;
            Ok(())
        };

        if incremental {
            let blocks_path = index_dir.join("blocks.json");
            let mut existing_blocks = BlockIndex::default();

            if blocks_path.exists() {
                if let Ok(content) = fs::read_to_string(&blocks_path) {
                    if let Ok(loaded) = serde_json::from_str::<BlockIndex>(&content) {
                        existing_blocks = loaded;
                    }
                }
            }

            let block_idx = self.block_index.lock().unwrap();
            for (height, block) in block_idx.blocks.range((last_saved + 1)..=current_height) {
                existing_blocks.blocks.insert(*height, block.clone());
                if let Some(hash) = block_idx
                    .hash_to_height
                    .iter()
                    .find(|(_, &h)| h == *height)
                    .map(|(hash, _)| hash.clone())
                {
                    existing_blocks.hash_to_height.insert(hash, *height);
                }
            }
            drop(block_idx);

            let content = serde_json::to_string_pretty(&existing_blocks)
                .map_err(|e| ZeakingError::Storage(format!("Failed to serialize blocks: {}", e)))?;
            atomic_write(&blocks_path, content)?;

            let txs_path = index_dir.join("transactions.json");
            let mut existing_txs = TransactionIndex::default();

            if txs_path.exists() {
                if let Ok(content) = fs::read_to_string(&txs_path) {
                    if let Ok(loaded) = serde_json::from_str::<TransactionIndex>(&content) {
                        existing_txs = loaded;
                    }
                }
            }

            let tx_idx = self.transaction_index.lock().unwrap();
            for height in (last_saved + 1)..=current_height {
                if let Some(txids) = tx_idx.block_to_txs.get(&height) {
                    for txid in txids {
                        if let Some(tx) = tx_idx.transactions.get(txid) {
                            existing_txs.transactions.insert(txid.clone(), tx.clone());
                        }
                    }
                    existing_txs.block_to_txs.insert(height, txids.clone());
                }
            }

            for (nullifier, txid) in &tx_idx.nullifier_to_tx {
                if let Some(tx) = tx_idx.transactions.get(txid) {
                    if tx.block_height > last_saved {
                        existing_txs
                            .nullifier_to_tx
                            .insert(nullifier.clone(), txid.clone());
                    }
                }
            }
            drop(tx_idx);

            let content = serde_json::to_string_pretty(&existing_txs).map_err(|e| {
                ZeakingError::Storage(format!("Failed to serialize transactions: {}", e))
            })?;
            atomic_write(&txs_path, content)?;
        } else {
            let blocks_path = index_dir.join("blocks.json");
            let block_idx = self.block_index.lock().unwrap();
            let content = serde_json::to_string_pretty(&*block_idx)
                .map_err(|e| ZeakingError::Storage(format!("Failed to serialize blocks: {}", e)))?;
            drop(block_idx);
            atomic_write(&blocks_path, content)?;

            let txs_path = index_dir.join("transactions.json");
            let tx_idx = self.transaction_index.lock().unwrap();
            let content = serde_json::to_string_pretty(&*tx_idx).map_err(|e| {
                ZeakingError::Storage(format!("Failed to serialize transactions: {}", e))
            })?;
            drop(tx_idx);
            atomic_write(&txs_path, content)?;
        }

        let metadata_path = index_dir.join("metadata.json");
        let mut meta = self.metadata.lock().unwrap();
        meta.last_saved_height = current_height;
        meta.last_saved_time = Utc::now();
        let content = serde_json::to_string_pretty(&*meta)
            .map_err(|e| ZeakingError::Storage(format!("Failed to serialize metadata: {}", e)))?;
        drop(meta);
        atomic_write(&metadata_path, content)?;

        Ok(())
    }

    pub fn save_index_incremental_only(&self) -> ZeakingResult<()> {
        self.save_index_incremental(true)
    }

    pub async fn index_block(&self, height: u32) -> ZeakingResult<()> {
        {
            let block_idx = self.block_index.lock().unwrap();
            if block_idx.blocks.contains_key(&height) {
                return Ok(());
            }
        }

        let block_data = self.block_source.get_block(height).await?;
        let block_hash = self.block_source.get_block_hash(height).await?;

        let transactions = self.block_parser.parse_block(block_data.clone()).await?;

        let mut orchard_action_count = 0;
        for tx in &transactions {
            orchard_action_count += tx.orchard_actions.len() as u32;
        }

        let indexed_block = IndexedBlock {
            height,
            hash: block_hash.clone(),
            time: block_data.time,
            size: block_data.size,
            tx_count: transactions.len() as u32,
            orchard_action_count,
            indexed_at: Utc::now(),
        };

        {
            let mut block_idx = self.block_index.lock().unwrap();
            block_idx.blocks.insert(height, indexed_block.clone());
            block_idx.hash_to_height.insert(block_hash.clone(), height);
        }

        for (tx_index, tx) in transactions.iter().enumerate() {
            self.index_transaction(tx, height, tx_index as u32, &block_hash)
                .await?;
        }

        {
            let mut meta = self.metadata.lock().unwrap();
            meta.last_indexed_height = meta.last_indexed_height.max(height);
            meta.last_indexed_time = Utc::now();
            meta.total_blocks_indexed += 1;
        }

        Ok(())
    }

    async fn index_transaction(
        &self,
        tx: &TransactionData,
        block_height: u32,
        tx_index: u32,
        block_hash: &str,
    ) -> ZeakingResult<()> {
        {
            let tx_idx = self.transaction_index.lock().unwrap();
            if tx_idx.transactions.contains_key(&tx.txid) {
                return Ok(());
            }
        }

        let mut orchard_actions = Vec::new();
        for action in &tx.orchard_actions {
            let has_nullifier = action.nullifier.is_some();
            orchard_actions.push(OrchardActionIndex {
                action_type: if has_nullifier { "spend" } else { "output" }.to_string(),
                nullifier: action.nullifier.map(|n| hex::encode(&n)),
                commitment: Some(hex::encode(&action.cmx)),
                cv: Some(hex::encode(&action.cv)),
            });
        }

        let indexed_tx = IndexedTransaction {
            txid: tx.txid.clone(),
            block_height,
            block_hash: block_hash.to_string(),
            index: tx_index,
            size: tx.raw_data.len() as u32,
            fee: None,
            orchard_actions,
            transparent_inputs: 0,
            transparent_outputs: 0,
            indexed_at: Utc::now(),
        };

        {
            let mut tx_idx = self.transaction_index.lock().unwrap();
            tx_idx
                .transactions
                .insert(tx.txid.clone(), indexed_tx.clone());

            tx_idx
                .block_to_txs
                .entry(block_height)
                .or_insert_with(Vec::new)
                .push(tx.txid.clone());

            for action in &indexed_tx.orchard_actions {
                if let Some(ref nullifier) = action.nullifier {
                    tx_idx
                        .nullifier_to_tx
                        .insert(nullifier.clone(), tx.txid.clone());
                }
            }
        }

        {
            let mut meta = self.metadata.lock().unwrap();
            meta.total_transactions_indexed += 1;
        }

        Ok(())
    }

    pub async fn index_range(
        &self,
        start_height: u32,
        end_height: u32,
    ) -> ZeakingResult<IndexStats> {
        self.index_range_with_auto_save(start_height, end_height, Some(1000))
            .await
    }

    pub async fn index_range_with_auto_save(
        &self,
        start_height: u32,
        end_height: u32,
        auto_save_interval: Option<u32>,
    ) -> ZeakingResult<IndexStats> {
        let mut indexed = 0;
        let mut errors = 0;
        let mut last_save_height = start_height.saturating_sub(1);

        for height in start_height..=end_height {
            match self.index_block(height).await {
                Ok(_) => {
                    indexed += 1;

                    // Auto-save incrementally if interval is set
                    if let Some(interval) = auto_save_interval {
                        if height - last_save_height >= interval {
                            if let Err(e) = self.save_index_incremental_only() {
                                eprintln!("Warning: Incremental save failed, will retry with full save: {}", e);
                            } else {
                                last_save_height = height;
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to index block {}: {}", height, e);
                    errors += 1;
                }
            }
        }

        if last_save_height < end_height {
            if let Err(_) = self.save_index_incremental_only() {
                self.save_index()?;
            }
        }

        let tx_count = {
            let tx_idx = self.transaction_index.lock().unwrap();
            tx_idx.transactions.len() as u64
        };

        Ok(IndexStats {
            blocks_indexed: indexed,
            transactions_indexed: tx_count,
            errors,
            start_height,
            end_height,
        })
    }

    pub fn get_block(&self, height: u32) -> Option<IndexedBlock> {
        let block_idx = self.block_index.lock().unwrap();
        block_idx.blocks.get(&height).cloned()
    }

    pub fn get_transaction(&self, txid: &str) -> Option<IndexedTransaction> {
        let tx_idx = self.transaction_index.lock().unwrap();
        tx_idx.transactions.get(txid).cloned()
    }

    pub fn get_block_transactions(&self, height: u32) -> Vec<IndexedTransaction> {
        let tx_idx = self.transaction_index.lock().unwrap();
        tx_idx
            .block_to_txs
            .get(&height)
            .map(|txids| {
                txids
                    .iter()
                    .filter_map(|txid| tx_idx.transactions.get(txid).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn find_transaction_by_nullifier(&self, nullifier: &str) -> Option<IndexedTransaction> {
        let tx_idx = self.transaction_index.lock().unwrap();
        tx_idx
            .nullifier_to_tx
            .get(nullifier)
            .and_then(|txid| tx_idx.transactions.get(txid).cloned())
    }

    pub fn get_stats(&self) -> IndexStats {
        let block_idx = self.block_index.lock().unwrap();
        let tx_idx = self.transaction_index.lock().unwrap();
        let meta = self.metadata.lock().unwrap();

        IndexStats {
            blocks_indexed: block_idx.blocks.len() as u64,
            transactions_indexed: tx_idx.transactions.len() as u64,
            errors: 0,
            start_height: block_idx.blocks.keys().next().copied().unwrap_or(0),
            end_height: meta.last_indexed_height,
        }
    }

    pub fn get_last_indexed_height(&self) -> u32 {
        let meta = self.metadata.lock().unwrap();
        meta.last_indexed_height
    }

    pub async fn sync_to_tip(&self) -> ZeakingResult<IndexStats> {
        let last_height = self.get_last_indexed_height();
        let tip_height = self.block_source.get_block_count().await?;

        if last_height >= tip_height {
            return Ok(IndexStats {
                blocks_indexed: 0,
                transactions_indexed: 0,
                errors: 0,
                start_height: last_height,
                end_height: tip_height,
            });
        }

        self.index_range(last_height + 1, tip_height).await
    }
}
