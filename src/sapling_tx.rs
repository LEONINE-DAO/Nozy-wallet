//! Sapling shielded sends via `zcash_primitives::transaction::Builder` (ZIP-317 fees, Groth16 proofs).

use crate::error::{NozyError, NozyResult};
use crate::notes::NoteScanner;
use crate::sapling_notes::SpendableSaplingNote;
use crate::sapling_tree_codec::sapling_incremental_witness_from_bytes;
use crate::sapling_witness::{
    advance_sapling_witness_with_nodes, node_from_cmu_bytes, sapling_merkle_path_from_witness,
    sapling_witness_root_matches_anchor,
};
use crate::zebra_integration::ZebraClient;
use async_trait::async_trait;
use core::convert::Infallible;
use rand::rngs::OsRng;
use sapling::zip32::ExtendedSpendingKey;
use sapling::{Anchor as SaplingAnchor, PaymentAddress};
use zcash_address::unified::{Container, Encoding};
use zcash_primitives::transaction::builder::{BuildConfig, Builder};
use zcash_primitives::transaction::fees::zip317::FeeRule as Zip317FeeRule;
use zcash_proofs::prover::LocalTxProver;
use zcash_protocol::consensus::{BlockHeight, NetworkType, Parameters, MAIN_NETWORK, TEST_NETWORK};
use zcash_protocol::memo::MemoBytes;
use zcash_protocol::value::Zatoshis;
use zcash_transparent::builder::TransparentSigningSet;
use zip32::Scope;

/// Result of building a Sapling-only v5 transaction.
#[derive(Debug, Clone)]
pub struct SaplingBuiltSpend {
    pub raw_transaction: Vec<u8>,
    pub txid: String,
}

#[async_trait]
pub trait SaplingWitnessProvider: Send + Sync {
    async fn prepare_spend_anchor_and_path(
        &self,
        zebra: &ZebraClient,
        note: &SpendableSaplingNote,
        anchor_height: u32,
    ) -> NozyResult<(SaplingAnchor, sapling::MerklePath)>;
}

pub struct ZebraJsonRpcSaplingWitnessProvider;

async fn advance_sapling_witness_from_zebra_blocks(
    witness: &mut crate::sapling_tree_codec::SaplingIncrementalWitness,
    zebra: &ZebraClient,
    from_height_exclusive: u32,
    to_height_inclusive: u32,
) -> NozyResult<()> {
    if from_height_exclusive >= to_height_inclusive {
        return Ok(());
    }
    for h in (from_height_exclusive + 1)..=to_height_inclusive {
        let hash = zebra.get_block_hash(h).await?;
        let block_data = zebra.get_block_by_hash(&hash, 2).await?;
        let v = serde_json::to_value(&block_data)
            .map_err(|e| NozyError::InvalidOperation(format!("block JSON: {}", e)))?;
        let txs = NoteScanner::parse_block_data(&v, h)?;
        for tx in &txs {
            for out in &tx.sapling_outputs {
                let n = node_from_cmu_bytes(&out.cmu)?;
                advance_sapling_witness_with_nodes(witness, std::iter::once(n))?;
            }
        }
    }
    Ok(())
}

#[async_trait]
impl SaplingWitnessProvider for ZebraJsonRpcSaplingWitnessProvider {
    async fn prepare_spend_anchor_and_path(
        &self,
        zebra: &ZebraClient,
        note: &SpendableSaplingNote,
        anchor_height: u32,
    ) -> NozyResult<(SaplingAnchor, sapling::MerklePath)> {
        let witness_hex = note
            .sapling_incremental_witness_hex
            .as_ref()
            .ok_or_else(|| {
                NozyError::InvalidOperation(
                    "Missing Sapling incremental witness on note: rescan with JSON-RPC Zebra."
                        .to_string(),
                )
            })?;
        let bytes = hex::decode(witness_hex).map_err(|e| {
            NozyError::InvalidOperation(format!("sapling_incremental_witness_hex decode: {}", e))
        })?;
        let mut witness = sapling_incremental_witness_from_bytes(&bytes)?;

        let stored_tip = note.sapling_witness_tip_height.unwrap_or(0);
        if stored_tip < anchor_height {
            advance_sapling_witness_from_zebra_blocks(
                &mut witness,
                zebra,
                stored_tip,
                anchor_height,
            )
            .await?;
        }

        let ts = zebra.get_sapling_tree_state(anchor_height).await?;
        let mut anchor_arr = [0u8; 32];
        anchor_arr.copy_from_slice(&ts.anchor);
        if !sapling_witness_root_matches_anchor(&witness, &anchor_arr) {
            return Err(NozyError::InvalidOperation(
                "Sapling witness does not match z_gettreestate (rescan or wait for sync)."
                    .to_string(),
            ));
        }

        sapling_merkle_path_from_witness(&witness)
    }
}

pub struct SaplingTransactionBuilder {
    prover: LocalTxProver,
}

impl SaplingTransactionBuilder {
    pub fn new() -> Self {
        Self {
            prover: LocalTxProver::bundled(),
        }
    }

