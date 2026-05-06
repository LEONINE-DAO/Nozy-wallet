//! JSON shapes for Zebra `z_gettreestate` / `z_getsubtreesbyindex` (see `zebra-rpc` `trees.rs`).

use crate::error::{NozyError, NozyResult};
use serde::Deserialize;
use serde_json::Value;

/// Parsed Orchard pool data from `z_gettreestate`.
#[derive(Debug, Clone)]
pub struct OrchardTreestateParsed {
    pub height: u32,
    /// Anchor (Merkle root) at end of `height`, 32 bytes.
    pub anchor: [u8; 32],
    /// Number of Orchard note commitments in the tree after block `height`.
    pub commitment_count: u64,
    /// Serialized `incrementalmerkletree::CommitmentTree`, if present.
    pub final_state: Option<Vec<u8>>,
}

#[derive(Debug, Deserialize)]
pub struct ZebraSubtreeEntry {
    pub root: String,
    #[serde(rename = "endHeight", default)]
    pub end_height: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct ZebraSubtreesByIndex {
    pub pool: String,
    #[serde(rename = "startIndex", default)]
    pub start_index: Option<u64>,
    #[serde(default)]
    pub subtrees: Vec<ZebraSubtreeEntry>,
}

fn hex32_to_anchor(bytes: &[u8]) -> NozyResult<[u8; 32]> {
    if bytes.len() != 32 {
        return Err(NozyError::InvalidOperation(format!(
            "Expected 32-byte Orchard anchor/root, got {} bytes",
            bytes.len()
        )));
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(bytes);
    Ok(out)
}

fn decode_hex_field(v: &Value, label: &str) -> NozyResult<Option<Vec<u8>>> {
    let Some(s) = v.as_str() else {
        return Ok(None);
    };
    let s = s.trim();
    if s.is_empty() {
        return Ok(None);
    }
    let decoded = hex::decode(s)
        .map_err(|e| NozyError::InvalidOperation(format!("Invalid hex for {}: {}", label, e)))?;
    Ok(Some(decoded))
}

/// Parse `z_gettreestate` JSON `result` into Orchard treestate fields.
pub fn parse_z_gettreestate_orchard(result: &Value) -> NozyResult<OrchardTreestateParsed> {
    let height = result
        .get("height")
        .and_then(|h| h.as_u64())
        .ok_or_else(|| {
            NozyError::InvalidOperation("z_gettreestate: missing or invalid height".to_string())
        })? as u32;

    let orchard = result.get("orchard").ok_or_else(|| {
        NozyError::InvalidOperation("z_gettreestate: missing orchard field".to_string())
    })?;

    let commitments = orchard.get("commitments").ok_or_else(|| {
        NozyError::InvalidOperation("z_gettreestate: missing orchard.commitments".to_string())
    })?;

    let final_state = decode_hex_field(
        commitments.get("finalState").unwrap_or(&Value::Null),
        "orchard.commitments.finalState",
    )?;

    let final_root_hex = commitments.get("finalRoot");
    let anchor_from_root = final_root_hex
        .and_then(|v| decode_hex_field(v, "orchard.commitments.finalRoot").transpose())
        .transpose()?
        .filter(|b| !b.is_empty())
        .map(|b| hex32_to_anchor(&b))
        .transpose()?;

    let (anchor, commitment_count) = if let Some(ref state) = final_state {
        let tree = crate::orchard_tree_codec::orchard_commitment_tree_from_final_state(state)?;
        let root = tree.root();
        let count = tree.size() as u64;
        let anchor_bytes = root.to_bytes();
        (anchor_bytes, count)
    } else if let Some(a) = anchor_from_root {
        (a, 0)
    } else {
        return Err(NozyError::InvalidOperation(
            "z_gettreestate: Orchard pool has neither finalState nor finalRoot".to_string(),
        ));
    };

    Ok(OrchardTreestateParsed {
        height,
        anchor,
        commitment_count,
        final_state,
    })
}

/// Parse `z_getsubtreesbyindex` JSON `result`.
pub fn parse_z_get_subtrees_by_index(result: &Value) -> NozyResult<ZebraSubtreesByIndex> {
    serde_json::from_value(result.clone()).map_err(|e| {
        NozyError::InvalidOperation(format!("z_getsubtreesbyindex: invalid JSON shape: {}", e))
    })
}
