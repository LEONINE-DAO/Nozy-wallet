//! Sapling / Orchard / Ironwood commitment tree sizes for `ChainMetadata`.
//!
//! Compact blocks carry the tree size **after** the block (lightwalletd semantics).
//! Ingest counts new notes in the block and adds them to the previous height’s
//! sizes. When the previous compact is missing or still all-zero (pre-S11 cache),
//! sizes are seeded from Zebra `z_gettreestate` (JSON `size` or `finalState`).

use std::io::Cursor;

use incrementalmerkletree::frontier::CommitmentTree;
use serde_json::Value;
use zcash_primitives::merkle_tree::read_commitment_tree;

use crate::error::IndexerResult;
use crate::proto::{ChainMetadata, CompactBlock};
use crate::rpc::RpcClient;
use crate::store::IndexerStore;
use crate::treestate::pool_final_state_hex;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TreeSizes {
    pub sapling: u32,
    pub orchard: u32,
    pub ironwood: u32,
}

impl TreeSizes {
    pub fn is_zero(self) -> bool {
        self.sapling == 0 && self.orchard == 0 && self.ironwood == 0
    }

    pub fn saturating_add(self, delta: TreeSizes) -> TreeSizes {
        TreeSizes {
            sapling: self.sapling.saturating_add(delta.sapling),
            orchard: self.orchard.saturating_add(delta.orchard),
            ironwood: self.ironwood.saturating_add(delta.ironwood),
        }
    }

    pub fn into_chain_metadata(self) -> ChainMetadata {
        ChainMetadata {
            sapling_commitment_tree_size: self.sapling,
            orchard_commitment_tree_size: self.orchard,
            ironwood_commitment_tree_size: self.ironwood,
        }
    }
}

impl From<&ChainMetadata> for TreeSizes {
    fn from(m: &ChainMetadata) -> Self {
        TreeSizes {
            sapling: m.sapling_commitment_tree_size,
            orchard: m.orchard_commitment_tree_size,
            ironwood: m.ironwood_commitment_tree_size,
        }
    }
}

/// New commitments in this compact block (Sapling outputs, Orchard/Ironwood actions).
pub fn commitment_counts(block: &CompactBlock) -> TreeSizes {
    let mut sapling = 0u32;
    let mut orchard = 0u32;
    let mut ironwood = 0u32;
    for tx in &block.vtx {
        sapling = sapling.saturating_add(tx.outputs.len() as u32);
        orchard = orchard.saturating_add(tx.actions.len() as u32);
        ironwood = ironwood.saturating_add(tx.ironwood_actions.len() as u32);
    }
    TreeSizes {
        sapling,
        orchard,
        ironwood,
    }
}

pub fn sizes_from_compact(block: &CompactBlock) -> TreeSizes {
    block
        .chain_metadata
        .as_ref()
        .map(TreeSizes::from)
        .unwrap_or_default()
}

fn json_u32(v: &Value) -> Option<u32> {
    v.as_u64()
        .and_then(|n| u32::try_from(n).ok())
        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
}

fn pool_json_size(result: &Value, pool: &str) -> u32 {
    let null = Value::Null;
    let pool_v = result.get(pool).unwrap_or(&null);
    let commitments = pool_v.get("commitments").unwrap_or(pool_v);
    for key in ["size", "n", "count", "noteCount", "note_count"] {
        if let Some(n) = pool_v.get(key).and_then(json_u32) {
            return n;
        }
        if let Some(n) = commitments.get(key).and_then(json_u32) {
            return n;
        }
    }
    0
}

fn sapling_size_from_final_state(bytes: &[u8]) -> Option<u32> {
    let mut cursor = Cursor::new(bytes);
    let tree: CommitmentTree<sapling::Node, 32> = read_commitment_tree(&mut cursor).ok()?;
    u32::try_from(tree.size()).ok()
}

fn orchard_size_from_final_state(bytes: &[u8]) -> Option<u32> {
    let mut cursor = Cursor::new(bytes);
    let tree: CommitmentTree<orchard::tree::MerkleHashOrchard, 32> =
        read_commitment_tree(&mut cursor).ok()?;
    u32::try_from(tree.size()).ok()
}

/// Tree sizes **after** the block at `height` (z_gettreestate / getblock trees).
pub fn tree_sizes_from_zebra_json(result: &Value) -> TreeSizes {
    let mut sizes = TreeSizes {
        sapling: pool_json_size(result, "sapling"),
        orchard: pool_json_size(result, "orchard"),
        ironwood: pool_json_size(result, "ironwood"),
    };
    if sizes.sapling == 0 {
        let hex = pool_final_state_hex(result, "sapling");
        if let Ok(bytes) = hex::decode(hex.trim()) {
            if let Some(n) = sapling_size_from_final_state(&bytes) {
                sizes.sapling = n;
            }
        }
    }
    if sizes.orchard == 0 {
        let hex = pool_final_state_hex(result, "orchard");
        if let Ok(bytes) = hex::decode(hex.trim()) {
            if let Some(n) = orchard_size_from_final_state(&bytes) {
                sizes.orchard = n;
            }
        }
    }
    if sizes.ironwood == 0 {
        let hex = pool_final_state_hex(result, "ironwood");
        if let Ok(bytes) = hex::decode(hex.trim()) {
            if let Some(n) = orchard_size_from_final_state(&bytes) {
                sizes.ironwood = n;
            }
        }
    }
    sizes
}

