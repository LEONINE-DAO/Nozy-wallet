use crate::error::{NozyError, NozyResult};
use crate::hd_wallet::HDWallet;
use crate::transactions::TransactionDetails;
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::Aead;
use rand::RngCore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletData {
    pub mnemonic: String,
    pub addresses: Vec<String>,
    pub transactions: Vec<TransactionDetails>,
    pub balance: u64,
}

impl WalletData {
    pub fn new(mnemonic: String) -> Self {
        Self {
            mnemonic,
            addresses: Vec::new(),
            transactions: Vec::new(),
            balance: 0,
        }
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
}
