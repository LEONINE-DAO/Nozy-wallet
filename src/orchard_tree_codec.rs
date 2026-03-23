//! Deserialize Orchard `CommitmentTree` / `IncrementalWitness` from Zebra/zcashd RPC encodings.

use crate::error::{NozyError, NozyResult};
use core2::io::Cursor;
use incrementalmerkletree::frontier::CommitmentTree;
use incrementalmerkletree::witness::IncrementalWitness;
use orchard::tree::MerkleHashOrchard;
use zcash_primitives::merkle_tree::{
    read_commitment_tree, read_incremental_witness, write_incremental_witness,
};

pub type OrchardCommitmentTree = CommitmentTree<MerkleHashOrchard, 32>;
pub type OrchardIncrementalWitness = IncrementalWitness<MerkleHashOrchard, 32>;

pub fn orchard_commitment_tree_from_final_state(bytes: &[u8]) -> NozyResult<OrchardCommitmentTree> {
    let mut cursor = Cursor::new(bytes);
    read_commitment_tree(&mut cursor).map_err(|e| {
        NozyError::InvalidOperation(format!(
            "Failed to parse Orchard finalState CommitmentTree: {}",
            e
        ))
    })
}

pub fn orchard_incremental_witness_from_bytes(
    bytes: &[u8],
) -> NozyResult<OrchardIncrementalWitness> {
    let mut cursor = Cursor::new(bytes);
    read_incremental_witness(&mut cursor).map_err(|e| {
        NozyError::InvalidOperation(format!("Failed to parse Orchard IncrementalWitness: {}", e))
    })
}

pub fn orchard_incremental_witness_to_bytes(
    witness: &OrchardIncrementalWitness,
) -> NozyResult<Vec<u8>> {
    let mut buf = Vec::new();
    write_incremental_witness(witness, &mut buf).map_err(|e| {
        NozyError::InvalidOperation(format!(
            "Failed to serialize Orchard IncrementalWitness: {}",
            e
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
    fn orchard_commitment_tree_roundtrip_empty() {
        let t = OrchardCommitmentTree::empty();
        let mut v = Vec::new();
        write_commitment_tree(&t, &mut v).expect("write");
        let back = orchard_commitment_tree_from_final_state(&v).expect("read");
        assert_eq!(t.root().to_bytes(), back.root().to_bytes());
    }

    #[test]
    fn orchard_incremental_witness_roundtrip_empty_tree_fails_or_skips() {
        let t = OrchardCommitmentTree::empty();
        assert!(IncrementalWitness::<orchard::tree::MerkleHashOrchard, 32>::from_tree(t).is_none());
    }

    #[test]
    fn orchard_incremental_witness_roundtrip_single_leaf() {
        use orchard::tree::MerkleHashOrchard;
        let mut t = OrchardCommitmentTree::empty();
        let leaf = MerkleHashOrchard::empty_leaf();
        t.append(leaf).unwrap();
        let w = IncrementalWitness::<MerkleHashOrchard, 32>::from_tree(t.clone()).expect("witness");
        let bytes = orchard_incremental_witness_to_bytes(&w).expect("ser");
        let w2 = orchard_incremental_witness_from_bytes(&bytes).expect("de");
        assert_eq!(w.root().to_bytes(), w2.root().to_bytes());
    }
}