    /// Build, prove, and sign a single-note Sapling v5 transaction.
    pub async fn build_single_spend(
        &self,
        zebra_client: &ZebraClient,
        witness_provider: &dyn SaplingWitnessProvider,
        spendable_notes: &[SpendableSaplingNote],
        recipient_address: &str,
        amount_zatoshis: u64,
        fee_zatoshis: u64,
        memo: Option<&[u8]>,
    ) -> NozyResult<SaplingBuiltSpend> {
        let (network_type, recipient_decoded) =
            zcash_address::unified::Address::decode(recipient_address).map_err(|e| {
                NozyError::InvalidOperation(format!("Invalid recipient address: {}", e))
            })?;

        let total_input: u64 = spendable_notes.iter().map(|n| n.sapling_note.value).sum();
        let change_amount = total_input.saturating_sub(amount_zatoshis + fee_zatoshis);
        if total_input < amount_zatoshis + fee_zatoshis {
            return Err(NozyError::InvalidOperation(format!(
                "Insufficient Sapling funds: have {} zatoshis, need {}",
                total_input,
                amount_zatoshis + fee_zatoshis
            )));
        }

        let tip_height = zebra_client.get_best_block_height().await?;
        let sn = &spendable_notes[0];
        let target_height = BlockHeight::from_u32(tip_height);

        let (anchor, merkle_path) = witness_provider
            .prepare_spend_anchor_and_path(zebra_client, sn, tip_height)
            .await?;

        let dfvk = sn.extsk.to_diversifiable_full_viewing_key();
        let fvk = dfvk.fvk().clone();

        let mut recipient_pa: Option<PaymentAddress> = None;
        for item in recipient_decoded.items() {
            if let zcash_address::unified::Receiver::Sapling(data) = item {
                if data.len() == 43 {
                    let mut b = [0u8; 43];
                    b.copy_from_slice(&data);
                    recipient_pa = PaymentAddress::from_bytes(&b);
                    break;
                }
            }
        }
        let to = recipient_pa.ok_or_else(|| {
            NozyError::AddressParsing("No Sapling receiver in unified address.".to_string())
        })?;

        let build_config = BuildConfig::Standard {
            sapling_anchor: Some(anchor),
            orchard_anchor: None,
        };

        let ovk = Some(dfvk.to_ovk(Scope::External));
        let memo_field = if let Some(m) = memo {
            let mut a = [0u8; 512];
            let l = m.len().min(512);
            a[..l].copy_from_slice(&m[..l]);
            MemoBytes::from_bytes(&a).unwrap_or_else(|_| MemoBytes::empty())
        } else {
            MemoBytes::empty()
        };

        match network_type {
            NetworkType::Main => self.build_spend_inner(
                MAIN_NETWORK,
                target_height,
                build_config,
                sn,
                fvk,
                dfvk,
                merkle_path,
                to,
                ovk,
                amount_zatoshis,
                change_amount,
                memo_field,
            ),
            NetworkType::Test | NetworkType::Regtest => self.build_spend_inner(
                TEST_NETWORK,
                target_height,
                build_config,
                sn,
                fvk,
                dfvk,
                merkle_path,
                to,
                ovk,
                amount_zatoshis,
                change_amount,
                memo_field,
            ),
        }
    }

    fn build_spend_inner<P: Parameters + Copy>(
        &self,
        params: P,
        target_height: BlockHeight,
        build_config: BuildConfig,
        sn: &SpendableSaplingNote,
        fvk: sapling::keys::FullViewingKey,
        _dfvk: sapling::zip32::DiversifiableFullViewingKey,
        merkle_path: sapling::MerklePath,
        to: PaymentAddress,
        ovk: Option<sapling::keys::OutgoingViewingKey>,
        amount_zatoshis: u64,
        change_amount: u64,
        memo_field: MemoBytes,
    ) -> NozyResult<SaplingBuiltSpend> {
        let mut builder = Builder::new(params, target_height, build_config);

        let note = sn.sapling_note.note.clone();
        builder
            .add_sapling_spend::<Infallible>(fvk, note, merkle_path)
            .map_err(|e| NozyError::InvalidOperation(format!("add_sapling_spend: {}", e)))?;

        builder
            .add_sapling_output::<Infallible>(
                ovk,
                to,
                Zatoshis::from_u64(amount_zatoshis)
                    .map_err(|e| NozyError::InvalidOperation(format!("amount: {:?}", e)))?,
                memo_field,
            )
            .map_err(|e| NozyError::InvalidOperation(format!("add_sapling_output: {}", e)))?;

        if change_amount > 0 {
            let (_, change_addr) = sn.extsk.default_address();
            builder
                .add_sapling_output::<Infallible>(
                    ovk,
                    change_addr,
                    Zatoshis::from_u64(change_amount)
                        .map_err(|e| NozyError::InvalidOperation(format!("change: {:?}", e)))?,
                    MemoBytes::empty(),
                )
                .map_err(|e| NozyError::InvalidOperation(format!("change output: {}", e)))?;
        }

        let fee_rule = Zip317FeeRule::standard();
        let signing = TransparentSigningSet::new();
        let extsks: [ExtendedSpendingKey; 1] = [sn.extsk.clone()];
        let mut rng = OsRng;

        let built = builder
            .build(
                &signing,
                &extsks,
                &[],
                &mut rng,
                &self.prover,
                &self.prover,
                &fee_rule,
            )
            .map_err(|e| NozyError::InvalidOperation(format!("Sapling build: {}", e)))?;

        let tx = built.transaction();
        let txid = tx.txid().to_string();
        let mut raw = Vec::new();
        tx.write(&mut raw)
            .map_err(|e| NozyError::InvalidOperation(format!("tx serialize: {}", e)))?;

        Ok(SaplingBuiltSpend {
            raw_transaction: raw,
            txid,
        })
    }
}

impl Default for SaplingTransactionBuilder {
    fn default() -> Self {
        Self::new()
    }
}
