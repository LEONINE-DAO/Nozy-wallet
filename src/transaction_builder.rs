use crate::error::{NozyError, NozyResult};
use crate::fee_policy::{
    is_expiry_consensus_error, OrchardSendFeeShape, PilotSendOptions,
    PILOT_EXPIRY_MAX_REBUILD_ATTEMPTS,
};
use crate::ironwood_tx::{
    build_single_ironwood_spend, select_single_ironwood_spend_note,
    ZebraJsonRpcIronwoodWitnessProvider,
};
use crate::notes::SpendableNote;
use crate::orchard_tx::{
    select_single_spend_note, OrchardTransactionBuilder, ZebraJsonRpcOrchardWitnessProvider,
};
use crate::shielded_pool::ShieldedPool;
use crate::zebra_integration::ZebraClient;
use zcash_address::unified::{Container, Encoding};

#[derive(Debug, Clone)]
pub struct SignedTransaction {
    pub raw_transaction: Vec<u8>,
    pub txid: String,
    /// On-chain expiry height encoded in the transaction (for history / speed-up).
    pub expiry_height: u32,
    /// Canonical nullifier hex of the note that was spent (for local balance bookkeeping).
    pub spent_nullifier_hex: Option<String>,
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
        pilot: PilotSendOptions,
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
                "Insufficient shielded funds: need {:.8} ZEC, have {:.8} ZEC",
                amount_zec, available_zec
            )));
        }

        let chain_tip = zebra_client.get_best_block_height().await?;
        let info = zebra_client.get_blockchain_info().await?;
        let testnet = info
            .get("chain")
            .and_then(|v| v.as_str())
            .is_some_and(|chain| matches!(chain, "test" | "testnet" | "regtest"));

        if crate::ironwood::is_ironwood_active(chain_tip, testnet) {
            let ironwood_notes: Vec<&SpendableNote> = spendable_notes
                .iter()
                .filter(|n| !n.orchard_note.spent && n.pool == ShieldedPool::Ironwood)
                .collect();
            if ironwood_notes.is_empty() {
                return Err(NozyError::InvalidOperation(
                    "Ironwood (NU6.3) is active but no spendable Ironwood notes were found. \
                     Run `nozy sync --to-tip` to index Ironwood pool outputs."
                        .to_string(),
                ));
            }
            let ironwood_refs: Vec<SpendableNote> =
                ironwood_notes.iter().map(|n| (*n).clone()).collect();
            let spend_note =
                select_single_ironwood_spend_note(&ironwood_refs, amount_zatoshis, fee_zatoshis)?;
            crate::send_readiness::ensure_witness_fresh_for_send(spend_note, chain_tip)?;
            let has_change =
                spend_note.orchard_note.value > amount_zatoshis.saturating_add(fee_zatoshis);
            let shape = OrchardSendFeeShape::single_spend_send(has_change, memo);
            let expected_fee = crate::fee_policy::fee_zatoshis(&shape, pilot.priority);
            if fee_zatoshis < expected_fee {
                return Err(NozyError::InvalidOperation(format!(
                    "Fee {} zats is below ZIP-317 minimum {} zats for this transaction shape",
                    fee_zatoshis, expected_fee
                )));
            }

            let built = build_single_ironwood_spend(
                zebra_client,
                &ZebraJsonRpcIronwoodWitnessProvider,
                &ironwood_refs,
                recipient_address,
                amount_zatoshis,
                fee_zatoshis,
                memo,
                pilot.expiry_delta_blocks,
            )
            .await?;

            let fvk = orchard::keys::FullViewingKey::from(&spend_note.spending_key);
            let spent_nullifier_hex = Some(hex::encode(
                spend_note.orchard_note.note.nullifier(&fvk).to_bytes(),
            ));

            return Ok(SignedTransaction {
                raw_transaction: built.raw_transaction,
                txid: built.txid,
                expiry_height: built.expiry_height,
                spent_nullifier_hex,
            });
        }

        let total_available_orchard: u64 = spendable_notes
            .iter()
            .filter(|note| !note.orchard_note.spent && note.pool == ShieldedPool::Orchard)
            .map(|note| note.orchard_note.value)
            .sum();

        if total_available_orchard < total_needed {
            let amount_zec = amount_zatoshis as f64 / 100_000_000.0;
            let available_zec = total_available_orchard as f64 / 100_000_000.0;
            return Err(NozyError::InvalidOperation(format!(
                "Insufficient Orchard funds: need {:.8} ZEC, have {:.8} ZEC",
                amount_zec, available_zec
            )));
        }

        let orchard_notes: Vec<SpendableNote> = spendable_notes
            .iter()
            .filter(|n| !n.orchard_note.spent && n.pool == ShieldedPool::Orchard)
            .cloned()
            .collect();
        let spend_note = select_single_spend_note(&orchard_notes, amount_zatoshis, fee_zatoshis)?;
        crate::send_readiness::ensure_witness_fresh_for_send(spend_note, chain_tip)?;
        let has_change =
            spend_note.orchard_note.value > amount_zatoshis.saturating_add(fee_zatoshis);
        let shape = OrchardSendFeeShape::single_spend_send(has_change, memo);
        let expected_fee = crate::fee_policy::fee_zatoshis(&shape, pilot.priority);
        if fee_zatoshis < expected_fee {
            return Err(NozyError::InvalidOperation(format!(
                "Fee {} zats is below ZIP-317 minimum {} zats for this transaction shape",
                fee_zatoshis, expected_fee
            )));
        }

        let orchard_builder = OrchardTransactionBuilder::new(true);
        let built = orchard_builder
            .build_single_spend(
                zebra_client,
                &ZebraJsonRpcOrchardWitnessProvider,
                &orchard_notes,
                recipient_address,
                amount_zatoshis,
                fee_zatoshis,
                memo,
                pilot.expiry_delta_blocks,
            )
            .await?;

        let fvk = orchard::keys::FullViewingKey::from(&spend_note.spending_key);
        let spent_nullifier_hex = Some(hex::encode(
            spend_note.orchard_note.note.nullifier(&fvk).to_bytes(),
        ));

        Ok(SignedTransaction {
            raw_transaction: built.raw_transaction,
            txid: built.txid,
            expiry_height: built.expiry_height,
            spent_nullifier_hex,
        })
    }

    /// Build, then broadcast with automatic rebuild when the pilot expiry window is exceeded.
    pub async fn build_and_broadcast_send_transaction(
        &self,
        zebra_client: &ZebraClient,
        spendable_notes: &[SpendableNote],
        recipient_address: &str,
        amount_zatoshis: u64,
        fee_zatoshis: u64,
        memo: Option<&[u8]>,
        pilot: PilotSendOptions,
    ) -> NozyResult<SignedTransaction> {
        if !self.allow_mainnet_broadcast {
            return Err(NozyError::InvalidOperation(
                "Broadcasting disabled".to_string(),
            ));
        }

        let mut last_err: Option<NozyError> = None;

        for attempt in 1..=PILOT_EXPIRY_MAX_REBUILD_ATTEMPTS {
            if attempt > 1 {
                eprintln!(
                    "⚠️  Broadcast hit pilot expiry; rebuilding send ({attempt}/{})",
                    PILOT_EXPIRY_MAX_REBUILD_ATTEMPTS
                );
            }

            let transaction = self
                .build_send_transaction(
                    zebra_client,
                    spendable_notes,
                    recipient_address,
                    amount_zatoshis,
                    fee_zatoshis,
                    memo,
                    pilot,
                )
                .await?;

            match self.broadcast_transaction(zebra_client, &transaction).await {
                Ok(_network_txid) => return Ok(transaction),
                Err(e) => {
                    let msg = e.to_string();
                    if is_expiry_consensus_error(&msg)
                        && attempt < PILOT_EXPIRY_MAX_REBUILD_ATTEMPTS
                    {
                        last_err = Some(e);
                        continue;
                    }
                    return Err(e);
                }
            }
        }

        Err(last_err.unwrap_or_else(|| {
            NozyError::InvalidOperation(
                "Failed to broadcast within pilot expiry window after rebuild attempts".to_string(),
            )
        }))
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
