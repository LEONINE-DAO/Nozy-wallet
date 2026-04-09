//! Parse Zebra `getblock` JSON (verbosity 2) for Orchard actions in consensus order.

use serde_json::Value;

/// All Orchard note commitments in this block, in block tx order and per-tx action order.
pub fn orchard_cmx_bytes_from_block_json(block: &Value) -> Result<Vec<[u8; 32]>, String> {
    let mut out = Vec::new();
    let tx_array = block
        .get("tx")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "block.tx missing".to_string())?;

    for tx in tx_array {
        if tx.as_str().is_some() {
            continue;
        }
        let has_orchard = tx
            .get("orchard")
            .and_then(|o| o.as_object())
            .and_then(|o| o.get("actions"))
            .and_then(|a| a.as_array())
            .map(|a| !a.is_empty())
            .unwrap_or(false);
        if !has_orchard {
            continue;
        }
        let orchard = tx.get("orchard").ok_or_else(|| "orchard missing".to_string())?;
        let actions = orchard
            .get("actions")
            .and_then(|a| a.as_array())
            .ok_or_else(|| "orchard.actions missing".to_string())?;
        for action in actions {
            let cmx_hex = action
                .get("cmx")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "action.cmx missing".to_string())?;
            let cmx = hex::decode(cmx_hex.trim_start_matches("0x"))
                .map_err(|e| format!("cmx hex: {}", e))?;
            if cmx.len() != 32 {
                return Err(format!("cmx len {}", cmx.len()));
            }
            let mut b = [0u8; 32];
            b.copy_from_slice(&cmx);
            out.push(b);
        }
    }
    Ok(out)
}
