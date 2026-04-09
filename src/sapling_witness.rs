//! Local Sapling note commitment tree + incremental witnesses for Zebrad-safe spends.

use std::collections::HashMap;

use crate::error::{NozyError, NozyResult};
use crate::sapling_tree_codec::{
    sapling_incremental_witness_to_bytes, SaplingCommitmentTree, SaplingIncrementalWitness,
};
use sapling::note::ExtractedNoteCommitment;
use sapling::{Anchor, MerklePath, Node, Nullifier};

pub use crate::sapling_tree_codec::sapling_commitment_tree_from_final_state;

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

    pub fn append_cmu_node(&mut self, node: Node) -> NozyResult<()> {
        self.tree.append(node).map_err(|_| {
            NozyError::InvalidOperation("Sapling note commitment tree is full".to_string())
        })?;
        for w in self.witnesses.values_mut() {
            w.append(node).map_err(|_| {
                NozyError::InvalidOperation(
                    "Sapling incremental witness update failed (tree full)".to_string(),
                )
            })?;
        }
        Ok(())
    }

    pub fn register_discovered_note(&mut self, nullifier_bytes: [u8; 32]) -> NozyResult<()> {
        let w = SaplingIncrementalWitness::from_tree(self.tree.clone()).ok_or_else(|| {
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

pub fn node_from_cmu_bytes(bytes: &[u8; 32]) -> NozyResult<Node> {
    let cmu = ExtractedNoteCommitment::from_bytes(bytes)
        .into_option()
        .ok_or_else(|| NozyError::InvalidOperation("Invalid Sapling cmu bytes".to_string()))?;
    Ok(Node::from_cmu(&cmu))
}

pub fn advance_sapling_witness_with_nodes(
    witness: &mut SaplingIncrementalWitness,
    nodes: impl Iterator<Item = Node>,
) -> NozyResult<()> {
    for n in nodes {
        witness.append(n).map_err(|_| {
            NozyError::InvalidOperation("Failed to advance Sapling witness (tree full)".to_string())
        })?;
    }
    Ok(())
}

/// Compare witness root to Sapling anchor bytes from `z_gettreestate` / `z_getsaplingtree`.
pub fn sapling_witness_root_matches_anchor(
    witness: &SaplingIncrementalWitness,
    anchor: &[u8; 32],
) -> bool {
    let Some(expected) = Node::from_bytes(*anchor).into_option() else {
        return false;
    };
    witness.root() == expected
}

pub fn sapling_merkle_path_from_witness(
    witness: &SaplingIncrementalWitness,
) -> NozyResult<(Anchor, MerklePath)> {
    let inc_path = witness.path().ok_or_else(|| {
        NozyError::InvalidOperation("Sapling witness has no Merkle path (empty tree)".to_string())
    })?;
    let merkle_path = MerklePath::from(inc_path);
    let anchor = Anchor::from(witness.root());
    Ok((anchor, merkle_path))
}

pub fn sapling_nullifier_bytes(nf: &Nullifier) -> [u8; 32] {
    nf.0
}
