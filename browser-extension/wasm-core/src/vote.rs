//! NU7 coinholder vote helpers for the extension WASM wallet.
//!
//! Export/sign stay in-browser (seed never leaves the extension).
//! Prepare / PIR / cast still run on companion `nozy-vote` (same split as Desktop).

use orchard::keys::{FullViewingKey, SpendingKey};
use serde_json::{json, Value};
use wasm_bindgen::prelude::*;
use zip32::fingerprint::SeedFingerprint;
use zip32::AccountId;

use nozy::hd_wallet::HDWallet;
use zcash_protocol::consensus::NetworkType;

use crate::orchard_tree_codec::orchard_incremental_witness_from_bytes;
use crate::orchard_witness_local::merkle_path_from_witness;

#[wasm_bindgen]
pub fn sign_vote_delegation(mnemonic_str: &str, request_json: &str) -> Result<String, JsError> {
    let wallet = HDWallet::from_mnemonic(mnemonic_str)
        .map_err(|e| JsError::new(&format!("wallet: {e}")))?;
    let sig = nozy::sign_delegation_request_json(&wallet, request_json.as_bytes())
        .map_err(|e| JsError::new(&e.user_friendly_message()))?;
    serde_json::to_string(&sig).map_err(|e| JsError::new(&format!("serialize sig: {e}")))
}

