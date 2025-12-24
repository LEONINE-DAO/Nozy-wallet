// Secret Network Key Derivation
// Implements BIP44 key derivation for Secret Network (coin type 529)

use crate::error::{NozyError, NozyResult};
use crate::hd_wallet::HDWallet;
use bip32::{DerivationPath, XPrv, ChildNumber};
use secp256k1::{Secp256k1, SecretKey, PublicKey, Message, ecdsa::Signature};
use bech32::{self, ToBase32, Variant};
use sha2::{Sha256, Digest};
use ripemd::{Ripemd160, Digest as RipemdDigest};
use std::str::FromStr;

/// Secret Network BIP44 coin type
pub const SECRET_COIN_TYPE: u32 = 529;

/// Secret Network address prefix
pub const SECRET_ADDRESS_PREFIX: &str = "secret";

#[derive(Debug, Clone)]
pub struct SecretKeyDerivation {
    hd_wallet: HDWallet,
}

#[derive(Debug, Clone)]
pub struct SecretDerivationPath {
    pub account: u32,
    pub change: u32,  // 0 for external (receiving), 1 for internal (change)
    pub index: u32,
}

#[derive(Debug, Clone)]
pub struct SecretKeyPair {
    pub derivation_path: SecretDerivationPath,
    pub private_key: SecretKey,
    pub public_key: PublicKey,
    pub address: String,
}

impl SecretDerivationPath {
    /// Create BIP44 derivation path for Secret Network
    /// Format: m/44'/529'/account'/0/index
    pub fn to_bip44_path(&self) -> String {
        format!("m/44'/529'/{}/0/{}", self.account, self.index)
    }
    
    /// Create derivation path for HD wallet
    pub fn to_derivation_path(&self) -> NozyResult<DerivationPath> {
        let path_str = format!("m/44'/529'/{}/0/{}", self.account, self.index);
        DerivationPath::from_str(&path_str)
            .map_err(|e| NozyError::KeyDerivation(format!("Invalid derivation path: {}", e)))
    }
}

impl SecretKeyDerivation {
    pub fn new(hd_wallet: HDWallet) -> Self {
        Self { hd_wallet }
    }

    /// Derive Secret Network key pair from HD wallet
    pub fn derive_key_pair(&self, path: &SecretDerivationPath) -> NozyResult<SecretKeyPair> {
        // Get derived private key from HD wallet
        let derived_xprv = self.hd_wallet.derive_key(&path.to_bip44_path())?;
        
        // Extract secp256k1 private key
        let private_key_bytes = derived_xprv.private_key().to_bytes();
        let secp = Secp256k1::new();
        let private_key = SecretKey::from_slice(&private_key_bytes)
            .map_err(|e| NozyError::KeyDerivation(format!("Invalid private key: {}", e)))?;
        
        // Derive public key
        let public_key = PublicKey::from_secret_key(&secp, &private_key);
        
        // Generate address from public key
        let address = Self::public_key_to_address(&public_key)?;
        
        Ok(SecretKeyPair {
            derivation_path: path.clone(),
            private_key,
            public_key,
            address,
        })
    }

    /// Generate Secret Network address from public key
    /// Uses SHA256 + RIPEMD160 hash (Cosmos SDK standard), then bech32 encoding
    pub fn public_key_to_address(public_key: &PublicKey) -> NozyResult<String> {
        // Get compressed public key bytes (33 bytes)
        let pubkey_bytes = public_key.serialize();
        
        // Hash with SHA256
        let sha256_hash = Sha256::digest(&pubkey_bytes);
        
        // Then hash with RIPEMD160 (Cosmos SDK standard)
        let ripemd160_hash = Ripemd160::digest(&sha256_hash);
        
        // Encode with bech32
        let encoded = bech32::encode(
            SECRET_ADDRESS_PREFIX,
            ripemd160_hash.to_base32(),
            Variant::Bech32,
        )
        .map_err(|e| NozyError::KeyDerivation(format!("Failed to encode address: {}", e)))?;
        
        Ok(encoded)
    }

    /// Sign a message with the private key
    pub fn sign_message(key_pair: &SecretKeyPair, message: &[u8]) -> NozyResult<Vec<u8>> {
        let secp = Secp256k1::new();
        
        // Hash the message
        let mut hasher = Sha256::new();
        hasher.update(message);
        let message_hash = hasher.finalize();
        
        let msg = Message::from_slice(&message_hash)
            .map_err(|e| NozyError::KeyDerivation(format!("Invalid message: {}", e)))?;
        
        // Sign
        let signature = secp.sign_ecdsa(&msg, &key_pair.private_key);
        
        // Serialize signature (64 bytes: r || s)
        Ok(signature.serialize_compact().to_vec())
    }

    /// Verify a signature
    pub fn verify_signature(
        public_key: &PublicKey,
        message: &[u8],
        signature: &[u8],
    ) -> NozyResult<bool> {
        let secp = Secp256k1::new();
        
        // Hash the message
        let mut hasher = Sha256::new();
        hasher.update(message);
        let message_hash = hasher.finalize();
        
        let msg = Message::from_slice(&message_hash)
            .map_err(|e| NozyError::KeyDerivation(format!("Invalid message: {}", e)))?;
        
        // Parse signature
        let sig = Signature::from_compact(signature)
            .map_err(|e| NozyError::KeyDerivation(format!("Invalid signature: {}", e)))?;
        
        // Verify
        Ok(secp.verify_ecdsa(&msg, &sig, public_key).is_ok())
    }
}

impl HDWallet {
    /// Generate Secret Network address from HD wallet
    /// Uses BIP44 path: m/44'/529'/account'/0/index
    pub fn generate_secret_address(&self, account: u32, index: u32) -> NozyResult<String> {
        let path = SecretDerivationPath {
            account,
            change: 0, // External (receiving) address
            index,
        };
        
        let key_derivation = SecretKeyDerivation::new(self.clone());
        let key_pair = key_derivation.derive_key_pair(&path)?;
        
        Ok(key_pair.address)
    }

    /// Get Secret Network key pair for signing
    pub fn get_secret_key_pair(&self, account: u32, index: u32) -> NozyResult<SecretKeyPair> {
        let path = SecretDerivationPath {
            account,
            change: 0,
            index,
        };
        
        let key_derivation = SecretKeyDerivation::new(self.clone());
        key_derivation.derive_key_pair(&path)
    }
}
