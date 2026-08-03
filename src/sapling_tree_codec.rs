//! Deserialize Sapling `CommitmentTree` / `IncrementalWitness` from Zebra RPC encodings.

use crate::error::{NozyError, NozyResult};
use incrementalmerkletree::frontier::CommitmentTree;
use incrementalmerkletree::witness::IncrementalWitness;
use sapling::Node;
use std::io::Cursor;
use zcash_primitives::merkle_tree::{
    read_commitment_tree, read_incremental_witness, write_incremental_witness,
};

pub type SaplingCommitmentTree = CommitmentTree<Node, 32>;
pub type SaplingIncrementalWitness = IncrementalWitness<Node, 32>;

pub fn sapling_commitment_tree_from_final_state(bytes: &[u8]) -> NozyResult<SaplingCommitmentTree> {
    let mut cursor = Cursor::new(bytes);
    read_commitment_tree(&mut cursor).map_err(|e| {
        NozyError::InvalidOperation(format!(
            "Failed to parse Sapling finalState CommitmentTree: {e}"
        ))
    })
}

pub fn sapling_incremental_witness_from_bytes(
    bytes: &[u8],
) -> NozyResult<SaplingIncrementalWitness> {
    let mut cursor = Cursor::new(bytes);
    read_incremental_witness(&mut cursor).map_err(|e| {
        NozyError::InvalidOperation(format!("Failed to parse Sapling IncrementalWitness: {e}"))
    })
}

pub fn sapling_incremental_witness_to_bytes(
    witness: &SaplingIncrementalWitness,
) -> NozyResult<Vec<u8>> {
    let mut buf = Vec::new();
    write_incremental_witness(witness, &mut buf).map_err(|e| {
        NozyError::InvalidOperation(format!(
            "Failed to serialize Sapling IncrementalWitness: {e}"
        ))
    })?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use incrementalmerkletree::Hashable;
    use zcash_primitives::merkle_tree::write_commitment_tree;

    #[test]
    fn sapling_commitment_tree_roundtrip_empty() {
        let t = SaplingCommitmentTree::empty();
        let mut v = Vec::new();
        write_commitment_tree(&t, &mut v).expect("write");
        let back = sapling_commitment_tree_from_final_state(&v).expect("read");
        assert_eq!(t.root().to_bytes(), back.root().to_bytes());
    }

    #[test]
    fn sapling_incremental_witness_roundtrip_single_leaf() {
        let mut t = SaplingCommitmentTree::empty();
        let leaf = Node::empty_leaf();
        t.append(leaf).unwrap();
        let w = IncrementalWitness::<Node, 32>::from_tree(t.clone()).expect("witness");
        let bytes = sapling_incremental_witness_to_bytes(&w).expect("ser");
        let w2 = sapling_incremental_witness_from_bytes(&bytes).expect("de");
        assert_eq!(w.root().to_bytes(), w2.root().to_bytes());
    }
}
