use crate::error::{NozyError, NozyResult};
use crate::notes::SpendableNote;
use crate::orchard_tx::OrchardTransactionBuilder;
use crate::zebra_integration::ZebraClient;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct SignedTransaction {
    pub raw_transaction: Vec<u8>,
    pub txid: String,
}

pub struct ZcashTransactionBuilder {
    pub allow_mainnet_broadcast: bool,
    pub zebra_url: String,
}

impl ZcashTransactionBuilder {
    pub fn new() -> Self {
        Self {
            allow_mainnet_broadcast: false,
            zebra_url: "http://127.0.0.1:8232".to_string(),
        }
    }

    pub fn set_zebra_url(&mut self, url: &str) -> &mut Self {
        self.zebra_url = url.to_string();
        self
    }

    pub fn enable_mainnet_broadcast(&mut self) -> &mut Self {
        self.allow_mainnet_broadcast = true;
        self
    }

    pub async fn build_send_transaction(
        &self,
        zebra_client: &ZebraClient,
        spendable_notes: &[SpendableNote],
        recipient_address: &str,
        amount_zatoshis: u64,
        fee_zatoshis: u64,
        memo: Option<&[u8]>,
    ) -> NozyResult<SignedTransaction> {
        // Enforce privacy by validating address
        use crate::privacy::validate_shielded_address;
        validate_shielded_address(recipient_address)?;

        let total_available: u64 = spendable_notes
            .iter()
            .filter(|note| !note.orchard_note.spent)
            .map(|note| note.orchard_note.value)
            .sum();

        let total_needed = amount_zatoshis + fee_zatoshis;

        if total_available < total_needed {
            let amount_zec = amount_zatoshis as f64 / 100_000_000.0;
            let available_zec = total_available as f64 / 100_000_000.0;
            return Err(NozyError::InvalidOperation(format!(
                "Insufficient funds: need {:.8} ZEC, have {:.8} ZEC",
                amount_zec, available_zec
            )));
        }

        let orchard_builder = OrchardTransactionBuilder::new(true);
        let tx_data = orchard_builder
            .build_single_spend(
                &zebra_client,
                spendable_notes,
                recipient_address,
                amount_zatoshis,
                fee_zatoshis,
                memo,
            )
            .await?;
        let txid = self.calculate_txid(&tx_data)?;

        Ok(SignedTransaction {
            raw_transaction: tx_data,
            txid,
        })
    }

    fn calculate_txid(&self, tx_data: &[u8]) -> NozyResult<String> {
        let mut hasher = Sha256::new();
        hasher.update(tx_data);
        let hash1 = hasher.finalize();

        let mut hasher2 = Sha256::new();
        hasher2.update(hash1);
        let hash2 = hasher2.finalize();

        let mut txid_bytes = hash2.to_vec();
        txid_bytes.reverse();
        Ok(hex::encode(txid_bytes))
    }

    pub async fn broadcast_transaction(
        &self,
        zebra_client: &ZebraClient,
        transaction: &SignedTransaction,
    ) -> NozyResult<String> {
        if !self.allow_mainnet_broadcast {
            return Err(NozyError::InvalidOperation(
                "Broadcasting disabled".to_string(),
            ));
        }

        let raw_tx_hex = hex::encode(&transaction.raw_transaction);
        zebra_client.broadcast_transaction(&raw_tx_hex).await
    }
}
