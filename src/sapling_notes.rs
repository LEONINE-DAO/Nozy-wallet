//! Sapling note types and spendable wrappers (mirrors Orchard types in `notes.rs`).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct SaplingNote {
    pub note: sapling::Note,
    pub value: u64,
    pub payment_address: sapling::PaymentAddress,
    pub nullifier: sapling::Nullifier,
    pub block_height: u32,
    pub txid: String,
    pub spent: bool,
    pub memo: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct SpendableSaplingNote {
    pub sapling_note: SaplingNote,
    pub extsk: sapling::zip32::ExtendedSpendingKey,
    pub derivation_path: String,
    pub sapling_incremental_witness_hex: Option<String>,
    pub sapling_witness_tip_height: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableSaplingNote {
    pub nullifier_bytes: [u8; 32],
    pub value: u64,
    pub payment_address_bytes: Vec<u8>,
    pub block_height: u32,
    pub txid: String,
    pub spent: bool,
    pub memo: Vec<u8>,
    #[serde(default)]
    pub sapling_incremental_witness_hex: Option<String>,
    #[serde(default)]
    pub sapling_witness_tip_height: Option<u32>,
}

impl From<&SaplingNote> for SerializableSaplingNote {
    fn from(note: &SaplingNote) -> Self {
        Self {
            nullifier_bytes: note.nullifier.0,
            value: note.value,
            payment_address_bytes: note.payment_address.to_bytes().to_vec(),
            block_height: note.block_height,
            txid: note.txid.clone(),
            spent: note.spent,
            memo: note.memo.clone(),
            sapling_incremental_witness_hex: None,
            sapling_witness_tip_height: None,
        }
    }
}
