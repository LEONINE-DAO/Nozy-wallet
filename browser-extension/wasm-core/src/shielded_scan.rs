//! Block-by-block Orchard + Ironwood scan with incremental witnesses (Zebrad JSON-RPC).

use serde::Serialize;
use serde_json::{json, Value};
use wasm_bindgen::prelude::*;

use nozy::hd_wallet::{HDWallet, OrchardActionCompactData, OrchardDecryptionResult};

use crate::orchard_witness_local::{merkle_hash_from_cmx_bytes, OrchardWitnessTracker};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShieldedPool {
    Orchard,
    Ironwood,
}

impl ShieldedPool {
    fn as_str(self) -> &'static str {
        match self {
            ShieldedPool::Orchard => "orchard",
            ShieldedPool::Ironwood => "ironwood",
        }
    }
}

struct TrackerPair {
    orchard: OrchardWitnessTracker,
    ironwood: OrchardWitnessTracker,
}

impl TrackerPair {
    fn deserialize(raw: &str) -> Result<Self, String> {
        if raw.trim().is_empty() {
            return Ok(Self {
                orchard: OrchardWitnessTracker::from_final_state_hex(None)?,
                ironwood: OrchardWitnessTracker::from_final_state_hex(None)?,
            });
        }
        if let Ok(v) = serde_json::from_str::<Value>(raw) {
            if v.is_object() {
                let orchard = v.get("orchard").and_then(|x| x.as_str()).unwrap_or("");
                let ironwood = v.get("ironwood").and_then(|x| x.as_str()).unwrap_or("");
                return Ok(Self {
                    orchard: OrchardWitnessTracker::deserialize_json(orchard)?,
                    ironwood: OrchardWitnessTracker::deserialize_json(ironwood)?,
                });
            }
        }
        // Legacy: single orchard tracker string.
        Ok(Self {
            orchard: OrchardWitnessTracker::deserialize_json(raw)?,
            ironwood: OrchardWitnessTracker::from_final_state_hex(None)?,
        })
    }

    fn serialize_json(&self) -> Result<String, String> {
        Ok(json!({
            "orchard": self.orchard.serialize_json()?,
            "ironwood": self.ironwood.serialize_json()?,
        })
        .to_string())
    }
}

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
    let ephemeral_key = action
        .get("ephemeralKey")
        .or_else(|| action.get("ephemeral_key"))?;
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

fn tx_actions(tx: &Value, pool: ShieldedPool) -> Option<&Vec<Value>> {
    match pool {
        ShieldedPool::Orchard => tx
            .get("orchard")
            .and_then(|o| o.get("actions"))
            .and_then(|a| a.as_array()),
        ShieldedPool::Ironwood => {
            let pool_obj = tx
                .get("ironwood")
                .or_else(|| tx.get("ironwoodActions"))
                .or_else(|| tx.get("ironwood_actions"))?;
            if let Some(arr) = pool_obj.as_array() {
                return Some(arr);
            }
            pool_obj.get("actions").and_then(|a| a.as_array())
        }
    }
}

fn note_to_json(note: &OrchardDecryptionResult, pool: ShieldedPool) -> Value {
    let mut v = serde_json::to_value(note).unwrap_or(json!({}));
    if let Some(obj) = v.as_object_mut() {
        obj.insert("pool".to_string(), json!(pool.as_str()));
        if pool == ShieldedPool::Ironwood {
            if let Some(w) = obj.get("orchard_incremental_witness_hex").cloned() {
                obj.insert("ironwood_incremental_witness_hex".to_string(), w);
            }
            if let Some(h) = obj.get("orchard_witness_tip_height").cloned() {
                obj.insert("ironwood_witness_tip_height".to_string(), h);
            }
        }
    }
    v
}

fn nozy_pool(pool: ShieldedPool) -> nozy::shielded_pool::ShieldedPool {
    match pool {
        ShieldedPool::Orchard => nozy::shielded_pool::ShieldedPool::Orchard,
        ShieldedPool::Ironwood => nozy::shielded_pool::ShieldedPool::Ironwood,
    }
}

