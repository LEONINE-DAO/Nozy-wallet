use crate::error::{NozyError, NozyResult};
use crate::key_management::{SecureSeed, zeroize_bytes};
use bip39::Mnemonic;
use bip32::{XPrv, DerivationPath};
use std::str::FromStr;
use rand::RngCore;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{SaltString, rand_core::OsRng};
use sha2::{Sha256, Digest};
use zeroize::Zeroize;

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
        // Use secure seed that will be zeroized on drop
        let mut seed_bytes = self.mnemonic.to_seed("").to_vec();
        let secure_seed = SecureSeed::new(seed_bytes.clone());
        
        let account_id = AccountId::try_from(account)
            .map_err(|e| NozyError::KeyDerivation(format!("Invalid account ID: {:?}", e)))?;
        
        let orchard_sk = SpendingKey::from_zip32_seed(secure_seed.as_bytes(), 133, account_id)
            .map_err(|e| NozyError::KeyDerivation(format!("Failed to derive Orchard spending key: {:?}", e)))?;
        
        let orchard_fvk = FullViewingKey::from(&orchard_sk);
        let diversifier_index = DiversifierIndex::from(index);
        let orchard_address: OrchardAddress = orchard_fvk.address_at(diversifier_index, Scope::External);
        
        zeroize_bytes(&mut seed_bytes);
        
        self.create_unified_address(orchard_address)
    }

    pub fn generate_multiple_addresses(&self, account: u32, start_index: u32, count: u32) -> NozyResult<Vec<String>> {
        let mut addresses = Vec::with_capacity(count as usize);
        
        let mut seed_bytes = self.mnemonic.to_seed("").to_vec();
        let secure_seed = SecureSeed::new(seed_bytes.clone());
        
        let account_id = AccountId::try_from(account)
            .map_err(|e| NozyError::KeyDerivation(format!("Invalid account ID: {:?}", e)))?;
        
        let orchard_sk = SpendingKey::from_zip32_seed(secure_seed.as_bytes(), 133, account_id)
            .map_err(|e| NozyError::KeyDerivation(format!("Failed to derive Orchard spending key: {:?}", e)))?;
        
        let orchard_fvk = FullViewingKey::from(&orchard_sk);
        
        for i in 0..count {
            let diversifier_index = DiversifierIndex::from(start_index + i);
            let orchard_address: OrchardAddress = orchard_fvk.address_at(diversifier_index, Scope::External);
            let unified_address = self.create_unified_address(orchard_address)?;
            addresses.push(unified_address);
        }
        
        zeroize_bytes(&mut seed_bytes);
        
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
        
        let mut seed_bytes = self.mnemonic.to_seed(password).to_vec();
        let secure_seed = SecureSeed::new(seed_bytes.clone());
        
        let master_key = XPrv::new(secure_seed.as_bytes())
            .map_err(|e| NozyError::KeyDerivation(format!("Failed to derive master key: {}", e)))?;
        
        zeroize_bytes(&mut seed_bytes);
        
        Ok(master_key)
    }
    
    pub fn derive_private_key_for_note(&self, note: &crate::notes::OrchardNote) -> NozyResult<Vec<u8>> {
        let mut seed_bytes = self.mnemonic.to_seed("").to_vec();
        let secure_seed = SecureSeed::new(seed_bytes.clone());
        
        let mut hasher = Sha256::new();
        hasher.update(secure_seed.as_bytes());
        hasher.update(&note.nullifier.to_bytes());
        hasher.update(b"spending_key");
        
        let private_key_hash = hasher.finalize();
        let result = private_key_hash.to_vec();
        
        zeroize_bytes(&mut seed_bytes);
        
        Ok(result)
    }

    
    pub fn decrypt_note(&self, _encrypted_note: &[u8], address: &str) -> NozyResult<String> {
        let _ivk = self.derive_incoming_viewing_key_for_address(address)?;
        
        Ok(format!("Decrypted note for address: {} (key derived successfully)", address))
    }
    
    pub fn decrypt_orchard_action(
        &self,
        action: &crate::notes::OrchardActionData,
        address: &str,
        block_height: u32,
        txid: &str,
    ) -> NozyResult<Option<crate::notes::OrchardNote>> {
        use orchard::{
            note::{Nullifier, ExtractedNoteCommitment},
            note_encryption::{OrchardDomain, CompactAction},
            keys::PreparedIncomingViewingKey,
        };
        use zcash_note_encryption::{EphemeralKeyBytes, try_compact_note_decryption};
        
        let ivk = self.derive_incoming_viewing_key_for_address(address)?;
        
        let nullifier_result = Nullifier::from_bytes(&action.nullifier);
        let nullifier = match nullifier_result.into_option() {
            Some(n) => n,
            None => {
                return Err(NozyError::InvalidOperation("Invalid nullifier bytes".to_string()));
            }
        };
        
        let cmx_result = ExtractedNoteCommitment::from_bytes(&action.cmx);
        let cmx = match cmx_result.into_option() {
            Some(c) => c,
            None => {
                return Err(NozyError::InvalidOperation("Invalid cmx bytes".to_string()));
            }
        };
        
        let ephemeral_key = EphemeralKeyBytes::from(action.ephemeral_key);
        
        if action.encrypted_note.len() < 52 {
            return Err(NozyError::InvalidOperation(
                "Encrypted note too short for CompactAction".to_string()
            ));
        }
        
        let mut compact_enc_ciphertext = [0u8; 52];
        compact_enc_ciphertext.copy_from_slice(&action.encrypted_note[..52]);
        
        let compact_action = CompactAction::from_parts(
            nullifier,
            cmx,
            ephemeral_key,
            compact_enc_ciphertext,
        );
        
        let domain = OrchardDomain::for_compact_action(&compact_action);
        
        let prepared_ivk = PreparedIncomingViewingKey::new(&ivk);
        
        match try_compact_note_decryption(&domain, &prepared_ivk, &compact_action) {
            Some((note, note_address)) => {
                let orchard_note = crate::notes::OrchardNote {
                    note: note.clone(),
                    value: note.value().inner(),
                    address: note_address.clone(),
                    nullifier,
                    block_height,
                    txid: txid.to_string(),
                    spent: false,
                    memo: Vec::new(), 
                };
                Ok(Some(orchard_note))
            },
            None => Ok(None), 
        }
    }
    
    fn derive_incoming_viewing_key_for_address(&self, address: &str) -> NozyResult<orchard::keys::IncomingViewingKey> {
        use zcash_address::unified::{Address as UnifiedAddress, Encoding, Container};
        
        let (_network, ua) = UnifiedAddress::decode(address)
            .map_err(|e| NozyError::AddressParsing(format!("Invalid address: {}", e)))?;
        
        let mut orchard_receiver = None;
        for item in ua.items() {
            if let zcash_address::unified::Receiver::Orchard(data) = item {
                orchard_receiver = Some(data);
                break;
            }
        }
        
        let orchard_raw = orchard_receiver
            .ok_or_else(|| NozyError::AddressParsing("Address does not contain Orchard receiver".to_string()))?;
        
        let orchard_address_result = orchard::Address::from_raw_address_bytes(&orchard_raw);
        let is_valid: bool = orchard_address_result.is_some().into();
        if !is_valid {
            return Err(NozyError::AddressParsing("Invalid Orchard address bytes".to_string()));
        }
        
        let mut seed_bytes = self.mnemonic.to_seed("").to_vec();
        let secure_seed = SecureSeed::new(seed_bytes.clone());
        
        let account_id = zcash_primitives::zip32::AccountId::try_from(0)
            .map_err(|e| NozyError::KeyDerivation(format!("Invalid account ID: {:?}", e)))?;
        
        let orchard_sk = SpendingKey::from_zip32_seed(secure_seed.as_bytes(), 133, account_id)
            .map_err(|e| NozyError::KeyDerivation(format!("Failed to derive Orchard spending key: {:?}", e)))?;
        
        let orchard_fvk = FullViewingKey::from(&orchard_sk);
        
        let ivk = orchard_fvk.to_ivk(orchard::keys::Scope::External);
        
        zeroize_bytes(&mut seed_bytes);
        
        Ok(ivk)
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

    pub fn get_password_hash(&self) -> Option<&String> {
        self.password_hash.as_ref()
    }

    pub fn set_password_hash(&mut self, hash: String) -> NozyResult<()> {
        PasswordHash::new(&hash)
            .map_err(|e| NozyError::Cryptographic(format!("Invalid password hash format: {}", e)))?;
        self.password_hash = Some(hash);
        Ok(())
    }

    pub fn derive_key_with_password(&self, password: &str, path: &str) -> NozyResult<XPrv> {
        if self.is_password_protected() {
            self.verify_password(password)?;
        }
        
        self.derive_key(path)
    }
}