/// Build `nozy-vote-notes-v1` JSON from scanned Ironwood notes (cached witnesses).
#[wasm_bindgen]
pub fn export_ironwood_vote_notes_json(
    mnemonic_str: &str,
    notes_json: &str,
    network: &str,
) -> Result<String, JsError> {
    let wallet = HDWallet::from_mnemonic(mnemonic_str)
        .map_err(|e| JsError::new(&format!("wallet: {e}")))?;
    let net = if network.eq_ignore_ascii_case("testnet") {
        NetworkType::Test
    } else {
        NetworkType::Main
    };

    let seed = wallet.get_mnemonic_object().to_seed("").to_vec();
    let seed_fp = SeedFingerprint::from_seed(&seed)
        .ok_or_else(|| JsError::new("seed fingerprint: invalid seed length"))?;
    let account = AccountId::try_from(0).map_err(|e| JsError::new(&format!("account: {e:?}")))?;
    let sk = SpendingKey::from_zip32_seed(&seed, 133, account)
        .map_err(|e| JsError::new(&format!("spending key: {e:?}")))?;
    let fvk = FullViewingKey::from(&sk);
    let ufvk = wallet
        .generate_orchard_address(0, 0, net)
        .map_err(|e| JsError::new(&format!("address: {e}")))?;

    let raw: Value = serde_json::from_str(notes_json)
        .map_err(|e| JsError::new(&format!("notes json: {e}")))?;
    let rows = raw
        .as_array()
        .ok_or_else(|| JsError::new("notes json must be an array"))?;

    let mut notes = Vec::new();
    for row in rows {
        let note = row.get("note").unwrap_or(row);
        let pool = row
            .get("pool")
            .or_else(|| note.get("pool"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if pool != "ironwood" {
            continue;
        }
        notes.push(export_one_note(row, note)?);
    }

    if notes.is_empty() {
        return Err(JsError::new(
            "No unspent Ironwood notes in this extension wallet. Sync first, and migrate Orchard → Ironwood before the NU7 snapshot.",
        ));
    }

    let file = json!({
        "format": "nozy-vote-notes-v1",
        "network": if matches!(net, NetworkType::Test) { "testnet" } else { "mainnet" },
        "ufvk": ufvk,
        "orchard_fvk_hex": hex::encode(fvk.to_bytes()),
        "seed_fingerprint_hex": hex::encode(seed_fp.to_bytes()),
        "account_index": 0,
        "notes": notes,
    });
    serde_json::to_string(&file).map_err(|e| JsError::new(&format!("serialize export: {e}")))
}

fn export_one_note(row: &Value, note: &Value) -> Result<Value, JsError> {
    let value = json_u64(row, "value")
        .or_else(|| json_u64(note, "value"))
        .ok_or_else(|| JsError::new("Ironwood note missing value"))?;
    let txid = json_str(row, "txid")
        .or_else(|| json_str(note, "txid"))
        .unwrap_or_else(|| "unknown".into());
    let block_height = json_u64(row, "height")
        .or_else(|| json_u64(row, "block_height"))
        .or_else(|| json_u64(note, "block_height"))
        .unwrap_or(0) as u32;

    let wit_hex = json_str(note, "ironwood_incremental_witness_hex")
        .or_else(|| json_str(note, "orchard_incremental_witness_hex"))
        .ok_or_else(|| {
            JsError::new(&format!(
                "Ironwood note in tx {txid} has no witness — rescan, then export again"
            ))
        })?;
    let wbytes = hex::decode(wit_hex.trim_start_matches("0x"))
        .map_err(|e| JsError::new(&format!("witness hex: {e}")))?;
    let witness = orchard_incremental_witness_from_bytes(&wbytes)
        .map_err(|e| JsError::new(&e))?;
    let (anchor, merkle_path) =
        merkle_path_from_witness(&witness).map_err(|e| JsError::new(&e))?;
    let auth_path_hex: Vec<String> = merkle_path
        .auth_path()
        .iter()
        .map(|h| hex::encode(h.to_bytes()))
        .collect();
    if auth_path_hex.len() != 32 {
        return Err(JsError::new(&format!(
            "expected 32 auth path elems, got {}",
            auth_path_hex.len()
        )));
    }

    let cmx = json_bytes32(note, "cmx")
        .ok_or_else(|| JsError::new(&format!("note {txid} missing cmx")))?;
    let nullifier = json_bytes32(note, "nullifier")
        .ok_or_else(|| JsError::new(&format!("note {txid} missing nullifier")))?;
    let rho = json_bytes32(note, "rho")
        .ok_or_else(|| JsError::new(&format!("note {txid} missing rho — rescan")))?;
    let rseed = json_bytes32(note, "rseed")
        .ok_or_else(|| JsError::new(&format!("note {txid} missing rseed — rescan")))?;
    let addr_raw = json_bytes(note, "orchard_address_raw").unwrap_or_default();
    if addr_raw.len() < 11 {
        return Err(JsError::new(&format!(
            "note {txid} missing orchard address / diversifier"
        )));
    }
    let diversifier = &addr_raw[..11];

    Ok(json!({
        "commitment_hex": hex::encode(cmx),
        "nullifier_hex": hex::encode(nullifier),
        "value": value,
        "position": u64::from(u32::from(merkle_path.position())),
        "diversifier_hex": hex::encode(diversifier),
        "rho_hex": hex::encode(rho),
        "rseed_hex": hex::encode(rseed),
        "scope": 0,
        "root_hex": hex::encode(anchor.to_bytes()),
        "auth_path_hex": auth_path_hex,
        "txid": txid,
        "block_height": block_height,
    }))
}

fn json_str(v: &Value, key: &str) -> Option<String> {
    v.get(key)?.as_str().map(|s| s.to_string())
}

fn json_u64(v: &Value, key: &str) -> Option<u64> {
    let n = v.get(key)?;
    n.as_u64()
        .or_else(|| n.as_f64().map(|f| f as u64))
        .or_else(|| n.as_i64().map(|i| i as u64))
}

fn json_bytes32(v: &Value, key: &str) -> Option<[u8; 32]> {
    let bytes = json_bytes(v, key)?;
    if bytes.len() != 32 {
        return None;
    }
    let mut a = [0u8; 32];
    a.copy_from_slice(&bytes);
    Some(a)
}

fn json_bytes(v: &Value, key: &str) -> Option<Vec<u8>> {
    let n = v.get(key)?;
    if let Some(s) = n.as_str() {
        return hex::decode(s.trim_start_matches("0x")).ok();
    }
    if let Some(arr) = n.as_array() {
        let mut out = Vec::with_capacity(arr.len());
        for x in arr {
            out.push(x.as_u64()? as u8);
        }
        return Some(out);
    }
    None
}
