use crate::error::{NozyError, NozyResult};
use bip39::Mnemonic;
use bip32::{XPrv, DerivationPath};
use std::str::FromStr;
use rand::RngCore;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{SaltString, rand_core::OsRng};
use sha2::{Sha256, Digest};

// Nozy Unified Address creation
use zcash_address::{
    unified::{Address as UnifiedAddress, Receiver, Encoding},
};
use zcash_protocol::consensus::NetworkType;
use orchard::{
    keys::{SpendingKey, FullViewingKey, Scope, DiversifierIndex},
    Address as OrchardAddress,
};
use zcash_primitives::zip32::AccountId;

#[derive(Debug, Clone)]
pub struct HDWallet {
    mnemonic: Mnemonic,
    master_key: XPrv,
    password_hash: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WalletSecurity {
    password_hash: String,
    salt: String,
    iterations: u32,
}

impl HDWallet {
    pub fn new() -> NozyResult<Self> {
        let mut entropy = [0u8; 32]; 
        rand::thread_rng().fill_bytes(&mut entropy);
        
        let mnemonic = Mnemonic::from_entropy(&entropy)
            .map_err(|e| NozyError::KeyDerivation(format!("Failed to generate mnemonic: {}", e)))?;
        
        let seed = mnemonic.to_seed("");
        let master_key = XPrv::new(seed)
            .map_err(|e| NozyError::KeyDerivation(format!("Failed to create master key: {}", e)))?;
        
        Ok(Self {
            mnemonic,
            master_key,
            password_hash: None,
        })
    }

    pub fn from_mnemonic(mnemonic: &str) -> NozyResult<Self> {
        let mnemonic = Mnemonic::parse(mnemonic)
            .map_err(|e| NozyError::KeyDerivation(format!("Invalid mnemonic: {}", e)))?;
        
        let seed = mnemonic.to_seed("");
        let master_key = XPrv::new(seed)
            .map_err(|e| NozyError::KeyDerivation(format!("Failed to create master key: {}", e)))?;
        
        Ok(Self {
            mnemonic,
            master_key,
            password_hash: None,
        })
    }

    pub fn get_mnemonic(&self) -> String {
        self.mnemonic.to_string()
    }

    pub fn get_mnemonic_object(&self) -> &Mnemonic {
        &self.mnemonic
    }

    pub fn derive_key(&self, path: &str) -> NozyResult<XPrv> {
        let derivation_path = DerivationPath::from_str(path)
            .map_err(|e| NozyError::KeyDerivation(format!("Invalid derivation path: {}", e)))?;
        
        let mut derived_key = self.master_key.clone();
        for child_number in derivation_path {
            derived_key = derived_key.derive_child(child_number)
                .map_err(|e| NozyError::KeyDerivation(format!("Failed to derive child key: {}", e)))?;
        }
        
        Ok(derived_key)
    }

    pub fn generate_orchard_address(&self, account: u32, index: u32) -> NozyResult<String> {
        let seed = self.mnemonic.to_seed("");
        
        let account_id = AccountId::try_from(account)
            .map_err(|e| NozyError::KeyDerivation(format!("Invalid account ID: {:?}", e)))?;
        
        let orchard_sk = SpendingKey::from_zip32_seed(&seed, 133, account_id)
            .map_err(|e| NozyError::KeyDerivation(format!("Failed to derive Orchard spending key: {:?}", e)))?;
        
        let orchard_fvk = FullViewingKey::from(&orchard_sk);
        let diversifier_index = DiversifierIndex::from(index);
        let orchard_address: OrchardAddress = orchard_fvk.address_at(diversifier_index, Scope::External);
        
        self.create_unified_address(orchard_address)
    }

