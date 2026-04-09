//! Sapling `CommitmentTree` / `IncrementalWitness` serialization (Zebra `z_gettreestate` `finalState`).

use crate::error::{NozyError, NozyResult};
use core2::io::Cursor;
use incrementalmerkletree::frontier::CommitmentTree;
use incrementalmerkletree::witness::IncrementalWitness;
use sapling::Node;
use zcash_primitives::merkle_tree::{
    read_commitment_tree, read_incremental_witness, write_incremental_witness,
};

pub type SaplingCommitmentTree = CommitmentTree<Node, 32>;
pub type SaplingIncrementalWitness = IncrementalWitness<Node, 32>;

pub fn sapling_commitment_tree_from_final_state(bytes: &[u8]) -> NozyResult<SaplingCommitmentTree> {
    let mut cursor = Cursor::new(bytes);
    read_commitment_tree(&mut cursor).map_err(|e| {
        NozyError::InvalidOperation(format!(
            "Failed to parse Sapling finalState CommitmentTree: {}",
            e
        ))
    })
}

pub fn sapling_incremental_witness_from_bytes(bytes: &[u8]) -> NozyResult<SaplingIncrementalWitness> {
    let mut cursor = Cursor::new(bytes);
    read_incremental_witness(&mut cursor).map_err(|e| {
        NozyError::InvalidOperation(format!("Failed to parse Sapling IncrementalWitness: {}", e))
    })
}

pub fn sapling_incremental_witness_to_bytes(
    witness: &SaplingIncrementalWitness,
) -> NozyResult<Vec<u8>> {
    let mut buf = Vec::new();
    write_incremental_witness(witness, &mut buf).map_err(|e| {
        NozyError::InvalidOperation(format!(
            "Failed to serialize Sapling IncrementalWitness: {}",
            e
        ))
    })?;
    Ok(buf)
}
