//! Decode Orchard note commitments from compact block protobuf blobs.

use prost::Message;
use serde_json::{json, Value};

use crate::error::{ZeakingError, ZeakingResult};
use crate::lwd::proto::CompactBlock;

/// One compact Orchard or Ironwood action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrchardCompactActionBytes {
    pub nullifier: [u8; 32],
    pub cmx: [u8; 32],
    pub ephemeral_key: [u8; 32],
    pub ciphertext: Vec<u8>,
}

/// Compact Orchard/Ironwood material from one transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrchardCompactTxSlice {
    pub txid_bytes: Vec<u8>,
    pub orchard_actions: Vec<OrchardCompactActionBytes>,
    pub ironwood_actions: Vec<OrchardCompactActionBytes>,
}

/// Parsed Orchard fields for one compact block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrchardCompactBlockSlice {
    pub height: u64,
    pub txs: Vec<OrchardCompactTxSlice>,
}

fn action_bytes(
    nf: &[u8],
    cmx: &[u8],
    epk: &[u8],
    ciphertext: Vec<u8>,
) -> Option<OrchardCompactActionBytes> {
    if nf.len() != 32 || cmx.len() != 32 || epk.len() != 32 {
        return None;
    }
    let mut nullifier = [0u8; 32];
    nullifier.copy_from_slice(nf);
    let mut cmx_arr = [0u8; 32];
    cmx_arr.copy_from_slice(cmx);
    let mut ephemeral_key = [0u8; 32];
    ephemeral_key.copy_from_slice(epk);
    Some(OrchardCompactActionBytes {
        nullifier,
        cmx: cmx_arr,
        ephemeral_key,
        ciphertext,
    })
}

fn action_to_scan_json(a: &OrchardCompactActionBytes) -> Value {
    json!({
        "nullifier": hex::encode(a.nullifier),
        "cmx": hex::encode(a.cmx),
        "ephemeralKey": hex::encode(a.ephemeral_key),
        "encCiphertext": hex::encode(&a.ciphertext),
    })
}

/// Shape expected by extension WASM `orchard_scan_tracker_apply_block` (slim getblock JSON).
pub fn orchard_slice_to_scan_block_json(slice: &OrchardCompactBlockSlice) -> Value {
    let mut txs = Vec::new();
    for tx in &slice.txs {
        if tx.orchard_actions.is_empty() && tx.ironwood_actions.is_empty() {
            continue;
        }
        let txid = if tx.txid_bytes.len() == 32 {
            // lightwalletd stores tx hash little-endian; wallets display big-endian hex.
            let mut rev = tx.txid_bytes.clone();
            rev.reverse();
            hex::encode(rev)
        } else {
            hex::encode(&tx.txid_bytes)
        };
        let mut obj = serde_json::Map::new();
        obj.insert("txid".into(), json!(txid));
        if !tx.orchard_actions.is_empty() {
            obj.insert(
                "orchard".into(),
                json!({
                    "actions": tx.orchard_actions.iter().map(action_to_scan_json).collect::<Vec<_>>()
                }),
            );
        }
        if !tx.ironwood_actions.is_empty() {
            obj.insert(
                "ironwood".into(),
                json!({
                    "actions": tx.ironwood_actions.iter().map(action_to_scan_json).collect::<Vec<_>>()
                }),
            );
        }
        txs.push(Value::Object(obj));
    }
    json!({
        "height": slice.height,
        "tx": txs,
    })
}

/// Decode Orchard and Ironwood compact actions from a compact block protobuf.
pub fn orchard_slice_from_compact_block(data: &[u8]) -> ZeakingResult<OrchardCompactBlockSlice> {
    let block: CompactBlock = CompactBlock::decode(data)
        .map_err(|e| ZeakingError::InvalidOperation(format!("compact block decode failed: {e}")))?;

    let mut txs = Vec::with_capacity(block.vtx.len());
    for tx in block.vtx {
        let mut orchard_actions = Vec::new();
        for action in tx.actions {
            if let Some(parsed) = action_bytes(
                &action.nullifier,
                &action.cmx,
                &action.ephemeral_key,
                action.ciphertext,
            ) {
                orchard_actions.push(parsed);
            }
        }
        let mut ironwood_actions = Vec::new();
        for action in tx.ironwood_actions {
            if let Some(parsed) = action_bytes(
                &action.nullifier,
                &action.cmx,
                &action.ephemeral_key,
                action.ciphertext,
            ) {
                ironwood_actions.push(parsed);
            }
        }
        txs.push(OrchardCompactTxSlice {
            txid_bytes: tx.hash,
            orchard_actions,
            ironwood_actions,
        });
    }

    Ok(OrchardCompactBlockSlice {
        height: block.height,
        txs,
    })
}

/// Orchard `cmx` values in consensus order (block vtx order, actions within each tx).
pub fn orchard_cmx_bytes_from_compact_block(data: &[u8]) -> ZeakingResult<Vec<[u8; 32]>> {
    let block: CompactBlock = CompactBlock::decode(data)
        .map_err(|e| ZeakingError::InvalidOperation(format!("compact block decode failed: {e}")))?;

    let mut out = Vec::new();
    for tx in block.vtx {
        for action in tx.actions {
            if action.cmx.len() == 32 {
                let mut cmx = [0u8; 32];
                cmx.copy_from_slice(&action.cmx);
                out.push(cmx);
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slice_to_scan_json_roundtrip_shape() {
        let slice = OrchardCompactBlockSlice {
            height: 10,
            txs: vec![OrchardCompactTxSlice {
                txid_bytes: vec![1u8; 32],
                orchard_actions: vec![OrchardCompactActionBytes {
                    nullifier: [2u8; 32],
                    cmx: [3u8; 32],
                    ephemeral_key: [4u8; 32],
                    ciphertext: vec![5u8; 52],
                }],
                ironwood_actions: vec![],
            }],
        };
        let v = orchard_slice_to_scan_block_json(&slice);
        assert_eq!(v["height"], 10);
        assert!(v["tx"][0]["orchard"]["actions"][0]["cmx"].is_string());
    }
}
