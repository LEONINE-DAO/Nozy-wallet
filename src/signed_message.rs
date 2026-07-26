//! Domain-separated Orchard SpendAuth message signatures (`nozy-sm-v1`).
//!
//! This is **not** a ZIP-standardized message-signing scheme. It uses the wallet's
//! Orchard `SpendAuthorizingKey` (ZIP-32 coin type 133, account 0) to produce a
//! RedDSA SpendAuth signature over `SHA256("NozyWallet_SignedMessage_v1" || 0x00 || msg)`.
//! Encoding: `nozy-sm-v1:<ak_hex32>:<sig_hex64>`.

use crate::error::{NozyError, NozyResult};
use group::ff::Field;
use orchard::keys::{SpendAuthorizingKey, SpendingKey};
use orchard::primitives::redpallas::{self, SpendAuth};
use pasta_curves::pallas;
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};
use zip32::AccountId;

pub const SCHEME_ID: &str = "nozy-sm-v1";
const DOMAIN: &[u8] = b"NozyWallet_SignedMessage_v1";

fn digest_message(message: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(DOMAIN);
    hasher.update([0u8]);
    hasher.update(message.as_bytes());
    hasher.finalize().into()
}

fn spending_key_from_seed(seed_bytes: &[u8]) -> NozyResult<SpendingKey> {
    let account_id = AccountId::try_from(0u32)
        .map_err(|e| NozyError::KeyDerivation(format!("Invalid ZIP32 account id: {e}")))?;
    SpendingKey::from_zip32_seed(seed_bytes, 133, account_id).map_err(|e| {
        NozyError::KeyDerivation(format!("Failed to derive Orchard spending key: {e:?}"))
    })
}

/// Sign `message` with the Orchard SpendAuth key for account 0.
///
/// `seed_bytes` is the BIP-39 seed (`mnemonic.to_seed("")`).
pub fn sign_with_seed(seed_bytes: &[u8], message: &str) -> NozyResult<String> {
    let spending_key = spending_key_from_seed(seed_bytes)?;
    let ask = SpendAuthorizingKey::from(&spending_key);
    let msg = digest_message(message);
    let signing_key = ask.randomize(&pallas::Scalar::ZERO);
    let verifying_key = redpallas::VerificationKey::<SpendAuth>::from(&signing_key);
    let ak_bytes: [u8; 32] = (&verifying_key).into();
    let signature = signing_key.sign(&mut OsRng, &msg);
    let sig_bytes: [u8; 64] = (&signature).into();
    Ok(format!(
        "{}:{}:{}",
        SCHEME_ID,
        hex::encode(ak_bytes),
        hex::encode(sig_bytes)
    ))
}

/// Verify a `sign_with_seed` output.
pub fn verify(signature: &str, message: &str) -> NozyResult<bool> {
    let parts: Vec<&str> = signature.split(':').collect();
    if parts.len() != 3 || parts[0] != SCHEME_ID {
        return Ok(false);
    }
    let ak_vec = hex::decode(parts[1])
        .map_err(|e| NozyError::KeyDerivation(format!("Invalid ak hex: {e}")))?;
    let sig_vec = hex::decode(parts[2])
        .map_err(|e| NozyError::KeyDerivation(format!("Invalid sig hex: {e}")))?;
    if ak_vec.len() != 32 || sig_vec.len() != 64 {
        return Ok(false);
    }
    let mut ak = [0u8; 32];
    let mut sig_bytes = [0u8; 64];
    ak.copy_from_slice(&ak_vec);
    sig_bytes.copy_from_slice(&sig_vec);

    let msg = digest_message(message);
    let vk = match redpallas::VerificationKey::<SpendAuth>::try_from(ak) {
        Ok(v) => v,
        Err(_) => return Ok(false),
    };
    let sig = redpallas::Signature::<SpendAuth>::from(sig_bytes);
    Ok(vk.verify(&msg, &sig).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bip39::{Language, Mnemonic};

    #[test]
    fn sign_and_verify_roundtrip() {
        let entropy = [7u8; 16];
        let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy).expect("mnemonic");
        let seed = mnemonic.to_seed("");
        let msg = "hello nozy dapp";
        let sig = sign_with_seed(&seed, msg).expect("sign");
        assert!(sig.starts_with("nozy-sm-v1:"));
        assert!(verify(&sig, msg).expect("verify"));
        assert!(!verify(&sig, "tampered").expect("verify tampered"));
    }
}
