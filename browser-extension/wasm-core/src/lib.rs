use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

mod orchard_block_parse;
mod orchard_tree_codec;
mod orchard_witness_local;
mod orchard_scan;

#[derive(Serialize, Deserialize)]
pub struct OrchardActionInput {
    pub nullifier: Vec<u8>,
    pub cmx: Vec<u8>,
    pub ephemeral_key: Vec<u8>,
    pub encrypted_note: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct ScanResult {
    pub scanned_actions: usize,
    pub decrypted_notes: usize,
    pub total_value_zats: u64,
    pub notes: Vec<nozy::hd_wallet::OrchardDecryptionResult>,
}

#[derive(Serialize, Deserialize)]
pub struct WalletCreationResult {
    pub mnemonic: String,
    pub address: String,
    pub encrypted_seed: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct WalletUnlockResult {
    pub address: String,
}

#[derive(Serialize, Deserialize)]
pub struct ProveOrchardTxResult {
    pub txid: String,
    #[serde(rename = "rawTxHex")]
    pub raw_tx_hex: String,
    pub bundle_actions: usize,
    pub proof_generated: bool,
}

#[derive(Deserialize)]
struct OrchardWitnessAuthPath {
    anchor: String,
    position: u32,
    auth_path: Vec<String>,
}

/// Zebrad-safe incremental witness, or zcashd-style auth path witness.
fn parse_orchard_witness_for_spend(
    witness_json: &str,
) -> Result<(orchard::tree::Anchor, orchard::tree::MerklePath), JsError> {
    use orchard::tree::{Anchor, MerkleHashOrchard, MerklePath};
    let v: serde_json::Value = serde_json::from_str(witness_json)
        .map_err(|e| JsError::new(&format!("witness json: {e}")))?;
    if let Some(inc_hex) = v.get("incremental_witness_hex").and_then(|x| x.as_str()) {
        let anchor_hex = v
            .get("anchor_hex")
            .and_then(|x| x.as_str())
            .ok_or_else(|| JsError::new("anchor_hex required with incremental_witness_hex"))?;
        let w = crate::orchard_tree_codec::orchard_incremental_witness_from_bytes(
            &hex::decode(inc_hex.trim_start_matches("0x"))
                .map_err(|e| JsError::new(&format!("witness hex: {e}")))?,
        )
        .map_err(|e| JsError::new(&e))?;
        let ab = hex::decode(anchor_hex.trim_start_matches("0x"))
            .map_err(|e| JsError::new(&format!("anchor hex: {e}")))?;
        if ab.len() != 32 {
            return Err(JsError::new("anchor must be 32 bytes"));
        }
        let mut ah = [0u8; 32];
        ah.copy_from_slice(&ab);
        if !crate::orchard_witness_local::witness_root_matches_anchor(&w, &ah) {
            return Err(JsError::new(
                "Orchard witness root does not match anchor; advance witness with advance_orchard_witness_hex",
            ));
        }
        return crate::orchard_witness_local::merkle_path_from_witness(&w)
            .map_err(|e| JsError::new(&e));
    }

    let witness: OrchardWitnessAuthPath = serde_json::from_str(witness_json).map_err(|e| {
        JsError::new(&format!("Invalid auth-path witness_json: {e}"))
    })?;

    let anchor_bytes_vec = hex::decode(witness.anchor.trim_start_matches("0x"))
        .map_err(|e| JsError::new(&format!("Invalid witness anchor hex: {e}")))?;
    if anchor_bytes_vec.len() != 32 {
        return Err(JsError::new("Witness anchor must be 32 bytes"));
    }
    let anchor_bytes: [u8; 32] = anchor_bytes_vec
        .as_slice()
        .try_into()
        .map_err(|_| JsError::new("Witness anchor length mismatch"))?;
    let anchor = Anchor::from_bytes(anchor_bytes)
        .into_option()
        .ok_or_else(|| JsError::new("Invalid witness anchor bytes"))?;
    if witness.auth_path.len() != 32 {
        return Err(JsError::new("auth_path must have 32 elements"));
    }
    let merkle_hashes_vec: Vec<MerkleHashOrchard> = witness
        .auth_path
        .iter()
        .enumerate()
        .map(|(i, h)| {
            let bytes_vec = hex::decode(h.trim_start_matches("0x")).map_err(|e| {
                JsError::new(&format!("Invalid auth_path[{i}] hex: {e}"))
            })?;
            if bytes_vec.len() != 32 {
                return Err(JsError::new(&format!("auth_path[{i}] must be 32 bytes")));
            }
            let arr: [u8; 32] = bytes_vec
                .as_slice()
                .try_into()
                .map_err(|_| JsError::new(&format!("auth_path[{i}] length mismatch")))?;
            MerkleHashOrchard::from_bytes(&arr)
                .into_option()
                .ok_or_else(|| JsError::new(&format!("Invalid merkle hash at {i}")))
        })
        .collect::<Result<Vec<_>, JsError>>()?;
    let merkle_hashes: [MerkleHashOrchard; 32] = merkle_hashes_vec
        .try_into()
        .map_err(|_| JsError::new("auth_path conversion failed"))?;
    let merkle_path = MerklePath::from_parts(witness.position, merkle_hashes);
    Ok((anchor, merkle_path))
}

#[wasm_bindgen]
pub fn advance_orchard_witness_hex(witness_hex: &str, block_json: &str) -> Result<String, JsError> {
    use crate::orchard_block_parse::orchard_cmx_bytes_from_block_json;
    use crate::orchard_tree_codec::orchard_incremental_witness_to_bytes;
    use crate::orchard_witness_local::{advance_witness_with_cmxs, merkle_hash_from_cmx_bytes};
    use serde_json::Value;
    let mut w = crate::orchard_tree_codec::orchard_incremental_witness_from_bytes(
        &hex::decode(witness_hex.trim_start_matches("0x"))
            .map_err(|e| JsError::new(&format!("witness hex: {e}")))?,
    )
    .map_err(|e| JsError::new(&e))?;
    let block: Value = serde_json::from_str(block_json)
        .map_err(|e| JsError::new(&format!("block json: {e}")))?;
    let cmxs = orchard_cmx_bytes_from_block_json(&block).map_err(|e| JsError::new(&e))?;
    for cmx in cmxs {
        let node = merkle_hash_from_cmx_bytes(&cmx).map_err(|e| JsError::new(&e))?;
        advance_witness_with_cmxs(&mut w, std::iter::once(node)).map_err(|e| JsError::new(&e))?;
    }
    let bytes = orchard_incremental_witness_to_bytes(&w).map_err(|e| JsError::new(&e))?;
    Ok(hex::encode(bytes))
}

#[wasm_bindgen]
pub fn orchard_witness_matches_anchor_hex(witness_hex: &str, anchor_hex: &str) -> Result<bool, JsError> {
    let w = crate::orchard_tree_codec::orchard_incremental_witness_from_bytes(
        &hex::decode(witness_hex.trim_start_matches("0x"))
            .map_err(|e| JsError::new(&format!("witness hex: {e}")))?,
    )
    .map_err(|e| JsError::new(&e))?;
    let ab = hex::decode(anchor_hex.trim_start_matches("0x"))
        .map_err(|e| JsError::new(&format!("anchor hex: {e}")))?;
    if ab.len() != 32 {
        return Err(JsError::new("anchor len"));
    }
    let mut a = [0u8; 32];
    a.copy_from_slice(&ab);
    Ok(crate::orchard_witness_local::witness_root_matches_anchor(&w, &a))
}

#[wasm_bindgen]
pub fn create_wallet(password: &str) -> Result<JsValue, JsError> {
    use bip39::Mnemonic;
    use nozy::hd_wallet::HDWallet;
    use rand::RngCore;
    use zcash_protocol::consensus::NetworkType;

    let mut entropy = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut entropy);
    let mnemonic = Mnemonic::from_entropy_in(bip39::Language::English, &entropy)
        .map_err(|e| JsError::new(&format!("Mnemonic generation failed: {}", e)))?;

