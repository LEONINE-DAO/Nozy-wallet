use crate::error::{NozyError, NozyResult};
use crate::hd_wallet::HDWallet;
use crate::transactions::TransactionDetails;
use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Legacy blob: salt(16) || nonce(12) || ciphertext (iterated SHA-256 KDF).
const LEGACY_HEADER_LEN: usize = 28;
/// Versioned Argon2id blob: magic(4) || salt(16) || nonce(12) || ciphertext.
const VAULT_MAGIC_V2: &[u8; 4] = b"NZK2";
const V2_HEADER_LEN: usize = 4 + 16 + 12;
/// Pre-security-fix v1.0 vault: raw AES key(32) || nonce(12) || ciphertext(+tag).
/// Password was ignored at encrypt time; key bytes lived in the hex blob.
const V1_EMBEDDED_KEY_MIN_LEN: usize = 32 + 12 + 16;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletData {
    pub mnemonic: String,
    pub addresses: Vec<String>,
    pub transactions: Vec<TransactionDetails>,
    pub balance: u64,
    #[serde(default)]
    pub created_at: u64,
    #[serde(default)]
    pub last_updated: u64,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub password_protected: bool,
    #[serde(default)]
    pub password_hash: Option<String>,
}

fn default_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

impl WalletData {
    pub fn new(mnemonic: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            mnemonic,
            addresses: Vec::new(),
            transactions: Vec::new(),
            balance: 0,
            created_at: now,
            last_updated: now,
            version: env!("CARGO_PKG_VERSION").to_string(),
            password_protected: false,
            password_hash: None,
        }
    }

    pub fn ensure_timestamps(&mut self) {
        if self.created_at == 0 {
            self.created_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
        }
        if self.last_updated == 0 {
            self.last_updated = self.created_at;
        }
    }

    pub fn update_timestamp(&mut self) {
        self.last_updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }
}

pub struct WalletStorage {
    data_dir: PathBuf,
}

impl WalletStorage {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    pub fn with_xdg_dir() -> Self {
        use crate::paths::{get_wallet_base_dir, get_wallet_data_dir};
        let base_dir = get_wallet_base_dir();
        Self::migrate_from_insecure_location(&base_dir);
        let _ = crate::wallet_profiles::ensure_profiles_initialized();
        let secure_dir = get_wallet_data_dir();
        Self::new(secure_dir)
    }

    fn migrate_from_insecure_location(secure_dir: &PathBuf) {
        let old_wallet_path = PathBuf::from("wallet_data").join("wallet.dat");
        let new_wallet_path = secure_dir.join("wallet.dat");

        if old_wallet_path.exists() && !new_wallet_path.exists() {
            if let Err(e) = std::fs::create_dir_all(secure_dir) {
                eprintln!(
                    "⚠️  Warning: Failed to create secure wallet directory: {}",
                    e
                );
                return;
            }

            match std::fs::copy(&old_wallet_path, &new_wallet_path) {
                Ok(_) => {
                    println!("✅ Migrated wallet from insecure location to secure XDG directory");
                    println!("   Old location: {}", old_wallet_path.display());
                    println!("   New location: {}", new_wallet_path.display());
                    println!("   ⚠️  Please delete the old wallet_data/ directory to prevent accidental commits");
                }
                Err(e) => {
                    eprintln!("⚠️  Warning: Failed to migrate wallet: {}", e);
                    eprintln!(
                        "   Your wallet is still in the insecure location: {}",
                        old_wallet_path.display()
                    );
                }
            }
        }
    }

    pub async fn save_wallet(&self, wallet: &HDWallet, password: &str) -> NozyResult<()> {
        let data_dir = self.data_dir.clone();
        let mnemonic = wallet.get_mnemonic();
        let password_protected = wallet.is_password_protected();
        let password_hash = wallet.get_password_hash().cloned();
        let password = password.to_string();

        // AES-GCM + Argon2id (or legacy SHA-256 decrypt) must not block the async runtime.
        tokio::task::spawn_blocking(move || {
            let storage = WalletStorage::new(data_dir);
            storage.persist_wallet_data_blocking(
                mnemonic,
                password_protected,
                password_hash,
                &password,
            )
        })
        .await
        .map_err(|e| NozyError::Storage(format!("Wallet save task failed: {e}")))?
    }

