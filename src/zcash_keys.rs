use crate::error::{NozyError, NozyResult};
use crate::hd_wallet::HDWallet;
use bip32::{DerivationPath, XPrv, XPub};
use sha2::{Sha256, Digest};
use std::str::FromStr;
use hex;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone)]
pub struct ZcashKeyDerivation {
    hd_wallet: HDWallet,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ZcashAddressType {
    Orchard,
    Sapling,
    Transparent,
    Unified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZcashDerivationPath {
    pub address_type: ZcashAddressType,
    pub account: u32,
    pub diversifier_index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZcashSpendingKey {
    pub address_type: ZcashAddressType,
    pub derivation_path: ZcashDerivationPath,
    pub private_key: Vec<u8>,
    pub public_key: Vec<u8>,
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZcashViewingKey {
    pub address_type: ZcashAddressType,
    pub derivation_path: ZcashDerivationPath,
    pub viewing_key: Vec<u8>,
    pub address: String,
}

impl ZcashKeyDerivation {
    pub fn new(hd_wallet: HDWallet) -> Self {
        Self { hd_wallet }
    }

    pub fn generate_derivation_path(&self, address_type: ZcashAddressType, account: u32, diversifier_index: u32) -> ZcashDerivationPath {
        ZcashDerivationPath {
            address_type,
            account,
            diversifier_index,
        }
    }

    pub fn path_to_string(&self, path: &ZcashDerivationPath) -> String {
        match path.address_type {
            ZcashAddressType::Orchard => format!("m/32/133'/{}/{}", path.account, path.diversifier_index),
            ZcashAddressType::Sapling => format!("m/32/133'/{}/{}", path.account, path.diversifier_index),
            ZcashAddressType::Transparent => format!("m/44/133'/{}/0/{}", path.account, path.diversifier_index),
            ZcashAddressType::Unified => format!("m/32/133'/{}/{}", path.account, path.diversifier_index),
        }
    }

    pub fn derive_spending_key(&self, path: &ZcashDerivationPath, password: &str) -> NozyResult<ZcashSpendingKey> {
        let master_key = self.hd_wallet.get_master_key(password)?;
        
        let mut derivation_path = Vec::new();
        
        derivation_path.push(bip32::ChildNumber::new(133, true)?);
        
        derivation_path.push(bip32::ChildNumber::new(path.account, true)?);
        
        match path.address_type {
            ZcashAddressType::Orchard => {
                derivation_path.push(bip32::ChildNumber::new(0, true)?);
                derivation_path.push(bip32::ChildNumber::new(path.diversifier_index, false)?);
            },
            ZcashAddressType::Sapling => {
                derivation_path.push(bip32::ChildNumber::new(1, true)?);
                derivation_path.push(bip32::ChildNumber::new(path.diversifier_index, false)?);
            },
            ZcashAddressType::Transparent => {
                derivation_path.push(bip32::ChildNumber::new(0, false)?);
                derivation_path.push(bip32::ChildNumber::new(path.diversifier_index, false)?);
            },
            ZcashAddressType::Unified => {
                derivation_path.push(bip32::ChildNumber::new(3, true)?);
                derivation_path.push(bip32::ChildNumber::new(path.diversifier_index, false)?);
            },
        }

        let mut child_key = master_key;
        for child_number in &derivation_path {
            child_key = child_key.derive_child(*child_number)
                .map_err(|e| NozyError::InvalidOperation(format!("Failed to derive key: {}", e)))?;
        }

        let private_key = child_key.private_key().to_bytes().to_vec();
        let public_key = child_key.public_key().to_bytes().to_vec();
        
        let address = self.generate_address_from_key(&child_key, path.address_type)?;

        Ok(ZcashSpendingKey {
            address_type: path.address_type,
            derivation_path: path.clone(),
            private_key,
            public_key,
            address,
        })
    }

    pub fn generate_address_from_key(&self, key: &XPrv, address_type: ZcashAddressType) -> NozyResult<String> {
        match address_type {
            ZcashAddressType::Orchard => self.generate_orchard_address(key),
            ZcashAddressType::Sapling => self.generate_sapling_address(key),
            ZcashAddressType::Transparent => self.generate_transparent_address(key),
            ZcashAddressType::Unified => self.generate_unified_address(key),
        }
    }

    pub fn generate_orchard_address(&self, key: &XPrv) -> NozyResult<String> {
        let public_key = key.public_key();
        let address_bytes = Sha256::digest(public_key.to_bytes());
        Ok(format!("u1{}", bs58::encode(&address_bytes[..20]).into_string()))
    }

    pub fn generate_sapling_address(&self, key: &XPrv) -> NozyResult<String> {
        let public_key = key.public_key();
        let address_bytes = Sha256::digest(public_key.to_bytes());
        Ok(format!("zs{}", bs58::encode(&address_bytes[..20]).into_string()))
    }

    pub fn generate_transparent_address(&self, key: &XPrv) -> NozyResult<String> {
        let public_key = key.public_key();
        let address_bytes = Sha256::digest(public_key.to_bytes());
        Ok(format!("t1{}", bs58::encode(&address_bytes[..20]).into_string()))
    }

    pub fn generate_unified_address(&self, key: &XPrv) -> NozyResult<String> {
        let public_key = key.public_key();
        let address_bytes = Sha256::digest(public_key.to_bytes());
        Ok(format!("u1{}", bs58::encode(&address_bytes[..20]).into_string()))
    }

    pub fn generate_note_spending_key(&self, spending_key: &ZcashSpendingKey, note_commitment: &[u8], rseed: &[u8]) -> NozyResult<Vec<u8>> {
        let mut hasher = Sha256::new();
        hasher.update(&spending_key.private_key);
        hasher.update(note_commitment);
        hasher.update(rseed);
        Ok(hasher.finalize().to_vec())
    }

    pub fn generate_note_nullifier(&self, note_spending_key: &[u8], note_commitment: &[u8]) -> NozyResult<Vec<u8>> {
        let mut hasher = Sha256::new();
        hasher.update(note_spending_key);
        hasher.update(note_commitment);
        Ok(hasher.finalize().to_vec())
    }

    pub fn generate_note_commitment(&self, note_value: u64, note_recipient: &[u8], note_rseed: &[u8]) -> NozyResult<Vec<u8>> {
        let mut hasher = Sha256::new();
        hasher.update(&note_value.to_le_bytes());
        hasher.update(note_recipient);
        hasher.update(note_rseed);
        Ok(hasher.finalize().to_vec())
    }
} 