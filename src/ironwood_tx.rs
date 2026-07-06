//! Ironwood (NU6.3) V6 transaction building for normal sends.

use crate::error::{NozyError, NozyResult};
use crate::fee_policy::{pilot_expiry_height, PILOT_EXPIRY_MAX_REBUILD_ATTEMPTS};
use crate::ironwood_tree_codec::{
    ironwood_incremental_witness_from_bytes, ironwood_incremental_witness_to_bytes,
};
use crate::ironwood_witness::ironwood_merkle_path_from_witness;
use crate::notes::{NoteScanner, SpendableNote};
use crate::orchard_tx::OrchardBuiltSpend;
use crate::orchard_witness::{
    advance_witness_with_cmxs, merkle_hash_from_cmx_bytes, witness_root_matches_anchor,
};
use crate::shielded_pool::ShieldedPool;
use crate::zebra_integration::ZebraClient;
use async_trait::async_trait;
use futures::future::join_all;
use orchard::keys::{FullViewingKey, SpendAuthorizingKey};
use orchard::tree::{Anchor, MerklePath};
use orchard::Address as OrchardAddress;
use pczt::roles::{
    creator::Creator, io_finalizer::IoFinalizer, prover::Prover, signer::Signer,
    tx_extractor::TransactionExtractor,
};
use zcash_address::unified::{Container, Encoding};
use zcash_primitives::transaction::builder::{BuildConfig, Builder};
use zcash_primitives::transaction::fees::{transparent::InputSize, FeeRule};
use zcash_protocol::consensus::{BlockHeight, MainNetwork, NetworkType, Parameters, TestNetwork};
use zcash_protocol::memo::MemoBytes;
use zcash_protocol::value::Zatoshis;

struct FixedSendFeeRule {
    fee: Zatoshis,
}

impl FeeRule for FixedSendFeeRule {
    type Error = core::convert::Infallible;

    fn fee_required<P: Parameters>(
        &self,
        _params: &P,
        _target_height: BlockHeight,
        _transparent_input_sizes: impl IntoIterator<Item = InputSize>,
        _transparent_output_sizes: impl IntoIterator<Item = usize>,
        _sapling_input_count: usize,
        _sapling_output_count: usize,
        _orchard_action_count: usize,
        _ironwood_action_count: usize,
    ) -> Result<Zatoshis, Self::Error> {
        Ok(self.fee)
    }
}

/// Supplies Ironwood anchor + Merkle path for a spend.
#[async_trait]
pub trait IronwoodWitnessProvider: Send + Sync {
    async fn prepare_spend_anchor_and_path(
        &self,
        zebra: &ZebraClient,
        note: &SpendableNote,
        anchor_height: u32,
    ) -> NozyResult<(Anchor, MerklePath)>;
}

pub struct ZebraJsonRpcIronwoodWitnessProvider;

async fn fetch_ironwood_cmx_nodes_for_height(
    zebra: &ZebraClient,
    height: u32,
) -> NozyResult<Vec<orchard::tree::MerkleHashOrchard>> {
    let hash = zebra.get_block_hash(height).await?;
    let block_data = zebra.get_block_by_hash(&hash, 2).await?;
    let v = serde_json::to_value(&block_data)
        .map_err(|e| NozyError::InvalidOperation(format!("block JSON: {e}")))?;
    let txs = NoteScanner::parse_block_data(&v, height)?;
    let mut nodes = Vec::new();
    for tx in &txs {
        for action in &tx.ironwood_actions {
            nodes.push(merkle_hash_from_cmx_bytes(&action.cmx)?);
        }
    }
    Ok(nodes)
}