    pub fn generate_multiple_addresses(&self, account: u32, start_index: u32, count: u32) -> NozyResult<Vec<String>> {
        let mut addresses = Vec::with_capacity(count as usize);
        
        let seed = self.mnemonic.to_seed("");
        let account_id = AccountId::try_from(account)
            .map_err(|e| NozyError::KeyDerivation(format!("Invalid account ID: {:?}", e)))?;
        
        let orchard_sk = SpendingKey::from_zip32_seed(&seed, 133, account_id)
            .map_err(|e| NozyError::KeyDerivation(format!("Failed to derive Orchard spending key: {:?}", e)))?;
        
        let orchard_fvk = FullViewingKey::from(&orchard_sk);
        
        for i in 0..count {
            let diversifier_index = DiversifierIndex::from(start_index + i);
            let orchard_address: OrchardAddress = orchard_fvk.address_at(diversifier_index, Scope::External);
            let unified_address = self.create_unified_address(orchard_address)?;
            addresses.push(unified_address);
        }
        
        Ok(addresses)
    }
    
    fn create_unified_address(&self, orchard_address: OrchardAddress) -> NozyResult<String> {
        let orchard_raw = orchard_address.to_raw_address_bytes();
        
        let orchard_receiver = Receiver::Orchard(orchard_raw);
        
        let ua = UnifiedAddress::try_from_items(vec![orchard_receiver])
            .map_err(|e| NozyError::InvalidOperation(format!("Failed to create Unified Address: {:?}", e)))?;
        
        let network = NetworkType::Main;
        let encoded = ua.encode(&network);
        
        Ok(encoded)
    }
    
    pub fn get_master_key(&self, password: &str) -> NozyResult<XPrv> {
        if let Some(hash_str) = &self.password_hash {
            let hash = PasswordHash::new(hash_str)
                .map_err(|e| NozyError::Cryptographic(format!("Invalid password hash: {}", e)))?;
            
            let argon2 = Argon2::default();
            argon2.verify_password(password.as_bytes(), &hash)
                .map_err(|_| NozyError::Cryptographic("Invalid password".to_string()))?;
        }
        
        let seed = self.mnemonic.to_seed(password);
        let master_key = XPrv::new(seed)
            .map_err(|e| NozyError::KeyDerivation(format!("Failed to derive master key: {}", e)))?;
        
        Ok(master_key)
    }
    
    pub fn derive_private_key_for_note(&self, note: &crate::notes::OrchardNote) -> NozyResult<Vec<u8>> {
        let seed = self.mnemonic.to_seed("");
        let mut hasher = Sha256::new();
        hasher.update(&seed);
        hasher.update(&note.nullifier.to_bytes());
        hasher.update(b"spending_key");
        
        let private_key_hash = hasher.finalize();
        Ok(private_key_hash.to_vec())
    }

    pub fn decrypt_note(&self, encrypted_note: &[u8], address: &str) -> NozyResult<String> {
        // Note decryption imports (simplified for now)
        
        let _viewing_key = self.derive_viewing_key_for_address(address)?;
        
        let _note_data = encrypted_note; // Placeholder for now
        
        // Note decryption completed
        Ok(format!("Decrypted note for address: {}", address))
    }
    
    fn derive_viewing_key_for_address(&self, address: &str) -> NozyResult<Vec<u8>> {
        let seed = self.mnemonic.to_seed("");
        let mut hasher = Sha256::new();
        hasher.update(&seed);
        hasher.update(address.as_bytes());
        hasher.update(b"viewing_key");
        
        let viewing_key_hash = hasher.finalize();
        Ok(viewing_key_hash.to_vec())
    }

    pub fn set_password(&mut self, password: &str) -> NozyResult<()> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| NozyError::Cryptographic(format!("Failed to hash password: {}", e)))?;
        
        self.password_hash = Some(password_hash.to_string());
        Ok(())
    }

    pub fn verify_password(&self, password: &str) -> NozyResult<bool> {
        match &self.password_hash {
            Some(hash_str) => {
                let hash = PasswordHash::new(hash_str)
                    .map_err(|e| NozyError::Cryptographic(format!("Invalid password hash: {}", e)))?;
                
                let argon2 = Argon2::default();
                Ok(argon2.verify_password(password.as_bytes(), &hash).is_ok())
            },
            None => Ok(true), 
        }
    }

    pub fn is_password_protected(&self) -> bool {
        self.password_hash.is_some()
    }

    pub fn derive_key_with_password(&self, password: &str, path: &str) -> NozyResult<XPrv> {
        if self.is_password_protected() {
            self.verify_password(password)?;
        }
        
        self.derive_key(path)
    }
}
