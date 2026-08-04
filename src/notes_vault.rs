//! Encrypted Orchard note cache (`notes.json`) â€” finding F-02.
//!
//! Format on disk (hex): `NZN1 || nonce(12) || AES-256-GCM(ciphertext)`
//! Key: Argon2id(password, persistent `notes.salt`) cached for the unlock session.
//! Plaintext JSON files still load; next save upgrades to NZN1.

use crate::error::{NozyError, NozyResult};
use crate::paths::get_wallet_data_dir;
use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};
use rand::rngs::OsRng;
use rand::RngCore;
use std::sync::Mutex;
use zeroize::Zeroize;

const NOTES_MAGIC: &[u8; 4] = b"NZN1";
/// Ironwood migration schedule on-disk magic (F-13 residual).
const SCHEDULE_MAGIC: &[u8; 4] = b"NZS1";
const SALT_FILE: &str = "notes.salt";

static NOTES_AES_KEY: Mutex<Option<[u8; 32]>> = Mutex::new(None);

/// Unlock / refresh the in-memory notes AES key from the wallet password.
pub fn unlock_notes_vault(password: &str) -> NozyResult<()> {
    let salt = load_or_create_notes_salt()?;
    let key = derive_notes_key(password, &salt)?;
    if let Ok(mut guard) = NOTES_AES_KEY.lock() {
        if let Some(old) = guard.as_mut() {
            old.zeroize();
        }
        *guard = Some(key);
    }
    Ok(())
}

/// Clear the cached notes key (wallet lock).
pub fn clear_notes_vault() {
    if let Ok(mut guard) = NOTES_AES_KEY.lock() {
        if let Some(key) = guard.as_mut() {
            key.zeroize();
        }
        *guard = None;
    }
}

fn notes_key() -> NozyResult<[u8; 32]> {
    let guard = NOTES_AES_KEY
        .lock()
        .map_err(|_| NozyError::Storage("notes vault lock poisoned".into()))?;
    guard.as_ref().copied().ok_or_else(|| {
        NozyError::Storage(
            "Notes cache is locked. Unlock the wallet (or re-enter password) before sync/save."
                .into(),
        )
    })
}

fn derive_notes_key(password: &str, salt: &[u8]) -> NozyResult<[u8; 32]> {
    let params = Params::new(19 * 1024, 2, 1, Some(32))
        .map_err(|e| NozyError::Storage(format!("Invalid Argon2 params: {e}")))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| NozyError::Storage(format!("Argon2id notes KDF failed: {e}")))?;
    Ok(key)
}

fn load_or_create_notes_salt() -> NozyResult<[u8; 16]> {
    let path = get_wallet_data_dir().join(SALT_FILE);
    if path.exists() {
        let bytes = std::fs::read(&path)
            .map_err(|e| NozyError::Storage(format!("Failed to read notes.salt: {e}")))?;
        if bytes.len() != 16 {
            return Err(NozyError::Storage(
                "notes.salt has unexpected length".into(),
            ));
        }
        let mut salt = [0u8; 16];
        salt.copy_from_slice(&bytes);
        return Ok(salt);
    }
    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(&path, salt)
        .map_err(|e| NozyError::Storage(format!("Failed to write notes.salt: {e}")))?;
    Ok(salt)
}

fn session_notes_key() -> NozyResult<[u8; 32]> {
    match notes_key() {
        Ok(k) => Ok(k),
        Err(_) => {
            // Best-effort: empty-password unlock for unprotected wallets.
            unlock_notes_vault("")?;
            notes_key()
        }
    }
}

fn encrypt_json_with_magic(plaintext_json: &str, magic: &[u8; 4]) -> NozyResult<String> {
    let key = session_notes_key()?;
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| NozyError::Storage(format!("Failed to create notes cipher: {e}")))?;
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), plaintext_json.as_bytes())
        .map_err(|e| NozyError::Storage(format!("Vault encryption failed: {e}")))?;
    let mut out = Vec::with_capacity(4 + 12 + ciphertext.len());
    out.extend_from_slice(magic);
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ciphertext);
    Ok(hex::encode(out))
}