    let wallet = HDWallet::from_mnemonic(&mnemonic.to_string())
        .map_err(|e| JsError::new(&format!("Wallet creation failed: {}", e)))?;

    let address = wallet.generate_orchard_address(0, 0, NetworkType::Main)
        .map_err(|e| JsError::new(&format!("Address generation failed: {}", e)))?;

    let seed_bytes = mnemonic.to_seed(password).to_vec();
    let encrypted_seed = encrypt_data(&seed_bytes, password)?;

    let result = WalletCreationResult {
        mnemonic: mnemonic.to_string(),
        address,
        encrypted_seed,
    };

    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsError::new(&format!("Serialization failed: {}", e)))
}

#[wasm_bindgen]
pub fn restore_wallet(mnemonic_str: &str, password: &str) -> Result<JsValue, JsError> {
    use nozy::hd_wallet::HDWallet;
    use bip39::Mnemonic;
    use zcash_protocol::consensus::NetworkType;

    let mnemonic: Mnemonic = mnemonic_str.parse()
        .map_err(|e| JsError::new(&format!("Invalid mnemonic: {}", e)))?;

    let wallet = HDWallet::from_mnemonic(&mnemonic.to_string())
        .map_err(|e| JsError::new(&format!("Wallet restore failed: {}", e)))?;

    let address = wallet.generate_orchard_address(0, 0, NetworkType::Main)
        .map_err(|e| JsError::new(&format!("Address generation failed: {}", e)))?;

    let seed_bytes = mnemonic.to_seed(password).to_vec();
    let encrypted_seed = encrypt_data(&seed_bytes, password)?;

    let result = WalletCreationResult {
        mnemonic: mnemonic.to_string(),
        address,
        encrypted_seed,
    };

    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsError::new(&format!("Serialization failed: {}", e)))
}

#[wasm_bindgen]
pub fn unlock_wallet(encrypted_seed: &[u8], password: &str) -> Result<JsValue, JsError> {
    use nozy::hd_wallet::HDWallet;
    use zcash_protocol::consensus::NetworkType;

    let seed = decrypt_data(encrypted_seed, password)?;

    let mnemonic = bip39::Mnemonic::from_entropy(&seed[..32])
        .map_err(|e| JsError::new(&format!("Seed decode failed: {}", e)))?;

    let wallet = HDWallet::from_mnemonic(&mnemonic.to_string())
        .map_err(|e| JsError::new(&format!("Wallet unlock failed: {}", e)))?;

    let address = wallet.generate_orchard_address(0, 0, NetworkType::Main)
        .map_err(|e| JsError::new(&format!("Address generation failed: {}", e)))?;

    let result = WalletUnlockResult { address };

    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsError::new(&format!("Serialization failed: {}", e)))
}

