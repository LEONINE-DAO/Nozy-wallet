//! Deserialize Ironwood `CommitmentTree` / `IncrementalWitness` encodings.
//!
//! Ironwood uses the Orchard protocol's note commitment tree primitives, but it
//! is a distinct value pool with a distinct tree. Keep separate type aliases so
//! scan and migration code cannot accidentally conflate pool state.

use crate::error::NozyResult;
use crate::orchard_tree_codec::{
    orchard_commitment_tree_from_final_state, orchard_incremental_witness_from_bytes,
    orchard_incremental_witness_to_bytes, OrchardCommitmentTree, OrchardIncrementalWitness,
};

pub type IronwoodCommitmentTree = OrchardCommitmentTree;
pub type IronwoodIncrementalWitness = OrchardIncrementalWitness;

pub fn ironwood_commitment_tree_from_final_state(
    bytes: &[u8],
) -> NozyResult<IronwoodCommitmentTree> {
    orchard_commitment_tree_from_final_state(bytes)
}

pub fn ironwood_incremental_witness_from_bytes(
    bytes: &[u8],
) -> NozyResult<IronwoodIncrementalWitness> {
    orchard_incremental_witness_from_bytes(bytes)
}

pub fn ironwood_incremental_witness_to_bytes(
    witness: &IronwoodIncrementalWitness,
) -> NozyResult<Vec<u8>> {
    orchard_incremental_witness_to_bytes(witness)
}
