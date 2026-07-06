//! Ironwood note commitment tree witness tracking.
//!
//! Ironwood actions use Orchard note commitment primitives (`MerkleHashOrchard`,
//! `Anchor`, `MerklePath`) in a separate value pool. This wrapper keeps pool
//! ownership explicit while reusing the proven Orchard witness machinery.

use orchard::tree::{Anchor, MerkleHashOrchard, MerklePath};

use crate::error::NozyResult;
use crate::ironwood_tree_codec::{IronwoodCommitmentTree, IronwoodIncrementalWitness};
use crate::orchard_witness::{
    advance_witness_with_cmxs, merkle_hash_from_cmx_bytes, merkle_path_from_witness,
    witness_root_matches_anchor, OrchardWitnessTracker,
};

#[derive(Debug, Clone)]
pub struct IronwoodWitnessTracker {
    inner: OrchardWitnessTracker,
}

impl IronwoodWitnessTracker {
    pub fn new(initial_tree: IronwoodCommitmentTree) -> Self {
        Self {
            inner: OrchardWitnessTracker::new(initial_tree),
        }
    }

    pub fn append_cmx(&mut self, cmx: MerkleHashOrchard) -> NozyResult<()> {
        self.inner.append_cmx(cmx)
    }

    pub fn register_discovered_note(&mut self, nullifier_bytes: [u8; 32]) -> NozyResult<()> {
        self.inner.register_discovered_note(nullifier_bytes)
    }

    pub fn serialized_witness_for_nullifier(
        &self,
        nullifier_bytes: &[u8; 32],
    ) -> NozyResult<Option<Vec<u8>>> {
        self.inner.serialized_witness_for_nullifier(nullifier_bytes)
    }

    pub fn root_at_tip(&self) -> MerkleHashOrchard {
        self.inner.root_at_tip()
    }
}

pub fn ironwood_merkle_hash_from_cmx_bytes(bytes: &[u8; 32]) -> NozyResult<MerkleHashOrchard> {
    merkle_hash_from_cmx_bytes(bytes)
}

pub fn advance_ironwood_witness_with_cmxs(
    witness: &mut IronwoodIncrementalWitness,
    cmxs: impl Iterator<Item = MerkleHashOrchard>,
) -> NozyResult<()> {
    advance_witness_with_cmxs(witness, cmxs)
}

pub fn ironwood_witness_root_matches_anchor(
    witness: &IronwoodIncrementalWitness,
    anchor: &[u8; 32],
) -> bool {
    witness_root_matches_anchor(witness, anchor)
}

pub fn ironwood_merkle_path_from_witness(
    witness: &IronwoodIncrementalWitness,
) -> NozyResult<(Anchor, MerklePath)> {
    merkle_path_from_witness(witness)
}