    pub async fn load_wallet(&self, password: &str) -> NozyResult<HDWallet> {
        let data_dir = self.data_dir.clone();
        let password = password.to_string();

        // Decrypt may run Argon2id or legacy 100k-iter SHA-256; offload so IPC never stalls.
        tokio::task::spawn_blocking(move || {
            let storage = WalletStorage::new(data_dir);
            storage.load_wallet_blocking(&password)
        })
        .await
        .map_err(|e| NozyError::Storage(format!("Wallet load task failed: {e}")))?
    }

    /// Synchronous wallet load for blocking-pool workers.
    pub fn load_wallet_blocking(&self, password: &str) -> NozyResult<HDWallet> {
        let wallet_path = self.data_dir.join("wallet.dat");
        let encrypted = std::fs::read(&wallet_path).map_err(|e| {
            NozyError::Storage(format!(
                "Failed to read wallet file {}: {}",
                wallet_path.display(),
                e
            ))
        })?;
        let size = encrypted.len();
        let kind = vault_kind_label(&encrypted);
        let already_nzk2 = matches!(
            decode_vault_bytes(&encrypted),
            Ok(ref data) if data.len() >= V2_HEADER_LEN && data.starts_with(VAULT_MAGIC_V2)
        );

        let mut enc_passwords = password_unlock_candidates(password);
        // Wallets saved with an empty vault key but a real password_hash inside
        // AES-GCM-fail for the typed password. Unlock by decrypting with "" then
        // verifying the hash — then rewrite NZK2 so the next unlock is normal.
        if !password.is_empty() {
            enc_passwords.push(String::new());
        }

        let mut last_err: Option<NozyError> = None;
        for enc_pw in &enc_passwords {
            match self.decrypt_wallet_bytes(&encrypted, enc_pw) {
                Ok(plain) => match self.wallet_from_plaintext(&plain, password, enc_pw) {
                    Ok(wallet) => {
                        // Upgrade empty-key, legacy SHA-256/Argon2, and v1 embedded-key
                        // vaults to NZK2 on successful unlock.
                        if enc_pw != password || !already_nzk2 {
                            let verified = password_unlock_candidates(password)
                                .into_iter()
                                .find(|pw| wallet.verify_password(pw).unwrap_or(false))
                                .unwrap_or_else(|| password.to_string());
                            if let Err(e) = self.persist_wallet_data_blocking(
                                wallet.get_mnemonic(),
                                wallet.is_password_protected(),
                                wallet.get_password_hash().cloned(),
                                &verified,
                            ) {
                                eprintln!(
                                    "⚠️  Unlocked via compatibility path but failed to rewrite {}: {e}",
                                    wallet_path.display()
                                );
                            } else {
                                eprintln!(
                                    "✅ Rewrote {} as NZK2 using your password (previous vault was a compatibility/legacy format).",
                                    wallet_path.display()
                                );
                            }
                        }
                        return Ok(wallet);
                    }
                    Err(e) => last_err = Some(e),
                },
                Err(e) => last_err = Some(e),
            }
        }

        Err(self.annotate_unlock_error(last_err, &wallet_path, size, kind, password))
    }

    fn persist_wallet_data_blocking(
        &self,
        mnemonic: String,
        password_protected: bool,
        password_hash: Option<String>,
        password: &str,
    ) -> NozyResult<()> {
        let mut wallet_data = WalletData::new(mnemonic);
        wallet_data.password_protected = password_protected;
        wallet_data.password_hash = password_hash;

        let serialized = serde_json::to_string(&wallet_data)
            .map_err(|e| NozyError::Storage(format!("Failed to serialize wallet: {}", e)))?;

        let encrypted = self.encrypt_data(&serialized, password)?;
        std::fs::write(self.data_dir.join("wallet.dat"), encrypted)
            .map_err(|e| NozyError::Storage(format!("Failed to write wallet file: {}", e)))?;
        Ok(())
    }