pub fn tree_sizes_from_getblock_json(block: &Value) -> TreeSizes {
    let trees = block.get("trees").unwrap_or(&Value::Null);
    TreeSizes {
        sapling: pool_json_size(trees, "sapling"),
        orchard: pool_json_size(trees, "orchard"),
        ironwood: pool_json_size(trees, "ironwood"),
    }
}

async fn seed_sizes_at_height(rpc: &RpcClient, height: u64) -> IndexerResult<TreeSizes> {
    if let Ok(ts) = rpc.z_gettreestate(height).await {
        let sizes = tree_sizes_from_zebra_json(&ts);
        if !sizes.is_zero() {
            return Ok(sizes);
        }
    }
    if let Ok(hash) = rpc.get_block_hash(height).await {
        if let Ok(block) = rpc.get_block_verbose(&hash).await {
            let sizes = tree_sizes_from_getblock_json(&block);
            if !sizes.is_zero() {
                return Ok(sizes);
            }
        }
    }
    Ok(TreeSizes::default())
}

/// Sizes **after** `height` (this compact block).
pub async fn tree_sizes_after_block(
    rpc: &RpcClient,
    store: &IndexerStore,
    height: u64,
    compact: &CompactBlock,
) -> IndexerResult<TreeSizes> {
    let counts = commitment_counts(compact);
    let prev = if let Some(prev_h) = height.checked_sub(1) {
        let from_store = match store.get_compact_block(prev_h)? {
            Some(bytes) => crate::compact::decode_compact_block(&bytes)
                .ok()
                .map(|b| sizes_from_compact(&b))
                .filter(|s| !s.is_zero()),
            None => None,
        };
        match from_store {
            Some(s) => s,
            None => seed_sizes_at_height(rpc, prev_h).await?,
        }
    } else {
        TreeSizes::default()
    };
    Ok(prev.saturating_add(counts))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::{ChainMetadata, CompactOrchardAction, CompactSaplingOutput, CompactTx};

    fn empty_output() -> CompactSaplingOutput {
        CompactSaplingOutput {
            cmu: vec![0; 32],
            ephemeral_key: vec![0; 32],
            ciphertext: vec![0; 52],
        }
    }

    fn empty_action() -> CompactOrchardAction {
        CompactOrchardAction {
            nullifier: vec![0; 32],
            cmx: vec![0; 32],
            ephemeral_key: vec![0; 32],
            ciphertext: vec![0; 52],
        }
    }

    #[test]
    fn counts_outputs_and_actions() {
        let block = CompactBlock {
            proto_version: 4,
            height: 10,
            hash: vec![1; 32],
            prev_hash: vec![0; 32],
            time: 1,
            header: vec![],
            vtx: vec![CompactTx {
                index: 1,
                hash: vec![2; 32],
                fee: 0,
                spends: vec![],
                outputs: vec![empty_output(), empty_output()],
                actions: vec![empty_action()],
                ironwood_actions: vec![empty_action(), empty_action(), empty_action()],
            }],
            chain_metadata: None,
        };
        assert_eq!(
            commitment_counts(&block),
            TreeSizes {
                sapling: 2,
                orchard: 1,
                ironwood: 3,
            }
        );
    }

    #[test]
    fn adds_to_previous_sizes() {
        let prev = TreeSizes {
            sapling: 100,
            orchard: 50,
            ironwood: 7,
        };
        let delta = TreeSizes {
            sapling: 2,
            orchard: 1,
            ironwood: 3,
        };
        assert_eq!(
            prev.saturating_add(delta),
            TreeSizes {
                sapling: 102,
                orchard: 51,
                ironwood: 10,
            }
        );
    }

    #[test]
    fn json_size_fields() {
        let v = serde_json::json!({
            "height": 99,
            "sapling": { "size": 12 },
            "orchard": { "commitments": { "size": 34 } },
            "ironwood": { "n": "5" }
        });
        assert_eq!(
            tree_sizes_from_zebra_json(&v),
            TreeSizes {
                sapling: 12,
                orchard: 34,
                ironwood: 5,
            }
        );
    }

    #[test]
    fn getblock_trees_object() {
        let v = serde_json::json!({
            "trees": {
                "sapling": { "size": 8 },
                "orchard": { "size": 9 },
                "ironwood": { "size": 1 }
            }
        });
        assert_eq!(
            tree_sizes_from_getblock_json(&v),
            TreeSizes {
                sapling: 8,
                orchard: 9,
                ironwood: 1,
            }
        );
    }

    #[test]
    fn metadata_roundtrip() {
        let s = TreeSizes {
            sapling: 3,
            orchard: 4,
            ironwood: 5,
        };
        let meta: ChainMetadata = s.into_chain_metadata();
        assert_eq!(TreeSizes::from(&meta), s);
    }

    #[test]
    fn empty_final_state_decodes_to_zero() {
        let mut v = Vec::new();
        let t = CommitmentTree::<sapling::Node, 32>::empty();
        zcash_primitives::merkle_tree::write_commitment_tree(&t, &mut v).unwrap();
        assert_eq!(sapling_size_from_final_state(&v), Some(0));
    }
}