#[derive(Default)]
struct ScanDebug {
    ow_seen: u32,
    ow_compact_ok: u32,
    ow_decrypt_ok: u32,
    ow_other_domain: u32,
    iw_seen: u32,
    iw_compact_ok: u32,
    iw_decrypt_ok: u32,
    iw_other_domain: u32,
}

impl ScanDebug {
    fn note_seen(&mut self, pool: ShieldedPool, n: u32) {
        match pool {
            ShieldedPool::Orchard => self.ow_seen = self.ow_seen.saturating_add(n),
            ShieldedPool::Ironwood => self.iw_seen = self.iw_seen.saturating_add(n),
        }
    }

    fn to_json(&self) -> Value {
        json!({
            "owSeen": self.ow_seen,
            "owCompactOk": self.ow_compact_ok,
            "owDecryptOk": self.ow_decrypt_ok,
            "owOtherDomain": self.ow_other_domain,
            "iwSeen": self.iw_seen,
            "iwCompactOk": self.iw_compact_ok,
            "iwDecryptOk": self.iw_decrypt_ok,
            "iwOtherDomain": self.iw_other_domain,
        })
    }
}

fn apply_pool_actions(
    tracker: &mut OrchardWitnessTracker,
    wallet: &HDWallet,
    wallet_address: &str,
    block_height: u32,
    txid: &str,
    actions: &[Value],
    pool: ShieldedPool,
    debug: &mut ScanDebug,
) -> Result<Vec<Value>, String> {
    let mut discovered = Vec::new();
    debug.note_seen(pool, actions.len() as u32);
    let other_pool = match pool {
        ShieldedPool::Orchard => ShieldedPool::Ironwood,
        ShieldedPool::Ironwood => ShieldedPool::Orchard,
    };
    for action in actions {
        let Some(compact) = action_json_to_compact(action) else {
            continue;
        };
        match pool {
            ShieldedPool::Orchard => debug.ow_compact_ok = debug.ow_compact_ok.saturating_add(1),
            ShieldedPool::Ironwood => debug.iw_compact_ok = debug.iw_compact_ok.saturating_add(1),
        }
        let cmx_node = merkle_hash_from_cmx_bytes(&compact.cmx)?;
        tracker.append_cmx(cmx_node)?;

        let decrypted = wallet
            .decrypt_orchard_action_compact(
                &compact,
                wallet_address,
                block_height,
                txid,
                nozy_pool(pool),
            )
            .map_err(|e| format!("{:?}", e))?;
        let decrypted = match decrypted {
            Some(note) => {
                match pool {
                    ShieldedPool::Orchard => {
                        debug.ow_decrypt_ok = debug.ow_decrypt_ok.saturating_add(1)
                    }
                    ShieldedPool::Ironwood => {
                        debug.iw_decrypt_ok = debug.iw_decrypt_ok.saturating_add(1)
                    }
                }
                Some(note)
            }
            None => {
                let other = wallet
                    .decrypt_orchard_action_compact(
                        &compact,
                        wallet_address,
                        block_height,
                        txid,
                        nozy_pool(other_pool),
                    )
                    .map_err(|e| format!("{:?}", e))?;
                if other.is_some() {
                    match pool {
                        ShieldedPool::Orchard => {
                            debug.ow_other_domain = debug.ow_other_domain.saturating_add(1)
                        }
                        ShieldedPool::Ironwood => {
                            debug.iw_other_domain = debug.iw_other_domain.saturating_add(1)
                        }
                    }
                }
                None
            }
        };
        if let Some(mut note) = decrypted {
            tracker.register_discovered_note(note.nullifier)?;
            let wh = tracker
                .serialized_witness_for_nullifier(&note.nullifier)?
                .ok_or_else(|| "witness missing after register".to_string())?;
            note.orchard_incremental_witness_hex = Some(hex::encode(wh));
            note.orchard_witness_tip_height = Some(block_height);
            discovered.push(note_to_json(&note, pool));
        }
    }
    Ok(discovered)
}

