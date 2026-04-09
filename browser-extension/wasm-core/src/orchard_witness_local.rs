//! Orchard witness tracker + Merkle helpers (WASM copy of `nozy::orchard_witness`).

use std::collections::HashMap;
use serde_json::json;

use incrementalmerkletree::witness::IncrementalWitness;
use orchard::note::ExtractedNoteCommitment;
use orchard::tree::{Anchor, MerkleHashOrchard, MerklePath};

use crate::orchard_tree_codec::{
    orchard_commitment_tree_from_final_state, orchard_commitment_tree_to_bytes,
    orchard_incremental_witness_from_bytes, orchard_incremental_witness_to_bytes,
    OrchardCommitmentTree, OrchardIncrementalWitness,
};

pub struct OrchardWitnessTracker {
    tree: OrchardCommitmentTree,
    witnesses: HashMap<[u8; 32], OrchardIncrementalWitness>,
}

impl OrchardWitnessTracker {
    pub fn new(initial_tree: OrchardCommitmentTree) -> Self {
        Self {
            tree: initial_tree,
            witnesses: HashMap::new(),
        }
    }

    pub fn from_final_state_hex(final_state_hex: Option<&str>) -> Result<Self, String> {
        let initial = if let Some(h) = final_state_hex {
            let s = h.trim().trim_start_matches("0x");
            if s.is_empty() {
                OrchardCommitmentTree::empty()
            } else {
                let bytes = hex::decode(s).map_err(|e| format!("finalState hex: {}", e))?;
                orchard_commitment_tree_from_final_state(&bytes)?
            }
        } else {
            OrchardCommitmentTree::empty()
        };
        Ok(Self::new(initial))
    }

    pub fn append_cmx(&mut self, cmx: MerkleHashOrchard) -> Result<(), String> {
        self.tree.append(cmx).map_err(|_| "Orchard tree full".to_string())?;
        for w in self.witnesses.values_mut() {
            w.append(cmx).map_err(|_| "witness append failed".to_string())?;
        }
        Ok(())
    }

    pub fn register_discovered_note(&mut self, nullifier_bytes: [u8; 32]) -> Result<(), String> {
        let w = IncrementalWitness::<MerkleHashOrchard, 32>::from_tree(self.tree.clone())
            .ok_or_else(|| "cannot create witness from tree".to_string())?;
        self.witnesses.insert(nullifier_bytes, w);
        Ok(())
    }

    pub fn serialized_witness_for_nullifier(&self, nf: &[u8; 32]) -> Result<Option<Vec<u8>>, String> {
        let Some(w) = self.witnesses.get(nf) else {
            return Ok(None);
        };
        Ok(Some(orchard_incremental_witness_to_bytes(w)?))
    }

    pub fn serialize_json(&self) -> Result<String, String> {
        let tree_hex = hex::encode(orchard_commitment_tree_to_bytes(&self.tree)?);
        let mut map = serde_json::Map::new();
        for (nf, w) in &self.witnesses {
            let wh = hex::encode(orchard_incremental_witness_to_bytes(w)?);
            map.insert(hex::encode(nf), serde_json::Value::String(wh));
        }
        let v = json!({ "tree_hex": tree_hex, "witnesses": map });
        serde_json::to_string(&v).map_err(|e| e.to_string())
    }

    pub fn deserialize_json(s: &str) -> Result<Self, String> {
        let v: serde_json::Value =
            serde_json::from_str(s).map_err(|e| format!("tracker json: {}", e))?;
        let tree_hex = v
            .get("tree_hex")
            .and_then(|x| x.as_str())
            .ok_or_else(|| "missing tree_hex".to_string())?;
        let tree_bytes = hex::decode(tree_hex.trim_start_matches("0x"))
            .map_err(|e| format!("tree hex: {}", e))?;
        let tree = crate::orchard_tree_codec::orchard_commitment_tree_from_bytes(&tree_bytes)?;
        let mut witnesses = HashMap::new();
        if let Some(wm) = v.get("witnesses").and_then(|x| x.as_object()) {
            for (nf_hex, wval) in wm {
                let nf_b = hex::decode(nf_hex.trim_start_matches("0x"))
                    .map_err(|e| format!("nf hex: {}", e))?;
                if nf_b.len() != 32 {
                    return Err("nf len".to_string());
                }
                let mut nf_a = [0u8; 32];
                nf_a.copy_from_slice(&nf_b);
                let wh = wval
                    .as_str()
                    .ok_or_else(|| "witness str".to_string())?;
                let wbytes = hex::decode(wh.trim_start_matches("0x"))
                    .map_err(|e| format!("witness hex: {}", e))?;
                let w = orchard_incremental_witness_from_bytes(&wbytes)?;
                witnesses.insert(nf_a, w);
            }
        }
        Ok(Self { tree, witnesses })
    }
}

pub fn merkle_hash_from_cmx_bytes(bytes: &[u8; 32]) -> Result<MerkleHashOrchard, String> {
    let cmx = ExtractedNoteCommitment::from_bytes(bytes)
        .into_option()
        .ok_or_else(|| "invalid cmx".to_string())?;
    Ok(MerkleHashOrchard::from_cmx(&cmx))
}

pub fn advance_witness_with_cmxs(
    witness: &mut OrchardIncrementalWitness,
    cmxs: impl Iterator<Item = MerkleHashOrchard>,
) -> Result<(), String> {
    for cmx in cmxs {
        witness
            .append(cmx)
            .map_err(|_| "witness advance failed".to_string())?;
    }
    Ok(())
}

pub fn witness_root_matches_anchor(witness: &OrchardIncrementalWitness, anchor: &[u8; 32]) -> bool {
    let Some(expected) = MerkleHashOrchard::from_bytes(anchor).into_option() else {
        return false;
    };
    witness.root() == expected
}

pub fn merkle_path_from_witness(
    witness: &OrchardIncrementalWitness,
) -> Result<(Anchor, MerklePath), String> {
    let inc_path = witness
        .path()
        .ok_or_else(|| "witness has no path".to_string())?;
    let merkle_path = MerklePath::from(inc_path);
    let anchor = Anchor::from(witness.root());
    Ok((anchor, merkle_path))
}
