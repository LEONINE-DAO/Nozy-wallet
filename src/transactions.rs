use crate::error::NozyResult;
use crate::hd_wallet::HDWallet;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionDetails {
    pub txid: String,
    pub amount: u64,
    pub fee: u64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedTransaction {
    pub raw_transaction: Vec<u8>,
    pub txid: String,
}

pub struct TransactionBuilder {
    #[allow(dead_code)]
    wallet: HDWallet,
}

impl TransactionBuilder {
    pub fn new(wallet: HDWallet) -> Self {
        Self { wallet }
    }

    pub async fn build_transaction(
        &self,
        recipient: &str,
        amount: u64,
        fee: u64,
    ) -> NozyResult<SignedTransaction> {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(recipient.as_bytes());
        hasher.update(&amount.to_le_bytes());
        hasher.update(&fee.to_le_bytes());
        hasher.update(&chrono::Utc::now().timestamp().to_le_bytes());
        let tx_hash = hasher.finalize();

        let mut raw_transaction = Vec::new();
        raw_transaction.extend_from_slice(&5u32.to_le_bytes());
        raw_transaction.extend_from_slice(&0u32.to_le_bytes());
        raw_transaction.extend_from_slice(&amount.to_le_bytes());
        raw_transaction.extend_from_slice(&fee.to_le_bytes());
        raw_transaction.extend_from_slice(recipient.as_bytes());

        Ok(SignedTransaction {
            raw_transaction,
            txid: hex::encode(tx_hash),
        })
    }

    pub async fn send_transaction(&self, transaction: &SignedTransaction) -> NozyResult<String> {
        if transaction.raw_transaction.len() < 16 {
            return Err(crate::error::NozyError::InvalidOperation(
                "Transaction too small to be valid".to_string(),
            ));
        }

        if transaction.txid.len() != 64 {
            return Err(crate::error::NozyError::InvalidOperation(
                "Invalid transaction ID format".to_string(),
            ));
        }

        Ok(transaction.txid.clone())
    }
}