/// Apply one block to Orchard + Ironwood trackers.
pub fn shielded_scan_tracker_apply_block_json(
    tracker_state_json: &str,
    mnemonic_str: &str,
    wallet_address: &str,
    block_height: u32,
    block_json: &str,
) -> Result<String, String> {
    let mut trackers = TrackerPair::deserialize(tracker_state_json)?;
    let wallet = HDWallet::from_mnemonic(mnemonic_str).map_err(|e| e.to_string())?;
    let block: Value =
        serde_json::from_str(block_json).map_err(|e| format!("block json: {}", e))?;

    let tx_array = block
        .get("tx")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "block.tx missing".to_string())?;

    let mut discovered: Vec<Value> = Vec::new();
    let mut scan_debug = ScanDebug::default();

    for tx in tx_array {
        if tx.as_str().is_some() {
            continue;
        }
        let txid = tx
            .get("txid")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "tx.txid".to_string())?
            .to_string();

        if let Some(actions) = tx_actions(tx, ShieldedPool::Orchard) {
            if !actions.is_empty() {
                discovered.extend(apply_pool_actions(
                    &mut trackers.orchard,
                    &wallet,
                    wallet_address,
                    block_height,
                    &txid,
                    actions,
                    ShieldedPool::Orchard,
                    &mut scan_debug,
                )?);
            }
        }

        if let Some(actions) = tx_actions(tx, ShieldedPool::Ironwood) {
            if !actions.is_empty() {
                discovered.extend(apply_pool_actions(
                    &mut trackers.ironwood,
                    &wallet,
                    wallet_address,
                    block_height,
                    &txid,
                    actions,
                    ShieldedPool::Ironwood,
                    &mut scan_debug,
                )?);
            }
        }
    }

    let out = json!({
        "tracker_state": trackers.serialize_json()?,
        "orchard_tracker_state": trackers.orchard.serialize_json()?,
        "ironwood_tracker_state": trackers.ironwood.serialize_json()?,
        "notes": discovered,
        "scanDebug": scan_debug.to_json(),
    });
    serde_json::to_string(&out).map_err(|e| e.to_string())
}

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

/// Initialize dual-pool trackers from Zebrad `z_gettreestate` final states.
#[wasm_bindgen]
pub fn shielded_scan_tracker_new(
    orchard_final_state_hex: &str,
    ironwood_final_state_hex: &str,
) -> Result<String, JsError> {
    let pair = TrackerPair {
        orchard: OrchardWitnessTracker::from_final_state_hex(
            if orchard_final_state_hex.is_empty() {
                None
            } else {
                Some(orchard_final_state_hex)
            },
        )
        .map_err(|e| JsError::new(&e))?,
        ironwood: OrchardWitnessTracker::from_final_state_hex(
            if ironwood_final_state_hex.is_empty() {
                None
            } else {
                Some(ironwood_final_state_hex)
            },
        )
        .map_err(|e| JsError::new(&e))?,
    };
    pair.serialize_json().map_err(|e| JsError::new(&e))
}

#[wasm_bindgen]
pub fn orchard_scan_tracker_apply_block(
    tracker_state_json: &str,
    mnemonic_str: &str,
    wallet_address: &str,
    block_height: u32,
    block_json: &str,
) -> Result<JsValue, JsError> {
    let s = shielded_scan_tracker_apply_block_json(
        tracker_state_json,
        mnemonic_str,
        wallet_address,
        block_height,
        block_json,
    )
    .map_err(|e| JsError::new(&e))?;
    let v: Value = serde_json::from_str(&s).map_err(|e| JsError::new(&format!("json: {}", e)))?;
    // serde_json::Value objects become JS Maps by default; the service worker reads
    // `out.notes` as a plain object property.
    v.serialize(&serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true))
        .map_err(|e| JsError::new(&format!("{}", e)))
}
//Nozy people dont use zebra, they use zcashd