    fn wallet_from_plaintext(
        &self,
        decrypted: &str,
        supplied_password: &str,
        enc_pw: &str,
    ) -> NozyResult<HDWallet> {
        let mut wallet_data: WalletData = serde_json::from_str(decrypted)
            .map_err(|e| NozyError::Storage(format!("Failed to deserialize wallet: {}", e)))?;

        wallet_data.ensure_timestamps();

        let mut wallet = HDWallet::from_mnemonic(&wallet_data.mnemonic)?;
        let supplied = password_unlock_candidates(supplied_password);

        if let Some(hash) = wallet_data.password_hash {
            wallet.set_password_hash(hash)?;
            let matched = supplied
                .iter()
                .any(|pw| wallet.verify_password(pw).unwrap_or(false));
            if !matched {
                return Err(NozyError::Cryptographic("Invalid password".to_string()));
            }
        } else if !supplied.iter().any(|pw| pw == enc_pw) {
            // Empty-key fallback must not open an unprotected vault when the
            // caller typed a non-empty password.
            return Err(NozyError::Cryptographic("Invalid password".to_string()));
        }

        Ok(wallet)
    }

    fn annotate_unlock_error(
        &self,
        last_err: Option<NozyError>,
        wallet_path: &std::path::Path,
        size: usize,
        kind: &str,
        password: &str,
    ) -> NozyError {
        let detail = match last_err {
            Some(NozyError::Cryptographic(msg)) | Some(NozyError::Storage(msg)) => msg,
            Some(other) => other.to_string(),
            None => "Decryption failed: Invalid password or corrupted data".to_string(),
        };
        let hint = if password.is_empty() {
            "Tried the empty vault key and v1 embedded-key layout."
        } else {
            "Tried your password, trimmed password, empty vault key, NZK2 Argon2id, unversioned Argon2id, legacy SHA-256, and v1 embedded-key (raw AES key in blob)."
        };
        NozyError::Cryptographic(format!(
            "Decryption failed: Invalid password or corrupted data ({detail})\n  File: {}\n  Size: {size} bytes\n  Vault: {kind}\n  {hint}",
            wallet_path.display()
        ))
    }

    /// True when the wallet cannot be opened with an empty encryption password.
    pub async fn requires_password(&self) -> bool {
        self.load_wallet("").await.is_err()
    }