async fn advance_ironwood_witness_from_zebra_blocks(
    witness: &mut crate::ironwood_tree_codec::IronwoodIncrementalWitness,
    zebra: &ZebraClient,
    from_height_exclusive: u32,
    to_height_inclusive: u32,
) -> NozyResult<()> {
    if from_height_exclusive >= to_height_inclusive {
        return Ok(());
    }
    let heights: Vec<u32> = (from_height_exclusive + 1..=to_height_inclusive).collect();
    let parallel = crate::send_readiness::WITNESS_CATCHUP_PARALLEL_BLOCKS.max(1);
    for chunk in heights.chunks(parallel) {
        let fetch_futures: Vec<_> = chunk
            .iter()
            .map(|&h| fetch_ironwood_cmx_nodes_for_height(zebra, h))
            .collect();
        let batch = join_all(fetch_futures).await;
        for nodes in batch {
            for node in nodes? {
                advance_witness_with_cmxs(witness, std::iter::once(node))?;
            }
        }
    }
    Ok(())
}

#[async_trait]
impl IronwoodWitnessProvider for ZebraJsonRpcIronwoodWitnessProvider {
    async fn prepare_spend_anchor_and_path(
        &self,
        zebra: &ZebraClient,
        note: &SpendableNote,
        anchor_height: u32,
    ) -> NozyResult<(Anchor, MerklePath)> {
        let witness_hex = note.ironwood_incremental_witness_hex.as_ref().ok_or_else(|| {
            NozyError::InvalidOperation(
                "Missing Ironwood incremental witness on note: scan with JSON-RPC Zebra (rescan if needed)."
                    .to_string(),
            )
        })?;
        let bytes = hex::decode(witness_hex).map_err(|e| {
            NozyError::InvalidOperation(format!("ironwood_incremental_witness_hex decode: {e}"))
        })?;
        let mut witness = ironwood_incremental_witness_from_bytes(&bytes)?;

        let stored_tip = note.ironwood_witness_tip_height.unwrap_or(0);
        if stored_tip < anchor_height {
            advance_ironwood_witness_from_zebra_blocks(
                &mut witness,
                zebra,
                stored_tip,
                anchor_height,
            )
            .await?;
        }

        let ts = zebra.get_ironwood_tree_state(anchor_height).await?;
        if !witness_root_matches_anchor(&witness, &ts.anchor) {
            return Err(NozyError::InvalidOperation(
                "Ironwood witness does not match z_gettreestate (rescan or wait for sync)."
                    .to_string(),
            ));
        }

        ironwood_merkle_path_from_witness(&witness)
    }
}

pub fn select_single_ironwood_spend_note<'a>(
    spendable_notes: &'a [SpendableNote],
    amount_zatoshis: u64,
    fee_zatoshis: u64,
) -> NozyResult<&'a SpendableNote> {
    let needed = amount_zatoshis.saturating_add(fee_zatoshis);
    let mut best: Option<&SpendableNote> = None;
    for note in spendable_notes
        .iter()
        .filter(|n| !n.orchard_note.spent && n.pool == ShieldedPool::Ironwood)
    {
        if note.orchard_note.value >= needed {
            best = match best {
                None => Some(note),
                Some(current) if note.orchard_note.value < current.orchard_note.value => Some(note),
                _ => best,
            };
        }
    }
    best.ok_or_else(|| {
        NozyError::InvalidOperation(format!(
            "No single Ironwood note covers {needed} zats (multi-note spends are not supported yet)"
        ))
    })
}