fn decrypt_file_content_with_magic(
    content: &str,
    magic: &[u8; 4],
    file_label: &str,
) -> NozyResult<String> {
    let trimmed = content.trim();
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        return Ok(trimmed.to_string());
    }

    let magic_label = std::str::from_utf8(magic).unwrap_or("????");
    let raw = hex::decode(trimmed).map_err(|e| {
        NozyError::Storage(format!(
            "{file_label} is neither JSON nor {magic_label} hex: {e}"
        ))
    })?;
    if raw.len() < 4 + 12 + 16 || !raw.starts_with(magic) {
        return Err(NozyError::Storage(format!(
            "{file_label} encrypted blob has invalid {magic_label} header"
        )));
    }
    let nonce = &raw[4..16];
    let ciphertext = &raw[16..];

    let try_decrypt = |key: &[u8; 32]| -> NozyResult<String> {
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| NozyError::Storage(format!("Failed to create notes cipher: {e}")))?;
        let plain = cipher
            .decrypt(Nonce::from_slice(nonce), ciphertext)
            .map_err(|_| {
                NozyError::Storage(format!(
                    "Failed to decrypt {file_label}: wrong password or corrupted file. Unlock wallet and retry."
                ))
            })?;
        String::from_utf8(plain)
            .map_err(|e| NozyError::Storage(format!("Invalid UTF-8 in {file_label}: {e}")))
    };

    if let Ok(key) = notes_key() {
        return try_decrypt(&key);
    }
    unlock_notes_vault("")?;
    try_decrypt(&notes_key()?)
}

/// Encrypt NoteIndex JSON for disk (uses session key).
pub fn encrypt_notes_json(plaintext_json: &str) -> NozyResult<String> {
    encrypt_json_with_magic(plaintext_json, NOTES_MAGIC)
}

/// Decode file content: NZN1 hex blob or legacy plaintext JSON.
pub fn decrypt_notes_file_content(content: &str) -> NozyResult<String> {
    decrypt_file_content_with_magic(content, NOTES_MAGIC, "notes.json")
}

/// Encrypt Ironwood migration schedule JSON (NZS1, same session key as notes).
pub fn encrypt_schedule_json(plaintext_json: &str) -> NozyResult<String> {
    encrypt_json_with_magic(plaintext_json, SCHEDULE_MAGIC)
}

/// Decode schedule file: NZS1 hex blob or legacy plaintext JSON.
pub fn decrypt_schedule_file_content(content: &str) -> NozyResult<String> {
    decrypt_file_content_with_magic(content, SCHEDULE_MAGIC, "ironwood migration schedule")
}

#[cfg(test)]
static NOTES_VAULT_TEST_LOCK: Mutex<()> = Mutex::new(());

/// Serialize tests that touch the global notes AES session key / salt dir.
#[cfg(test)]
pub(crate) fn lock_notes_vault_for_test() -> std::sync::MutexGuard<'static, ()> {
    NOTES_VAULT_TEST_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::with_wallet_data_dir;

    #[test]
    fn notes_vault_roundtrip() {
        let _g = super::lock_notes_vault_for_test();
        let dir =
            std::env::temp_dir().join(format!("nozy-notes-vault-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        with_wallet_data_dir(&dir, || {
            clear_notes_vault();
            unlock_notes_vault("secret").unwrap();
            let enc = encrypt_notes_json(r#"{"version":2,"notes":[]}"#).unwrap();
            clear_notes_vault();
            unlock_notes_vault("secret").unwrap();
            let plain = decrypt_notes_file_content(&enc).unwrap();
            assert!(plain.contains("version"));
            clear_notes_vault();
            unlock_notes_vault("wrong").unwrap();
            assert!(decrypt_notes_file_content(&enc).is_err());
            clear_notes_vault();
        });
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn plaintext_json_still_loads() {
        let plain = decrypt_notes_file_content("{\"version\":2,\"notes\":[]}").unwrap();
        assert!(plain.starts_with('{'));
    }

    #[test]
    fn schedule_vault_roundtrip() {
        let _g = super::lock_notes_vault_for_test();
        let dir =
            std::env::temp_dir().join(format!("nozy-schedule-vault-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        with_wallet_data_dir(&dir, || {
            clear_notes_vault();
            unlock_notes_vault("secret").unwrap();
            let sample = r#"{"version":1,"transfers":[]}"#;
            let enc = encrypt_schedule_json(sample).unwrap();
            assert!(enc.starts_with("4e5a5331") || !enc.starts_with('{'));
            clear_notes_vault();
            unlock_notes_vault("secret").unwrap();
            let plain = decrypt_schedule_file_content(&enc).unwrap();
            assert!(plain.contains("transfers"));
            clear_notes_vault();
        });
        let _ = std::fs::remove_dir_all(&dir);
    }
}
