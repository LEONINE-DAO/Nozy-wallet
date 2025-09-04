use crate::error::{NozyError, NozyResult};
use bip39::Mnemonic;
use bip32::{XPrv, XPub, DerivationPath};
use sha2::{Sha256, Digest};
use hex;
use std::str::FromStr;
use rand::RngCore;

// Nozy Unified Address creation
use zcash_address::{
    unified::{Address as UnifiedAddress, Receiver, Encoding},
    ZcashAddress,
};
use zcash_protocol::consensus::NetworkType;
use orchard::{
    keys::{SpendingKey, FullViewingKey, IncomingViewingKey, Scope, DiversifierIndex},
    Address as OrchardAddress,
};
use zcash_primitives::zip32::AccountId;

#[derive(Debug, Clone)]
pub struct HDWallet {
    mnemonic: Mnemonic,
    master_key: XPrv,
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
        // Use proper ZIP-32 Orchard derivation as specified in the official documentation
        // This follows the exact specification from https://zcash.github.io/orchard/design/keys.html
        
        // Get the mnemonic seed for Orchard key derivation
        let seed = self.mnemonic.to_seed("");
        
        // Use the official Orchard ZIP-32 derivation with proper AccountId
        // This is the correct coin type for Zcash (133) as specified in ZIP-32
        let account_id = AccountId::try_from(account)
            .map_err(|e| NozyError::KeyDerivation(format!("Invalid account ID: {:?}", e)))?;
        
        // Generate Orchard spending key using the official from_zip32_seed method
        // This follows the hardened-only derivation specified in ZIP-32 for Orchard
        let orchard_sk = SpendingKey::from_zip32_seed(&seed, 133, account_id)
            .map_err(|e| NozyError::KeyDerivation(format!("Failed to derive Orchard spending key: {:?}", e)))?;
        
        // Generate Orchard address from spending key using proper diversification
        let orchard_fvk = FullViewingKey::from(&orchard_sk);
        let diversifier_index = DiversifierIndex::from(index);
        let orchard_address: OrchardAddress = orchard_fvk.address_at(diversifier_index, Scope::External);
        
        // Convert to proper Unified Address format as specified in ZIP-316
        // This creates a valid UA that wallets like YWallet will accept
        self.create_unified_address(orchard_address)
    }
    
    /// Create a proper Unified Address containing the Orchard receiver
    /// This uses the OFFICIAL zcash_address API for valid ZIP-316 addresses
    fn create_unified_address(&self, orchard_address: OrchardAddress) -> NozyResult<String> {
        // Get the raw Orchard address bytes (43 bytes)
        let orchard_raw = orchard_address.to_raw_address_bytes();
        
        // Create Orchard receiver using the official API
        let orchard_receiver = Receiver::Orchard(orchard_raw);
        
        // Create unified address using the official zcash_address API
        // This handles F4Jumble, ZIP-316 encoding, and all validation automatically
        let ua = UnifiedAddress::try_from_items(vec![orchard_receiver])
            .map_err(|e| NozyError::InvalidOperation(format!("Failed to create Unified Address: {:?}", e)))?;
        
        // Encode for mainnet
        let network = NetworkType::Main;
        let encoded = ua.encode(&network);
        
        Ok(encoded)
    }
    
    /// Get master key with password (for compatibility)
    pub fn get_master_key(&self, _password: &str) -> NozyResult<XPrv> {
        // For now, ignore password - in production this would decrypt with password
        Ok(self.master_key.clone())
    }
    
    /// Derive private key for a specific note (for spending)
    pub fn derive_private_key_for_note(&self, note: &crate::notes::OrchardNote) -> NozyResult<Vec<u8>> {
        // Derive spending key for the note based on its nullifier
        use sha2::{Sha256, Digest};
        
        let seed = self.mnemonic.to_seed("");
        let mut hasher = Sha256::new();
        hasher.update(&seed);
        hasher.update(&note.nullifier.to_bytes());
        hasher.update(b"spending_key");
        
        let private_key_hash = hasher.finalize();
        Ok(private_key_hash.to_vec())
    }

    pub fn decrypt_note(&self, _encrypted_note: &[u8], _address: &str) -> NozyResult<String> {
        // This is a placeholder - in production implement proper decryption
        Err(NozyError::KeyDerivation("Decryption not implemented in this example".to_string()))
    }
}
