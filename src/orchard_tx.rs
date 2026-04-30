use crate::error::{NozyError, NozyResult};
use crate::notes::{NoteScanner, SpendableNote};
use crate::orchard_tree_codec::{
    orchard_incremental_witness_from_bytes, OrchardIncrementalWitness,
};
use crate::orchard_witness::{
    advance_witness_with_cmxs, merkle_hash_from_cmx_bytes, merkle_path_from_witness,
    witness_root_matches_anchor,
};
use crate::proving::{OrchardProvingManager, ProvingStatus};
use crate::zebra_integration::ZebraClient;
use async_trait::async_trait;
use orchard::{
    builder::Builder as OrchardBuilder, builder::BundleType, bundle::Flags, circuit::ProvingKey,
    keys::FullViewingKey, keys::SpendAuthorizingKey, tree::Anchor, tree::MerklePath,
    value::NoteValue, Address as OrchardAddress,
};
use rand::rngs::OsRng;
use std::sync::{Arc, OnceLock};
use zcash_address::unified::{Container, Encoding};
use zcash_primitives::transaction::builder::DEFAULT_TX_EXPIRY_DELTA;
use zcash_primitives::transaction::sighash::{signature_hash, SignableInput};
use zcash_primitives::transaction::txid::TxIdDigester;
use zcash_primitives::transaction::{Authorized, TransactionData, TxVersion, Unauthorized};
use zcash_protocol::consensus::{BlockHeight, BranchId, NetworkType, MAIN_NETWORK, TEST_NETWORK};
use zcash_protocol::value::ZatBalance;

static ORCHARD_PROVING_KEY: OnceLock<Arc<ProvingKey>> = OnceLock::new();

fn orchard_proving_key() -> &'static Arc<ProvingKey> {
    ORCHARD_PROVING_KEY.get_or_init(|| Arc::new(ProvingKey::build()))
}

/// Result of building a single Orchard spend: canonical v5 raw bytes and ZIP-244 txid hex.
#[derive(Debug, Clone)]
pub struct OrchardBuiltSpend {
    pub raw_transaction: Vec<u8>,
    pub txid: String,
}

/// Supplies Orchard anchor + Merkle path for a spend (local witness + `z_gettreestate` verification).
#[async_trait]
pub trait OrchardWitnessProvider: Send + Sync {
    async fn prepare_spend_anchor_and_path(
        &self,
        zebra: &ZebraClient,
        note: &SpendableNote,
        anchor_height: u32,
    ) -> NozyResult<(Anchor, MerklePath)>;
}

/// Default: deserialize incremental witness from the note, catch up to `anchor_height` via Zebra blocks, verify root.
pub struct ZebraJsonRpcOrchardWitnessProvider;

async fn advance_orchard_witness_from_zebra_blocks(
    witness: &mut OrchardIncrementalWitness,
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
            for action in &tx.orchard_actions {
                let node = merkle_hash_from_cmx_bytes(&action.cmx)?;
                advance_witness_with_cmxs(witness, std::iter::once(node))?;
            }
        }
    }
    Ok(())
}

#[async_trait]
impl OrchardWitnessProvider for ZebraJsonRpcOrchardWitnessProvider {
    async fn prepare_spend_anchor_and_path(
        &self,
        zebra: &ZebraClient,
        note: &SpendableNote,
        anchor_height: u32,
    ) -> NozyResult<(Anchor, MerklePath)> {
        let witness_hex = note.orchard_incremental_witness_hex.as_ref().ok_or_else(|| {
            NozyError::InvalidOperation(
                "Missing Orchard incremental witness on note: scan with JSON-RPC Zebra (rescan if needed)."
                    .to_string(),
            )
        })?;
        let bytes = hex::decode(witness_hex).map_err(|e| {
            NozyError::InvalidOperation(format!("orchard_incremental_witness_hex decode: {}", e))
        })?;
        let mut witness = orchard_incremental_witness_from_bytes(&bytes)?;

        let stored_tip = note.orchard_witness_tip_height.unwrap_or(0);
        if stored_tip < anchor_height {
            advance_orchard_witness_from_zebra_blocks(
                &mut witness,
                zebra,
                stored_tip,
                anchor_height,
            )
            .await?;
        }

        let ts = zebra.get_orchard_tree_state(anchor_height).await?;
        if !witness_root_matches_anchor(&witness, &ts.anchor) {
            return Err(NozyError::InvalidOperation(
                "Orchard witness does not match z_gettreestate (rescan or wait for sync)."
                    .to_string(),
            ));
        }

        merkle_path_from_witness(&witness)
    }
}

#[derive(Debug)]
pub struct OrchardTransactionBuilder {
    proving_manager: OrchardProvingManager,
}