pub async fn build_single_ironwood_spend(
    zebra_client: &ZebraClient,
    witness_provider: &dyn IronwoodWitnessProvider,
    spendable_notes: &[SpendableNote],
    recipient_address: &str,
    amount_zatoshis: u64,
    fee_zatoshis: u64,
    memo: Option<&[u8]>,
    expiry_delta_blocks: u32,
) -> NozyResult<OrchardBuiltSpend> {
    let (network_type, recipient_decoded) =
        zcash_address::unified::Address::decode(recipient_address)
            .map_err(|e| NozyError::InvalidOperation(format!("Invalid recipient address: {e}")))?;

    let spend_note =
        select_single_ironwood_spend_note(spendable_notes, amount_zatoshis, fee_zatoshis)?;
    let total_input_value = spend_note.orchard_note.value;
    let change_amount = total_input_value.saturating_sub(amount_zatoshis + fee_zatoshis);

    let recipient_orchard_address = {
        let mut orchard_receiver = None;
        for item in recipient_decoded.items() {
            if let zcash_address::unified::Receiver::Orchard(data) = item {
                orchard_receiver = Some(data);
                break;
            }
        }
        OrchardAddress::from_raw_address_bytes(&orchard_receiver.ok_or_else(|| {
            NozyError::AddressParsing(
                "Recipient must include an Orchard receiver (ZIP-316).".to_string(),
            )
        })?)
        .into_option()
        .ok_or_else(|| {
            NozyError::AddressParsing("Invalid Orchard receiver in unified address".to_string())
        })?
    };

    let transfer_value = Zatoshis::from_u64(amount_zatoshis)
        .map_err(|_| NozyError::InvalidOperation("Invalid Ironwood send amount".to_string()))?;
    let fee = Zatoshis::from_u64(fee_zatoshis)
        .map_err(|_| NozyError::InvalidOperation("Invalid Ironwood send fee".to_string()))?;
    let fee_rule = FixedSendFeeRule { fee };
    let memo_bytes = MemoBytes::from_bytes(memo.unwrap_or(&[])).map_err(|e| {
        NozyError::InvalidOperation(format!("Invalid memo for Ironwood send: {e:?}"))
    })?;

    let fvk = FullViewingKey::from(&spend_note.spending_key);
    let change_address = spend_note.orchard_note.address.clone();

    for attempt in 1..=PILOT_EXPIRY_MAX_REBUILD_ATTEMPTS {
        if attempt > 1 {
            println!(
                "Ironwood proof outran pilot expiry; rebuilding ({attempt}/{PILOT_EXPIRY_MAX_REBUILD_ATTEMPTS})"
            );
        }

        let chain_tip = zebra_client.get_best_block_height().await?;
        let (ironwood_anchor, merkle_path) = witness_provider
            .prepare_spend_anchor_and_path(zebra_client, spend_note, chain_tip)
            .await?;
        let target_height = BlockHeight::from_u32(chain_tip.saturating_add(1));
        let build_config = BuildConfig::Standard {
            sapling_anchor: None,
            orchard_anchor: None,
            ironwood_anchor: Some(ironwood_anchor),
        };

        macro_rules! build_pczt_for_network {
            ($params:expr) => {{
                let mut builder = Builder::new($params, target_height, build_config);
                builder
                    .add_ironwood_spend::<core::convert::Infallible>(
                        fvk.clone(),
                        spend_note.orchard_note.note.clone(),
                        merkle_path,
                    )
                    .map_err(|e| {
                        NozyError::InvalidOperation(format!("add_ironwood_spend: {e:?}"))
                    })?;
                builder
                    .add_ironwood_output::<core::convert::Infallible>(
                        None,
                        recipient_orchard_address.clone(),
                        transfer_value,
                        memo_bytes.clone(),
                    )
                    .map_err(|e| {
                        NozyError::InvalidOperation(format!("add_ironwood_output: {e:?}"))
                    })?;

                if change_amount > 0 {
                    let change_value = Zatoshis::from_u64(change_amount).map_err(|_| {
                        NozyError::InvalidOperation("Invalid Ironwood change amount".to_string())
                    })?;
                    builder
                        .add_ironwood_output::<core::convert::Infallible>(
                            None,
                            change_address.clone(),
                            change_value,
                            MemoBytes::empty(),
                        )
                        .map_err(|e| {
                            NozyError::InvalidOperation(format!(
                                "add_ironwood_output (change): {e:?}"
                            ))
                        })?;
                }

                let parts = builder
                    .build_for_pczt(rand::rngs::OsRng, &fee_rule)
                    .map_err(|e| NozyError::InvalidOperation(format!("build_for_pczt: {e:?}")))?
                    .pczt_parts;
                Creator::build_from_parts(parts).ok_or_else(|| {
                    NozyError::InvalidOperation(
                        "PCZT creator: incompatible V6 Ironwood send parts".to_string(),
                    )
                })
            }};
        }

        let pczt = match network_type {
            NetworkType::Main => build_pczt_for_network!(MainNetwork)?,
            NetworkType::Test | NetworkType::Regtest => build_pczt_for_network!(TestNetwork)?,
        };

        let proving_key =
            orchard::circuit::ProvingKey::build(orchard::circuit::OrchardCircuitVersion::PostNu6_3);
        let pczt = Prover::new(pczt)
            .create_ironwood_proof(&proving_key)
            .map_err(|e| NozyError::InvalidOperation(format!("create_ironwood_proof: {e:?}")))?
            .finish();

        let pczt = IoFinalizer::new(pczt).finalize_io().map_err(|e| {
            NozyError::InvalidOperation(format!("ironwood send io_finalize: {e:?}"))
        })?;

        let ask = SpendAuthorizingKey::from(&spend_note.spending_key);
        let ironwood_action_count = pczt.ironwood().actions().len();
        let mut signer = Signer::new(pczt)
            .map_err(|e| NozyError::InvalidOperation(format!("ironwood signer init: {e:?}")))?;
        let mut signed_any = false;
        for index in 0..ironwood_action_count {
            if signer.sign_ironwood(index, &ask).is_ok() {
                signed_any = true;
            }
        }
        if !signed_any {
            return Err(NozyError::InvalidOperation(
                "No Ironwood spend signatures were applied.".to_string(),
            ));
        }
        let pczt = signer.finish();

        let verifying_key = orchard::circuit::VerifyingKey::build(
            orchard::circuit::OrchardCircuitVersion::PostNu6_3,
        );
        let tx = TransactionExtractor::new(pczt)
            .with_orchard(&verifying_key)
            .extract()
            .map_err(|e| NozyError::InvalidOperation(format!("ironwood tx extract: {e:?}")))?;

        let expiry_tip = zebra_client.get_best_block_height().await?;
        let expiry_height_u32 = pilot_expiry_height(expiry_tip, expiry_delta_blocks);
        let txid = tx.txid().to_string();
        let mut raw_transaction = Vec::new();
        tx.write(&mut raw_transaction)
            .map_err(|e| NozyError::InvalidOperation(format!("ironwood tx serialize: {e}")))?;

        return Ok(OrchardBuiltSpend {
            raw_transaction,
            txid,
            expiry_height: expiry_height_u32,
        });
    }

    Err(NozyError::InvalidOperation(
        "Ironwood send exceeded pilot expiry rebuild attempts".to_string(),
    ))
}

