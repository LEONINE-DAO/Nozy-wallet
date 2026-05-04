//! Block-by-block Orchard scan with incremental witnesses for Zebrad-based flows.

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

#[cfg(test)]
mod tests {
    use super::orchard_scan_tracker_apply_block_json;
    use crate::orchard_witness_local::OrchardWitnessTracker;
    use bip39::Mnemonic;
    use nozy::hd_wallet::HDWallet;
    use orchard::keys::{DiversifierIndex, FullViewingKey, Scope, SpendingKey};
    use orchard::note::{ExtractedNoteCommitment, Nullifier, Note, RandomSeed, Rho};
    use orchard::note_encryption::{CompactAction, OrchardDomain, OrchardNoteEncryption};
    use orchard::value::NoteValue;
    use rand::rngs::StdRng;
    use rand::{RngCore, SeedableRng};
    use zcash_note_encryption::{Domain, ShieldedOutput};
    use zcash_primitives::zip32::AccountId;
    use zcash_protocol::consensus::NetworkType;

    const TEST_MNEMONIC: &str =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    fn tracker_empty_json() -> String {
        OrchardWitnessTracker::from_final_state_hex(None)
            .expect("empty tree")
            .serialize_json()
            .expect("serialize")
    }

    fn sample_nullifier(rng: &mut impl RngCore) -> Nullifier {
        loop {
            let mut b = [0u8; 32];
            rng.fill_bytes(&mut b);
            if let Some(nf) = Nullifier::from_bytes(&b).into() {
                return nf;
            }
        }
    }

    fn compact_action_for_recipient(
        rng: &mut impl RngCore,
        fvk: &FullViewingKey,
        recipient: orchard::Address,
        nf_old: Nullifier,
        value: NoteValue,
    ) -> CompactAction {
        // For Orchard outputs, ρ is derived from the spent note's nullifier (same base encoding).
        let rho = Option::from(Rho::from_bytes(&nf_old.to_bytes())).expect("rho from nf_old");
        let rseed = loop {
            let mut bytes = [0u8; 32];
            rng.fill_bytes(&mut bytes);
            if let Some(rs) = RandomSeed::from_bytes(bytes, &rho).into() {
                break rs;
            }
        };
        let note = Note::from_parts(recipient, value, rho, rseed).expect("valid note");
        let cmx = ExtractedNoteCommitment::from(note.commitment());
        let ne = OrchardNoteEncryption::new(
            Some(fvk.to_ovk(Scope::External)),
            note,
            [0u8; 512],
        );
        let ephemeral_key = OrchardDomain::epk_bytes(ne.epk());
        let enc_full = ne.encrypt_note_plaintext();
        let enc_compact: [u8; 52] = enc_full[..52].try_into().expect("compact enc len");
        CompactAction::from_parts(nf_old, cmx, ephemeral_key, enc_compact)
    }

    fn block_json_from_compact(
        compact: &CompactAction,
        txid_hex: &str,
        enc_field: &str,
        epk_field: &str,
    ) -> String {
        let nullifier = hex::encode(compact.nullifier().to_bytes());
        let cmx = hex::encode(compact.cmx().to_bytes());
        let ephemeral_key = hex::encode(compact.ephemeral_key().0);
        let enc = hex::encode(compact.enc_ciphertext().as_slice());
        let mut action = serde_json::Map::new();
        action.insert("nullifier".to_string(), serde_json::Value::String(nullifier));
        action.insert("cmx".to_string(), serde_json::Value::String(cmx));
        action.insert(enc_field.to_string(), serde_json::Value::String(enc));
        action.insert(
            epk_field.to_string(),
            serde_json::Value::String(ephemeral_key),
        );
        serde_json::json!({
            "tx": [{
                "txid": txid_hex,
                "orchard": { "actions": [ serde_json::Value::Object(action) ] }
            }]
        })
        .to_string()
    }

