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
            let mut wallet_data = WalletData::new(mnemonic);
            wallet_data.password_protected = password_protected;
            wallet_data.password_hash = password_hash;

            let serialized = serde_json::to_string(&wallet_data)
                .map_err(|e| NozyError::Storage(format!("Failed to serialize wallet: {}", e)))?;

            let encrypted = storage.encrypt_data(&serialized, &password)?;
            std::fs::write(storage.data_dir.join("wallet.dat"), encrypted)
                .map_err(|e| NozyError::Storage(format!("Failed to write wallet file: {}", e)))?;

            Ok(())
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
        let encrypted = std::fs::read(self.data_dir.join("wallet.dat"))
            .map_err(|e| NozyError::Storage(format!("Failed to read wallet file: {}", e)))?;

        let decrypted = self.decrypt_data(&String::from_utf8_lossy(&encrypted), password)?;
        let mut wallet_data: WalletData = serde_json::from_str(&decrypted)
            .map_err(|e| NozyError::Storage(format!("Failed to deserialize wallet: {}", e)))?;

        wallet_data.ensure_timestamps();

        let mut wallet = HDWallet::from_mnemonic(&wallet_data.mnemonic)?;

        if let Some(hash) = wallet_data.password_hash {
            wallet.set_password_hash(hash.clone())?;

            let is_valid = wallet.verify_password(password).map_err(|e| {
                NozyError::Cryptographic(format!("Password verification failed: {}", e))
            })?;

            if !is_valid {
                return Err(NozyError::Cryptographic("Invalid password".to_string()));
            }
        }

        Ok(wallet)
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
        let data = hex::decode(encrypted_data)
            .map_err(|e| NozyError::Storage(format!("Failed to decode hex: {}", e)))?;

        if data.len() >= V2_HEADER_LEN && data.starts_with(VAULT_MAGIC_V2) {
            let salt = &data[4..20];
            let nonce = &data[20..32];
            let ciphertext = &data[32..];
            let key = self.derive_key_argon2id(password, salt)?;
            return self.decrypt_aes_gcm(&key, nonce, ciphertext);
        }

        // Legacy iterated-SHA256 vault (pre-F-05). Still readable; next save upgrades to NZK2.
        if data.len() < LEGACY_HEADER_LEN {
            return Err(NozyError::Storage(
                "Invalid encrypted data length".to_string(),
            ));
        }

        let salt = &data[0..16];
        let nonce = &data[16..28];
        let ciphertext = &data[28..];
        let key = self.derive_key_legacy_sha256(password, salt);
        self.decrypt_aes_gcm(&key, nonce, ciphertext)
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
                NozyError::Storage(
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