#[wasm_bindgen]
pub fn generate_address(mnemonic_str: &str, account: u32, index: u32) -> Result<String, JsError> {
    use nozy::hd_wallet::HDWallet;
    use zcash_protocol::consensus::NetworkType;

    let wallet = HDWallet::from_mnemonic(mnemonic_str)
        .map_err(|e| JsError::new(&format!("Wallet creation failed: {}", e)))?;

    wallet.generate_orchard_address(account, index, NetworkType::Main)
        .map_err(|e| JsError::new(&format!("Address generation failed: {}", e)))
}

#[wasm_bindgen]
pub fn get_zcash_chain_id() -> String {
    "0x5ba3".to_string()
}

#[wasm_bindgen]
pub fn get_nu5_activation_height() -> u32 {
    use zcash_protocol::consensus::{MainNetwork, Parameters, NetworkUpgrade};
    MainNetwork
        .activation_height(NetworkUpgrade::Nu5)
        .map(|h| u32::from(h))
        .unwrap_or(0)
}

#[wasm_bindgen]
pub fn sign_message(mnemonic_str: &str, message: &str) -> Result<String, JsError> {
    use nozy::hd_wallet::HDWallet;
    use sha2::{Sha256, Digest};

    let wallet = HDWallet::from_mnemonic(mnemonic_str)
        .map_err(|e| JsError::new(&format!("Wallet creation failed: {}", e)))?;

    let seed_bytes = wallet.get_mnemonic_object().to_seed("");
    let mut hasher = Sha256::new();
    hasher.update(&seed_bytes);
    hasher.update(message.as_bytes());
    let signature = hasher.finalize();

    Ok(hex::encode(signature))
}

#[wasm_bindgen]
pub fn scan_orchard_actions(
    mnemonic_str: &str,
    address: &str,
    actions_json: &str,
    block_height: u32,
    txid: &str,
) -> Result<JsValue, JsError> {
    use nozy::hd_wallet::{HDWallet, OrchardActionCompactData};

    let wallet = HDWallet::from_mnemonic(mnemonic_str)
        .map_err(|e| JsError::new(&format!("Wallet creation failed: {}", e)))?;

    let actions: Vec<OrchardActionInput> = serde_json::from_str(actions_json)
        .map_err(|e| JsError::new(&format!("Invalid actions JSON: {}", e)))?;

    let mut notes = Vec::new();
    let mut total_value = 0u64;

    for action in &actions {
        if action.nullifier.len() != 32 || action.cmx.len() != 32 || action.ephemeral_key.len() != 32 {
            continue;
        }

        let compact = OrchardActionCompactData {
            nullifier: action.nullifier.clone().try_into().map_err(|_| JsError::new("Invalid nullifier length"))?,
            cmx: action.cmx.clone().try_into().map_err(|_| JsError::new("Invalid cmx length"))?,
            ephemeral_key: action.ephemeral_key.clone().try_into().map_err(|_| JsError::new("Invalid ephemeral_key length"))?,
            encrypted_note: action.encrypted_note.clone(),
        };

        if let Some(note) = wallet
            .decrypt_orchard_action_compact(&compact, address, block_height, txid)
            .map_err(|e| JsError::new(&format!("Decrypt action failed: {}", e)))?
        {
            total_value = total_value.saturating_add(note.value);
            notes.push(note);
        }
    }

    let result = ScanResult {
        scanned_actions: actions.len(),
        decrypted_notes: notes.len(),
        total_value_zats: total_value,
        notes,
    };

    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsError::new(&format!("Serialization failed: {}", e)))
}