/// Advance persisted Ironwood note witnesses through chain tip.
pub async fn refresh_ironwood_cached_witnesses_to_tip(
    zebra: &ZebraClient,
    notes: &mut [crate::notes::SerializableOrchardNote],
    chain_tip: u32,
) -> NozyResult<u32> {
    let mut updated = 0u32;
    for note in notes
        .iter_mut()
        .filter(|n| !n.spent && n.pool == ShieldedPool::Ironwood)
    {
        let Some(ref witness_hex) = note.ironwood_incremental_witness_hex else {
            continue;
        };
        if witness_hex.is_empty() {
            continue;
        }
        let stored_tip = note.ironwood_witness_tip_height.unwrap_or(0);
        if stored_tip >= chain_tip {
            continue;
        }
        let bytes = hex::decode(witness_hex).map_err(|e| {
            NozyError::InvalidOperation(format!("ironwood_incremental_witness_hex decode: {e}"))
        })?;
        let mut witness = ironwood_incremental_witness_from_bytes(&bytes)?;
        advance_ironwood_witness_from_zebra_blocks(&mut witness, zebra, stored_tip, chain_tip)
            .await?;
        note.ironwood_incremental_witness_hex = Some(hex::encode(
            ironwood_incremental_witness_to_bytes(&witness)?,
        ));
        note.ironwood_witness_tip_height = Some(chain_tip);
        updated += 1;
    }
    Ok(updated)
}
