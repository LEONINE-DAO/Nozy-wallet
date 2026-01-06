use crate::error::{NozyError, NozyResult};
use crate::notes::SpendableNote;
use crate::transactions::{SignedTransaction, TransactionSignature, SignatureType};
use crate::key_management::{SecureSpendingKeyBytes, zeroize_bytes};

use redjubjub::{SigningKey, VerificationKey, Signature, Binding, SpendAuth};
use pallas::redjubjub::{PrivateKey, PublicKey};
use orchard::{ pczt::CtOption,
    keys::{SpendingKey, FullViewingKey},
    note::Nullifier,
    value::NoteValue,
};
use rand::Rng;
use std::collections::HashMap;

pub struct ZcashTransactionSigner {
}

impl ZcashTransactionSigner {
    pub fn new() -> Self {
        Self {}
    }

    pub fn create_orchard_signature(
        &self,
        spending_key: &SpendingKey,
        message: &[u8],
        rng: &mut impl Rng,
    ) -> NozyResult<[u8; 64]> {
        let key_bytes_array = spending_key.to_bytes();
        let mut key_bytes = key_bytes_array.to_vec();
        let secure_key_bytes = SecureSpendingKeyBytes::new(key_bytes.clone());
        
        let signing_key = SigningKey::<SpendAuth>::from_bytes(secure_key_bytes.as_bytes());
        let signature = signing_key.sign(rng, message);
        
        zeroize_bytes(&mut key_bytes);
        
        let sig_bytes: [u8; 64] = signature.into();
        Ok(sig_bytes)
    }

    pub fn create_binding_signature(
        &self,
        value_balance: i64,
        sighash: &[u8],
        rng: &mut impl Rng,
    ) -> NozyResult<[u8; 64]> {
        let signing_key = SigningKey::<Binding>::new(rng);
        let signature = signing_key.sign(rng, sighash);
        
        let sig_bytes: [u8; 64] = signature.into();
        Ok(sig_bytes)
    }

    pub fn verify_orchard_signature(
        &self,
        verification_key: &[u8; 32],
        message: &[u8],
        signature: &[u8; 64],
    ) -> NozyResult<bool> {
        let vk = VerificationKey::<SpendAuth>::try_from(*verification_key)
            .map_err(|e| NozyError::KeyDerivation(format!("Invalid verification key: {}", e)))?;
        
        let sig = Signature::<SpendAuth>::from(*signature);
        
        match vk.verify(message, &sig) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    pub fn verify_binding_signature(
        &self,
        verification_key: &[u8; 32],
        message: &[u8],
        signature: &[u8; 64],
    ) -> NozyResult<bool> {
        let vk = VerificationKey::<Binding>::try_from(*verification_key)
            .map_err(|e| NozyError::KeyDerivation(format!("Invalid verification key: {}", e)))?;
        
        let sig = Signature::<Binding>::from(*signature);
        
        match vk.verify(message, &sig) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    pub fn sign_transaction_with_notes(
        &self,
        spend_notes: &[SpendableNote],
        output_notes: &[NoteValue],
        sighash: &[u8],
        rng: &mut impl Rng,
    ) -> NozyResult<Vec<TransactionSignature>> {
        let mut signatures = Vec::new();
        
        for spend_note in spend_notes {
            let mut private_key_bytes = spend_note.private_key.clone();
            let secure_key_bytes = SecureSpendingKeyBytes::new(private_key_bytes.clone());
            
            let spending_key = SpendingKey::from_bytes(secure_key_bytes.as_bytes())
                .map_err(|e| NozyError::KeyDerivation(format!("Invalid spending key: {}", e)))?;
            
            let signature_bytes = self.create_orchard_signature(&spending_key, sighash, rng)?;
            
            let full_viewing_key = FullViewingKey::from(&spending_key);
            let verification_key_bytes: [u8; 32] = full_viewing_key.to_bytes();
            
            zeroize_bytes(&mut private_key_bytes);
            
            let signature = TransactionSignature {
                signature_type: SignatureType::Orchard,
                data: signature_bytes.to_vec(),
                public_key: verification_key_bytes.to_vec(),
            };
            
            signatures.push(signature);
        }
        
        Ok(signatures)
    }

    pub fn create_note_signature(
        &self,
        spending_key: &SpendingKey,
        nullifier: &Nullifier,
        rk: &[u8; 32],
        rng: &mut impl Rng,
    ) -> NozyResult<[u8; 64]> {
        let message = self.create_note_signature_message(nullifier, rk);
        self.create_orchard_signature(spending_key, &message, rng)
    }

    fn create_note_signature_message(
        &self,
        nullifier: &Nullifier,
        rk: &[u8; 32],
    ) -> Vec<u8> {
        let mut message = Vec::new();
        message.extend_from_slice(&nullifier.to_bytes());
        message.extend_from_slice(rk);
        message
    }

    pub fn derive_randomized_key(
        &self,
        spending_key: &SpendingKey,
        alpha: &[u8; 32],
        rng: &mut impl Rng,
    ) -> NozyResult<[u8; 32]> {
        let full_viewing_key = FullViewingKey::from(spending_key);
        let rk = full_viewing_key.to_rk(spending_key, rng);
        Ok(rk.to_bytes())
    }

    pub fn create_authorization_signature(
        &self,
        spending_key: &SpendingKey,
        bundle_hash: &[u8; 32],
        rng: &mut impl Rng,
    ) -> NozyResult<[u8; 64]> {
        self.create_orchard_signature(spending_key, bundle_hash, rng)
    }

    pub fn batch_verify_signatures(
        &self,
        verification_keys: &[[u8; 32]],
        messages: &[Vec<u8>],
        signatures: &[[u8; 64]],
    ) -> NozyResult<Vec<bool>> {
        if verification_keys.len() != messages.len() || messages.len() != signatures.len() {
            return Err(NozyError::KeyDerivation("Mismatched batch sizes".to_string()));
        }
        
        let mut results = Vec::new();
        for i in 0..verification_keys.len() {
            let result = self.verify_orchard_signature(
                &verification_keys[i],
                &messages[i],
                &signatures[i],
            )?;
            results.push(result);
        }
        
        Ok(results)
    }

    pub fn create_spend_auth_signature(
        &self,
        spending_key: &SpendingKey,
        message: &[u8],
        rng: &mut impl Rng,
    ) -> NozyResult<[u8; 64]> {
        self.create_orchard_signature(spending_key, message, rng)
    }

    pub fn create_bundle_authorization(
        &self,
        bundle_hash: &[u8; 32],
        value_balance: i64,
        rng: &mut impl Rng,
    ) -> NozyResult<[u8; 64]> {
        let mut message = Vec::new();
        message.extend_from_slice(bundle_hash);
        message.extend_from_slice(&value_balance.to_le_bytes());
        
        self.create_binding_signature(value_balance, &message, rng)
    }
}