#[wasm_bindgen]
pub fn prove_orchard_transaction_dummy(
    recipient_address: &str,
    amount_zatoshis: u64,
    memo: &str,
) -> Result<JsValue, JsError> {
    use orchard::{
        builder::Builder as OrchardBuilder,
        builder::BundleType,
        bundle::Flags,
        circuit::ProvingKey,
        tree::Anchor,
        value::NoteValue,
    };
    use rand::rngs::OsRng;
    use sha2::{Digest, Sha256};
    use zcash_address::unified::{Address as UnifiedAddress, Receiver, Encoding, Container};

    let (_network, ua) = UnifiedAddress::decode(recipient_address).map_err(|e| {
        JsError::new(&format!("Invalid recipient address: {e}"))
    })?;

    let mut orchard_receiver_raw = None;
    for item in ua.items() {
        if let Receiver::Orchard(data) = item {
            orchard_receiver_raw = Some(data);
            break;
        }
    }

    let orchard_receiver_raw = orchard_receiver_raw.ok_or_else(|| {
        JsError::new("Recipient address does not contain an Orchard receiver")
    })?;

    let orchard_address = orchard::Address::from_raw_address_bytes(&orchard_receiver_raw)
        .into_option()
        .ok_or_else(|| JsError::new("Invalid Orchard receiver bytes"))?;

    let memo_bytes = {
        let mut out = [0u8; 512];
        let m = memo.as_bytes();
        let len = m.len().min(512);
        out[..len].copy_from_slice(&m[..len]);
        out
    };

    let bundle_type = BundleType::Transactional {
        flags: Flags::ENABLED,
        bundle_required: true,
    };

    let anchor = Anchor::from_bytes([0u8; 32])
        .into_option()
        .ok_or_else(|| JsError::new("Invalid dummy anchor bytes"))?;

    let mut builder = OrchardBuilder::new(bundle_type, anchor);
    builder
        .add_output(
            None,
            orchard_address,
            NoteValue::from_raw(amount_zatoshis),
            memo_bytes,
        )
        .map_err(|e| JsError::new(&format!("Failed to add output: {e}")))?;

    let mut rng = OsRng;
    let bundle_result = builder
        .build::<i64>(&mut rng)
        .map_err(|e| JsError::new(&format!("Failed to build Orchard bundle: {e}")))?;
    let (unauthorized_bundle, _bundle_meta) = bundle_result.ok_or_else(|| {
        JsError::new("Orchard builder did not produce a bundle")
    })?;

    let pk = ProvingKey::build();
    let proved = unauthorized_bundle
        .create_proof(&pk, &mut rng)
        .map_err(|e| JsError::new(&format!("Failed to create Orchard proof: {e}")))?;

    let prepared = proved.prepare(&mut rng, [0; 32]);
    let authorized = prepared.finalize().map_err(|e| {
        JsError::new(&format!("Failed to finalize Orchard bundle signatures: {e}"))
    })?;

    let bundle_actions = authorized.actions().len();

    let now_ms = js_sys::Date::now() as u64;
    let mut hasher = Sha256::new();
    hasher.update(recipient_address.as_bytes());
    hasher.update(&amount_zatoshis.to_le_bytes());
    hasher.update(memo.as_bytes());
    hasher.update(&now_ms.to_le_bytes());
    let txid = hex::encode(hasher.finalize());

    let result = ProveOrchardTxResult {
        txid,
        raw_tx_hex: String::new(),
        bundle_actions,
        proof_generated: true,
    };

    serde_wasm_bindgen::to_value(&result).map_err(|e| {
        JsError::new(&format!("Serialization failed: {e}"))
    })
}

#[wasm_bindgen]
pub fn prove_orchard_transaction_spend_from_note(
    mnemonic_str: &str,
    recipient_address: &str,
    amount_zatoshis: u64,
    memo: &str,
    spend_note_json: &str,
    witness_json: &str,
) -> Result<JsValue, JsError> {
    use bip39::Mnemonic;
    use nozy::hd_wallet::OrchardDecryptionResult;
    use orchard::{
        builder::Builder as OrchardBuilder,
        builder::BundleType,
        bundle::Flags,
        circuit::ProvingKey,
        keys::{FullViewingKey, SpendingKey},
        note::{Note, RandomSeed, Rho},
        value::NoteValue,
    };
    use rand::rngs::OsRng;
    use sha2::Digest;
    use zcash_address::unified::{Address as UnifiedAddress, Container, Encoding, Receiver};
    use zcash_primitives::zip32::AccountId;

    let (anchor, merkle_path) = parse_orchard_witness_for_spend(witness_json)?;

    let spend_note: OrchardDecryptionResult = serde_json::from_str(spend_note_json).map_err(|e| {
        JsError::new(&format!("Invalid spend_note_json: {e}"))
    })?;

    if amount_zatoshis > spend_note.value {
        return Err(JsError::new(
            "Requested amount exceeds selected note value (prototype limitation).",
        ));
    }

    let (_network, ua) = UnifiedAddress::decode(recipient_address).map_err(|e| {
        JsError::new(&format!("Invalid recipient address: {e}"))
    })?;

    let mut orchard_receiver_raw: Option<[u8; 43]> = None;
    for item in ua.items() {
        if let Receiver::Orchard(data) = item {
            orchard_receiver_raw = Some(data);
            break;
        }
    }

    let orchard_receiver_raw = orchard_receiver_raw.ok_or_else(|| {
        JsError::new("Recipient address does not contain an Orchard receiver")
    })?;

    let recipient_orchard_address = orchard::Address::from_raw_address_bytes(&orchard_receiver_raw)
        .into_option()
        .ok_or_else(|| JsError::new("Invalid Orchard receiver bytes"))?;

    let spend_orchard_receiver_raw: [u8; 43] = spend_note
        .orchard_address_raw
        .as_slice()
        .try_into()
        .map_err(|_| JsError::new("Invalid spend note orchard receiver raw bytes length"))?;

    let note_recipient = orchard::Address::from_raw_address_bytes(&spend_orchard_receiver_raw)
        .into_option()
        .ok_or_else(|| JsError::new("Invalid spend note recipient bytes"))?;

    let rho = Rho::from_bytes(&spend_note.rho)
        .into_option()
        .ok_or_else(|| JsError::new("Invalid spend note rho bytes"))?;

    let rseed = RandomSeed::from_bytes(spend_note.rseed, &rho)
        .into_option()
        .ok_or_else(|| JsError::new("Invalid spend note rseed bytes"))?;

    let note_value = NoteValue::from_raw(spend_note.value);
    let note = Note::from_parts(note_recipient, note_value, rho, rseed)
        .into_option()
        .ok_or_else(|| JsError::new("Invalid spend note reconstruction"))?;

    let mnemonic: Mnemonic = mnemonic_str
        .parse()
        .map_err(|e| JsError::new(&format!("Invalid mnemonic: {e}")))?;
    let seed_bytes = mnemonic.to_seed("").to_vec();

    let account_id = AccountId::try_from(0).map_err(|e| {
        JsError::new(&format!("Invalid ZIP32 account id: {e}"))
    })?;
    let spending_key = SpendingKey::from_zip32_seed(&seed_bytes, 133, account_id).map_err(|e| {
        JsError::new(&format!("Failed to derive Orchard spending key: {e:?}"))
    })?;
    let fvk = FullViewingKey::from(&spending_key);

    // 512-byte memo.
    let memo_bytes = {
        let mut out = [0u8; 512];
        let m = memo.as_bytes();
        let len = m.len().min(512);
        out[..len].copy_from_slice(&m[..len]);
        out
    };

    let change_amount = spend_note.value.saturating_sub(amount_zatoshis);

    let bundle_type = BundleType::Transactional {
        flags: Flags::ENABLED,
        bundle_required: true,
    };
    let mut builder = OrchardBuilder::new(bundle_type, anchor);

    builder
        .add_spend(fvk, note, merkle_path)
        .map_err(|e| JsError::new(&format!("Failed to add spend: {e}")))?;

    builder
        .add_output(
            None,
            recipient_orchard_address,
            NoteValue::from_raw(amount_zatoshis),
            memo_bytes,
        )
        .map_err(|e| JsError::new(&format!("Failed to add output: {e}")))?;

    if change_amount > 0 {
        builder
            .add_output(
                None,
                note_recipient,
                NoteValue::from_raw(change_amount),
                [0u8; 512],
            )
            .map_err(|e| JsError::new(&format!("Failed to add change output: {e}")))?;
    }

    let mut rng = OsRng;
    let bundle_result = builder
        .build::<i64>(&mut rng)
        .map_err(|e| JsError::new(&format!("Failed to build Orchard bundle: {e}")))?;
    let (unauthorized_bundle, _bundle_meta) = bundle_result.ok_or_else(|| {
        JsError::new("Orchard builder did not produce a bundle")
    })?;

    let pk = ProvingKey::build();
    let proved = unauthorized_bundle
        .create_proof(&pk, &mut rng)
        .map_err(|e| JsError::new(&format!("Failed to create Orchard proof: {e}")))?;
    let prepared = proved.prepare(&mut rng, [0; 32]);
    let authorized = prepared.finalize().map_err(|e| {
        JsError::new(&format!("Failed to finalize Orchard bundle signatures: {e}"))
    })?;

    let bundle_actions = authorized.actions().len();

    let now_ms = js_sys::Date::now() as u64;
    let mut hasher = sha2::Sha256::new();
    hasher.update(recipient_address.as_bytes());
    hasher.update(&amount_zatoshis.to_le_bytes());
    hasher.update(memo.as_bytes());
    hasher.update(&now_ms.to_le_bytes());
    let txid = hex::encode(hasher.finalize());

    let result = ProveOrchardTxResult {
        txid,
        raw_tx_hex: String::new(),
        bundle_actions,
        proof_generated: true,
    };

    serde_wasm_bindgen::to_value(&result).map_err(|e| {
        JsError::new(&format!("Serialization failed: {e}"))
    })
}

