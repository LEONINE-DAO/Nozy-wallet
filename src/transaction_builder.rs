use crate::error::{NozyError, NozyResult};
use crate::notes::SpendableNote;
use crate::orchard_tx::{OrchardTransactionBuilder, ZebraJsonRpcOrchardWitnessProvider};
use crate::zebra_integration::ZebraClient;
use zcash_address::unified::{Container, Encoding};

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

    /// Build an Orchard shielded send (Orchard-only wallet).
    pub async fn build_send_transaction(
        &self,
        zebra_client: &ZebraClient,
        spendable_notes: &[SpendableNote],
        recipient_address: &str,
        amount_zatoshis: u64,
        fee_zatoshis: u64,
        memo: Option<&[u8]>,
    ) -> NozyResult<SignedTransaction> {
        use crate::privacy::validate_shielded_address;
        validate_shielded_address(recipient_address)?;

        let (_, decoded) =
            zcash_address::unified::Address::decode(recipient_address).map_err(|e| {
                NozyError::InvalidOperation(format!("Invalid recipient address: {}", e))
            })?;

        let has_orchard = decoded
            .items()
            .iter()
            .any(|i| matches!(i, zcash_address::unified::Receiver::Orchard(_)));

        if !has_orchard {
            return Err(NozyError::InvalidOperation(
                "Recipient must include an Orchard receiver (ZIP-316). Sapling-only addresses are not supported."
                    .to_string(),
            ));
        }

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
                "Insufficient Orchard funds: need {:.8} ZEC, have {:.8} ZEC",
                amount_zec, available_zec
            )));
        }

        let orchard_builder = OrchardTransactionBuilder::new(true);
        let built = orchard_builder
            .build_single_spend(
                zebra_client,
                &ZebraJsonRpcOrchardWitnessProvider,
                spendable_notes,
                recipient_address,
                amount_zatoshis,
                fee_zatoshis,
                memo,
            )
            .await?;

        Ok(SignedTransaction {
            raw_transaction: built.raw_transaction,
            txid: built.txid,
        })
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
