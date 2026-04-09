//! Block-by-block Orchard scan with incremental witnesses (Zebrad-safe, no zcashd witness RPCs).

use serde_json::Value;
use wasm_bindgen::prelude::*;

use nozy::hd_wallet::{HDWallet, OrchardActionCompactData, OrchardDecryptionResult};

use crate::orchard_witness_local::{merkle_hash_from_cmx_bytes, OrchardWitnessTracker};

fn hex32(s: &str) -> Result<[u8; 32], String> {
    let v = hex::decode(s.trim_start_matches("0x")).map_err(|e| format!("hex: {}", e))?;
    if v.len() != 32 {
        return Err(format!("expected 32 bytes, got {}", v.len()));
    }
    let mut a = [0u8; 32];
    a.copy_from_slice(&v);
    Ok(a)
}

fn action_json_to_compact(action: &Value) -> Option<OrchardActionCompactData> {
    let nullifier = action.get("nullifier")?.as_str()?;
    let cmx = action.get("cmx")?.as_str()?;
    let ephemeral_key = action.get("ephemeralKey").or_else(|| action.get("ephemeral_key"))?;
    let ephemeral_key = ephemeral_key.as_str()?;
    let enc_hex = action
        .get("encCiphertext")
        .or_else(|| action.get("enc_ciphertext"))?
        .as_str()?;
    let enc = hex::decode(enc_hex.trim_start_matches("0x")).ok()?;
    if enc.len() < 52 {
        return None;
    }
    Some(OrchardActionCompactData {
        nullifier: hex32(nullifier).ok()?,
        cmx: hex32(cmx).ok()?,
        ephemeral_key: hex32(ephemeral_key).ok()?,
        encrypted_note: enc,
    })
}

/// Apply one block to the scan tracker: append Orchard commitments in chain order, decrypt ours, attach witnesses.
pub fn orchard_scan_tracker_apply_block_json(
    tracker_state_json: &str,
    mnemonic_str: &str,
    wallet_address: &str,
    block_height: u32,
    block_json: &str,
) -> Result<String, String> {
    let mut tracker = OrchardWitnessTracker::deserialize_json(tracker_state_json)?;
    let wallet = HDWallet::from_mnemonic(mnemonic_str).map_err(|e| e.to_string())?;
    let block: Value = serde_json::from_str(block_json).map_err(|e| format!("block json: {}", e))?;

    let tx_array = block
        .get("tx")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "block.tx missing".to_string())?;

    let mut discovered: Vec<OrchardDecryptionResult> = Vec::new();

    for tx in tx_array {
        if tx.as_str().is_some() {
            continue;
        }
        let txid = tx
            .get("txid")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "tx.txid".to_string())?
            .to_string();

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

        let orchard = tx.get("orchard").ok_or_else(|| "orchard".to_string())?;
        let actions = orchard
            .get("actions")
            .and_then(|a| a.as_array())
            .ok_or_else(|| "actions".to_string())?;

        for action in actions {
            let Some(compact) = action_json_to_compact(action) else {
                continue;
            };
            let cmx_node = merkle_hash_from_cmx_bytes(&compact.cmx)?;
            tracker.append_cmx(cmx_node)?;

            let decrypted = wallet
                .decrypt_orchard_action_compact(&compact, wallet_address, block_height, &txid)
                .map_err(|e| format!("{:?}", e))?;
            if let Some(mut note) = decrypted {
                tracker.register_discovered_note(note.nullifier)?;
                let wh = tracker
                    .serialized_witness_for_nullifier(&note.nullifier)?
                    .ok_or_else(|| "witness missing after register".to_string())?;
                note.orchard_incremental_witness_hex = Some(hex::encode(wh));
                note.orchard_witness_tip_height = Some(block_height);
                discovered.push(note);
            }
        }
    }

    let out = serde_json::json!({
        "tracker_state": tracker.serialize_json()?,
        "notes": discovered,
    });
    serde_json::to_string(&out).map_err(|e| e.to_string())
}

/// Pass empty string to start from an empty Orchard tree (only valid when scanning from chain genesis / NU5).
#[wasm_bindgen]
pub fn orchard_scan_tracker_new(final_state_hex: &str) -> Result<String, JsError> {
    let t = OrchardWitnessTracker::from_final_state_hex(if final_state_hex.is_empty() {
        None
    } else {
        Some(final_state_hex)
    })
    .map_err(|e| JsError::new(&e))?;
    t.serialize_json().map_err(|e| JsError::new(&e))
}

#[wasm_bindgen]
pub fn orchard_scan_tracker_apply_block(
    tracker_state_json: &str,
    mnemonic_str: &str,
    wallet_address: &str,
    block_height: u32,
    block_json: &str,
) -> Result<JsValue, JsError> {
    let s = orchard_scan_tracker_apply_block_json(
        tracker_state_json,
        mnemonic_str,
        wallet_address,
        block_height,
        block_json,
    )
    .map_err(|e| JsError::new(&e))?;
    let v: serde_json::Value =
        serde_json::from_str(&s).map_err(|e| JsError::new(&format!("json: {}", e)))?;
    serde_wasm_bindgen::to_value(&v).map_err(|e| JsError::new(&format!("{}", e)))
}