#[derive(Serialize, Deserialize)]
pub struct BuiltOrchardTxResult {
    pub txid: String,
    #[serde(rename = "rawTxHex")]
    pub raw_tx_hex: String,
    pub bundle_actions: usize,
    pub proof_generated: bool,
}


#[wasm_bindgen]
pub fn build_orchard_v5_tx_from_note(
    mnemonic_str: &str,
    recipient_address: &str,
    amount_zatoshis: u64,
    fee_zatoshis: u64,
    memo: &str,
    spend_note_json: &str,
    witness_json: &str,
) -> Result<JsValue, JsError> {
    use bip39::Mnemonic;
    use core2::io::Cursor;
    use nozy::hd_wallet::OrchardDecryptionResult;
    use orchard::{
        keys::SpendAuthorizingKey,
        note::{Note, RandomSeed, Rho},
        value::NoteValue,
        Address as OrchardAddress,
    };
    use rand::rngs::OsRng;
    use transparent::builder::TransparentSigningSet;
    use zcash_address::unified::{Address as UnifiedAddress, Container, Encoding, Receiver};
    use zcash_primitives::transaction::builder::{BuildConfig, Builder as TxBuilder};
    use zcash_primitives::transaction::fees::{FeeRule, transparent::InputSize};
    use zcash_protocol::{
        consensus::{MainNetwork, BlockHeight},
        memo::MemoBytes,
        value::Zatoshis,
    };

    // Dummy Sapling provers. Since this builder config omits Sapling entirely,
    // these are never used at runtime, but are required by the generic builder API.
    struct DummySaplingSpendProver;
    struct DummySaplingOutputProver;

    struct FixedFeeRule {
        fee: Zatoshis,
    }

    impl FeeRule for FixedFeeRule {
        type Error = core::convert::Infallible;

        fn fee_required<P: zcash_protocol::consensus::Parameters>(
            &self,
            _params: &P,
            _target_height: BlockHeight,
            _transparent_input_sizes: impl IntoIterator<Item = InputSize>,
            _transparent_output_sizes: impl IntoIterator<Item = usize>,
            _sapling_input_count: usize,
            _sapling_output_count: usize,
            _orchard_action_count: usize,
        ) -> Result<Zatoshis, Self::Error> {
            Ok(self.fee)
        }
    }

    impl sapling::prover::SpendProver for DummySaplingSpendProver {
        type Proof = sapling::bundle::GrothProofBytes;

        fn prepare_circuit(
            _proof_generation_key: sapling::ProofGenerationKey,
            _diversifier: sapling::Diversifier,
            _rseed: sapling::Rseed,
            _value: sapling::value::NoteValue,
            _alpha: jubjub::Fr,
            _rcv: sapling::value::ValueCommitTrapdoor,
            _anchor: bls12_381::Scalar,
            _merkle_path: sapling::MerklePath,
        ) -> Option<sapling::circuit::Spend> {
            None
        }

        fn create_proof<R: rand::RngCore>(
            &self,
            _circuit: sapling::circuit::Spend,
            _rng: &mut R,
        ) -> Self::Proof {
            [0u8; 192]
        }

        fn encode_proof(proof: Self::Proof) -> sapling::bundle::GrothProofBytes {
            proof
        }
    }

    impl sapling::prover::OutputProver for DummySaplingOutputProver {
        type Proof = sapling::bundle::GrothProofBytes;

        fn prepare_circuit(
            _esk: &sapling::keys::EphemeralSecretKey,
            _payment_address: sapling::PaymentAddress,
            _rcm: jubjub::Fr,
            _value: sapling::value::NoteValue,
            _rcv: sapling::value::ValueCommitTrapdoor,
        ) -> sapling::circuit::Output {
            unreachable!("DummySaplingOutputProver should not be called");
        }

        fn create_proof<R: rand::RngCore>(
            &self,
            _circuit: sapling::circuit::Output,
            _rng: &mut R,
        ) -> Self::Proof {
            [0u8; 192]
        }

        fn encode_proof(proof: Self::Proof) -> sapling::bundle::GrothProofBytes {
            proof
        }
    }

    let witnesses: Vec<serde_json::Value> = match serde_json::from_str::<serde_json::Value>(witness_json)
        .map_err(|e| JsError::new(&format!("Invalid witness_json: {e}")))? {
        serde_json::Value::Array(a) => a,
        other => vec![other],
    };

    let spend_notes: Vec<OrchardDecryptionResult> = match serde_json::from_str::<serde_json::Value>(spend_note_json)
        .map_err(|e| JsError::new(&format!("Invalid spend_note_json: {e}")))? {
        serde_json::Value::Array(_) => serde_json::from_str(spend_note_json)
            .map_err(|e| JsError::new(&format!("Invalid spend_note_json array: {e}")))?,
        _ => vec![serde_json::from_str(spend_note_json)
            .map_err(|e| JsError::new(&format!("Invalid spend_note_json object: {e}")))?],
    };

    if spend_notes.is_empty() {
        return Err(JsError::new("No spend notes provided"));
    }
    if witnesses.is_empty() {
        return Err(JsError::new("No witnesses provided"));
    }
    if spend_notes.len() != witnesses.len() {
        return Err(JsError::new("Spend notes and witnesses length mismatch"));
    }

    let target_height_u32 = witnesses[0]
        .get("target_height")
        .and_then(|v| v.as_u64())
        .or_else(|| witnesses[0].get("anchor_height").and_then(|v| v.as_u64()))
        .ok_or_else(|| {
            JsError::new("witness[0] must include target_height or anchor_height (chain height for tx expiry)")
        })? as u32;
    let target_height_bh: BlockHeight = BlockHeight::from(target_height_u32);

    let (_network, ua) = UnifiedAddress::decode(recipient_address).map_err(|e| {
        JsError::new(&format!("Invalid recipient address: {e}"))
    })?;

    let mut orchard_receiver_raw: Option<[u8; 43]> = None;
    for item in ua.items() {
        if let Receiver::Orchard(data) = item {
            orchard_receiver_raw = Some(data);
            break;
        }
    }

    let orchard_receiver_raw = orchard_receiver_raw.ok_or_else(|| {
        JsError::new("Recipient address does not contain an Orchard receiver")
    })?;

    let recipient_orchard_address = OrchardAddress::from_raw_address_bytes(&orchard_receiver_raw)
        .into_option()
        .ok_or_else(|| JsError::new("Invalid Orchard receiver bytes"))?;

    let first_spend_note = &spend_notes[0];
    let spend_orchard_receiver_raw: [u8; 43] = first_spend_note
        .orchard_address_raw
        .as_slice()
        .try_into()
        .map_err(|_| JsError::new("Invalid spend note orchard receiver raw bytes length"))?;

    let note_recipient = OrchardAddress::from_raw_address_bytes(&spend_orchard_receiver_raw)
        .into_option()
        .ok_or_else(|| JsError::new("Invalid spend note recipient bytes"))?;

    let first_witness_str = serde_json::to_string(&witnesses[0])
        .map_err(|e| JsError::new(&format!("witness json: {e}")))?;
    let (anchor, _) = parse_orchard_witness_for_spend(&first_witness_str)?;
    let anchor_bytes: [u8; 32] = anchor.to_bytes();

    let mnemonic: Mnemonic = mnemonic_str
        .parse()
        .map_err(|e| JsError::new(&format!("Invalid mnemonic: {e}")))?;

    let seed_bytes = mnemonic.to_seed("").to_vec();
    let account_id = zcash_primitives::zip32::AccountId::try_from(0).map_err(|e| {
        JsError::new(&format!("Invalid ZIP32 account id: {e}"))
    })?;

    let spending_key = orchard::keys::SpendingKey::from_zip32_seed(&seed_bytes, 133, account_id)
        .map_err(|e| JsError::new(&format!("Failed to derive Orchard spending key: {e:?}")))?;
    let fvk = orchard::keys::FullViewingKey::from(&spending_key);

    let spend_auth_key: SpendAuthorizingKey = SpendAuthorizingKey::from(&spending_key);
    let orchard_saks = vec![spend_auth_key];

    let build_config = BuildConfig::Standard {
        sapling_anchor: None,
        orchard_anchor: Some(anchor),
    };

    let tx_builder = TxBuilder::new(MainNetwork, target_height_bh, build_config);
    let transparent_signing_set = TransparentSigningSet::new();
    let sapling_extsks: &[sapling::zip32::ExtendedSpendingKey] = &[];

    let fee = Zatoshis::from_u64(fee_zatoshis)
        .map_err(|_| JsError::new("Invalid fee amount"))?;
    let fee_rule = FixedFeeRule { fee };

    // Add Orchard spends + outputs.
    let mut builder = tx_builder;
    let mut total_input_value: u64 = 0;
    for (idx, (spend_note, witness_val)) in spend_notes.iter().zip(witnesses.iter()).enumerate() {
        let witness_str = serde_json::to_string(witness_val)
            .map_err(|e| JsError::new(&format!("witness json at {idx}: {e}")))?;
        let (w_anchor, merkle_path) = parse_orchard_witness_for_spend(&witness_str)?;
        if w_anchor.to_bytes() != anchor_bytes {
            return Err(JsError::new("All spend witnesses must share the same Orchard anchor"));
        }

        let spend_orchard_receiver_raw: [u8; 43] = spend_note
            .orchard_address_raw
            .as_slice()
            .try_into()
            .map_err(|_| JsError::new(&format!("Invalid spend note receiver bytes at index {idx}")))?;
        let spend_note_recipient = OrchardAddress::from_raw_address_bytes(&spend_orchard_receiver_raw)
            .into_option()
            .ok_or_else(|| JsError::new(&format!("Invalid spend note recipient at index {idx}")))?;

        let rho = Rho::from_bytes(&spend_note.rho)
            .into_option()
            .ok_or_else(|| JsError::new(&format!("Invalid spend note rho bytes at index {idx}")))?;
        let rseed = RandomSeed::from_bytes(spend_note.rseed, &rho)
            .into_option()
            .ok_or_else(|| JsError::new(&format!("Invalid spend note rseed bytes at index {idx}")))?;
        let note_value = NoteValue::from_raw(spend_note.value);
        let note = Note::from_parts(spend_note_recipient, note_value, rho, rseed)
            .into_option()
            .ok_or_else(|| JsError::new(&format!("Invalid spend note reconstruction at index {idx}")))?;

        builder
            .add_orchard_spend::<core::convert::Infallible>(
                fvk.clone(),
                note,
                merkle_path,
            )
            .map_err(|e| JsError::new(&format!("Failed to add Orchard spend at index {idx}: {e:?}")))?;

        total_input_value = total_input_value.saturating_add(spend_note.value);
    }

    let recipient_value = amount_zatoshis;
    let total_required = recipient_value.saturating_add(fee_zatoshis);
    if total_input_value < total_required {
        return Err(JsError::new(&format!(
            "Insufficient input value for amount+fee: inputs={} required={}",
            total_input_value, total_required
        )));
    }
    let memo_bytes = MemoBytes::from_bytes(memo.as_bytes()).map_err(|e| {
        JsError::new(&format!("Invalid memo: {e}"))
    })?;

    builder
        .add_orchard_output::<core::convert::Infallible>(
            None,
            recipient_orchard_address,
            recipient_value,
            memo_bytes,
        )
        .map_err(|e| JsError::new(&format!("Failed to add Orchard output: {e:?}")))?;

    let change_amount = total_input_value.saturating_sub(total_required);
    if change_amount > 0 {
        builder
            .add_orchard_output::<core::convert::Infallible>(
                None,
                note_recipient,
                change_amount,
                MemoBytes::empty(),
            )
            .map_err(|e| {
                JsError::new(&format!("Failed to add change Orchard output: {e:?}"))
            })?;
    }

    let rng = OsRng;
    let spend_prover = DummySaplingSpendProver;
    let output_prover = DummySaplingOutputProver;

    let build_result = builder
        .build(
            &transparent_signing_set,
            sapling_extsks,
            &orchard_saks,
            rng,
            &spend_prover,
            &output_prover,
            &fee_rule,
        )
        .map_err(|e| JsError::new(&format!("Transaction build failed: {e:?}")))?;

    let tx = build_result.transaction();
    let txid = tx.txid().to_string();

    // Serialize to raw tx bytes for broadcasting later.
    let mut cursor = Cursor::new(Vec::<u8>::new());
    tx.write(&mut cursor).map_err(|e| JsError::new(&format!("Tx serialization failed: {e}")))?;
    let raw_tx_bytes = cursor.into_inner();
    let raw_tx_hex = hex::encode(raw_tx_bytes);

    let result = BuiltOrchardTxResult {
        txid,
        raw_tx_hex,
        bundle_actions: 0,
        proof_generated: true,
    };

    serde_wasm_bindgen::to_value(&result).map_err(|e| JsError::new(&format!("Serialization failed: {e}")))
}

