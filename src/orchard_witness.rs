//! Local Orchard note commitment tree + [`IncrementalWitness`] updates for shielded spends.

use std::collections::HashMap;

use incrementalmerkletree::witness::IncrementalWitness;
use orchard::note::ExtractedNoteCommitment;
use orchard::tree::{Anchor, MerkleHashOrchard, MerklePath};

use crate::error::{NozyError, NozyResult};
use crate::orchard_tree_codec::{
    orchard_incremental_witness_to_bytes, OrchardCommitmentTree, OrchardIncrementalWitness,
};

pub use crate::orchard_tree_codec::orchard_commitment_tree_from_final_state;

/// Tracks the global Orchard commitment tree and per-nullifier incremental witnesses during sync.
#[derive(Debug, Clone)]
pub struct OrchardWitnessTracker {
    tree: OrchardCommitmentTree,
    /// Witnesses for our notes, updated as new Orchard commitments are appended.
    witnesses: HashMap<[u8; 32], OrchardIncrementalWitness>,
}

impl OrchardWitnessTracker {
    pub fn new(initial_tree: OrchardCommitmentTree) -> Self {
        Self {
            tree: initial_tree,
            witnesses: HashMap::new(),
        }
    }

    pub fn tree(&self) -> &OrchardCommitmentTree {
        &self.tree
    }

    /// Append one Orchard note commitment (chain order). Updates all tracked witnesses, then the tree.
    pub fn append_cmx(&mut self, cmx: MerkleHashOrchard) -> NozyResult<()> {
        self.tree.append(cmx).map_err(|_| {
            NozyError::InvalidOperation("Orchard note commitment tree is full".to_string())
        })?;
        for w in self.witnesses.values_mut() {
            w.append(cmx).map_err(|_| {
                NozyError::InvalidOperation(
                    "Orchard incremental witness update failed (tree full)".to_string(),
                )
            })?;
        }
        Ok(())
    }

    /// After [`Self::append_cmx`], register a newly discovered spendable note at the current leaf.
    pub fn register_discovered_note(&mut self, nullifier_bytes: [u8; 32]) -> NozyResult<()> {
        let w = IncrementalWitness::<MerkleHashOrchard, 32>::from_tree(self.tree.clone()).ok_or_else(|| {
            NozyError::InvalidOperation(
                "Cannot create Orchard witness: tree empty after note discovery".to_string(),
            )
        })?;
        self.witnesses.insert(nullifier_bytes, w);
        Ok(())
    }

    pub fn witness_for_nullifier(&self, nullifier_bytes: &[u8; 32]) -> Option<&OrchardIncrementalWitness> {
        self.witnesses.get(nullifier_bytes)
    }

    pub fn serialized_witness_for_nullifier(
        &self,
        nullifier_bytes: &[u8; 32],
    ) -> NozyResult<Option<Vec<u8>>> {
        let Some(w) = self.witnesses.get(nullifier_bytes) else {
            return Ok(None);
        };
        Ok(Some(orchard_incremental_witness_to_bytes(w)?))
    }

    pub fn root_at_tip(&self) -> MerkleHashOrchard {
        self.tree.root()
    }
}

pub fn merkle_hash_from_cmx_bytes(bytes: &[u8; 32]) -> NozyResult<MerkleHashOrchard> {
    let cmx = ExtractedNoteCommitment::from_bytes(bytes)
        .into_option()
        .ok_or_else(|| NozyError::InvalidOperation("Invalid Orchard cmx bytes".to_string()))?;
    Ok(MerkleHashOrchard::from_cmx(&cmx))
}

/// Advance a witness through additional Orchard commitments (e.g. new blocks since last scan).
pub fn advance_witness_with_cmxs(
    witness: &mut OrchardIncrementalWitness,
    cmxs: impl Iterator<Item = MerkleHashOrchard>,
) -> NozyResult<()> {
    for cmx in cmxs {
        witness.append(cmx).map_err(|_| {
            NozyError::InvalidOperation("Failed to advance Orchard witness (tree full)".to_string())
        })?;
    }
    Ok(())
}

/// Compare witness root to [`crate::zebra_integration::OrchardTreeState::anchor`] from `z_gettreestate`.
pub fn witness_root_matches_anchor(witness: &OrchardIncrementalWitness, anchor: &[u8; 32]) -> bool {
    let Some(expected) = MerkleHashOrchard::from_bytes(anchor).into_option() else {
        return false;
    };
    witness.root() == expected
}

/// Build spend [`Anchor`] and [`MerklePath`] after the witness root matches the node anchor.
pub fn merkle_path_from_witness(witness: &OrchardIncrementalWitness) -> NozyResult<(Anchor, MerklePath)> {
    let inc_path = witness.path().ok_or_else(|| {
        NozyError::InvalidOperation("Orchard witness has no Merkle path (empty tree)".to_string())
    })?;
    let merkle_path = MerklePath::from(inc_path);
    let anchor = Anchor::from(witness.root());
    Ok((anchor, merkle_path))
}
