use crate::error::{NozyError, NozyResult};
use crate::hd_wallet::HDWallet;
use crate::transactions::TransactionDetails;
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::Aead;
use rand::RngCore;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletData {
    pub mnemonic: String,
    pub addresses: Vec<String>,
    pub transactions: Vec<TransactionDetails>,
    pub balance: u64,
    pub created_at: u64,
    pub last_updated: u64,
    pub version: String,
    pub password_protected: bool,
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
        let wallet_data: WalletData = serde_json::from_str(&decrypted)
            .map_err(|e| NozyError::Storage(format!("Failed to deserialize wallet: {}", e)))?;
        
        HDWallet::from_mnemonic(&wallet_data.mnemonic)
    }

    fn encrypt_data(&self, data: &str, _password: &str) -> NozyResult<String> {
        let mut key = [0u8; 32];
        let mut nonce = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut key);
        rand::thread_rng().fill_bytes(&mut nonce);
        
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| NozyError::Storage(format!("Failed to create cipher: {}", e)))?;
        let ciphertext = cipher.encrypt(Nonce::from_slice(&nonce), data.as_bytes())
            .map_err(|e| NozyError::Storage(format!("Encryption failed: {}", e)))?;
        
        let mut result = Vec::new();
        result.extend_from_slice(&key);
        result.extend_from_slice(&nonce);
        result.extend_from_slice(&ciphertext);
        
        Ok(hex::encode(result))
    }

    fn decrypt_data(&self, encrypted_data: &str, _password: &str) -> NozyResult<String> {
        let data = hex::decode(encrypted_data)
            .map_err(|e| NozyError::Storage(format!("Failed to decode hex: {}", e)))?;
        
        if data.len() < 44 {
            return Err(NozyError::Storage("Invalid encrypted data length".to_string()));
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
    
    /// Create a backup of the wallet
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
    
    /// Restore wallet from backup
    pub async fn restore_from_backup(&self, backup_path: &str) -> NozyResult<()> {
        let backup_file = PathBuf::from(backup_path);
        if !backup_file.exists() {
            return Err(NozyError::Storage("Backup file not found".to_string()));
        }
        
        let wallet_path = self.data_dir.join("wallet.dat");
        
        // Create backup of current wallet if it exists
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
    
    /// List available backups
    pub fn list_backups(&self) -> NozyResult<Vec<String>> {
        let mut backups = Vec::new();
        
        // Check in data directory
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