#[wasm_bindgen]
pub fn encrypt_for_storage(data: &[u8], password: &str) -> Result<Vec<u8>, JsError> {
    encrypt_data(data, password)
}

#[wasm_bindgen]
pub fn decrypt_from_storage(encrypted: &[u8], password: &str) -> Result<Vec<u8>, JsError> {
    decrypt_data(encrypted, password)
}

fn encrypt_data(data: &[u8], password: &str) -> Result<Vec<u8>, JsError> {
    use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
    use aes_gcm::aead::Aead;
    use argon2::Argon2;
    use rand::RngCore;

    let mut salt = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut salt);

    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(password.as_bytes(), &salt, &mut key)
        .map_err(|e| JsError::new(&format!("Key derivation failed: {}", e)))?;

    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| JsError::new(&format!("Cipher init failed: {}", e)))?;

    let mut nonce_bytes = [0u8; 12];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, data)
        .map_err(|e| JsError::new(&format!("Encryption failed: {}", e)))?;

    // Format: [16 bytes salt][12 bytes nonce][ciphertext]
    let mut result = Vec::with_capacity(16 + 12 + ciphertext.len());
    result.extend_from_slice(&salt);
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

fn decrypt_data(encrypted: &[u8], password: &str) -> Result<Vec<u8>, JsError> {
    use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
    use aes_gcm::aead::Aead;
    use argon2::Argon2;

    if encrypted.len() < 28 {
        return Err(JsError::new("Encrypted data too short"));
    }

    let salt = &encrypted[..16];
    let nonce_bytes = &encrypted[16..28];
    let ciphertext = &encrypted[28..];

    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| JsError::new(&format!("Key derivation failed: {}", e)))?;

    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| JsError::new(&format!("Cipher init failed: {}", e)))?;

    let nonce = Nonce::from_slice(nonce_bytes);

    cipher.decrypt(nonce, ciphertext)
        .map_err(|_| JsError::new("Decryption failed: wrong password or corrupted data"))
}