impl OrchardTransactionBuilder {
    pub fn new(_download_params: bool) -> Self {
        let params_dir = std::path::PathBuf::from("orchard_params");
        let proving_manager = OrchardProvingManager::new(params_dir);

        Self { proving_manager }
    }

    pub async fn new_async(download_params: bool) -> NozyResult<Self> {
        let params_dir = std::path::PathBuf::from("orchard_params");
        let mut proving_manager = OrchardProvingManager::new(params_dir);

        proving_manager.initialize().await?;

        if download_params {
            proving_manager.download_parameters().await?;
        }

        Ok(Self { proving_manager })
    }

    /// Build, prove, and sign a single-note Orchard v5 transaction (ZIP-225 serialization, ZIP-244 txid).
    pub async fn build_single_spend(
        &self,
        zebra_client: &ZebraClient,
        witness_provider: &dyn OrchardWitnessProvider,
        spendable_notes: &[SpendableNote],
        recipient_address: &str,
        amount_zatoshis: u64,
        fee_zatoshis: u64,
        memo: Option<&[u8]>,
    ) -> NozyResult<OrchardBuiltSpend> {
        println!("Building Orchard transaction...");

        let (network_type, recipient_decoded) =
            zcash_address::unified::Address::decode(recipient_address).map_err(|e| {
                NozyError::InvalidOperation(format!("Invalid recipient address: {}", e))
            })?;

        let total_input_value: u64 = spendable_notes
            .iter()
            .map(|note| note.orchard_note.value)
            .sum();

        let change_amount = total_input_value.saturating_sub(amount_zatoshis + fee_zatoshis);

        if total_input_value < amount_zatoshis + fee_zatoshis {
            return Err(NozyError::InvalidOperation(format!(
                "Insufficient funds: have {} zatoshis, need {} zatoshis",
                total_input_value,
                amount_zatoshis + fee_zatoshis
            )));
        }

        let bundle_type = BundleType::Transactional {
            flags: Flags::ENABLED,
            bundle_required: true,
        };
        let tip_height = zebra_client.get_best_block_height().await?;
        // Mempool consensus checks use the next block context, not the current tip block.
        let tx_build_height = tip_height.saturating_add(1);

        let spendable_note = &spendable_notes[0];

        println!(
            "Adding spend action for {} zatoshis",
            spendable_note.orchard_note.value
        );

        let fvk = FullViewingKey::from(&spendable_note.spending_key);

        let (anchor, merkle_path) = witness_provider
            .prepare_spend_anchor_and_path(zebra_client, spendable_note, tip_height)
            .await?;

        let mut builder = OrchardBuilder::new(bundle_type, anchor);

        builder
            .add_spend(fvk, spendable_note.orchard_note.note.clone(), merkle_path)
            .map_err(|e| {
                NozyError::InvalidOperation(format!("Failed to add spend action: {}", e))
            })?;

        let recipient_orchard_address = {
            let mut orchard_receiver = None;
            for item in recipient_decoded.items() {
                if let zcash_address::unified::Receiver::Orchard(data) = item {
                    orchard_receiver = Some(data);
                    break;
                }
            }

            match orchard_receiver {
                Some(data) => OrchardAddress::from_raw_address_bytes(&data)
                    .into_option()
                    .ok_or_else(|| {
                        NozyError::AddressParsing(
                            "Invalid Orchard receiver in unified address".to_string(),
                        )
                    })?,
                None => {
                    let has_sapling = recipient_decoded
                        .items()
                        .iter()
                        .any(|i| matches!(i, zcash_address::unified::Receiver::Sapling(_)));
                    return Err(NozyError::AddressParsing(if has_sapling {
                        "Unified address has Sapling but no Orchard receiver. Use a Sapling-capable send path or a UA that includes Orchard."
                            .to_string()
                    } else {
                        "No Orchard receiver found in unified address".to_string()
                    }));
                }
            }
        };
        let recipient_note_value = NoteValue::from_raw(amount_zatoshis);
        let recipient_memo = memo
            .map(|m| {
                let mut memo_bytes = [0u8; 512];
                let len = m.len().min(512);
                memo_bytes[..len].copy_from_slice(&m[..len]);
                memo_bytes
            })
            .unwrap_or([0u8; 512]);

        builder
            .add_output(
                None,
                recipient_orchard_address,
                recipient_note_value,
                recipient_memo,
            )
            .map_err(|e| {
                NozyError::InvalidOperation(format!("Failed to add recipient output: {}", e))
            })?;

        if change_amount > 0 {
            let change_orchard_address = spendable_note.orchard_note.address;
            let change_note_value = NoteValue::from_raw(change_amount);
            let change_memo = [0u8; 512];

            builder
                .add_output(None, change_orchard_address, change_note_value, change_memo)
                .map_err(|e| {
                    NozyError::InvalidOperation(format!("Failed to add change output: {}", e))
                })?;
        }

        let mut rng = OsRng;
        let bundle_result = builder.build::<i64>(&mut rng);

        let (unauthorized, _metadata) = match bundle_result {
            Ok(Some((bundle, metadata))) => (bundle, metadata),
            Ok(None) => {
                return Err(NozyError::InvalidOperation(
                    "Failed to build Orchard bundle - no bundle returned".to_string(),
                ))
            }
            Err(e) => {
                return Err(NozyError::InvalidOperation(format!(
                    "Failed to build Orchard bundle: {}",
                    e
                )))
            }
        };

        println!("✅ Orchard bundle built successfully!");

        let status = self.proving_manager.get_status();
        if !status.can_prove {
            return Err(NozyError::InvalidOperation(
                "Cannot create proofs: proving parameters not available. Run 'nozy proving --download' first.".to_string()
            ));
        }

        println!("🔧 Proving Status: {}", status.status_message());

        let branch_id = match network_type {
            NetworkType::Main => {
                BranchId::for_height(&MAIN_NETWORK, BlockHeight::from_u32(tx_build_height))
            }
            NetworkType::Test | NetworkType::Regtest => {
                BranchId::for_height(&TEST_NETWORK, BlockHeight::from_u32(tx_build_height))
            }
        };
        let expiry_height =
            BlockHeight::from_u32(tx_build_height.saturating_add(DEFAULT_TX_EXPIRY_DELTA));
        let unauthorized_zat = unauthorized.clone().try_map_value_balance(|vb| {
            ZatBalance::from_i64(vb).map_err(|e| {
                NozyError::InvalidOperation(format!(
                    "Orchard unauthorized value balance out of range: {:?}",
                    e
                ))
            })
        })?;
        let unauthed_tx_data = TransactionData::<Unauthorized>::from_parts(
            TxVersion::V5,
            branch_id,
            0,
            expiry_height,
            None,
            None,
            None,
            Some(unauthorized_zat),
        );
        let txid_parts = unauthed_tx_data.digest(TxIdDigester);
        let zip244_shielded_sighash =
            signature_hash(&unauthed_tx_data, &SignableInput::Shielded, &txid_parts);
        let zip244_sighash_arr = *zip244_shielded_sighash.as_ref();

        let sighash: [u8; 32] = zip244_sighash_arr;
        let pk = orchard_proving_key();
        let proven = unauthorized
            .create_proof(pk.as_ref(), &mut rng)
            .map_err(|e| {
                NozyError::InvalidOperation(format!("Orchard proof generation failed: {:?}", e))
            })?;

        let authorized_bundle = proven
            .apply_signatures(
                &mut rng,
                sighash,
                &[SpendAuthorizingKey::from(&spendable_note.spending_key)],
            )
            .map_err(|e| {
                NozyError::InvalidOperation(format!("Orchard spend authorization failed: {}", e))
            })?;

        let bundle_zat = authorized_bundle.try_map_value_balance(|vb| {
            ZatBalance::from_i64(vb).map_err(|e| {
                NozyError::InvalidOperation(format!("Orchard value balance out of range: {:?}", e))
            })
        })?;

        let bundle_actions_count = bundle_zat.actions().len();

        let tx_data = TransactionData::<Authorized>::from_parts(
            TxVersion::V5,
            branch_id,
            0,
            expiry_height,
            None,
            None,
            None,
            Some(bundle_zat),
        );

        let tx = tx_data
            .freeze()
            .map_err(|e| NozyError::InvalidOperation(format!("transaction freeze: {}", e)))?;

        let txid = tx.txid().to_string();

        let mut raw_transaction = Vec::new();
        tx.write(&mut raw_transaction).map_err(|e| {
            NozyError::InvalidOperation(format!("transaction serialization: {}", e))
        })?;

        println!("🔐 Bundle authorized (Orchard proof + binding + spend signatures)");
        println!("✅ Transaction signed and serialized (ZIP-225 v5)");
        println!("   Bundle contains {} actions", bundle_actions_count);
        println!("   TXID: {}", txid);
        println!("   Transaction size: {} bytes", raw_transaction.len());

        Ok(OrchardBuiltSpend {
            raw_transaction,
            txid,
        })
    }

    pub fn get_proving_status(&self) -> ProvingStatus {
        self.proving_manager.get_status()
    }

    pub fn can_prove(&self) -> bool {
        self.proving_manager.can_prove()
    }

    pub async fn initialize_proving(&mut self) -> NozyResult<()> {
        self.proving_manager.initialize().await?;
        Ok(())
    }

    pub async fn download_parameters(&mut self) -> NozyResult<()> {
        self.proving_manager.download_parameters().await?;
        Ok(())
    }

    pub fn get_proving_key_info(&self) -> Option<String> {
        self.proving_manager.can_prove().then_some(
            "Orchard Halo2 ProvingKey: built in-process on first spend (cached globally)"
                .to_string(),
        )
    }
}
