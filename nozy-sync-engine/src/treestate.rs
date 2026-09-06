//! Map Zebra `z_gettreestate` / `z_getsubtreesbyindex` JSON → lightwalletd protos.

use serde_json::Value;

use crate::error::{IndexerError, IndexerResult};
use crate::proto::{ShieldedProtocol, SubtreeRoot, TreeState};
use crate::rpc::RpcClient;

pub(crate) fn pool_final_state_hex(result: &Value, pool: &str) -> String {
    let null = Value::Null;
    let pool_value = result.get(pool).unwrap_or(&null);
    let commitments = pool_value.get("commitments").unwrap_or(pool_value);
    for key in ["finalState", "final_state"] {
        if let Some(s) = commitments.get(key).and_then(|v| v.as_str()) {
            if !s.trim().is_empty() {
                return s.trim().to_string();
            }
        }
        if let Some(s) = pool_value.get(key).and_then(|v| v.as_str()) {
            if !s.trim().is_empty() {
                return s.trim().to_string();
            }
        }
    }
    String::new()
}

/// Build lightwalletd `TreeState` from a `z_gettreestate` result object.
pub fn tree_state_from_zebra_json(network: &str, result: &Value) -> IndexerResult<TreeState> {
    let height = result
        .get("height")
        .and_then(|h| h.as_u64())
        .ok_or_else(|| IndexerError::Rpc("z_gettreestate: missing height".into()))?;
    let hash = result
        .get("hash")
        .and_then(|h| h.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let time = result.get("time").and_then(|t| t.as_u64()).unwrap_or(0) as u32;

    Ok(TreeState {
        network: network.to_string(),
        height,
        hash,
        time,
        sapling_tree: pool_final_state_hex(result, "sapling"),
        orchard_tree: pool_final_state_hex(result, "orchard"),
    })
}

pub async fn fetch_tree_state(
    rpc: &RpcClient,
    network: &str,
    height: u64,
) -> IndexerResult<TreeState> {
    let result = rpc.z_gettreestate(height).await?;
    tree_state_from_zebra_json(network, &result)
}

fn protocol_pool_name(proto: i32) -> IndexerResult<&'static str> {
    match ShieldedProtocol::try_from(proto) {
        Ok(ShieldedProtocol::Sapling) => Ok("sapling"),
        Ok(ShieldedProtocol::Orchard) => Ok("orchard"),
        Err(_) => Err(IndexerError::Rpc(format!(
            "unknown ShieldedProtocol value {proto}"
        ))),
    }
}

/// Fetch subtree roots and resolve completing block hashes when heights are present.
pub async fn fetch_subtree_roots(
    rpc: &RpcClient,
    protocol: i32,
    start_index: u32,
    max_entries: u32,
) -> IndexerResult<Vec<SubtreeRoot>> {
    let pool = protocol_pool_name(protocol)?;
    let result = rpc
        .z_getsubtreesbyindex(pool, start_index, max_entries)
        .await?;
    let subtrees = result
        .get("subtrees")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut out = Vec::with_capacity(subtrees.len());
    for entry in subtrees {
        let root_hex = entry
            .get("root")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();
        let root_hash = if root_hex.is_empty() {
            Vec::new()
        } else {
            hex::decode(root_hex).unwrap_or_default()
        };
        let end_height = entry
            .get("endHeight")
            .or_else(|| entry.get("end_height"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let mut completing_block_hash = Vec::new();
        if end_height > 0 {
            if let Ok(h) = rpc.get_block_hash(end_height).await {
                completing_block_hash = hex::decode(h.trim()).unwrap_or_default();
            }
        }
        out.push(SubtreeRoot {
            root_hash,
            completing_block_hash,
            completing_block_height: end_height,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn maps_final_state_fields() {
        let v = json!({
            "height": 100,
            "hash": "abcd",
            "time": 42,
            "sapling": { "commitments": { "finalState": "aa" } },
            "orchard": { "finalState": "bb" }
        });
        let ts = tree_state_from_zebra_json("main", &v).unwrap();
        assert_eq!(ts.height, 100);
        assert_eq!(ts.sapling_tree, "aa");
        assert_eq!(ts.orchard_tree, "bb");
        assert_eq!(ts.time, 42);
    }
}