    #[test]
    fn fixture_missing_tx_errors() {
        let fixture = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/zebrad_getblock_missing_tx.json"
        ));
        let err = orchard_scan_tracker_apply_block_json(
            &tracker_empty_json(),
            TEST_MNEMONIC,
            "u1abc", // unused for parse error
            1,
            fixture,
        )
        .expect_err("expected block.tx error");
        assert!(err.contains("block.tx"), "{}", err);
    }

    #[test]
    fn fixture_empty_orchard_actions_no_notes_tree_unchanged() {
        let fixture = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/zebrad_getblock_empty_orchard_actions.json"
        ));
        let t0 = tracker_empty_json();
        let out = orchard_scan_tracker_apply_block_json(
            &t0,
            TEST_MNEMONIC,
            "u1abc",
            1,
            fixture,
        )
        .expect("apply");
        let v: serde_json::Value = serde_json::from_str(&out).expect("json");
        let notes = v.get("notes").and_then(|n| n.as_array()).expect("notes array");
        assert!(notes.is_empty());
        let t1 = v
            .get("tracker_state")
            .and_then(|x| x.as_str())
            .expect("tracker_state");
        assert_eq!(t0, t1);
    }

    #[test]
    fn fixture_short_enc_skipped_no_append() {
        let fixture = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/zebrad_getblock_action_short_enc.json"
        ));
        let t0 = tracker_empty_json();
        let out = orchard_scan_tracker_apply_block_json(
            &t0,
            TEST_MNEMONIC,
            "u1abc",
            1,
            fixture,
        )
        .expect("apply");
        let v: serde_json::Value = serde_json::from_str(&out).expect("json");
        assert!(v
            .get("notes")
            .and_then(|n| n.as_array())
            .expect("notes")
            .is_empty());
        assert_eq!(
            t0,
            v.get("tracker_state")
                .and_then(|x| x.as_str())
                .expect("tracker_state")
        );
    }

    #[test]
    fn zebrad_shape_decrypts_note_and_snake_case_fields() {
        let wallet = HDWallet::from_mnemonic(TEST_MNEMONIC).expect("wallet");
        let ua = wallet
            .generate_orchard_address(0, 0, NetworkType::Test)
            .expect("ua");

        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).expect("parse");
        let seed = mnemonic.to_seed("");
        let orchard_sk =
            SpendingKey::from_zip32_seed(&seed, 133, AccountId::try_from(0).expect("account"))
                .expect("orchard sk");
        let fvk = FullViewingKey::from(&orchard_sk);
        let recipient = fvk.address_at(DiversifierIndex::from(0u32), Scope::External);

        let mut rng = StdRng::seed_from_u64(99);
        let nf_old = sample_nullifier(&mut rng);
        let compact = compact_action_for_recipient(
            &mut rng,
            &fvk,
            recipient,
            nf_old,
            NoteValue::from_raw(5000),
        );

        let txid_hex = "a1".repeat(32);
        let tracker_json = tracker_empty_json();

        let camel = block_json_from_compact(&compact, &txid_hex, "encCiphertext", "ephemeralKey");
        let out_camel = orchard_scan_tracker_apply_block_json(
            &tracker_json,
            TEST_MNEMONIC,
            &ua,
            7,
            &camel,
        )
        .expect("camelCase");
        let v: serde_json::Value = serde_json::from_str(&out_camel).expect("json");
        let notes = v.get("notes").and_then(|n| n.as_array()).expect("notes");
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].get("value").and_then(|x| x.as_u64()), Some(5000));
        assert!(
            notes[0]
                .get("orchard_incremental_witness_hex")
                .and_then(|x| x.as_str())
                .is_some(),
            "witness hex"
        );

        let snake = block_json_from_compact(
            &compact,
            &txid_hex,
            "enc_ciphertext",
            "ephemeral_key",
        );
        let out_snake = orchard_scan_tracker_apply_block_json(
            &tracker_json,
            TEST_MNEMONIC,
            &ua,
            7,
            &snake,
        )
        .expect("snake_case");
        let v2: serde_json::Value = serde_json::from_str(&out_snake).expect("json");
        assert_eq!(
            v.get("notes").expect("notes"),
            v2.get("notes").expect("notes2"),
            "camel vs snake decrypt parity"
        );
    }

}
