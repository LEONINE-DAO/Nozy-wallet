use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

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
