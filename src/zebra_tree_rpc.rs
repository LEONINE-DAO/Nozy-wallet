//! JSON shapes for Zebra `z_gettreestate` / `z_getsubtreesbyindex` (see `zebra-rpc` `trees.rs`).

use crate::error::{NozyError, NozyResult};
use crate::shielded_pool::ShieldedPool;
use serde::Deserialize;
use serde_json::Value;

/// Parsed shielded pool data from `z_gettreestate`.
#[derive(Debug, Clone)]
pub struct ShieldedTreestateParsed {
    pub pool: ShieldedPool,
    pub height: u32,
    /// Anchor (Merkle root) at end of `height`, 32 bytes.
    pub anchor: [u8; 32],
    /// Number of note commitments in the tree after block `height`.
    pub commitment_count: u64,
    /// Serialized `incrementalmerkletree::CommitmentTree`, if present.
    pub final_state: Option<Vec<u8>>,
}

pub type OrchardTreestateParsed = ShieldedTreestateParsed;

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
            "Expected 32-byte shielded pool anchor/root, got {} bytes",
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

fn first_hex_field<'a, I, S>(values: I) -> NozyResult<Option<Vec<u8>>>
where
    I: IntoIterator<Item = (&'a Value, S)>,
    S: AsRef<str>,
{
    for (value, label) in values {
        if let Some(decoded) = decode_hex_field(value, label.as_ref())? {
            return Ok(Some(decoded));
        }
    }
    Ok(None)
}

/// Parse `z_gettreestate` JSON `result` into shielded pool treestate fields.
pub fn parse_z_gettreestate_pool(
    result: &Value,
    pool: ShieldedPool,
) -> NozyResult<ShieldedTreestateParsed> {
    let height = result
        .get("height")
        .and_then(|h| h.as_u64())
        .ok_or_else(|| {
            NozyError::InvalidOperation("z_gettreestate: missing or invalid height".to_string())
        })? as u32;

    let pool_id = pool.value_pool_id();
    let pool_value = result.get(pool_id).ok_or_else(|| {
        NozyError::InvalidOperation(format!("z_gettreestate: missing {pool_id} field"))
    })?;

    let commitments = pool_value.get("commitments").unwrap_or(pool_value);

    let null = Value::Null;
    let final_state = first_hex_field([
        (
            commitments.get("finalState").unwrap_or(&null),
            format!("{pool_id}.commitments.finalState"),
        ),
        (
            commitments.get("final_state").unwrap_or(&null),
            format!("{pool_id}.commitments.final_state"),
        ),
        (
            pool_value.get("finalState").unwrap_or(&null),
            format!("{pool_id}.finalState"),
        ),
        (
            pool_value.get("final_state").unwrap_or(&null),
            format!("{pool_id}.final_state"),
        ),
    ])?;

    let anchor_from_root = first_hex_field([
        (
            commitments.get("finalRoot").unwrap_or(&null),
            format!("{pool_id}.commitments.finalRoot"),
        ),
        (
            commitments.get("final_root").unwrap_or(&null),
            format!("{pool_id}.commitments.final_root"),
        ),
        (
            commitments.get("root").unwrap_or(&null),
            format!("{pool_id}.commitments.root"),
        ),
        (
            pool_value.get("finalRoot").unwrap_or(&null),
            format!("{pool_id}.finalRoot"),
        ),
        (
            pool_value.get("final_root").unwrap_or(&null),
            format!("{pool_id}.final_root"),
        ),
        (
            pool_value.get("root").unwrap_or(&null),
            format!("{pool_id}.root"),
        ),
    ])?
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
        return Err(NozyError::InvalidOperation(format!(
            "z_gettreestate: {pool_id} pool has neither finalState nor finalRoot"
        )));
    };

    Ok(ShieldedTreestateParsed {
        pool,
        height,
        anchor,
        commitment_count,
        final_state,
    })
}

/// Parse `z_gettreestate` JSON `result` into Orchard treestate fields.
pub fn parse_z_gettreestate_orchard(result: &Value) -> NozyResult<OrchardTreestateParsed> {
    parse_z_gettreestate_pool(result, ShieldedPool::Orchard)
}

/// Parse `z_getsubtreesbyindex` JSON `result`.
pub fn parse_z_get_subtrees_by_index(result: &Value) -> NozyResult<ZebraSubtreesByIndex> {
    serde_json::from_value(result.clone()).map_err(|e| {
        NozyError::InvalidOperation(format!("z_getsubtreesbyindex: invalid JSON shape: {}", e))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ironwood_treestate_from_final_root() {
        let final_root = "11".repeat(32);
        let result = serde_json::json!({
            "height": 3_000_000,
            "ironwood": {
                "commitments": {
                    "finalRoot": final_root,
                    "finalState": ""
                }
            }
        });

        let parsed = parse_z_gettreestate_pool(&result, ShieldedPool::Ironwood).unwrap();

        assert_eq!(parsed.pool, ShieldedPool::Ironwood);
        assert_eq!(parsed.height, 3_000_000);
        assert_eq!(parsed.anchor, [0x11; 32]);
        assert_eq!(parsed.commitment_count, 0);
        assert!(parsed.final_state.is_none());
    }

    #[test]
    fn parses_pool_level_snake_case_root() {
        let final_root = "33".repeat(32);
        let result = serde_json::json!({
            "height": 4_134_000,
            "orchard": {
                "final_root": final_root,
                "commitments": {}
            }
        });

        let parsed = parse_z_gettreestate_pool(&result, ShieldedPool::Orchard).unwrap();

        assert_eq!(parsed.pool, ShieldedPool::Orchard);
        assert_eq!(parsed.height, 4_134_000);
        assert_eq!(parsed.anchor, [0x33; 32]);
        assert_eq!(parsed.commitment_count, 0);
        assert!(parsed.final_state.is_none());
    }

    #[test]
    fn parses_commitments_root_alias() {
        let root = "44".repeat(32);
        let result = serde_json::json!({
            "height": 4_134_000,
            "orchard": {
                "commitments": {
                    "root": root
                }
            }
        });

        let parsed = parse_z_gettreestate_orchard(&result).unwrap();

        assert_eq!(parsed.anchor, [0x44; 32]);
        assert!(parsed.final_state.is_none());
    }

    #[test]
    fn errors_when_requested_pool_is_missing() {
        let result = serde_json::json!({
            "height": 3_000_000,
            "orchard": {
                "commitments": {
                    "finalRoot": "22".repeat(32)
                }
            }
        });

        let err = parse_z_gettreestate_pool(&result, ShieldedPool::Ironwood).unwrap_err();
        assert!(err
            .to_string()
            .contains("z_gettreestate: missing ironwood field"));
    }
}
