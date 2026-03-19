use wasm_bindgen::prelude::*;

/// Test 1: BIP39 mnemonic generation (key derivation)
#[wasm_bindgen]
pub fn test_key_derivation() -> String {
    use bip39::Mnemonic;
    use rand::RngCore;

    let mut entropy = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut entropy);
    let mnemonic = Mnemonic::from_entropy_in(bip39::Language::English, &entropy)
        .expect("mnemonic generation failed");
    mnemonic.to_string()
}

/// Test 2: AES-256-GCM encryption round-trip
/// Pure Rust crypto, should compile.
#[wasm_bindgen]
pub fn test_encryption() -> bool {
    use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
    use aes_gcm::aead::Aead;
    use rand::RngCore;

    let mut key_bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut key_bytes);
    let cipher = Aes256Gcm::new_from_slice(&key_bytes).expect("cipher init failed");

    let mut nonce_bytes = [0u8; 12];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let plaintext = b"nozy wallet test";
    let ciphertext = cipher.encrypt(nonce, plaintext.as_ref()).expect("encrypt failed");
    let decrypted = cipher.decrypt(nonce, ciphertext.as_ref()).expect("decrypt failed");

    decrypted == plaintext
}

/// Test 3: Orchard address generation from seed
/// This is the critical test - it pulls in the full orchard + halo2_proofs stack.
#[wasm_bindgen]
pub fn test_orchard_address() -> String {
    use orchard::keys::{SpendingKey, FullViewingKey, Scope};

    let seed = [42u8; 32];
    let sk = SpendingKey::from_bytes(seed).expect("spending key failed");
    let fvk = FullViewingKey::from(&sk);
    let address = fvk.address_at(0u64, Scope::External);

    format!("orchard address generated: {:?}", address)
}

/// Test 4: Zcash protocol types
#[wasm_bindgen]
pub fn test_zcash_protocol() -> String {
    use zcash_protocol::consensus::{MainNetwork, Parameters};

    let activation = MainNetwork.activation_height(zcash_protocol::consensus::NetworkUpgrade::Nu5);
    format!("NU5 activation: {:?}", activation)
}
