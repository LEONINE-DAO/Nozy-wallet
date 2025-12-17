use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedBlock {
    pub height: u32,
    pub hash: String,
    pub time: i64,
    pub size: u32,
    pub tx_count: u32,
    pub orchard_action_count: u32,
    pub indexed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedTransaction {
    pub txid: String,
    pub block_height: u32,
    pub block_hash: String,
    pub index: u32,
    pub size: u32,
    pub fee: Option<u64>,
    pub orchard_actions: Vec<OrchardActionIndex>,
    pub transparent_inputs: u32,
    pub transparent_outputs: u32,
    pub indexed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchardActionIndex {
    pub action_type: String, 
    pub nullifier: Option<String>,
    pub commitment: Option<String>,
    pub cv: Option<String>,
}

#[derive(Debug, Clone)]
pub struct IndexStats {
    pub blocks_indexed: u64,
    pub transactions_indexed: u64,
    pub errors: u64,
    pub start_height: u32,
    pub end_height: u32,
}

#[derive(Debug, Clone)]
pub struct BlockData {
    pub height: u32,
    pub hash: String,
    pub time: i64,
    pub size: u32,
    pub transactions: Vec<TransactionData>,
}

#[derive(Debug, Clone)]
pub struct TransactionData {
    pub txid: String,
    pub raw_data: Vec<u8>,
    pub orchard_actions: Vec<OrchardActionData>,
}

#[derive(Debug, Clone)]
pub struct OrchardActionData {
    pub nullifier: Option<[u8; 32]>,
    pub cmx: [u8; 32],
    pub cv: [u8; 32],
}

