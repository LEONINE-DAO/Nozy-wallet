//! Sapling → Ironwood/Orchard shield (Phase 4 spend).
//!
//! Builds Merkle witnesses from LWD compact Sapling outputs + Zebra `z_gettreestate`,
//! then constructs a transaction that spends one Sapling note into the wallet's
//! own Orchard/Ironwood address (quiet legacy migration; not marketed as Sapling).

use crate::error::{NozyError, NozyResult};
use crate::fee_policy::{
    pilot_expiry_height, GRACE_ACTIONS, MARGINAL_FEE_ZATOSHIS, PILOT_EXPIRY_MAX_REBUILD_ATTEMPTS,
    PRIORITY_MULTIPLIER,
};
use crate::ironwood::{IronwoodAwareMainNetwork, NU6_3_MAINNET_ACTIVATION_HEIGHT};
use crate::sapling_scan::{
    reconstruct_sapling_note, sapling_note_has_rseed, SerializableSaplingNote,
};
use crate::sapling_tree_codec::{
    sapling_incremental_witness_from_bytes, sapling_incremental_witness_to_bytes,
};
use crate::sapling_witness::{
    advance_witness_with_cmus, merkle_node_from_cmu_bytes, merkle_path_from_witness,
    sapling_commitment_tree_from_final_state, witness_root_matches_anchor, SaplingWitnessTracker,
};
use crate::zebra_integration::ZebraClient;
use orchard::keys::SpendAuthorizingKey;
use orchard::Address as OrchardAddress;
use rand::rngs::OsRng;
use sapling::zip32::ExtendedSpendingKey;
use sapling::{Anchor as SaplingAnchor, Note};
use zcash_primitives::transaction::builder::{BuildConfig, Builder};
use zcash_primitives::transaction::fees::{transparent::InputSize, FeeRule};
use zcash_protocol::consensus::{BlockHeight, NetworkType, Parameters, TestNetwork};
use zcash_protocol::memo::MemoBytes;
use zcash_protocol::value::Zatoshis;
use zcash_transparent::builder::TransparentSigningSet;
use zeaking::lwd::{sapling_slice_from_compact_block, LwdCompactStore};

/// ZIP-317-ish fee for 1 Sapling spend + 1 shielded output (padded to 2 actions).
pub fn sapling_shield_fee_zatoshis() -> u64 {
    // logical = 1 sapling spend + 2 orchard/ironwood actions (bundle padding)
    let logical = 1u32.saturating_add(2);
    let billable = logical.max(GRACE_ACTIONS) as u64;
    MARGINAL_FEE_ZATOSHIS
        .saturating_mul(billable)
        .saturating_mul(PRIORITY_MULTIPLIER)
}

struct FixedShieldFeeRule {
    fee: Zatoshis,
}