#[cfg(test)]
mod multi_input_parsing_tests {
    use super::*;
    use nozy::hd_wallet::OrchardDecryptionResult;

    #[allow(dead_code)]
    #[derive(Debug, Deserialize)]
    struct OrchardWitnessInputTest {
        anchor: String,
        position: u32,
        auth_path: Vec<String>,
        target_height: u32,
    }

    fn parse_payloads(
        spend_note_json: &str,
        witness_json: &str,
    ) -> Result<(Vec<OrchardDecryptionResult>, Vec<OrchardWitnessInputTest>), String> {
        let witnesses: Vec<OrchardWitnessInputTest> = match serde_json::from_str::<serde_json::Value>(witness_json)
            .map_err(|e| format!("Invalid witness_json: {e}"))? {
            serde_json::Value::Array(_) => serde_json::from_str(witness_json)
                .map_err(|e| format!("Invalid witness_json array: {e}"))?,
            _ => vec![serde_json::from_str(witness_json)
                .map_err(|e| format!("Invalid witness_json object: {e}"))?],
        };

        let notes: Vec<OrchardDecryptionResult> = match serde_json::from_str::<serde_json::Value>(spend_note_json)
            .map_err(|e| format!("Invalid spend_note_json: {e}"))? {
            serde_json::Value::Array(_) => serde_json::from_str(spend_note_json)
                .map_err(|e| format!("Invalid spend_note_json array: {e}"))?,
            _ => vec![serde_json::from_str(spend_note_json)
                .map_err(|e| format!("Invalid spend_note_json object: {e}"))?],
        };

        if notes.len() != witnesses.len() {
            return Err("Spend notes and witnesses length mismatch".to_string());
        }
        Ok((notes, witnesses))
    }

