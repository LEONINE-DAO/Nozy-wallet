//! Local Sapling note commitment tree + [`IncrementalWitness`] updates for legacy spends.

use std::collections::HashMap;

use incrementalmerkletree::witness::IncrementalWitness;
use sapling::note::ExtractedNoteCommitment;
use sapling::{Anchor, MerklePath, Node};

use crate::error::{NozyError, NozyResult};
use crate::sapling_tree_codec::{
    sapling_incremental_witness_to_bytes, SaplingCommitmentTree, SaplingIncrementalWitness,
};

pub use crate::sapling_tree_codec::sapling_commitment_tree_from_final_state;

/// Tracks the global Sapling commitment tree and per-nullifier incremental witnesses during sync.
#[derive(Debug, Clone)]
pub struct SaplingWitnessTracker {
    tree: SaplingCommitmentTree,
    witnesses: HashMap<[u8; 32], SaplingIncrementalWitness>,
}

impl SaplingWitnessTracker {
    pub fn new(initial_tree: SaplingCommitmentTree) -> Self {
        Self {
            tree: initial_tree,
            witnesses: HashMap::new(),
        }
    }

    pub fn tree(&self) -> &SaplingCommitmentTree {
        &self.tree
    }

    /// Append one Sapling note commitment (chain order). Updates all tracked witnesses, then the tree.
    pub fn append_cmu(&mut self, cmu: Node) -> NozyResult<()> {
        self.tree.append(cmu).map_err(|_| {
            NozyError::InvalidOperation("Sapling note commitment tree is full".to_string())
        })?;
        for w in self.witnesses.values_mut() {
            w.append(cmu).map_err(|_| {
                NozyError::InvalidOperation(
                    "Sapling incremental witness update failed (tree full)".to_string(),
                )
            })?;
        }
        Ok(())
    }

    /// After [`Self::append_cmu`], register a newly discovered spendable note at the current leaf.
    pub fn register_discovered_note(&mut self, nullifier_bytes: [u8; 32]) -> NozyResult<()> {
        let w = IncrementalWitness::<Node, 32>::from_tree(self.tree.clone()).ok_or_else(|| {
            NozyError::InvalidOperation(
                "Cannot create Sapling witness: tree empty after note discovery".to_string(),
            )
        })?;
        self.witnesses.insert(nullifier_bytes, w);
        Ok(())
    }

    pub fn witness_for_nullifier(
        &self,
        nullifier_bytes: &[u8; 32],
    ) -> Option<&SaplingIncrementalWitness> {
        self.witnesses.get(nullifier_bytes)
    }

    pub fn serialized_witness_for_nullifier(
        &self,
        nullifier_bytes: &[u8; 32],
    ) -> NozyResult<Option<Vec<u8>>> {
        let Some(w) = self.witnesses.get(nullifier_bytes) else {
            return Ok(None);
        };
        Ok(Some(sapling_incremental_witness_to_bytes(w)?))
    }

    pub fn root_at_tip(&self) -> Node {
        self.tree.root()
    }
}

pub fn merkle_node_from_cmu_bytes(bytes: &[u8; 32]) -> NozyResult<Node> {
    let cmu = ExtractedNoteCommitment::from_bytes(bytes)
        .into_option()
        .ok_or_else(|| NozyError::InvalidOperation("Invalid Sapling cmu bytes".to_string()))?;
    Ok(Node::from_cmu(&cmu))
}

/// Advance a witness through additional Sapling commitments.
pub fn advance_witness_with_cmus(
    witness: &mut SaplingIncrementalWitness,
    cmus: impl Iterator<Item = Node>,
) -> NozyResult<()> {
    for cmu in cmus {
        witness.append(cmu).map_err(|_| {
            NozyError::InvalidOperation("Failed to advance Sapling witness (tree full)".to_string())
        })?;
    }
    Ok(())
}

/// Compare witness root to Sapling `z_gettreestate` anchor bytes.
pub fn witness_root_matches_anchor(witness: &SaplingIncrementalWitness, anchor: &[u8; 32]) -> bool {
    let Some(expected) = Node::from_bytes(*anchor).into_option() else {
        return false;
    };
    witness.root() == expected
}

/// Build spend [`Anchor`] and [`MerklePath`] after the witness root matches the node anchor.
pub fn merkle_path_from_witness(
    witness: &SaplingIncrementalWitness,
) -> NozyResult<(Anchor, MerklePath)> {
    let path = witness.path().ok_or_else(|| {
        NozyError::InvalidOperation("Sapling witness has no Merkle path (empty tree)".to_string())
    })?;
    let anchor = Anchor::from(witness.root());
    Ok((anchor, path))
}
