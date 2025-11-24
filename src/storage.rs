use crate::error::{NozyError, NozyResult};
use crate::hd_wallet::HDWallet;
use crate::transactions::TransactionDetails;
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::Aead;
use rand::RngCore;
use rand::rngs::OsRng;
use sha2::{Sha256, Digest};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

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
        use crate::paths::get_wallet_data_dir;
        let secure_dir = get_wallet_data_dir();
        
        Self::migrate_from_insecure_location(&secure_dir);
        
        Self::new(secure_dir)
    }
    
    fn migrate_from_insecure_location(secure_dir: &PathBuf) {
        let old_wallet_path = PathBuf::from("wallet_data").join("wallet.dat");
        let new_wallet_path = secure_dir.join("wallet.dat");
        
       
        if old_wallet_path.exists() && !new_wallet_path.exists() {
            if let Err(e) = std::fs::create_dir_all(secure_dir) {
                eprintln!("⚠️  Warning: Failed to create secure wallet directory: {}", e);
                return;
            }
            
            match std::fs::copy(&old_wallet_path, &new_wallet_path) {
                Ok(_) => {
                    println!("✅ Migrated wallet from insecure location to secure XDG directory");
                    println!("   Old location: {}", old_wallet_path.display());
                    println!("   New location: {}", new_wallet_path.display());
                    println!("   ⚠️  Please delete the old wallet_data/ directory to prevent accidental commits");
                },
                Err(e) => {
                    eprintln!("⚠️  Warning: Failed to migrate wallet: {}", e);
                    eprintln!("   Your wallet is still in the insecure location: {}", old_wallet_path.display());
                }
            }
        }
    }

    pub async fn save_wallet(&self, wallet: &HDWallet, password: &str) -> NozyResult<()> {
        let wallet_data = WalletData::new(wallet.get_mnemonic());
        let serialized = serde_json::to_string(&wallet_data)
            .map_err(|e| NozyError::Storage(format!("Failed to serialize wallet: {}", e)))?;
        
        let encrypted = self.encrypt_data(&serialized, password)?;
        std::fs::write(self.data_dir.join("wallet.dat"), encrypted)
            .map_err(|e| NozyError::Storage(format!("Failed to write wallet file: {}", e)))?;
        
        Ok(())
    }

    pub async fn load_wallet(&self, password: &str) -> NozyResult<HDWallet> {
        let encrypted = std::fs::read(self.data_dir.join("wallet.dat"))
            .map_err(|e| NozyError::Storage(format!("Failed to read wallet file: {}", e)))?;
        
        let decrypted = self.decrypt_data(&String::from_utf8_lossy(&encrypted), password)?;
        let mut wallet_data: WalletData = serde_json::from_str(&decrypted)
            .map_err(|e| NozyError::Storage(format!("Failed to deserialize wallet: {}", e)))?;
        
        wallet_data.ensure_timestamps();
        
        HDWallet::from_mnemonic(&wallet_data.mnemonic)
    }

    fn encrypt_data(&self, data: &str, password: &str) -> NozyResult<String> {
        let mut salt = [0u8; 16];
        OsRng.fill_bytes(&mut salt);
        
        let key = self.derive_key_from_password(password, &salt);
        
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);
        
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| NozyError::Storage(format!("Failed to create cipher: {}", e)))?;
        let ciphertext = cipher.encrypt(Nonce::from_slice(&nonce), data.as_bytes())
            .map_err(|e| NozyError::Storage(format!("Encryption failed: {}", e)))?;
        
        let mut result = Vec::new();
        result.extend_from_slice(&salt);
        result.extend_from_slice(&nonce);
        result.extend_from_slice(&ciphertext);
        
        Ok(hex::encode(result))
    }

    fn decrypt_data(&self, encrypted_data: &str, password: &str) -> NozyResult<String> {
        let data = hex::decode(encrypted_data)
            .map_err(|e| NozyError::Storage(format!("Failed to decode hex: {}", e)))?;
        
       
        if data.len() >= 44 && data.len() < 50 {
            if let Ok(result) = self.decrypt_old_format(&data) {
                return Ok(result);
            }
        }
        
        if data.len() < 28 {
            return Err(NozyError::Storage("Invalid encrypted data length".to_string()));
        }
        
        let salt = &data[0..16];
        let nonce = &data[16..28];
        let ciphertext = &data[28..];
        
        let key = self.derive_key_from_password(password, salt);
        
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| NozyError::Storage(format!("Failed to create cipher: {}", e)))?;
        let plaintext = cipher.decrypt(Nonce::from_slice(nonce), ciphertext)
            .map_err(|_| NozyError::Storage("Decryption failed: Invalid password or corrupted data".to_string()))?;
        
        String::from_utf8(plaintext)
            .map_err(|e| NozyError::Storage(format!("Invalid UTF-8: {}", e)))
    }
    
 
    fn derive_key_from_password(&self, password: &str, salt: &[u8]) -> [u8; 32] {
       
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
    
  
    fn decrypt_old_format(&self, data: &[u8]) -> NozyResult<String> {
        if data.len() < 44 {
            return Err(NozyError::Storage("Invalid old format data length".to_string()));
        }
        
        let key = &data[0..32];
        let nonce = &data[32..44];
        let ciphertext = &data[44..];
        
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| NozyError::Storage(format!("Failed to create cipher: {}", e)))?;
        let plaintext = cipher.decrypt(Nonce::from_slice(nonce), ciphertext)
            .map_err(|e| NozyError::Storage(format!("Decryption failed: {}", e)))?;
        
        String::from_utf8(plaintext)
            .map_err(|e| NozyError::Storage(format!("Invalid UTF-8: {}", e)))
    }
    
    pub async fn create_backup(&self, backup_path: &str) -> NozyResult<()> {
        let wallet_path = self.data_dir.join("wallet.dat");
        if !wallet_path.exists() {
            return Err(NozyError::Storage("No wallet found to backup".to_string()));
        }
        
        let backup_dir = PathBuf::from(backup_path);
        if !backup_dir.exists() {
            fs::create_dir_all(&backup_dir)
                .map_err(|e| NozyError::Storage(format!("Failed to create backup directory: {}", e)))?;
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
            fs::copy(&wallet_path, &current_backup)
                .map_err(|e| NozyError::Storage(format!("Failed to backup current wallet: {}", e)))?;
            println!("📦 Current wallet backed up to: {}", current_backup.display());
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
                if path.is_file() && path.file_name().unwrap().to_string_lossy().starts_with("wallet_backup_") {
                    backups.push(path.to_string_lossy().to_string());
                }
            }
        }
        
        Ok(backups)
    }
}