    fn sample_note_json() -> String {
        serde_json::json!({
            "value": 12000u64,
            "address": "u1sample",
            "cmx": vec![0; 32],
            "rho": vec![0; 32],
            "rseed": vec![0; 32],
            "orchard_address_raw": vec![0; 43],
            "nullifier": vec![0; 32],
            "block_height": 1,
            "txid": "abc"
        })
        .to_string()
    }

    fn sample_witness_json() -> String {
        serde_json::json!({
            "anchor": format!("{:064x}", 0),
            "position": 0,
            "auth_path": vec![format!("{:064x}", 0); 32],
            "target_height": 1
        })
        .to_string()
    }

    #[test]
    fn parses_single_object_payloads() {
        let parsed = parse_payloads(&sample_note_json(), &sample_witness_json()).unwrap();
        assert_eq!(parsed.0.len(), 1);
        assert_eq!(parsed.1.len(), 1);
    }

    #[test]
    fn parses_array_payloads() {
        let notes = format!("[{},{}]", sample_note_json(), sample_note_json());
        let witnesses = format!("[{},{}]", sample_witness_json(), sample_witness_json());
        let parsed = parse_payloads(&notes, &witnesses).unwrap();
        assert_eq!(parsed.0.len(), 2);
        assert_eq!(parsed.1.len(), 2);
    }

    #[test]
    fn rejects_length_mismatch() {
        let notes = format!("[{},{}]", sample_note_json(), sample_note_json());
        let err = parse_payloads(&notes, &sample_witness_json()).unwrap_err();
        assert!(err.contains("length mismatch"));
    }
}
