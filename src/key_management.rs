// Secure key management with zeroization
// This module provides utilities for securely handling and zeroizing sensitive keys

use zeroize::{Zeroize, ZeroizeOnDrop};
use crate::error::{NozyError, NozyResult};

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecureSeed {
    #[zeroize(on_drop)]
    seed: Vec<u8>,
}

impl SecureSeed {
    pub fn new(seed: Vec<u8>) -> Self {
        Self { seed }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.seed
    }

    pub fn len(&self) -> usize {
        self.seed.len()
    }
}

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecureSpendingKeyBytes {
    #[zeroize(on_drop)]
    key_bytes: Vec<u8>,
}

impl SecureSpendingKeyBytes {
    pub fn new(key_bytes: Vec<u8>) -> Self {
        Self { key_bytes }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.key_bytes
    }
    
    pub fn len(&self) -> usize {
        self.key_bytes.len()
    }
}

pub fn zeroize_bytes(bytes: &mut [u8]) {
    use zeroize::Zeroize;
    bytes.zeroize();
}

pub fn create_secure_seed_from_mnemonic(mnemonic: &bip39::Mnemonic, passphrase: &str) -> SecureSeed {
    let seed = mnemonic.to_seed(passphrase);
    SecureSeed::new(seed.to_vec())
}

pub fn derive_and_use_spending_key<F, T>(
    seed: &[u8],
    account_id: u32,
    f: F,
) -> NozyResult<T>
where
    F: FnOnce(&orchard::keys::SpendingKey) -> NozyResult<T>,
{
    use orchard::keys::SpendingKey;
    use zcash_primitives::zip32::AccountId;

    let account_id = AccountId::try_from(account_id)
        .map_err(|e| NozyError::KeyDerivation(format!("Invalid account ID: {:?}", e)))?;
    
    let spending_key = SpendingKey::from_zip32_seed(seed, 133, account_id)
        .map_err(|e| NozyError::KeyDerivation(format!("Failed to derive Orchard spending key: {:?}", e)))?;

    let result = f(&spending_key)?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_seed_zeroization() {
        let mut seed = SecureSeed::new(vec![1, 2, 3, 4, 5]);
        assert_eq!(seed.as_bytes(), &[1, 2, 3, 4, 5]);
        
        drop(seed);
    }

    #[test]
    fn test_zeroize_bytes() {
        let mut bytes = vec![1, 2, 3, 4, 5];
        zeroize_bytes(&mut bytes);
        assert_eq!(bytes, vec![0, 0, 0, 0, 0]);
    }
}

