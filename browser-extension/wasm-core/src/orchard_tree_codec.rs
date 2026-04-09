//! Orchard `CommitmentTree` / `IncrementalWitness` (zcash-compatible), WASM-only copy of nozy `orchard_tree_codec`.

use core2::io::Cursor;
use incrementalmerkletree::frontier::CommitmentTree;
use incrementalmerkletree::witness::IncrementalWitness;
use orchard::tree::MerkleHashOrchard;
use zcash_primitives::merkle_tree::{
    read_commitment_tree, read_incremental_witness, write_commitment_tree, write_incremental_witness,
};

pub type OrchardCommitmentTree = CommitmentTree<MerkleHashOrchard, 32>;
pub type OrchardIncrementalWitness = IncrementalWitness<MerkleHashOrchard, 32>;

pub fn orchard_commitment_tree_from_final_state(bytes: &[u8]) -> Result<OrchardCommitmentTree, String> {
    let mut cursor = Cursor::new(bytes);
    read_commitment_tree(&mut cursor).map_err(|e| format!("finalState CommitmentTree: {}", e))
}

pub fn orchard_commitment_tree_from_bytes(bytes: &[u8]) -> Result<OrchardCommitmentTree, String> {
    let mut cursor = Cursor::new(bytes);
    read_commitment_tree(&mut cursor).map_err(|e| format!("CommitmentTree: {}", e))
}

pub fn orchard_incremental_witness_from_bytes(bytes: &[u8]) -> Result<OrchardIncrementalWitness, String> {
    let mut cursor = Cursor::new(bytes);
    read_incremental_witness(&mut cursor).map_err(|e| format!("IncrementalWitness: {}", e))
}

pub fn orchard_incremental_witness_to_bytes(
    witness: &OrchardIncrementalWitness,
) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    write_incremental_witness(witness, &mut buf)
        .map_err(|e| format!("serialize witness: {}", e))?;
    Ok(buf)
}

pub fn orchard_commitment_tree_to_bytes(tree: &OrchardCommitmentTree) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    write_commitment_tree(tree, &mut buf).map_err(|e| format!("write tree: {}", e))?;
    Ok(buf)
}