    fn encrypt_data(&self, data: &str, password: &str) -> NozyResult<String> {
        // Empty password remains supported for legacy "no password" create UX, but Argon2id
        // still memory-hardens the blob vs the old iterated-SHA256 empty-password key.
        let mut salt = [0u8; 16];
        OsRng.fill_bytes(&mut salt);

        let key = self.derive_key_argon2id(password, &salt)?;

        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);

        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| NozyError::Storage(format!("Failed to create cipher: {}", e)))?;
        let ciphertext = cipher
            .encrypt(Nonce::from_slice(&nonce), data.as_bytes())
            .map_err(|e| NozyError::Storage(format!("Encryption failed: {}", e)))?;

        let mut result = Vec::with_capacity(V2_HEADER_LEN + ciphertext.len());
        result.extend_from_slice(VAULT_MAGIC_V2);
        result.extend_from_slice(&salt);
        result.extend_from_slice(&nonce);
        result.extend_from_slice(&ciphertext);

        Ok(hex::encode(result))
    }

    fn decrypt_data(&self, encrypted_data: &str, password: &str) -> NozyResult<String> {
        self.decrypt_wallet_bytes(encrypted_data.as_bytes(), password)
    }

    fn decrypt_wallet_bytes(&self, raw: &[u8], password: &str) -> NozyResult<String> {
        let blob = decode_vault_bytes(raw)?;
        self.decrypt_decoded_blob(&blob, password)
    }

    fn decrypt_decoded_blob(&self, data: &[u8], password: &str) -> NozyResult<String> {
        if data.len() >= V2_HEADER_LEN && data.starts_with(VAULT_MAGIC_V2) {
            let salt = &data[4..20];
            let nonce = &data[20..32];
            let ciphertext = &data[32..];
            let key = self.derive_key_argon2id(password, salt)?;
            return self.decrypt_aes_gcm(&key, nonce, ciphertext);
        }

        if data.len() < LEGACY_HEADER_LEN {
            return Err(NozyError::Storage(
                "Invalid encrypted data length".to_string(),
            ));
        }

        let salt = &data[0..16];
        let nonce = &data[16..28];
        let ciphertext = &data[28..];

        // Pre-F-05 native vaults used iterated SHA-256 with no magic.
        let sha_key = self.derive_key_legacy_sha256(password, salt);
        if let Ok(plain) = self.decrypt_aes_gcm(&sha_key, nonce, ciphertext) {
            return Ok(plain);
        }

        // WASM / unversioned Argon2id used the same salt||nonce||ct layout as SHA-256
        // but Argon2::default() (same 19MiB params as NZK2) — without the NZK2 prefix.
        if let Ok(argon_key) = self.derive_key_argon2id(password, salt) {
            if let Ok(plain) = self.decrypt_aes_gcm(&argon_key, nonce, ciphertext) {
                return Ok(plain);
            }
        }

        // v1.0 (e1218f6b): encrypt ignored the password and prepended the raw AES key.
        // Removed briefly after the Nov 2025 storage security fix; still needed to open
        // wallets created before password-derived keys existed (e.g. Gilmore's 554-byte file).
        self.decrypt_v1_embedded_key(data)
    }

    /// Pre-password vault: `key(32) || nonce(12) || ciphertext`. Password is unused.
    fn decrypt_v1_embedded_key(&self, data: &[u8]) -> NozyResult<String> {
        if data.len() < V1_EMBEDDED_KEY_MIN_LEN {
            return Err(NozyError::Cryptographic(
                "Decryption failed: Invalid password or corrupted data".to_string(),
            ));
        }
        let key = &data[0..32];
        let nonce = &data[32..44];
        let ciphertext = &data[44..];
        let mut key_arr = [0u8; 32];
        key_arr.copy_from_slice(key);
        self.decrypt_aes_gcm(&key_arr, nonce, ciphertext)
    }

    fn decrypt_aes_gcm(
        &self,
        key: &[u8; 32],
        nonce: &[u8],
        ciphertext: &[u8],
    ) -> NozyResult<String> {
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| NozyError::Storage(format!("Failed to create cipher: {}", e)))?;
        let plaintext = cipher
            .decrypt(Nonce::from_slice(nonce), ciphertext)
            .map_err(|_| {
                NozyError::Cryptographic(
                    "Decryption failed: Invalid password or corrupted data".to_string(),
                )
            })?;

        String::from_utf8(plaintext)
            .map_err(|e| NozyError::Storage(format!("Invalid UTF-8: {}", e)))
    }

    /// Argon2id (F-05) — memory-hard KDF for new wallet.dat blobs (`NZK2`).
    fn derive_key_argon2id(&self, password: &str, salt: &[u8]) -> NozyResult<[u8; 32]> {
        // m_cost=19 MiB, t_cost=2, p=1 — interactive unlock; OWASP-adjacent defaults for 0.5.
        let params = Params::new(19 * 1024, 2, 1, Some(32))
            .map_err(|e| NozyError::Storage(format!("Invalid Argon2 params: {e}")))?;
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
        let mut key = [0u8; 32];
        argon2
            .hash_password_into(password.as_bytes(), salt, &mut key)
            .map_err(|e| NozyError::Storage(format!("Argon2id KDF failed: {e}")))?;
        Ok(key)
    }

    /// Legacy iterated SHA-256 (pre-F-05) — decrypt only.
    fn derive_key_legacy_sha256(&self, password: &str, salt: &[u8]) -> [u8; 32] {
        const ITERATIONS: u32 = 100000;

        let mut hash = {
            let mut hasher = Sha256::new();
            hasher.update(password.as_bytes());
            hasher.update(salt);
            hasher.update(&0u32.to_be_bytes());
            hasher.finalize()
        };

        for i in 1..ITERATIONS {
            let mut hasher = Sha256::new();
            hasher.update(&hash);
            hasher.update(password.as_bytes());
            hasher.update(salt);
            hasher.update(&i.to_be_bytes());
            hash = hasher.finalize();
        }

        let mut key = [0u8; 32];
        key.copy_from_slice(&hash[..32]);
        key
    }

    pub async fn create_backup(&self, backup_path: &str) -> NozyResult<()> {
        let wallet_path = self.data_dir.join("wallet.dat");
        if !wallet_path.exists() {
            return Err(NozyError::Storage("No wallet found to backup".to_string()));
        }

        let backup_dir = PathBuf::from(backup_path);
        if !backup_dir.exists() {
            fs::create_dir_all(&backup_dir).map_err(|e| {
                NozyError::Storage(format!("Failed to create backup directory: {}", e))
            })?;
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let backup_file = backup_dir.join(format!("wallet_backup_{}.dat", timestamp));

        fs::copy(&wallet_path, &backup_file)
            .map_err(|e| NozyError::Storage(format!("Failed to create backup: {}", e)))?;

        println!("✅ Wallet backup created: {}", backup_file.display());
        Ok(())
    }

    pub async fn restore_from_backup(&self, backup_path: &str) -> NozyResult<()> {
        let backup_file = PathBuf::from(backup_path);
        if !backup_file.exists() {
            return Err(NozyError::Storage("Backup file not found".to_string()));
        }

        let wallet_path = self.data_dir.join("wallet.dat");

        if wallet_path.exists() {
            let current_backup = self.data_dir.join("wallet_current_backup.dat");
            fs::copy(&wallet_path, &current_backup).map_err(|e| {
                NozyError::Storage(format!("Failed to backup current wallet: {}", e))
            })?;
            println!(
                "📦 Current wallet backed up to: {}",
                current_backup.display()
            );
        }

        fs::copy(&backup_file, &wallet_path)
            .map_err(|e| NozyError::Storage(format!("Failed to restore from backup: {}", e)))?;

        println!("✅ Wallet restored from backup: {}", backup_file.display());
        Ok(())
    }

    pub fn list_backups(&self) -> NozyResult<Vec<String>> {
        let mut backups = Vec::new();

        if let Ok(entries) = fs::read_dir(&self.data_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(file_name) = path.file_name() {
                        if file_name.to_string_lossy().starts_with("wallet_backup_") {
                            backups.push(path.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }

        Ok(backups)
    }
}

fn password_unlock_candidates(password: &str) -> Vec<String> {
    let mut out = vec![password.to_string()];
    let trimmed = password.trim();
    if trimmed != password {
        out.push(trimmed.to_string());
    }
    out
}

fn decode_vault_bytes(raw: &[u8]) -> NozyResult<Vec<u8>> {
    let text = String::from_utf8_lossy(raw);
    let cleaned: String = text
        .trim()
        .trim_start_matches('\u{feff}')
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    if let Ok(data) = hex::decode(&cleaned) {
        if data.len() >= LEGACY_HEADER_LEN {
            return Ok(data);
        }
    }
    if raw.len() >= LEGACY_HEADER_LEN {
        return Ok(raw.to_vec());
    }
    Err(NozyError::Storage(
        "Failed to decode wallet.dat as hex. File is not a Nozy vault blob.".to_string(),
    ))
}

fn vault_kind_label(raw: &[u8]) -> &'static str {
    match decode_vault_bytes(raw) {
        Ok(data) if data.len() >= V2_HEADER_LEN && data.starts_with(VAULT_MAGIC_V2) => {
            "NZK2 (Argon2id)"
        }
        Ok(_) => "unversioned (SHA-256, Argon2id, or v1 embedded-key)",
        Err(_) => "not-hex / too short",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aes_gcm::aead::Aead;

    #[test]
    fn vault_nzk2_roundtrip_argon2id() {
        let storage = WalletStorage::new(PathBuf::from("."));
        let blob = storage
            .encrypt_data(r#"{"mnemonic":"test words only"}"#, "correct horse")
            .expect("encrypt");
        let raw = hex::decode(&blob).unwrap();
        assert!(raw.starts_with(VAULT_MAGIC_V2));
        let plain = storage
            .decrypt_data(&blob, "correct horse")
            .expect("decrypt");
        assert!(plain.contains("test words only"));
        assert!(storage.decrypt_data(&blob, "wrong").is_err());
    }

    #[test]
    fn vault_legacy_sha256_still_decrypts() {
        let storage = WalletStorage::new(PathBuf::from("."));
        // Build a legacy blob with the old KDF path.
        let password = "legacy-pass";
        let mut salt = [0u8; 16];
        OsRng.fill_bytes(&mut salt);
        let key = storage.derive_key_legacy_sha256(password, &salt);
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);
        let cipher = Aes256Gcm::new_from_slice(&key).unwrap();
        let ciphertext = cipher
            .encrypt(Nonce::from_slice(&nonce), b"legacy-wallet-json".as_slice())
            .unwrap();
        let mut raw = Vec::new();
        raw.extend_from_slice(&salt);
        raw.extend_from_slice(&nonce);
        raw.extend_from_slice(&ciphertext);
        let hex_blob = hex::encode(raw);
        let plain = storage
            .decrypt_data(&hex_blob, password)
            .expect("legacy decrypt");
        assert_eq!(plain, "legacy-wallet-json");
    }

    #[test]
    fn vault_hex_ignores_whitespace_and_bom() {
        let storage = WalletStorage::new(PathBuf::from("."));
        let blob = storage
            .encrypt_data(r#"{"mnemonic":"test words only"}"#, "pw")
            .expect("encrypt");
        let wrapped = format!("\u{feff}{blob}\n");
        let plain = storage
            .decrypt_data(&wrapped, "pw")
            .expect("decrypt wrapped");
        assert!(plain.contains("test words only"));
    }

    #[test]
    fn wrong_password_is_cryptographic_not_storage() {
        let storage = WalletStorage::new(PathBuf::from("."));
        let blob = storage.encrypt_data("{}", "right").expect("encrypt");
        let err = storage.decrypt_data(&blob, "wrong").unwrap_err();
        match err {
            NozyError::Cryptographic(ref msg) => {
                assert!(msg.to_ascii_lowercase().contains("password"))
            }
            other => panic!("expected Cryptographic, got {other:?}"),
        }
        assert!(!err.user_friendly_message().contains("permissions"));
    }

    fn temp_wallet_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "nozy-vault-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn encrypt_unversioned_argon2(storage: &WalletStorage, data: &str, password: &str) -> String {
        let mut salt = [0u8; 16];
        OsRng.fill_bytes(&mut salt);
        let key = storage.derive_key_argon2id(password, &salt).unwrap();
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);
        let cipher = Aes256Gcm::new_from_slice(&key).unwrap();
        let ciphertext = cipher
            .encrypt(Nonce::from_slice(&nonce), data.as_bytes())
            .unwrap();
        let mut raw = Vec::new();
        raw.extend_from_slice(&salt);
        raw.extend_from_slice(&nonce);
        raw.extend_from_slice(&ciphertext);
        hex::encode(raw)
    }

    #[test]
    fn vault_unversioned_argon2id_still_decrypts() {
        let storage = WalletStorage::new(PathBuf::from("."));
        let blob = encrypt_unversioned_argon2(&storage, "wasm-or-pre-nzk2-json", "secret");
        let raw = hex::decode(&blob).unwrap();
        assert!(!raw.starts_with(VAULT_MAGIC_V2));
        let plain = storage
            .decrypt_data(&blob, "secret")
            .expect("unversioned argon2 decrypt");
        assert_eq!(plain, "wasm-or-pre-nzk2-json");
    }

    /// v1.0 encrypt_data ignored the password and stored key||nonce||ct in the hex blob.
    fn encrypt_v1_embedded_key(data: &str) -> String {
        let mut key = [0u8; 32];
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut key);
        OsRng.fill_bytes(&mut nonce);
        let cipher = Aes256Gcm::new_from_slice(&key).unwrap();
        let ciphertext = cipher
            .encrypt(Nonce::from_slice(&nonce), data.as_bytes())
            .unwrap();
        let mut raw = Vec::new();
        raw.extend_from_slice(&key);
        raw.extend_from_slice(&nonce);
        raw.extend_from_slice(&ciphertext);
        hex::encode(raw)
    }

    #[test]
    fn vault_v1_embedded_key_still_decrypts_and_upgrades() {
        let dir = temp_wallet_dir();
        let storage = WalletStorage::new(dir.clone());
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let json = serde_json::to_string(&WalletData::new(mnemonic.to_string())).unwrap();
        let blob = encrypt_v1_embedded_key(&json);
        assert!(blob.len() > 100, "v1 hex blob should be wallet-sized");
        let raw = hex::decode(&blob).unwrap();
        assert!(!raw.starts_with(VAULT_MAGIC_V2));
        assert!(raw.len() >= V1_EMBEDDED_KEY_MIN_LEN);
        fs::write(dir.join("wallet.dat"), blob.as_bytes()).unwrap();

        // Password was never used for v1 encryption; typed password still unlocks,
        // then the file is rewritten as NZK2 under that password.
        let loaded = storage
            .load_wallet_blocking("gilmore-recovery-pw")
            .expect("v1 embedded-key unlock");
        assert_eq!(loaded.get_mnemonic(), mnemonic);

        let rewritten = fs::read_to_string(dir.join("wallet.dat")).unwrap();
        let rewritten_raw = hex::decode(rewritten.trim()).unwrap();
        assert!(
            rewritten_raw.starts_with(VAULT_MAGIC_V2),
            "v1 unlock should rewrite NZK2"
        );
        storage
            .load_wallet_blocking("gilmore-recovery-pw")
            .expect("unlock after NZK2 rewrite");
        storage
            .load_wallet_blocking("wrong")
            .expect_err("NZK2 must reject wrong password");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn empty_vault_key_unlocks_when_password_hash_matches() {
        let dir = temp_wallet_dir();
        let storage = WalletStorage::new(dir.clone());
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let mut wallet = HDWallet::from_mnemonic(mnemonic).unwrap();
        wallet.set_password("correct-horse").unwrap();
        let mut data = WalletData::new(mnemonic.to_string());
        data.password_protected = true;
        data.password_hash = wallet.get_password_hash().cloned();
        let json = serde_json::to_string(&data).unwrap();
        let blob = storage.encrypt_data(&json, "").expect("encrypt empty key");
        fs::write(dir.join("wallet.dat"), blob.as_bytes()).unwrap();

        storage
            .load_wallet_blocking("wrong-password")
            .expect_err("hash must not match");
        let loaded = storage
            .load_wallet_blocking("correct-horse")
            .expect("empty-key + matching hash");
        assert_eq!(loaded.get_mnemonic(), mnemonic);

        let rewritten = fs::read_to_string(dir.join("wallet.dat")).unwrap();
        let raw = hex::decode(rewritten.trim()).unwrap();
        assert!(
            raw.starts_with(VAULT_MAGIC_V2),
            "repair should rewrite NZK2 with the user password"
        );
        storage
            .load_wallet_blocking("correct-horse")
            .expect("unlock after NZK2 rewrite");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_wallet_blocking_names_file_on_decrypt_fail() {
        let dir = temp_wallet_dir();
        let storage = WalletStorage::new(dir.clone());
        let blob = storage.encrypt_data("{}", "right").expect("encrypt");
        fs::write(dir.join("wallet.dat"), blob.as_bytes()).unwrap();
        let err = storage.load_wallet_blocking("wrong").unwrap_err();
        match err {
            NozyError::Cryptographic(msg) => {
                assert!(msg.contains("Decryption failed"));
                assert!(msg.contains("File:"));
                assert!(msg.contains("Size:"));
                assert!(msg.contains("Vault:"));
            }
            other => panic!("expected Cryptographic, got {other:?}"),
        }
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn empty_key_fallback_does_not_open_unprotected_wallet_with_wrong_password() {
        let dir = temp_wallet_dir();
        let storage = WalletStorage::new(dir.clone());
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let data = WalletData::new(mnemonic.to_string());
        let json = serde_json::to_string(&data).unwrap();
        let blob = storage.encrypt_data(&json, "").expect("encrypt empty");
        fs::write(dir.join("wallet.dat"), blob.as_bytes()).unwrap();
        storage
            .load_wallet_blocking("guess")
            .expect_err("must not open unprotected vault via empty fallback");
        storage
            .load_wallet_blocking("")
            .expect("empty password still opens unprotected vault");
        let _ = fs::remove_dir_all(&dir);
    }
}