impl FeeRule for FixedShieldFeeRule {
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

/// Result of a proven Sapling shield transaction.
#[derive(Debug, Clone)]
pub struct SaplingShieldBuilt {
    pub raw_transaction: Vec<u8>,
    pub txid: String,
    pub expiry_height: u32,
    pub fee_zatoshis: u64,
    pub shielded_value_zatoshis: u64,
    pub spent_nullifier_hex: String,
}

fn collect_sapling_cmus_from_compact(data: &[u8]) -> NozyResult<Vec<[u8; 32]>> {
    let slice = sapling_slice_from_compact_block(data)
        .map_err(|e| NozyError::InvalidOperation(format!("sapling compact decode: {e}")))?;
    let mut out = Vec::new();
    for tx in &slice.txs {
        for o in &tx.outputs {
            out.push(o.cmu);
        }
    }
    Ok(out)
}

/// Cap Zebra tip to the highest compact height we can actually replay.
fn sapling_witness_replay_tip(store: &LwdCompactStore, zebra_tip: u32) -> NozyResult<u32> {
    let store_max = store
        .max_compact_height()
        .map_err(|e| NozyError::Storage(format!("compact max: {e}")))?
        .ok_or_else(|| {
            NozyError::InvalidOperation(
                "LWD compact store is empty — run `nozy lwd sync-to-tip` first".into(),
            )
        })?;
    Ok(u64::from(zebra_tip).min(store_max) as u32)
}

/// Append Sapling cmus for every height in `from..=to` (inclusive). Errors on gaps.
fn append_sapling_cmus_height_range(
    tracker: &mut SaplingWitnessTracker,
    store: &LwdCompactStore,
    from: u32,
    to: u32,
    mut on_after_append: impl FnMut(&mut SaplingWitnessTracker, u64, [u8; 32]) -> NozyResult<()>,
) -> NozyResult<()> {
    if from > to {
        return Ok(());
    }
    for h in from..=to {
        let Some(blob) = store
            .get_compact_block(u64::from(h))
            .map_err(|e| NozyError::Storage(format!("compact read {h}: {e}")))?
        else {
            return Err(NozyError::InvalidOperation(format!(
                "Missing compact block {h} in LWD store (needed for Sapling witnesses). \
                 Run `nozy lwd sync-to-tip` so the cache is contiguous through tip."
            )));
        };
        let cmus = collect_sapling_cmus_from_compact(&blob)?;
        for cmu in cmus {
            let node = merkle_node_from_cmu_bytes(&cmu)?;
            tracker.append_cmu(node)?;
            let position = (tracker.tree().size() as u64).saturating_sub(1);
            on_after_append(tracker, position, cmu)?;
        }
    }
    Ok(())
}

fn advance_witness_height_range(
    witness: &mut crate::sapling_tree_codec::SaplingIncrementalWitness,
    store: &LwdCompactStore,
    from: u32,
    to: u32,
) -> NozyResult<()> {
    if from > to {
        return Ok(());
    }
    for h in from..=to {
        let Some(blob) = store
            .get_compact_block(u64::from(h))
            .map_err(|e| NozyError::Storage(format!("compact read {h}: {e}")))?
        else {
            return Err(NozyError::InvalidOperation(format!(
                "Missing compact block {h} for Sapling witness catch-up. Run `nozy lwd sync-to-tip`."
            )));
        };
        let cmus = collect_sapling_cmus_from_compact(&blob)?;
        for cmu in cmus {
            let node = merkle_node_from_cmu_bytes(&cmu)?;
            advance_witness_with_cmus(witness, std::iter::once(node))?;
        }
    }
    Ok(())
}

/// Rebuild or advance Sapling incremental witnesses for unspent notes via LWD compact + Zebra.
pub async fn refresh_sapling_witnesses_from_compact_store(
    zebra: &ZebraClient,
    store: &LwdCompactStore,
    notes: &mut [SerializableSaplingNote],
    tip_height: u32,
) -> NozyResult<u32> {
    // Never verify against a Zebra tip we cannot replay from compact cache.
    let tip_height = sapling_witness_replay_tip(store, tip_height)?;

    let need_full: Vec<usize> = notes
        .iter()
        .enumerate()
        .filter(|(_, n)| {
            !n.spent
                && sapling_note_has_rseed(n)
                && n.sapling_incremental_witness_hex
                    .as_ref()
                    .map(|h| h.is_empty())
                    .unwrap_or(true)
        })
        .map(|(i, _)| i)
        .collect();

    let mut updated = 0u32;

    if !need_full.is_empty() {
        let min_note_height = need_full
            .iter()
            .map(|&i| notes[i].block_height)
            .min()
            .unwrap_or(1);
        if min_note_height > tip_height {
            return Err(NozyError::InvalidOperation(format!(
                "Sapling note height {min_note_height} is above compact tip {tip_height}; \
                 run `nozy lwd sync-to-tip`"
            )));
        }
        let checkpoint = min_note_height.saturating_sub(1);

        let store_min = store
            .min_compact_height()
            .map_err(|e| NozyError::Storage(format!("compact min: {e}")))?
            .unwrap_or(u64::from(min_note_height));
        if store_min > u64::from(min_note_height) {
            return Err(NozyError::InvalidOperation(format!(
                "LWD compact cache starts at {store_min}, but Sapling note is at {min_note_height}. \
                 Re-sync with a lower floor, e.g. `nozy lwd sync-to-tip --start-floor {}`",
                min_note_height.saturating_sub(1)
            )));
        }

        let parsed = zebra.get_sapling_treestate_parsed(checkpoint).await?;
        let Some(ref final_state) = parsed.final_state else {
            return Err(NozyError::InvalidOperation(format!(
                "z_gettreestate({checkpoint}) has no Sapling finalState (JSON-RPC Zebra required)"
            )));
        };
        let mut tracker =
            SaplingWitnessTracker::new(sapling_commitment_tree_from_final_state(final_state)?);
        let checkpoint_size = tracker.tree().size() as u64;

        // Match discoveries by note position (stable).
        let mut pending_by_pos: std::collections::HashMap<u64, [u8; 32]> =
            std::collections::HashMap::new();
        for &i in &need_full {
            let note = &notes[i];
            let Ok(nf) = <[u8; 32]>::try_from(note.nullifier_bytes.as_slice()) else {
                continue;
            };
            if note.position < checkpoint_size {
                return Err(NozyError::InvalidOperation(format!(
                    "Sapling note position {} is before checkpoint tree size {checkpoint_size} \
                     (height {}); rescan with `nozy lwd scan-sapling --full`",
                    note.position, checkpoint
                )));
            }
            pending_by_pos.insert(note.position, nf);
        }

        let start = checkpoint.saturating_add(1);
        // Replay through the discovery height first and verify against Zebra — isolates
        // catch-up gaps from a bad checkpoint / wrong note positions.
        append_sapling_cmus_height_range(
            &mut tracker,
            store,
            start,
            min_note_height,
            |tracker, position, _cmu| {
                if let Some(nf) = pending_by_pos.remove(&position) {
                    tracker.register_discovered_note(nf)?;
                }
                Ok(())
            },
        )?;
        {
            let note_ts = zebra.get_sapling_treestate_parsed(min_note_height).await?;
            let local_root = tracker.root_at_tip().to_bytes();
            let local_size = tracker.tree().size() as u64;
            let size_mismatch =
                note_ts.commitment_count > 0 && local_size != note_ts.commitment_count;
            if local_root != note_ts.anchor || size_mismatch {
                return Err(NozyError::InvalidOperation(format!(
                    "Sapling tree mismatch at note height {min_note_height}: \
                     local_size={local_size} zebra_size={} local_root={} zebra_root={}. \
                     Rescan with `nozy lwd scan-sapling --full` after a contiguous \
                     `nozy lwd sync-to-tip --start-floor {}`.",
                    note_ts.commitment_count,
                    hex::encode(local_root),
                    hex::encode(note_ts.anchor),
                    checkpoint
                )));
            }
        }
        if min_note_height < tip_height {
            append_sapling_cmus_height_range(
                &mut tracker,
                store,
                min_note_height.saturating_add(1),
                tip_height,
                |_tracker, _position, _cmu| Ok(()),
            )?;
        }

        let tip_ts = zebra.get_sapling_treestate_parsed(tip_height).await?;
        let local_root = tracker.root_at_tip().to_bytes();
        let local_size = tracker.tree().size() as u64;
        let size_mismatch = tip_ts.commitment_count > 0 && local_size != tip_ts.commitment_count;
        if local_root != tip_ts.anchor || size_mismatch {
            return Err(NozyError::InvalidOperation(format!(
                "Sapling witness tree mismatch at compact tip {tip_height}: \
                 local_size={local_size} zebra_size={} local_root={} zebra_root={}. \
                 Run `nozy lwd sync-to-tip` then retry (compact gaps after the note height).",
                tip_ts.commitment_count,
                hex::encode(local_root),
                hex::encode(tip_ts.anchor)
            )));
        }

        for &i in &need_full {
            let nf = match <[u8; 32]>::try_from(notes[i].nullifier_bytes.as_slice()) {
                Ok(nf) => nf,
                Err(_) => continue,
            };
            if let Some(bytes) = tracker.serialized_witness_for_nullifier(&nf)? {
                notes[i].sapling_incremental_witness_hex = Some(hex::encode(bytes));
                notes[i].sapling_witness_tip_height = Some(tip_height);
                updated += 1;
            }
        }
        if !pending_by_pos.is_empty() {
            return Err(NozyError::InvalidOperation(format!(
                "Could not locate {} Sapling note(s) by position in compact range {start}..={tip_height}; \
                 rescan with `nozy lwd scan-sapling --full`",
                pending_by_pos.len()
            )));
        }
    }

    for note in notes.iter_mut().filter(|n| !n.spent) {
        let Some(ref witness_hex) = note.sapling_incremental_witness_hex else {
            continue;
        };
        if witness_hex.is_empty() {
            continue;
        }
        let stored_tip = note.sapling_witness_tip_height.unwrap_or(0);
        if stored_tip >= tip_height {
            continue;
        }
        let bytes = hex::decode(witness_hex)
            .map_err(|e| NozyError::InvalidOperation(format!("sapling witness hex decode: {e}")))?;
        let mut witness = sapling_incremental_witness_from_bytes(&bytes)?;
        advance_witness_height_range(
            &mut witness,
            store,
            stored_tip.saturating_add(1),
            tip_height,
        )?;
        let tip_ts = zebra.get_sapling_tree_state(tip_height).await?;
        if !witness_root_matches_anchor(&witness, &tip_ts.anchor) {
            return Err(NozyError::InvalidOperation(format!(
                "Sapling witness does not match z_gettreestate after catch-up to {tip_height}"
            )));
        }
        note.sapling_incremental_witness_hex =
            Some(hex::encode(sapling_incremental_witness_to_bytes(&witness)?));
        note.sapling_witness_tip_height = Some(tip_height);
        updated += 1;
    }

    Ok(updated)
}

async fn prepare_sapling_anchor_and_path(
    zebra: &ZebraClient,
    note: &SerializableSaplingNote,
    anchor_height: u32,
    store: &LwdCompactStore,
) -> NozyResult<(SaplingAnchor, sapling::MerklePath)> {
    let witness_hex = note
        .sapling_incremental_witness_hex
        .as_ref()
        .ok_or_else(|| {
            NozyError::InvalidOperation(
                "Missing Sapling incremental witness on note (run shield again to rebuild)".into(),
            )
        })?;
    let bytes = hex::decode(witness_hex)
        .map_err(|e| NozyError::InvalidOperation(format!("sapling witness hex decode: {e}")))?;
    let mut witness = sapling_incremental_witness_from_bytes(&bytes)?;

    let anchor_height = sapling_witness_replay_tip(store, anchor_height)?;
    let stored_tip = note.sapling_witness_tip_height.unwrap_or(0);
    if stored_tip < anchor_height {
        advance_witness_height_range(
            &mut witness,
            store,
            stored_tip.saturating_add(1),
            anchor_height,
        )?;
    }

    let ts = zebra.get_sapling_tree_state(anchor_height).await?;
    if !witness_root_matches_anchor(&witness, &ts.anchor) {
        return Err(NozyError::InvalidOperation(format!(
            "Sapling witness does not match z_gettreestate at {anchor_height} (rebuild witnesses)"
        )));
    }
    merkle_path_from_witness(&witness)
}

fn select_shield_note(
    notes: &[SerializableSaplingNote],
    fee: u64,
) -> NozyResult<&SerializableSaplingNote> {
    let mut best: Option<&SerializableSaplingNote> = None;
    for note in notes.iter().filter(|n| {
        !n.spent
            && sapling_note_has_rseed(n)
            && n.sapling_incremental_witness_hex
                .as_ref()
                .is_some_and(|h| !h.is_empty())
    }) {
        if note.value > fee {
            best = match best {
                None => Some(note),
                Some(cur) if note.value < cur.value => Some(note),
                _ => best,
            };
        }
    }
    best.ok_or_else(|| {
        NozyError::InvalidOperation(format!(
            "No reconstructible Sapling note covers fee ({fee} zats) with a Merkle witness"
        ))
    })
}

fn destination_orchard_address(seed: &[u8], account: u32) -> NozyResult<OrchardAddress> {
    use orchard::keys::{DiversifierIndex, FullViewingKey, Scope, SpendingKey};
    use zip32::AccountId;
    let account_id = AccountId::try_from(account)
        .map_err(|e| NozyError::KeyDerivation(format!("Invalid account ID: {e:?}")))?;
    let sk = SpendingKey::from_zip32_seed(seed, 133, account_id).map_err(|e| {
        NozyError::KeyDerivation(format!("Orchard SpendingKey for shield destination: {e:?}"))
    })?;
    let fvk = FullViewingKey::from(&sk);
    Ok(fvk.address_at(DiversifierIndex::from(0u32), Scope::External))
}

/// Build, prove, and sign a Sapling → Ironwood (post-NU6.3) or Sapling → Orchard shield.
pub async fn build_sapling_shield_to_self(
    zebra: &ZebraClient,
    store: &LwdCompactStore,
    seed: &[u8],
    sapling_extsk: &ExtendedSpendingKey,
    notes: &mut [SerializableSaplingNote],
    network_type: NetworkType,
    expiry_delta_blocks: u32,
) -> NozyResult<SaplingShieldBuilt> {
    let fee = sapling_shield_fee_zatoshis();
    let fee_zat = Zatoshis::from_u64(fee)
        .map_err(|_| NozyError::InvalidOperation("Invalid Sapling shield fee".into()))?;
    let fee_rule = FixedShieldFeeRule { fee: fee_zat };

    let zebra_tip0 = zebra.get_best_block_height().await?;
    let tip0 = sapling_witness_replay_tip(store, zebra_tip0)?;
    if tip0 < zebra_tip0 {
        println!(
            "Compact cache tip {tip0} lags Zebra tip {zebra_tip0}; building Sapling witnesses to compact tip"
        );
    }
    refresh_sapling_witnesses_from_compact_store(zebra, store, notes, tip0).await?;

    let spend_note = select_shield_note(notes, fee)?.clone();
    let note: Note = reconstruct_sapling_note(&spend_note)?;
    let shielded_value = spend_note.value.saturating_sub(fee);
    let output_value = Zatoshis::from_u64(shielded_value)
        .map_err(|_| NozyError::InvalidOperation("Invalid Sapling shield output value".into()))?;

    let recipient = destination_orchard_address(seed, 0)?;
    let fvk = sapling_extsk
        .to_diversifiable_full_viewing_key()
        .fvk()
        .clone();

    let transparent_signing_set = TransparentSigningSet::new();
    let mut rng = OsRng;
    let prover = zcash_proofs::prover::LocalTxProver::bundled();
    let empty_saks: &[SpendAuthorizingKey] = &[];

    for attempt in 1..=PILOT_EXPIRY_MAX_REBUILD_ATTEMPTS {
        if attempt > 1 {
            println!(
                "Sapling shield proof outran pilot expiry; rebuilding ({attempt}/{PILOT_EXPIRY_MAX_REBUILD_ATTEMPTS})"
            );
        }

        let zebra_tip = zebra.get_best_block_height().await?;
        let tip = sapling_witness_replay_tip(store, zebra_tip)?;
        let (sapling_anchor, merkle_path) =
            prepare_sapling_anchor_and_path(zebra, &spend_note, tip, store).await?;

        let use_ironwood = match network_type {
            NetworkType::Main => tip >= NU6_3_MAINNET_ACTIVATION_HEIGHT,
            NetworkType::Test | NetworkType::Regtest => TestNetwork
                .activation_height(zcash_protocol::consensus::NetworkUpgrade::Nu6_3)
                .is_some_and(|h| tip >= u32::from(h)),
        };

        let target_height = BlockHeight::from_u32(tip.saturating_add(1));
        let expiry_height_u32 = pilot_expiry_height(tip, expiry_delta_blocks);

        let ironwood_anchor = if use_ironwood {
            let ts = zebra.get_ironwood_tree_state(tip).await?;
            Some(
                orchard::Anchor::from_bytes(ts.anchor)
                    .into_option()
                    .ok_or_else(|| {
                        NozyError::InvalidOperation("Invalid Ironwood tip anchor".into())
                    })?,
            )
        } else {
            None
        };
        let orchard_anchor = if use_ironwood {
            None
        } else {
            let ts = zebra.get_orchard_tree_state(tip).await?;
            Some(
                orchard::Anchor::from_bytes(ts.anchor)
                    .into_option()
                    .ok_or_else(|| {
                        NozyError::InvalidOperation("Invalid Orchard tip anchor".into())
                    })?,
            )
        };

        let build_config = BuildConfig::Standard {
            sapling_anchor: Some(sapling_anchor),
            orchard_anchor,
            ironwood_anchor,
        };

        let built = match network_type {
            NetworkType::Main => {
                let mut builder =
                    Builder::new(IronwoodAwareMainNetwork, target_height, build_config);
                builder = builder.with_expiry_height(BlockHeight::from_u32(expiry_height_u32));
                builder
                    .add_sapling_spend::<core::convert::Infallible>(
                        fvk.clone(),
                        note.clone(),
                        merkle_path.clone(),
                    )
                    .map_err(|e| {
                        NozyError::InvalidOperation(format!("add_sapling_spend: {e:?}"))
                    })?;
                if use_ironwood {
                    builder
                        .add_ironwood_output::<core::convert::Infallible>(
                            None,
                            recipient.clone(),
                            output_value,
                            MemoBytes::empty(),
                        )
                        .map_err(|e| {
                            NozyError::InvalidOperation(format!("add_ironwood_output: {e:?}"))
                        })?;
                } else {
                    builder
                        .add_orchard_output::<core::convert::Infallible>(
                            None,
                            recipient.clone(),
                            output_value,
                            MemoBytes::empty(),
                        )
                        .map_err(|e| {
                            NozyError::InvalidOperation(format!("add_orchard_output: {e:?}"))
                        })?;
                }
                builder
                    .build(
                        &transparent_signing_set,
                        std::slice::from_ref(sapling_extsk),
                        empty_saks,
                        &mut rng,
                        &prover,
                        &prover,
                        &fee_rule,
                    )
                    .map_err(|e| {
                        NozyError::InvalidOperation(format!("Sapling shield build: {e:?}"))
                    })?
            }
            NetworkType::Test | NetworkType::Regtest => {
                let mut builder = Builder::new(TestNetwork, target_height, build_config);
                builder = builder.with_expiry_height(BlockHeight::from_u32(expiry_height_u32));
                builder
                    .add_sapling_spend::<core::convert::Infallible>(
                        fvk.clone(),
                        note.clone(),
                        merkle_path.clone(),
                    )
                    .map_err(|e| {
                        NozyError::InvalidOperation(format!("add_sapling_spend: {e:?}"))
                    })?;
                if use_ironwood {
                    builder
                        .add_ironwood_output::<core::convert::Infallible>(
                            None,
                            recipient.clone(),
                            output_value,
                            MemoBytes::empty(),
                        )
                        .map_err(|e| {
                            NozyError::InvalidOperation(format!("add_ironwood_output: {e:?}"))
                        })?;
                } else {
                    builder
                        .add_orchard_output::<core::convert::Infallible>(
                            None,
                            recipient.clone(),
                            output_value,
                            MemoBytes::empty(),
                        )
                        .map_err(|e| {
                            NozyError::InvalidOperation(format!("add_orchard_output: {e:?}"))
                        })?;
                }
                builder
                    .build(
                        &transparent_signing_set,
                        std::slice::from_ref(sapling_extsk),
                        empty_saks,
                        &mut rng,
                        &prover,
                        &prover,
                        &fee_rule,
                    )
                    .map_err(|e| {
                        NozyError::InvalidOperation(format!("Sapling shield build: {e:?}"))
                    })?
            }
        };

        let tx = built.transaction();
        let txid = tx.txid().to_string();
        let mut raw_transaction = Vec::new();
        tx.write(&mut raw_transaction).map_err(|e| {
            NozyError::InvalidOperation(format!("serialize Sapling shield tx: {e}"))
        })?;

        let tip_after = zebra.get_best_block_height().await?;
        if tip_after >= expiry_height_u32 {
            continue;
        }

        return Ok(SaplingShieldBuilt {
            raw_transaction,
            txid,
            expiry_height: expiry_height_u32,
            fee_zatoshis: fee,
            shielded_value_zatoshis: shielded_value,
            spent_nullifier_hex: hex::encode(&spend_note.nullifier_bytes),
        });
    }

    Err(NozyError::InvalidOperation(
        "Sapling shield failed: pilot expiry rebuild attempts exhausted".into(),
    ))
}

/// True when a note is ready to be selected for shield (rseed + witness).
pub fn sapling_note_ready_to_shield(note: &SerializableSaplingNote) -> bool {
    !note.spent
        && sapling_note_has_rseed(note)
        && note
            .sapling_incremental_witness_hex
            .as_ref()
            .is_some_and(|h| !h.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shield_fee_is_priority_zip317() {
        assert_eq!(sapling_shield_fee_zatoshis(), 15_000 * PRIORITY_MULTIPLIER);
    }
}
