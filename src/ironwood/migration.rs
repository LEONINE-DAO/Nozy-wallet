//! Orchard → Ironwood turnstile migration (NU6.3).
//!
//! This module tracks migration state and prebuilds locked V6 turnstile
//! transactions with the NU6.3 `zcash_primitives` builder. Broadcast remains
//! gated until scheduled-window validation and retry handling are complete.
//!
//! Privacy-preserving flow (ZIP 318 schedule + Shielded Labs Appendix A):
//! note-splitting, canonical `{1,2,5}×10^k` denominations (active default),
//! shared anchor-height buckets, and persisted scheduled/pre-signed migration
//! transactions. ZIP 318 power-of-ten remains a compatibility ladder.

use super::network_privacy::{selected_amount_timing_algorithm, AmountTimingAlgorithm};
use crate::error::{NozyError, NozyResult};
use crate::notes::{load_wallet_notes, SerializableOrchardNote, SpendableNote};
use crate::orchard_tx::OrchardWitnessProvider;
use crate::paths::get_wallet_data_dir;
use crate::shielded_pool::ShieldedPool;
use chrono::Utc;
use orchard::keys::{FullViewingKey, Scope, SpendAuthorizingKey};
use pczt::roles::{
    creator::Creator, io_finalizer::IoFinalizer, prover::Prover, signer::Signer,
    tx_extractor::TransactionExtractor,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use zcash_primitives::transaction::builder::{BuildConfig, Builder};
use zcash_primitives::transaction::fees::{transparent::InputSize, FeeRule};
use zcash_protocol::consensus::{BlockHeight, MainNetwork, NetworkType, Parameters, TestNetwork};
use zcash_protocol::memo::MemoBytes;
use zcash_protocol::value::Zatoshis;

/// Draft ZIP 318 bucket interval. At ~75 seconds/block this is about 5.3 hours.
pub const ZIP318_ANCHOR_BUCKET_INTERVAL_BLOCKS: u32 = 256;

/// Default upper bound for repeated same-denomination sends in a single bucket.
///
/// The final value should come from ZIP 318 once `K_MAX` is finalized.
pub const ZIP318_DEFAULT_K_MAX: u8 = 4;

pub const IRONWOOD_MIGRATION_SCHEDULE_FILE: &str = "ironwood_migration_schedule.json";
pub const ZIP318_TRANSFER_EXPIRY_BLOCKS: u32 = ZIP318_ANCHOR_BUCKET_INTERVAL_BLOCKS;
pub const MIGRATION_SCHEDULE_VERSION: u32 = 1;

/// Smallest Shielded Labs / Zooko `{1,2,5}×10^k` bucket (0.001 ZEC).
/// Residuals below this are abandoned rather than emitted as one-off turnstile sizes.
pub const ZOOKO_RESIDUAL_ABANDON_ZAT: u64 = 100_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationTransfer {
    pub nullifier_hex: String,
    pub value_zat: u64,
    pub block_height: u32,
    pub txid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationDenomination {
    pub value_zat: u64,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zip318ScheduleSummary {
    pub note_split_required: bool,
    pub anchor_bucket_interval_blocks: u32,
    pub k_max: u8,
    pub denomination_transfers: Vec<MigrationDenomination>,
    pub total_transfer_count: u32,
    pub next_anchor_bucket_height: Option<u32>,
    pub scheduled_transfers: Vec<MigrationScheduledTransfer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationPlanSummary {
    pub orchard_notes_to_migrate: usize,
    pub total_zatoshis: u64,
    pub transfers: Vec<MigrationTransfer>,
    pub ironwood_active: bool,
    pub zip318: Zip318ScheduleSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MigrationTransferStatus {
    Pending,
    Presigned,
    Broadcast,
    Confirmed,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationScheduledTransfer {
    pub sequence: u32,
    pub value_zat: u64,
    pub anchor_bucket_height: u32,
    pub not_before_height: u32,
    pub bucket_slot: u8,
    pub status: MigrationTransferStatus,
    #[serde(default)]
    pub presigned_tx_hex: Option<String>,
    #[serde(default)]
    pub prepared_txid: Option<String>,
    #[serde(default)]
    pub broadcast_txid: Option<String>,
    #[serde(default)]
    pub source_nullifier_hex: Option<String>,
    #[serde(default)]
    pub prepared_at_height: Option<u32>,
    #[serde(default)]
    pub expires_at_height: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationSchedule {
    pub version: u32,
    pub created_at_unix: i64,
    pub ironwood_active_when_created: bool,
    pub chain_tip_when_created: u32,
    pub total_zatoshis: u64,
    pub source_notes: Vec<MigrationTransfer>,
    pub anchor_bucket_interval_blocks: u32,
    pub k_max: u8,
    pub transfers: Vec<MigrationScheduledTransfer>,
}

#[derive(Debug, Clone)]
pub struct PreparedMigrationTransaction {
    pub sequence: u32,
    pub value_zat: u64,
    pub txid: String,
    pub raw_tx_hex: String,
    pub source_nullifier_hex: String,
    pub prepared_at_height: u32,
    pub expires_at_height: u32,
}

#[derive(Debug, Clone)]
pub struct MigrationExecutionResult {
    pub orchard_notes_to_migrate: usize,
    pub total_zatoshis: u64,
    pub total_transfer_count: u32,
    pub schedule_path: Option<PathBuf>,
    pub prepared: Option<PreparedMigrationTransaction>,
    pub readiness_state: MigrationReadinessState,
    pub blockers: Vec<String>,
    pub rebuilt_transfer_windows: usize,
}

#[derive(Debug, Clone)]
pub struct MigrationBroadcastResult {
    pub sequence: u32,
    pub txid: String,
    pub broadcast_at_height: u32,
    pub schedule_path: PathBuf,
    pub confirmed: bool,
    pub readiness_state: MigrationReadinessState,
    pub blockers: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationReadinessState {
    PlanningOnly,
    NoOrchardNotes,
    SplitRequired,
    ReadyToPrebuild,
    WaitingForWindow,
    PresignedWaitingForBroadcast,
    ReadyToBroadcast,
    Blocked,
}

impl MigrationReadinessState {
    pub fn label(self) -> &'static str {
        match self {
            Self::PlanningOnly => "planning-only",
            Self::NoOrchardNotes => "no-orchard-notes",
            Self::SplitRequired => "split-required",
            Self::ReadyToPrebuild => "ready-to-prebuild",
            Self::WaitingForWindow => "waiting-for-window",
            Self::PresignedWaitingForBroadcast => "presigned-waiting-for-broadcast",
            Self::ReadyToBroadcast => "ready-to-broadcast",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Debug, Clone)]
pub struct MigrationScheduleValidation {
    pub valid: bool,
    pub errors: Vec<String>,
    pub expired_transfer_count: usize,
    pub stale_presigned_count: usize,
}

#[derive(Debug, Clone)]
pub struct MigrationReadinessReport {
    pub state: MigrationReadinessState,
    pub blockers: Vec<String>,
    pub validation: Option<MigrationScheduleValidation>,
    pub next_eligible_transfer: Option<MigrationScheduledTransfer>,
    pub next_waiting_transfer: Option<MigrationScheduledTransfer>,
    pub active_presigned_transfer: Option<MigrationScheduledTransfer>,
}

struct FixedMigrationFeeRule {
    fee: Zatoshis,
}

impl FeeRule for FixedMigrationFeeRule {
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

pub fn previous_zip318_anchor_boundary(height: u32) -> u32 {
    height - (height % ZIP318_ANCHOR_BUCKET_INTERVAL_BLOCKS)
}

pub fn next_zip318_anchor_boundary(height: u32) -> u32 {
    let previous = previous_zip318_anchor_boundary(height);
    if previous == height {
        previous + ZIP318_ANCHOR_BUCKET_INTERVAL_BLOCKS
    } else {
        previous + ZIP318_ANCHOR_BUCKET_INTERVAL_BLOCKS
    }
}

fn canonical_power_of_ten_denominations(mut amount_zat: u64) -> Vec<MigrationDenomination> {
    if amount_zat == 0 {
        return Vec::new();
    }

    let mut powers = Vec::new();
    let mut denom = 1u64;
    while denom <= amount_zat {
        powers.push(denom);
        match denom.checked_mul(10) {
            Some(next) => denom = next,
            None => break,
        }
    }

    let mut result = Vec::new();
    for denom in powers.into_iter().rev() {
        let count = amount_zat / denom;
        if count > 0 {
            result.push(MigrationDenomination {
                value_zat: denom,
                count: count as u32,
            });
            amount_zat -= count * denom;
        }
    }
    result
}

/// Build ascending `{1,2,5}×10^k` buckets in zatoshis (k >= -3 → 0.001 ZEC upward).
fn zooko_125_buckets_up_to(max_zat: u64) -> Vec<u64> {
    let mut buckets = Vec::new();
    let mut base = ZOOKO_RESIDUAL_ABANDON_ZAT; // 0.001 ZEC
    loop {
        for &digit in &[1u64, 2, 5] {
            let Some(value) = base.checked_mul(digit) else {
                continue;
            };
            if value > max_zat {
                return buckets;
            }
            buckets.push(value);
        }
        match base.checked_mul(10) {
            Some(next) if next > base => base = next,
            _ => break,
        }
    }
    buckets
}

/// Greedy Shielded Labs / Zooko `{1,2,5}×10^k` decomposition for schedule planning.
/// Residuals below 0.001 ZEC are abandoned (not scheduled as unique turnstile sizes).
fn canonical_125_denominations(mut amount_zat: u64) -> Vec<MigrationDenomination> {
    if amount_zat < ZOOKO_RESIDUAL_ABANDON_ZAT {
        return Vec::new();
    }

    let buckets = zooko_125_buckets_up_to(amount_zat);
    let mut result = Vec::new();
    for &denom in buckets.iter().rev() {
        let count = amount_zat / denom;
        if count > 0 {
            result.push(MigrationDenomination {
                value_zat: denom,
                count: count as u32,
            });
            amount_zat -= count * denom;
        }
    }
    // Intentionally abandon residual < ZOOKO_RESIDUAL_ABANDON_ZAT.
    let _ = amount_zat;
    result
}

/// Active canonical ladder (Shielded Labs `{1,2,5}×10^k` by default).
fn canonical_denominations(amount_zat: u64) -> Vec<MigrationDenomination> {
    match selected_amount_timing_algorithm() {
        AmountTimingAlgorithm::Zip318PowerOfTen => canonical_power_of_ten_denominations(amount_zat),
        AmountTimingAlgorithm::Zooko125 => canonical_125_denominations(amount_zat),
    }
}

/// Appendix A amount selection for one migration round (probabilistic step-down).
///
/// Starts at the largest bucket ≤ `current_balance_zat`, then fair-coin steps down
/// on heads until tails or the smallest bucket. Returns `None` when the residual
/// should be abandoned (< 0.001 ZEC).
pub fn select_zooko_round_amount<R: rand::RngCore>(
    current_balance_zat: u64,
    rng: &mut R,
) -> Option<u64> {
    if current_balance_zat < ZOOKO_RESIDUAL_ABANDON_ZAT {
        return None;
    }
    let buckets = zooko_125_buckets_up_to(current_balance_zat);
    if buckets.is_empty() {
        return None;
    }
    let mut idx = buckets.len() - 1;
    while idx > 0 {
        // Heads → step down; tails → stop.
        if rng.next_u32() & 1 == 0 {
            break;
        }
        idx -= 1;
    }
    Some(buckets[idx])
}

fn schedule_canonical_transfers(
    denomination_transfers: &[MigrationDenomination],
    chain_tip: u32,
    k_max: u8,
) -> Vec<MigrationScheduledTransfer> {
    if denomination_transfers.is_empty() {
        return Vec::new();
    }

    let first_bucket = next_zip318_anchor_boundary(chain_tip);
    let mut scheduled = Vec::new();
    let mut sequence = 0u32;

    for denomination in denomination_transfers {
        let mut remaining = denomination.count;
        let mut bucket_height = first_bucket;
        while remaining > 0 {
            let batch = remaining.min(u32::from(k_max.max(1)));
            for bucket_index in 0..batch {
                sequence += 1;
                scheduled.push(MigrationScheduledTransfer {
                    sequence,
                    value_zat: denomination.value_zat,
                    anchor_bucket_height: bucket_height,
                    not_before_height: bucket_height,
                    bucket_slot: (bucket_index + 1) as u8,
                    status: MigrationTransferStatus::Pending,
                    presigned_tx_hex: None,
                    prepared_txid: None,
                    broadcast_txid: None,
                    source_nullifier_hex: None,
                    prepared_at_height: None,
                    expires_at_height: None,
                });
            }
            remaining -= batch;
            bucket_height = bucket_height.saturating_add(ZIP318_ANCHOR_BUCKET_INTERVAL_BLOCKS);
        }
    }

    scheduled
}

pub fn build_schedule_from_plan(plan: &MigrationPlanSummary, chain_tip: u32) -> MigrationSchedule {
    MigrationSchedule {
        version: MIGRATION_SCHEDULE_VERSION,
        created_at_unix: Utc::now().timestamp(),
        ironwood_active_when_created: plan.ironwood_active,
        chain_tip_when_created: chain_tip,
        total_zatoshis: plan.total_zatoshis,
        source_notes: plan.transfers.clone(),
        anchor_bucket_interval_blocks: plan.zip318.anchor_bucket_interval_blocks,
        k_max: plan.zip318.k_max,
        transfers: plan.zip318.scheduled_transfers.clone(),
    }
}

pub fn ironwood_migration_schedule_path() -> PathBuf {
    get_wallet_data_dir().join(IRONWOOD_MIGRATION_SCHEDULE_FILE)
}

pub fn load_orchard_migration_schedule() -> NozyResult<Option<MigrationSchedule>> {
    let path = ironwood_migration_schedule_path();
    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| NozyError::Storage(format!("Failed to read Ironwood schedule: {e}")))?;
    serde_json::from_str(&content)
        .map(Some)
        .map_err(|e| NozyError::Storage(format!("Failed to parse Ironwood schedule: {e}")))
}

pub fn save_orchard_migration_schedule(schedule: &MigrationSchedule) -> NozyResult<PathBuf> {
    let path = ironwood_migration_schedule_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            NozyError::Storage(format!("Failed to create Ironwood schedule directory: {e}"))
        })?;
    }

    let serialized = serde_json::to_string_pretty(schedule)
        .map_err(|e| NozyError::Storage(format!("Failed to serialize Ironwood schedule: {e}")))?;
    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, serialized)
        .map_err(|e| NozyError::Storage(format!("Failed to write Ironwood schedule: {e}")))?;
    fs::rename(&temp_path, &path)
        .map_err(|e| NozyError::Storage(format!("Failed to save Ironwood schedule: {e}")))?;
    Ok(path)
}

pub fn save_orchard_migration_plan_at(
    ironwood_active: bool,
    chain_tip: u32,
) -> NozyResult<(MigrationSchedule, PathBuf)> {
    if let Some(existing) = load_orchard_migration_schedule()? {
        if active_presigned_transfer(&existing, chain_tip).is_some() {
            return Err(NozyError::InvalidOperation(
                "Refusing to rebuild the migration schedule while a presigned turnstile transaction \
                 is waiting for broadcast. Run `nozy ironwood broadcast` first.".to_string(),
            ));
        }
    }
    let plan = plan_orchard_migration_at(ironwood_active, chain_tip)?;
    let schedule = build_schedule_from_plan(&plan, chain_tip);
    let path = save_orchard_migration_schedule(&schedule)?;
    Ok((schedule, path))
}

fn transfer_count_by_value(transfers: &[MigrationScheduledTransfer]) -> HashMap<u64, u32> {
    let mut counts = HashMap::new();
    for transfer in transfers {
        *counts.entry(transfer.value_zat).or_insert(0) += 1;
    }
    counts
}

fn denomination_count_by_value(denominations: &[MigrationDenomination]) -> HashMap<u64, u32> {
    let mut counts = HashMap::new();
    for denomination in denominations {
        *counts.entry(denomination.value_zat).or_insert(0) += denomination.count;
    }
    counts
}

fn sorted_source_notes(transfers: &[MigrationTransfer]) -> Vec<(String, u64, u32)> {
    let mut notes: Vec<_> = transfers
        .iter()
        .map(|transfer| {
            (
                transfer.nullifier_hex.clone(),
                transfer.value_zat,
                transfer.block_height,
            )
        })
        .collect();
    notes.sort();
    notes
}

pub fn validate_orchard_migration_schedule(
    schedule: &MigrationSchedule,
    plan: &MigrationPlanSummary,
    chain_tip: u32,
) -> MigrationScheduleValidation {
    let mut errors = Vec::new();
    let mut expired_transfer_count = 0usize;
    let mut stale_presigned_count = 0usize;

    if schedule.version != MIGRATION_SCHEDULE_VERSION {
        errors.push(format!(
            "Unsupported schedule version {} (expected {})",
            schedule.version, MIGRATION_SCHEDULE_VERSION
        ));
    }
    if schedule.anchor_bucket_interval_blocks != ZIP318_ANCHOR_BUCKET_INTERVAL_BLOCKS {
        errors.push(format!(
            "Schedule bucket interval {} does not match ZIP 318 interval {}",
            schedule.anchor_bucket_interval_blocks, ZIP318_ANCHOR_BUCKET_INTERVAL_BLOCKS
        ));
    }
    if schedule.k_max == 0 || schedule.k_max != ZIP318_DEFAULT_K_MAX {
        errors.push(format!(
            "Schedule K_MAX {} does not match configured K_MAX {}",
            schedule.k_max, ZIP318_DEFAULT_K_MAX
        ));
    }
    if schedule.total_zatoshis != plan.total_zatoshis {
        errors.push(format!(
            "Schedule total {} zat no longer matches wallet Orchard total {} zat",
            schedule.total_zatoshis, plan.total_zatoshis
        ));
    }
    if sorted_source_notes(&schedule.source_notes) != sorted_source_notes(&plan.transfers) {
        errors
            .push("Schedule source notes no longer match active wallet Orchard notes".to_string());
    }
    if schedule.transfers.len() != plan.zip318.total_transfer_count as usize {
        errors.push(format!(
            "Schedule has {} transfers but current plan expects {}",
            schedule.transfers.len(),
            plan.zip318.total_transfer_count
        ));
    }

    let scheduled_total: u64 = schedule
        .transfers
        .iter()
        .map(|transfer| transfer.value_zat)
        .sum();
    if scheduled_total != schedule.total_zatoshis {
        errors.push(format!(
            "Scheduled transfer total {} zat does not match schedule total {} zat",
            scheduled_total, schedule.total_zatoshis
        ));
    }
    if transfer_count_by_value(&schedule.transfers)
        != denomination_count_by_value(&plan.zip318.denomination_transfers)
    {
        errors.push("Schedule denominations no longer match current ZIP 318 plan".to_string());
    }

    let mut bucket_counts: HashMap<(u64, u32), u32> = HashMap::new();
    let mut seen_slots: HashMap<(u64, u32, u8), u32> = HashMap::new();
    for (index, transfer) in schedule.transfers.iter().enumerate() {
        let expected_sequence = (index + 1) as u32;
        if transfer.sequence != expected_sequence {
            errors.push(format!(
                "Transfer sequence {} is out of order (expected {})",
                transfer.sequence, expected_sequence
            ));
        }
        if transfer.value_zat == 0 {
            errors.push(format!("Transfer #{} has zero value", transfer.sequence));
        }
        if transfer.anchor_bucket_height % ZIP318_ANCHOR_BUCKET_INTERVAL_BLOCKS != 0 {
            errors.push(format!(
                "Transfer #{} anchor bucket {} is not on a ZIP 318 boundary",
                transfer.sequence, transfer.anchor_bucket_height
            ));
        }
        if transfer.not_before_height != transfer.anchor_bucket_height {
            errors.push(format!(
                "Transfer #{} not-before height {} must equal anchor bucket {}",
                transfer.sequence, transfer.not_before_height, transfer.anchor_bucket_height
            ));
        }
        if transfer.bucket_slot == 0 || transfer.bucket_slot > schedule.k_max {
            errors.push(format!(
                "Transfer #{} bucket slot {} is outside 1..={}",
                transfer.sequence, transfer.bucket_slot, schedule.k_max
            ));
        }

        let bucket_key = (transfer.value_zat, transfer.anchor_bucket_height);
        *bucket_counts.entry(bucket_key).or_insert(0) += 1;
        let slot_key = (
            transfer.value_zat,
            transfer.anchor_bucket_height,
            transfer.bucket_slot,
        );
        *seen_slots.entry(slot_key).or_insert(0) += 1;

        if transfer_window_expired(transfer, chain_tip) {
            expired_transfer_count += 1;
            if transfer.status == MigrationTransferStatus::Presigned {
                stale_presigned_count += 1;
            }
        }

        match transfer.status {
            MigrationTransferStatus::Pending => {
                if transfer.presigned_tx_hex.is_some()
                    || transfer.prepared_txid.is_some()
                    || transfer.source_nullifier_hex.is_some()
                    || transfer.prepared_at_height.is_some()
                {
                    errors.push(format!(
                        "Pending transfer #{} contains presigned transaction state",
                        transfer.sequence
                    ));
                }
            }
            MigrationTransferStatus::Presigned => {
                if transfer.presigned_tx_hex.is_none() || transfer.prepared_txid.is_none() {
                    errors.push(format!(
                        "Presigned transfer #{} is missing tx hex or txid",
                        transfer.sequence
                    ));
                }
                if transfer.prepared_at_height.is_none() || transfer.expires_at_height.is_none() {
                    errors.push(format!(
                        "Presigned transfer #{} is missing prepared/expiry heights",
                        transfer.sequence
                    ));
                }
                if transfer.source_nullifier_hex.is_none() {
                    errors.push(format!(
                        "Presigned transfer #{} is missing source nullifier",
                        transfer.sequence
                    ));
                }
            }
            MigrationTransferStatus::Broadcast => {
                if transfer.broadcast_txid.is_none() {
                    errors.push(format!(
                        "Broadcast transfer #{} is missing broadcast txid",
                        transfer.sequence
                    ));
                }
            }
            MigrationTransferStatus::Confirmed | MigrationTransferStatus::Expired => {}
        }
    }

    for ((value_zat, bucket_height), count) in bucket_counts {
        if count > u32::from(schedule.k_max.max(1)) {
            errors.push(format!(
                "Bucket {} has {} transfers of {} zat, exceeding K_MAX {}",
                bucket_height, count, value_zat, schedule.k_max
            ));
        }
    }
    for ((value_zat, bucket_height, slot), count) in seen_slots {
        if count > 1 {
            errors.push(format!(
                "Bucket {} repeats slot {} for {} zat transfers",
                bucket_height, slot, value_zat
            ));
        }
    }

    MigrationScheduleValidation {
        valid: errors.is_empty(),
        errors,
        expired_transfer_count,
        stale_presigned_count,
    }
}

fn transfer_window_expired(transfer: &MigrationScheduledTransfer, chain_tip: u32) -> bool {
    matches!(
        transfer.status,
        MigrationTransferStatus::Pending | MigrationTransferStatus::Presigned
    ) && chain_tip
        > transfer
            .not_before_height
            .saturating_add(ZIP318_TRANSFER_EXPIRY_BLOCKS)
}

pub fn refresh_orchard_migration_schedule_at(
    ironwood_active: bool,
    chain_tip: u32,
) -> NozyResult<Option<(MigrationSchedule, PathBuf, usize)>> {
    let Some(existing) = load_orchard_migration_schedule()? else {
        return Ok(None);
    };

    let expired_count = existing
        .transfers
        .iter()
        .filter(|transfer| transfer_window_expired(transfer, chain_tip))
        .count();
    if expired_count == 0 {
        return Ok(Some((existing, ironwood_migration_schedule_path(), 0)));
    }

    let plan = plan_orchard_migration_at(ironwood_active, chain_tip)?;
    let rebuilt = build_schedule_from_plan(&plan, chain_tip);
    let path = save_orchard_migration_schedule(&rebuilt)?;
    Ok(Some((rebuilt, path, expired_count)))
}

fn next_eligible_pending_transfer(
    schedule: &MigrationSchedule,
    chain_tip: u32,
) -> Option<MigrationScheduledTransfer> {
    schedule
        .transfers
        .iter()
        .filter(|transfer| transfer_window_eligible(transfer, chain_tip))
        .min_by_key(|transfer| transfer.sequence)
        .cloned()
}

fn transfer_window_eligible(transfer: &MigrationScheduledTransfer, chain_tip: u32) -> bool {
    transfer.status == MigrationTransferStatus::Pending
        && chain_tip >= transfer.not_before_height
        && !transfer_window_expired(transfer, chain_tip)
}

/// Next in-window pending transfer that at least one Orchard note can fund (plus fee).
fn next_spendable_eligible_pending_transfer(
    schedule: &MigrationSchedule,
    chain_tip: u32,
    orchard_note_values: &[u64],
    fee_zatoshis: u64,
) -> Option<MigrationScheduledTransfer> {
    schedule
        .transfers
        .iter()
        .filter(|transfer| transfer_window_eligible(transfer, chain_tip))
        .filter(|transfer| {
            can_cover_transfer_with_note_values(
                transfer.value_zat,
                orchard_note_values,
                fee_zatoshis,
            )
        })
        .min_by_key(|transfer| transfer.sequence)
        .cloned()
}

fn can_cover_transfer_with_note_values(
    transfer_value_zat: u64,
    orchard_note_values: &[u64],
    fee_zatoshis: u64,
) -> bool {
    orchard_note_values.iter().any(|&value| {
        value >= transfer_value_zat.saturating_add(fee_zatoshis)
            || (value == transfer_value_zat && value > fee_zatoshis)
    })
}

fn next_waiting_pending_transfer(
    schedule: &MigrationSchedule,
) -> Option<MigrationScheduledTransfer> {
    schedule
        .transfers
        .iter()
        .filter(|transfer| transfer.status == MigrationTransferStatus::Pending)
        .min_by_key(|transfer| transfer.sequence)
        .cloned()
}

fn active_presigned_transfer(
    schedule: &MigrationSchedule,
    chain_tip: u32,
) -> Option<MigrationScheduledTransfer> {
    schedule
        .transfers
        .iter()
        .filter(|transfer| {
            transfer.status == MigrationTransferStatus::Presigned
                && !transfer_window_expired(transfer, chain_tip)
        })
        .min_by_key(|transfer| transfer.sequence)
        .cloned()
}

pub fn assess_orchard_migration_readiness(
    ironwood_active: bool,
    chain_tip: u32,
    plan: &MigrationPlanSummary,
    schedule: Option<&MigrationSchedule>,
    witness_lag_blocks: Option<u32>,
    max_witness_lag_blocks: Option<u32>,
) -> MigrationReadinessReport {
    assess_orchard_migration_readiness_with_spendability(
        ironwood_active,
        chain_tip,
        plan,
        schedule,
        witness_lag_blocks,
        max_witness_lag_blocks,
        None,
        None,
    )
}

pub fn assess_orchard_migration_readiness_with_spendability(
    ironwood_active: bool,
    chain_tip: u32,
    plan: &MigrationPlanSummary,
    schedule: Option<&MigrationSchedule>,
    witness_lag_blocks: Option<u32>,
    max_witness_lag_blocks: Option<u32>,
    orchard_note_values: Option<&[u64]>,
    migration_fee_zatoshis: Option<u64>,
) -> MigrationReadinessReport {
    let mut blockers = Vec::new();

    if plan.orchard_notes_to_migrate == 0 {
        return MigrationReadinessReport {
            state: MigrationReadinessState::NoOrchardNotes,
            blockers,
            validation: schedule.map(|s| validate_orchard_migration_schedule(s, plan, chain_tip)),
            next_eligible_transfer: None,
            next_waiting_transfer: None,
            active_presigned_transfer: None,
        };
    }

    if !ironwood_active {
        blockers.push(
            "Ironwood (NU6.3) is not active on this network yet; migration is planning-only."
                .to_string(),
        );
        return MigrationReadinessReport {
            state: MigrationReadinessState::PlanningOnly,
            blockers,
            validation: schedule.map(|s| validate_orchard_migration_schedule(s, plan, chain_tip)),
            next_eligible_transfer: None,
            next_waiting_transfer: schedule.and_then(next_waiting_pending_transfer),
            active_presigned_transfer: None,
        };
    }

    if let (Some(lag), Some(max_lag)) = (witness_lag_blocks, max_witness_lag_blocks) {
        if lag > max_lag {
            blockers.push(format!(
                "Orchard witnesses are {lag} blocks behind chain tip; sync before migration (max {max_lag})."
            ));
        }
    }

    if plan.zip318.note_split_required {
        blockers.push(
            "ZIP 318 note splitting is required before safe turnstile prebuilds. \
             Run planning/preflight only until the split phase is implemented."
                .to_string(),
        );
    }

    let validation = schedule.map(|s| validate_orchard_migration_schedule(s, plan, chain_tip));
    if let Some(validation) = validation.as_ref() {
        blockers.extend(validation.errors.iter().cloned());
    }

    if !blockers.is_empty() {
        let state = if plan.zip318.note_split_required
            && blockers.len() == 1
            && witness_lag_blocks
                .zip(max_witness_lag_blocks)
                .is_none_or(|(lag, max_lag)| lag <= max_lag)
            && validation.as_ref().is_none_or(|v| v.valid)
        {
            MigrationReadinessState::SplitRequired
        } else {
            MigrationReadinessState::Blocked
        };
        return MigrationReadinessReport {
            state,
            blockers,
            validation,
            next_eligible_transfer: None,
            next_waiting_transfer: schedule.and_then(next_waiting_pending_transfer),
            active_presigned_transfer: schedule
                .and_then(|s| active_presigned_transfer(s, chain_tip)),
        };
    }

    let active_presigned_transfer = schedule.and_then(|s| active_presigned_transfer(s, chain_tip));
    if let Some(ref transfer) = active_presigned_transfer {
        if presigned_transfer_broadcastable(transfer, chain_tip).is_ok() {
            return MigrationReadinessReport {
                state: MigrationReadinessState::ReadyToBroadcast,
                blockers: vec![
                    "Run `nozy ironwood broadcast` to submit the presigned turnstile transaction."
                        .to_string(),
                ],
                validation,
                next_eligible_transfer: None,
                next_waiting_transfer: schedule.and_then(next_waiting_pending_transfer),
                active_presigned_transfer: active_presigned_transfer.clone(),
            };
        }
        let blocker = presigned_transfer_broadcastable(transfer, chain_tip)
            .err()
            .unwrap_or_else(|| "Presigned transaction is not yet broadcastable.".to_string());
        return MigrationReadinessReport {
            state: MigrationReadinessState::PresignedWaitingForBroadcast,
            blockers: vec![blocker],
            validation,
            next_eligible_transfer: None,
            next_waiting_transfer: schedule.and_then(next_waiting_pending_transfer),
            active_presigned_transfer: active_presigned_transfer.clone(),
        };
    }

    let next_eligible_transfer =
        schedule.and_then(|s| next_eligible_pending_transfer(s, chain_tip));
    if let Some(sched) = schedule {
        if let (Some(values), Some(fee)) = (orchard_note_values, migration_fee_zatoshis) {
            if let Some(spendable) =
                next_spendable_eligible_pending_transfer(sched, chain_tip, values, fee)
            {
                if next_eligible_transfer
                    .as_ref()
                    .is_some_and(|t| t.sequence != spendable.sequence)
                {
                    blockers.push(format!(
                        "Scheduled transfer #{} ({} zat) is not fundable from current Orchard notes; \
                         next spendable transfer is #{} ({} zat).",
                        next_eligible_transfer.as_ref().map(|t| t.sequence).unwrap_or(0),
                        next_eligible_transfer
                            .as_ref()
                            .map(|t| t.value_zat)
                            .unwrap_or(0),
                        spendable.sequence,
                        spendable.value_zat
                    ));
                }
                return MigrationReadinessReport {
                    state: MigrationReadinessState::ReadyToPrebuild,
                    blockers,
                    validation,
                    next_eligible_transfer: Some(spendable),
                    next_waiting_transfer: next_waiting_pending_transfer(sched),
                    active_presigned_transfer: None,
                };
            }
            if next_eligible_transfer.is_some() {
                blockers.push(
                    "No in-window ZIP 318 transfer can be funded from current Orchard notes plus fee. \
                     Consolidate fee-dust notes or wait for note splitting support."
                        .to_string(),
                );
                return MigrationReadinessReport {
                    state: MigrationReadinessState::SplitRequired,
                    blockers,
                    validation,
                    next_eligible_transfer: None,
                    next_waiting_transfer: next_waiting_pending_transfer(sched),
                    active_presigned_transfer: None,
                };
            }
        } else if next_eligible_transfer.is_some() {
            return MigrationReadinessReport {
                state: MigrationReadinessState::ReadyToPrebuild,
                blockers,
                validation,
                next_eligible_transfer,
                next_waiting_transfer: next_waiting_pending_transfer(sched),
                active_presigned_transfer: None,
            };
        }
    } else if next_eligible_transfer.is_some() {
        return MigrationReadinessReport {
            state: MigrationReadinessState::ReadyToPrebuild,
            blockers,
            validation,
            next_eligible_transfer,
            next_waiting_transfer: schedule.and_then(next_waiting_pending_transfer),
            active_presigned_transfer: None,
        };
    }

    MigrationReadinessReport {
        state: MigrationReadinessState::WaitingForWindow,
        blockers,
        validation,
        next_eligible_transfer: None,
        next_waiting_transfer: schedule.and_then(next_waiting_pending_transfer),
        active_presigned_transfer: None,
    }
}

fn can_cover_transfer_with_current_notes(
    transfer: &MigrationScheduledTransfer,
    spendable_notes: &[SpendableNote],
    fee_zatoshis: u64,
) -> bool {
    select_migration_spend(spendable_notes, transfer.value_zat, fee_zatoshis).is_ok()
}

struct MigrationSpendSelection<'a> {
    spend_note: &'a SpendableNote,
    ironwood_output_zat: u64,
}

/// Pick one Orchard note for a ZIP 318 turnstile prebuild.
///
/// Prefer a note with headroom for `transfer + fee`. When the wallet holds an exact canonical
/// denomination note (e.g. 1.0 ZEC from note splitting), deduct the fee from the Ironwood output.
fn select_migration_spend<'a>(
    spendable_notes: &'a [SpendableNote],
    transfer_value_zat: u64,
    fee_zatoshis: u64,
) -> NozyResult<MigrationSpendSelection<'a>> {
    if let Ok(spend_note) = crate::orchard_tx::select_single_spend_note(
        spendable_notes,
        transfer_value_zat,
        fee_zatoshis,
    ) {
        return Ok(MigrationSpendSelection {
            spend_note,
            ironwood_output_zat: transfer_value_zat,
        });
    }

    if let Some(spend_note) = spendable_notes
        .iter()
        .filter(|note| !note.orchard_note.spent)
        .find(|note| {
            note.orchard_note.value == transfer_value_zat && note.orchard_note.value > fee_zatoshis
        })
    {
        return Ok(MigrationSpendSelection {
            spend_note,
            ironwood_output_zat: transfer_value_zat.saturating_sub(fee_zatoshis),
        });
    }

    Err(NozyError::InvalidOperation(format!(
        "No Orchard note covers ZIP 318 transfer {transfer_value_zat} zat plus {fee_zatoshis} zat fee."
    )))
}

fn load_or_rebuild_orchard_migration_schedule(
    _ironwood_active: bool,
    chain_tip: u32,
    plan: &MigrationPlanSummary,
) -> NozyResult<(MigrationSchedule, PathBuf, usize)> {
    if let Some(existing) = load_orchard_migration_schedule()? {
        let validation = validate_orchard_migration_schedule(&existing, plan, chain_tip);
        if validation.valid && validation.expired_transfer_count == 0 {
            return Ok((existing, ironwood_migration_schedule_path(), 0));
        }

        let rebuilt = build_schedule_from_plan(plan, chain_tip);
        let path = save_orchard_migration_schedule(&rebuilt)?;
        return Ok((rebuilt, path, validation.expired_transfer_count));
    }

    let rebuilt = build_schedule_from_plan(plan, chain_tip);
    let path = save_orchard_migration_schedule(&rebuilt)?;
    Ok((rebuilt, path, 0))
}

/// List unspent Orchard-pool notes that should migrate after NU6.3 activation.
pub fn plan_orchard_migration(ironwood_active: bool) -> NozyResult<MigrationPlanSummary> {
    plan_orchard_migration_at(ironwood_active, 0)
}

/// List unspent Orchard-pool notes and produce a ZIP 318 draft schedule at `chain_tip`.
pub fn plan_orchard_migration_at(
    ironwood_active: bool,
    chain_tip: u32,
) -> NozyResult<MigrationPlanSummary> {
    let notes = load_wallet_notes().unwrap_or_default();
    let orchard_unspent: Vec<&SerializableOrchardNote> = notes
        .iter()
        .filter(|n| !n.spent && n.pool == ShieldedPool::Orchard)
        .collect();

    let transfers: Vec<MigrationTransfer> = orchard_unspent
        .iter()
        .map(|n| MigrationTransfer {
            nullifier_hex: hex::encode(&n.nullifier_bytes),
            value_zat: n.value,
            block_height: n.block_height,
            txid: n.txid.clone(),
        })
        .collect();

    let total_zatoshis = transfers.iter().map(|t| t.value_zat).sum();
    let denomination_transfers = canonical_denominations(total_zatoshis);
    let total_transfer_count = denomination_transfers.iter().map(|d| d.count).sum();
    let scheduled_transfers =
        schedule_canonical_transfers(&denomination_transfers, chain_tip, ZIP318_DEFAULT_K_MAX);

    Ok(MigrationPlanSummary {
        orchard_notes_to_migrate: transfers.len(),
        total_zatoshis,
        transfers,
        ironwood_active,
        zip318: Zip318ScheduleSummary {
            note_split_required: orchard_wallet_needs_note_split(&notes),
            anchor_bucket_interval_blocks: ZIP318_ANCHOR_BUCKET_INTERVAL_BLOCKS,
            k_max: ZIP318_DEFAULT_K_MAX,
            denomination_transfers,
            total_transfer_count,
            next_anchor_bucket_height: scheduled_transfers
                .first()
                .map(|transfer| transfer.anchor_bucket_height),
            scheduled_transfers,
        },
    })
}

fn parse_network_type(info: &std::collections::HashMap<String, serde_json::Value>) -> NetworkType {
    let chain = info.get("chain").and_then(|v| v.as_str()).unwrap_or("main");
    if matches!(chain, "test" | "testnet" | "regtest") {
        NetworkType::Test
    } else {
        NetworkType::Main
    }
}

async fn build_migration_transaction_for_transfer(
    zebra: &crate::zebra_integration::ZebraClient,
    network: NetworkType,
    chain_tip: u32,
    transfer: &MigrationScheduledTransfer,
    spendable_notes: &[SpendableNote],
    fee_zatoshis: u64,
) -> NozyResult<PreparedMigrationTransaction> {
    let selection = select_migration_spend(spendable_notes, transfer.value_zat, fee_zatoshis)
        .map_err(|_| {
            NozyError::InvalidOperation(
                "No current Orchard note can cover the next scheduled ZIP 318 transfer plus fee. \
                     Run the note-splitting phase before prebuilding turnstile transactions."
                    .to_string(),
            )
        })?;
    let spend_note = selection.spend_note;
    let ironwood_output_zat = selection.ironwood_output_zat;
    let total_input = spend_note.orchard_note.value;
    let change = total_input
        .saturating_sub(ironwood_output_zat)
        .saturating_sub(fee_zatoshis);
    let source_nullifier_hex = hex::encode(spend_note.orchard_note.nullifier.to_bytes());

    let witness_provider = crate::orchard_tx::ZebraJsonRpcOrchardWitnessProvider;
    let (orchard_anchor, merkle_path) = witness_provider
        .prepare_spend_anchor_and_path(zebra, spend_note, chain_tip)
        .await?;

    let target_height = BlockHeight::from_u32(chain_tip.saturating_add(1));
    let fvk = FullViewingKey::from(&spend_note.spending_key);
    let self_ironwood_address = fvk.to_ivk(Scope::Internal).address_at(0u64);
    let transfer_value = Zatoshis::from_u64(ironwood_output_zat).map_err(|_| {
        NozyError::InvalidOperation("Invalid Ironwood migration transfer amount".to_string())
    })?;
    let fee = Zatoshis::from_u64(fee_zatoshis)
        .map_err(|_| NozyError::InvalidOperation("Invalid migration fee".to_string()))?;
    let fee_rule = FixedMigrationFeeRule { fee };

    let build_config = BuildConfig::Standard {
        sapling_anchor: None,
        orchard_anchor: Some(orchard_anchor),
        // Output-only Ironwood bundles use the empty Ironwood tree anchor.
        ironwood_anchor: Some(orchard::Anchor::empty_tree()),
    };

    macro_rules! build_pczt_for_network {
        ($params:expr) => {{
            let mut builder = Builder::new($params, target_height, build_config);
            builder
                .add_orchard_spend::<core::convert::Infallible>(
                    fvk.clone(),
                    spend_note.orchard_note.note.clone(),
                    merkle_path,
                )
                .map_err(|e| NozyError::InvalidOperation(format!("add_orchard_spend: {e:?}")))?;
            builder
                .add_ironwood_output::<core::convert::Infallible>(
                    None,
                    self_ironwood_address.clone(),
                    transfer_value,
                    MemoBytes::empty(),
                )
                .map_err(|e| NozyError::InvalidOperation(format!("add_ironwood_output: {e:?}")))?;

            if change > 0 {
                let change_value = Zatoshis::from_u64(change).map_err(|_| {
                    NozyError::InvalidOperation("Invalid Ironwood change amount".to_string())
                })?;
                builder
                    .add_ironwood_output::<core::convert::Infallible>(
                        None,
                        self_ironwood_address.clone(),
                        change_value,
                        MemoBytes::empty(),
                    )
                    .map_err(|e| {
                        NozyError::InvalidOperation(format!("add_ironwood_output (change): {e:?}"))
                    })?;
            }

            let parts = builder
                .build_for_pczt(rand::rngs::OsRng, &fee_rule)
                .map_err(|e| NozyError::InvalidOperation(format!("build_for_pczt: {e:?}")))?
                .pczt_parts;
            Creator::build_from_parts(parts).ok_or_else(|| {
                NozyError::InvalidOperation(
                    "PCZT creator: incompatible V6 migration parts".to_string(),
                )
            })
        }};
    }

    let pczt = match network {
        NetworkType::Main => build_pczt_for_network!(MainNetwork)?,
        NetworkType::Test | NetworkType::Regtest => build_pczt_for_network!(TestNetwork)?,
    };

    let proving_key =
        orchard::circuit::ProvingKey::build(orchard::circuit::OrchardCircuitVersion::PostNu6_3);
    let pczt = Prover::new(pczt)
        .create_orchard_proof(&proving_key)
        .map_err(|e| NozyError::InvalidOperation(format!("create_orchard_proof: {e:?}")))?
        .create_ironwood_proof(&proving_key)
        .map_err(|e| NozyError::InvalidOperation(format!("create_ironwood_proof: {e:?}")))?
        .finish();

    let pczt = IoFinalizer::new(pczt)
        .finalize_io()
        .map_err(|e| NozyError::InvalidOperation(format!("migration io_finalize: {e:?}")))?;

    let ask = SpendAuthorizingKey::from(&spend_note.spending_key);
    let orchard_action_count = pczt.orchard().actions().len();
    let mut signer = Signer::new(pczt)
        .map_err(|e| NozyError::InvalidOperation(format!("migration signer init: {e:?}")))?;
    let mut signed_any = false;
    for index in 0..orchard_action_count {
        if signer.sign_orchard(index, &ask).is_ok() {
            signed_any = true;
        }
    }
    if !signed_any {
        return Err(NozyError::InvalidOperation(
            "No Orchard migration spend signatures were applied.".to_string(),
        ));
    }
    let pczt = signer.finish();

    let verifying_key =
        orchard::circuit::VerifyingKey::build(orchard::circuit::OrchardCircuitVersion::PostNu6_3);
    let tx = TransactionExtractor::new(pczt)
        .with_orchard(&verifying_key)
        .extract()
        .map_err(|e| NozyError::InvalidOperation(format!("migration tx extract: {e:?}")))?;

    let txid = tx.txid().to_string();
    let mut raw_transaction = Vec::new();
    tx.write(&mut raw_transaction)
        .map_err(|e| NozyError::InvalidOperation(format!("migration tx serialize: {e}")))?;

    Ok(PreparedMigrationTransaction {
        sequence: transfer.sequence,
        value_zat: transfer.value_zat,
        txid,
        raw_tx_hex: hex::encode(raw_transaction),
        source_nullifier_hex,
        prepared_at_height: chain_tip,
        expires_at_height: transfer
            .not_before_height
            .saturating_add(ZIP318_TRANSFER_EXPIRY_BLOCKS),
    })
}

/// Execute migration — builds turnstile txs (one note per tx until multi-spend lands).
pub async fn execute_orchard_migration(
    zebra_url: &str,
    ironwood_active: bool,
    spendable_notes: &[SpendableNote],
) -> NozyResult<MigrationExecutionResult> {
    let zebra = crate::zebra_integration::ZebraClient::new(zebra_url.to_string());
    let chain_tip = zebra.get_best_block_height().await?;
    let info = zebra.get_blockchain_info().await?;
    let network = parse_network_type(&info);
    let plan = plan_orchard_migration_at(ironwood_active, chain_tip)?;

    if plan.orchard_notes_to_migrate == 0 {
        return Ok(MigrationExecutionResult {
            orchard_notes_to_migrate: 0,
            total_zatoshis: 0,
            total_transfer_count: 0,
            schedule_path: None,
            prepared: None,
            readiness_state: MigrationReadinessState::NoOrchardNotes,
            blockers: Vec::new(),
            rebuilt_transfer_windows: 0,
        });
    }

    if !ironwood_active {
        return Err(NozyError::InvalidOperation(
            "Ironwood (NU6.3) is not active on the network yet. Migration opens after activation."
                .to_string(),
        ));
    }

    let (mut schedule, schedule_path, rebuilt_transfer_windows) =
        load_or_rebuild_orchard_migration_schedule(ironwood_active, chain_tip, &plan)?;
    let migration_fee = crate::fee_policy::estimate_orchard_send_fee_zatoshis(
        None,
        crate::fee_policy::NOZY_WALLET_PRIORITY_FEE,
    );

    let readiness = assess_orchard_migration_readiness(
        ironwood_active,
        chain_tip,
        &plan,
        Some(&schedule),
        None,
        None,
    );
    match readiness.state {
        MigrationReadinessState::SplitRequired
        | MigrationReadinessState::WaitingForWindow
        | MigrationReadinessState::PresignedWaitingForBroadcast
        | MigrationReadinessState::ReadyToBroadcast => {
            return Ok(MigrationExecutionResult {
                orchard_notes_to_migrate: plan.orchard_notes_to_migrate,
                total_zatoshis: plan.total_zatoshis,
                total_transfer_count: plan.zip318.total_transfer_count,
                schedule_path: Some(schedule_path),
                prepared: None,
                readiness_state: readiness.state,
                blockers: readiness.blockers,
                rebuilt_transfer_windows,
            });
        }
        MigrationReadinessState::ReadyToPrebuild => {}
        MigrationReadinessState::PlanningOnly
        | MigrationReadinessState::NoOrchardNotes
        | MigrationReadinessState::Blocked => {
            return Err(NozyError::InvalidOperation(format!(
                "Ironwood migration is not ready: {}",
                readiness.blockers.join("; ")
            )));
        }
    }

    let orchard_note_values: Vec<u64> = spendable_notes
        .iter()
        .map(|note| note.orchard_note.value)
        .collect();
    let transfer = next_spendable_eligible_pending_transfer(
        &schedule,
        chain_tip,
        &orchard_note_values,
        migration_fee,
    )
    .or(readiness.next_eligible_transfer)
    .ok_or_else(|| {
        NozyError::InvalidOperation(
            "No eligible ZIP 318 transfer is ready for prebuild at the current height.".to_string(),
        )
    })?;
    if !can_cover_transfer_with_current_notes(&transfer, spendable_notes, migration_fee) {
        return Ok(MigrationExecutionResult {
            orchard_notes_to_migrate: plan.orchard_notes_to_migrate,
            total_zatoshis: plan.total_zatoshis,
            total_transfer_count: plan.zip318.total_transfer_count,
            schedule_path: Some(schedule_path),
            prepared: None,
            readiness_state: MigrationReadinessState::SplitRequired,
            blockers: vec![format!(
                "ZIP 318 transfer #{} ({} zat) cannot be covered by one current Orchard note plus fee. \
                 Consolidate fee-dust notes or run the note-splitting phase.",
                transfer.sequence, transfer.value_zat
            )],
            rebuilt_transfer_windows,
        });
    }

    let prepared = build_migration_transaction_for_transfer(
        &zebra,
        network,
        chain_tip,
        &transfer,
        spendable_notes,
        migration_fee,
    )
    .await?;
    if let Some(transfer) = schedule
        .transfers
        .iter_mut()
        .find(|t| t.sequence == prepared.sequence)
    {
        transfer.status = MigrationTransferStatus::Presigned;
        transfer.presigned_tx_hex = Some(prepared.raw_tx_hex.clone());
        transfer.prepared_txid = Some(prepared.txid.clone());
        transfer.source_nullifier_hex = Some(prepared.source_nullifier_hex.clone());
        transfer.prepared_at_height = Some(prepared.prepared_at_height);
        transfer.expires_at_height = Some(prepared.expires_at_height);
    }
    let schedule_path = save_orchard_migration_schedule(&schedule)?;

    // ZIP 318 turnstile flow:
    // 1. Orchard send-to-self note split into canonical denominations.
    // 2. Orchard spend → Ironwood output migration txs.
    // 3. Persist scheduled/pre-signed txs and broadcast in anchor-height buckets.
    // Tracked in IRONWOOD_WALLET_READINESS.md Phase 3.
    Ok(MigrationExecutionResult {
        orchard_notes_to_migrate: plan.orchard_notes_to_migrate,
        total_zatoshis: plan.total_zatoshis,
        total_transfer_count: plan.zip318.total_transfer_count,
        schedule_path: Some(schedule_path),
        prepared: Some(prepared),
        readiness_state: MigrationReadinessState::PresignedWaitingForBroadcast,
        blockers: vec![
            "Run `nozy ironwood broadcast` when the ZIP 318 bucket window is open.".to_string(),
        ],
        rebuilt_transfer_windows,
    })
}

/// Validate that a presigned transfer may be broadcast at `chain_tip`.
pub fn presigned_transfer_broadcastable(
    transfer: &MigrationScheduledTransfer,
    chain_tip: u32,
) -> Result<(), String> {
    if transfer.status != MigrationTransferStatus::Presigned {
        return Err(format!(
            "Transfer #{} is {:?}, not presigned",
            transfer.sequence, transfer.status
        ));
    }
    if transfer
        .presigned_tx_hex
        .as_deref()
        .is_none_or(|hex| hex.is_empty())
    {
        return Err(format!(
            "Transfer #{} is missing presigned_tx_hex",
            transfer.sequence
        ));
    }
    if transfer
        .prepared_txid
        .as_deref()
        .is_none_or(|txid| txid.is_empty())
    {
        return Err(format!(
            "Transfer #{} is missing prepared_txid",
            transfer.sequence
        ));
    }
    if chain_tip < transfer.not_before_height {
        return Err(format!(
            "ZIP 318 bucket for transfer #{} opens at height {} (tip {chain_tip})",
            transfer.sequence, transfer.not_before_height
        ));
    }
    if transfer_window_expired(transfer, chain_tip) {
        return Err(format!(
            "ZIP 318 window for transfer #{} expired at height {}",
            transfer.sequence,
            transfer
                .not_before_height
                .saturating_add(ZIP318_TRANSFER_EXPIRY_BLOCKS)
        ));
    }
    if transfer
        .expires_at_height
        .is_some_and(|expires| chain_tip > expires)
    {
        return Err(format!(
            "Presigned transaction for transfer #{} expired at height {}",
            transfer.sequence,
            transfer.expires_at_height.unwrap_or(0)
        ));
    }
    Ok(())
}

/// Mark broadcast transfers confirmed when Zebrad reports them in the active chain.
pub async fn reconcile_migration_broadcast_confirmations(
    zebra: &crate::zebra_integration::ZebraClient,
    schedule: &mut MigrationSchedule,
) -> NozyResult<usize> {
    let mut confirmed = 0usize;
    for transfer in &mut schedule.transfers {
        if transfer.status != MigrationTransferStatus::Broadcast {
            continue;
        }
        let Some(txid) = transfer
            .broadcast_txid
            .as_deref()
            .or(transfer.prepared_txid.as_deref())
        else {
            continue;
        };
        if zebra.transaction_in_active_chain(txid).await? {
            transfer.status = MigrationTransferStatus::Confirmed;
            confirmed += 1;
        }
    }
    Ok(confirmed)
}

/// Broadcast the next presigned ZIP 318 turnstile transaction inside its anchor bucket window.
///
/// Priority 1: real broadcasts require local Zebrad, detected Tor/I2P, user Nym/Tor attestation,
/// or an explicit `--force-clearnet` override (see `network_privacy`).
pub async fn execute_orchard_migration_broadcast(
    zebra_url: &str,
    ironwood_active: bool,
    dry_run: bool,
    wait_confirm: bool,
    network_privacy: &crate::ironwood::network_privacy::MigrationNetworkPrivacyOpts,
) -> NozyResult<MigrationBroadcastResult> {
    let zebra = crate::zebra_integration::ZebraClient::new(zebra_url.to_string());
    let chain_tip = zebra.get_best_block_height().await?;
    let plan = plan_orchard_migration_at(ironwood_active, chain_tip)?;

    if !ironwood_active {
        return Err(NozyError::InvalidOperation(
            "Ironwood (NU6.3) is not active on this network yet.".to_string(),
        ));
    }

    let (mut schedule, _, _rebuilt) =
        load_or_rebuild_orchard_migration_schedule(ironwood_active, chain_tip, &plan)?;

    let _ = reconcile_migration_broadcast_confirmations(&zebra, &mut schedule).await?;
    let mut schedule_path = save_orchard_migration_schedule(&schedule)?;

    let transfer = schedule
        .transfers
        .iter()
        .filter(|transfer| transfer.status == MigrationTransferStatus::Presigned)
        .filter(|transfer| !transfer_window_expired(transfer, chain_tip))
        .min_by_key(|transfer| transfer.sequence)
        .cloned()
        .ok_or_else(|| {
            NozyError::InvalidOperation(
                "No non-expired presigned Ironwood migration transaction is waiting for broadcast."
                    .to_string(),
            )
        })?;

    if let Err(reason) = presigned_transfer_broadcastable(&transfer, chain_tip) {
        return Ok(MigrationBroadcastResult {
            sequence: transfer.sequence,
            txid: transfer.prepared_txid.clone().unwrap_or_default(),
            broadcast_at_height: chain_tip,
            schedule_path,
            confirmed: false,
            readiness_state: MigrationReadinessState::PresignedWaitingForBroadcast,
            blockers: vec![reason],
        });
    }

    let raw_hex = transfer
        .presigned_tx_hex
        .as_ref()
        .expect("presigned tx validated");
    let prepared_txid = transfer
        .prepared_txid
        .as_ref()
        .expect("prepared txid validated");

    if dry_run {
        let privacy = crate::ironwood::network_privacy::assess_migration_network_privacy(
            zebra_url,
            network_privacy,
        )
        .await;
        let mut blockers = vec!["Dry run — transaction was not broadcast.".to_string()];
        blockers.extend(privacy.blockers.clone());
        blockers.extend(privacy.warnings.clone());
        return Ok(MigrationBroadcastResult {
            sequence: transfer.sequence,
            txid: prepared_txid.clone(),
            broadcast_at_height: chain_tip,
            schedule_path,
            confirmed: false,
            readiness_state: MigrationReadinessState::ReadyToBroadcast,
            blockers,
        });
    }

    let privacy = crate::ironwood::network_privacy::require_migration_network_privacy(
        zebra_url,
        network_privacy,
    )
    .await?;
    for warning in &privacy.warnings {
        eprintln!("⚠️  {warning}");
    }
    if let Some(mode) = privacy.mode {
        eprintln!("🔒 Migration network privacy: {}", mode.label());
    }

    let broadcast_txid = zebra.broadcast_transaction(raw_hex).await?;
    let txid = if broadcast_txid.is_empty() {
        prepared_txid.clone()
    } else {
        broadcast_txid
    };

    if let Some(source) = transfer.source_nullifier_hex.clone() {
        let _ = crate::notes::mark_wallet_notes_spent_by_nullifier_hex(
            std::slice::from_ref(&source),
            Some(&txid),
        );
    }

    if let Some(slot) = schedule
        .transfers
        .iter_mut()
        .find(|t| t.sequence == transfer.sequence)
    {
        slot.status = MigrationTransferStatus::Broadcast;
        slot.broadcast_txid = Some(txid.clone());
    }
    schedule_path = save_orchard_migration_schedule(&schedule)?;

    let mut confirmed = false;
    if wait_confirm {
        for _ in 0..45 {
            if zebra.transaction_in_active_chain(&txid).await? {
                confirmed = true;
                break;
            }
            tokio::time::sleep(std::time::Duration::from_secs(20)).await;
        }
        if confirmed {
            if let Some(slot) = schedule
                .transfers
                .iter_mut()
                .find(|t| t.sequence == transfer.sequence)
            {
                slot.status = MigrationTransferStatus::Confirmed;
            }
            schedule_path = save_orchard_migration_schedule(&schedule)?;
        }
    }

    Ok(MigrationBroadcastResult {
        sequence: transfer.sequence,
        txid,
        broadcast_at_height: chain_tip,
        schedule_path,
        confirmed,
        readiness_state: if confirmed {
            MigrationReadinessState::ReadyToPrebuild
        } else if wait_confirm {
            MigrationReadinessState::ReadyToPrebuild
        } else {
            MigrationReadinessState::ReadyToPrebuild
        },
        blockers: if wait_confirm && !confirmed {
            vec!["Broadcast submitted; confirmation still pending on chain.".to_string()]
        } else {
            Vec::new()
        },
    })
}

/// Flatten canonical denominations into per-note output values.
pub fn flatten_canonical_denomination_zatoshis(amount_zat: u64) -> Vec<u64> {
    canonical_denominations(amount_zat)
        .into_iter()
        .flat_map(|d| std::iter::repeat_n(d.value_zat, d.count as usize))
        .collect()
}

/// True when a note value decomposes into more than one canonical denomination.
pub fn note_requires_canonical_split(value_zat: u64) -> bool {
    flatten_canonical_denomination_zatoshis(value_zat).len() > 1
}

/// True when a note is a non-canonical composite that can still be split in a ZIP 318 split tx.
///
/// Fee-dust notes may fail [`note_requires_canonical_split`] but cannot fit a split fee; they do
/// not block turnstile prebuild once splittable composites are gone.
pub fn orchard_note_needs_splittable_split(value_zat: u64) -> bool {
    note_requires_canonical_split(value_zat) && plan_orchard_note_split_outputs(value_zat).is_ok()
}

/// True when any unspent Orchard note still needs a splittable ZIP 318 note-split transaction.
pub fn orchard_wallet_needs_note_split(notes: &[crate::notes::SerializableOrchardNote]) -> bool {
    notes
        .iter()
        .filter(|n| !n.spent && n.pool == crate::shielded_pool::ShieldedPool::Orchard)
        .any(|n| orchard_note_needs_splittable_split(n.value))
}

fn migration_split_fee_zatoshis(output_count: u32) -> u64 {
    let output_count = output_count.max(1);
    let shape = crate::fee_policy::OrchardSendFeeShape {
        orchard_actions: 1.max(output_count),
        memo_len: 0,
    };
    crate::fee_policy::fee_zatoshis(&shape, false)
}

/// Plan Orchard outputs for a ZIP 318 note-split transaction (fee deducted from the source note).
pub fn plan_orchard_note_split_outputs(input_value_zat: u64) -> NozyResult<(Vec<u64>, u64)> {
    if input_value_zat == 0 {
        return Err(NozyError::InvalidOperation(
            "Cannot split a zero-value note.".to_string(),
        ));
    }
    if !note_requires_canonical_split(input_value_zat) {
        return Err(NozyError::InvalidOperation(format!(
            "{input_value_zat} zat is already a single canonical denomination and does not require splitting."
        )));
    }

    let mut outputs = flatten_canonical_denomination_zatoshis(input_value_zat);
    let mut fee = migration_split_fee_zatoshis(outputs.len() as u32);

    for _ in 0..16 {
        let output_total: u64 = outputs.iter().sum();
        if output_total.saturating_add(fee) <= input_value_zat {
            let remainder = input_value_zat
                .saturating_sub(fee)
                .saturating_sub(output_total);
            if remainder > 0 {
                outputs.push(remainder);
                fee = migration_split_fee_zatoshis(outputs.len() as u32);
                continue;
            }
            if outputs.len() <= 1 {
                return Err(NozyError::InvalidOperation(format!(
                    "After deducting the {fee} zat split fee, {input_value_zat} zat does not decompose into multiple canonical notes."
                )));
            }
            return Ok((outputs, fee));
        }

        let deficit = output_total
            .saturating_add(fee)
            .saturating_sub(input_value_zat);
        let Some(last) = outputs.last_mut() else {
            return Err(NozyError::InvalidOperation(
                "Split planning produced no outputs.".to_string(),
            ));
        };
        if *last <= deficit {
            return Err(NozyError::InvalidOperation(format!(
                "Cannot fit ZIP 318 split outputs and {fee} zat fee into {input_value_zat} zat. \
                 Try consolidating notes or sync again before splitting."
            )));
        }
        *last -= deficit;
        if *last == 0 {
            outputs.pop();
            fee = migration_split_fee_zatoshis(outputs.len().max(1) as u32);
        }
    }

    Err(NozyError::InvalidOperation(
        "Split fee planning did not converge.".to_string(),
    ))
}

#[derive(Debug, Clone)]
pub struct OrchardNoteSplitResult {
    pub source_nullifier_hex: String,
    pub source_value_zat: u64,
    pub output_values_zat: Vec<u64>,
    pub fee_zat: u64,
    pub txid: String,
    pub raw_tx_hex: String,
    pub broadcast: bool,
    pub note_split_still_required: bool,
}

fn pick_orchard_note_for_split<'a>(notes: &'a [SpendableNote]) -> Option<&'a SpendableNote> {
    notes
        .iter()
        .filter(|note| !note.orchard_note.spent)
        .filter(|note| note_requires_canonical_split(note.orchard_note.value))
        .max_by_key(|note| note.orchard_note.value)
}

async fn build_orchard_split_transaction(
    zebra: &crate::zebra_integration::ZebraClient,
    network: NetworkType,
    chain_tip: u32,
    spend_note: &SpendableNote,
    output_values_zat: &[u64],
    fee_zatoshis: u64,
) -> NozyResult<(String, String)> {
    if output_values_zat.is_empty() {
        return Err(NozyError::InvalidOperation(
            "Split transaction requires at least one Orchard output.".to_string(),
        ));
    }

    let output_total: u64 = output_values_zat.iter().sum();
    if output_total.saturating_add(fee_zatoshis) > spend_note.orchard_note.value {
        return Err(NozyError::InvalidOperation(format!(
            "Split outputs ({output_total} zat) plus fee ({fee_zatoshis} zat) exceed note value ({} zat).",
            spend_note.orchard_note.value
        )));
    }

    crate::send_readiness::ensure_witness_fresh_for_send(spend_note, chain_tip)?;

    let witness_provider = crate::orchard_tx::ZebraJsonRpcOrchardWitnessProvider;
    let (orchard_anchor, merkle_path) = witness_provider
        .prepare_spend_anchor_and_path(zebra, spend_note, chain_tip)
        .await?;

    let target_height = BlockHeight::from_u32(chain_tip.saturating_add(1));
    let fvk = FullViewingKey::from(&spend_note.spending_key);
    // Preserve the source note's Orchard receiver (External vs Internal) — V6 rejects cross-scope outputs.
    let self_orchard_address = spend_note.orchard_note.address.clone();
    let fee = Zatoshis::from_u64(fee_zatoshis)
        .map_err(|_| NozyError::InvalidOperation("Invalid split fee".to_string()))?;
    let fee_rule = FixedMigrationFeeRule { fee };

    let build_config = BuildConfig::Standard {
        sapling_anchor: None,
        orchard_anchor: Some(orchard_anchor),
        ironwood_anchor: None,
    };

    macro_rules! build_split_pczt {
        ($params:expr) => {{
            let mut builder = Builder::new($params, target_height, build_config);
            builder
                .add_orchard_spend::<core::convert::Infallible>(
                    fvk.clone(),
                    spend_note.orchard_note.note.clone(),
                    merkle_path,
                )
                .map_err(|e| NozyError::InvalidOperation(format!("add_orchard_spend: {e:?}")))?;
            for value in output_values_zat {
                let output_value = Zatoshis::from_u64(*value).map_err(|_| {
                    NozyError::InvalidOperation(format!("Invalid split output value {value}"))
                })?;
                builder
                    .add_orchard_change_output::<core::convert::Infallible>(
                        fvk.clone(),
                        None,
                        self_orchard_address.clone(),
                        output_value,
                        MemoBytes::empty(),
                    )
                    .map_err(|e| {
                        NozyError::InvalidOperation(format!("add_orchard_change_output: {e:?}"))
                    })?;
            }
            let parts = builder
                .build_for_pczt(rand::rngs::OsRng, &fee_rule)
                .map_err(|e| NozyError::InvalidOperation(format!("build_for_pczt: {e:?}")))?
                .pczt_parts;
            Creator::build_from_parts(parts).ok_or_else(|| {
                NozyError::InvalidOperation(
                    "PCZT creator: incompatible split transaction version".to_string(),
                )
            })?
        }};
    }

    let pczt = match network {
        NetworkType::Main => build_split_pczt!(MainNetwork),
        NetworkType::Test | NetworkType::Regtest => build_split_pczt!(TestNetwork),
    };

    let proving_key =
        orchard::circuit::ProvingKey::build(orchard::circuit::OrchardCircuitVersion::PostNu6_3);
    let pczt = Prover::new(pczt)
        .create_orchard_proof(&proving_key)
        .map_err(|e| NozyError::InvalidOperation(format!("create_orchard_proof: {e:?}")))?
        .finish();

    let pczt = IoFinalizer::new(pczt)
        .finalize_io()
        .map_err(|e| NozyError::InvalidOperation(format!("split io_finalize: {e:?}")))?;

    let ask = SpendAuthorizingKey::from(&spend_note.spending_key);
    let orchard_action_count = pczt.orchard().actions().len();
    let mut signer = Signer::new(pczt)
        .map_err(|e| NozyError::InvalidOperation(format!("split signer init: {e:?}")))?;
    let mut signed_any = false;
    for index in 0..orchard_action_count {
        if signer.sign_orchard(index, &ask).is_ok() {
            signed_any = true;
        }
    }
    if !signed_any {
        return Err(NozyError::InvalidOperation(
            "No Orchard split spend signatures were applied.".to_string(),
        ));
    }
    let pczt = signer.finish();

    let verifying_key =
        orchard::circuit::VerifyingKey::build(orchard::circuit::OrchardCircuitVersion::PostNu6_3);
    let tx = TransactionExtractor::new(pczt)
        .with_orchard(&verifying_key)
        .extract()
        .map_err(|e| NozyError::InvalidOperation(format!("split tx extract: {e:?}")))?;

    let txid = tx.txid().to_string();
    let mut raw_transaction = Vec::new();
    tx.write(&mut raw_transaction)
        .map_err(|e| NozyError::InvalidOperation(format!("split tx serialize: {e}")))?;

    Ok((txid, hex::encode(raw_transaction)))
}

/// ZIP 318 Phase 1: Orchard send-to-self split into canonical denominations.
///
/// This path bypasses the post-activation Orchard-only send blocker because note splitting
/// is a migration-only operation, not a normal user send.
pub async fn execute_orchard_note_split(
    zebra_url: &str,
    ironwood_active: bool,
    spendable_notes: &[SpendableNote],
    broadcast: bool,
) -> NozyResult<OrchardNoteSplitResult> {
    if !ironwood_active {
        return Err(NozyError::InvalidOperation(
            "Ironwood (NU6.3) is not active yet. Note splitting opens after activation."
                .to_string(),
        ));
    }

    let spend_note = pick_orchard_note_for_split(spendable_notes).ok_or_else(|| {
        NozyError::InvalidOperation(
            "No Orchard note requires canonical splitting. Run `nozy ironwood preflight` to confirm."
                .to_string(),
        )
    })?;

    let (output_values_zat, fee_zat) =
        plan_orchard_note_split_outputs(spend_note.orchard_note.value)?;
    let source_nullifier_hex = hex::encode(spend_note.orchard_note.nullifier.to_bytes());
    let source_value_zat = spend_note.orchard_note.value;

    let zebra = crate::zebra_integration::ZebraClient::new(zebra_url.to_string());
    let chain_tip = zebra.get_best_block_height().await?;
    let info = zebra.get_blockchain_info().await?;
    let network = parse_network_type(&info);

    let (txid, raw_tx_hex) = build_orchard_split_transaction(
        &zebra,
        network,
        chain_tip,
        spend_note,
        &output_values_zat,
        fee_zat,
    )
    .await?;

    if broadcast {
        zebra.broadcast_transaction(&raw_tx_hex).await?;
        crate::notes::mark_wallet_notes_spent_from_spendables(
            std::slice::from_ref(spend_note),
            Some(&txid),
        )?;
    }

    let note_split_still_required = spendable_notes.iter().any(|note| {
        if broadcast && hex::encode(note.orchard_note.nullifier.to_bytes()) == source_nullifier_hex
        {
            return false;
        }
        !note.orchard_note.spent && orchard_note_needs_splittable_split(note.orchard_note.value)
    });

    Ok(OrchardNoteSplitResult {
        source_nullifier_hex,
        source_value_zat,
        output_values_zat,
        fee_zat,
        txid,
        raw_tx_hex,
        broadcast,
        note_split_still_required,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_plan(chain_tip: u32) -> MigrationPlanSummary {
        let denomination_transfers = vec![MigrationDenomination {
            value_zat: 10_000,
            count: 2,
        }];
        let scheduled_transfers =
            schedule_canonical_transfers(&denomination_transfers, chain_tip, ZIP318_DEFAULT_K_MAX);
        MigrationPlanSummary {
            orchard_notes_to_migrate: 2,
            total_zatoshis: 20_000,
            transfers: vec![
                MigrationTransfer {
                    nullifier_hex: "aa".repeat(32),
                    value_zat: 10_000,
                    block_height: 100,
                    txid: "tx-a".to_string(),
                },
                MigrationTransfer {
                    nullifier_hex: "bb".repeat(32),
                    value_zat: 10_000,
                    block_height: 101,
                    txid: "tx-b".to_string(),
                },
            ],
            ironwood_active: true,
            zip318: Zip318ScheduleSummary {
                note_split_required: false,
                anchor_bucket_interval_blocks: ZIP318_ANCHOR_BUCKET_INTERVAL_BLOCKS,
                k_max: ZIP318_DEFAULT_K_MAX,
                denomination_transfers,
                total_transfer_count: 2,
                next_anchor_bucket_height: scheduled_transfers
                    .first()
                    .map(|transfer| transfer.anchor_bucket_height),
                scheduled_transfers,
            },
        }
    }

    #[test]
    fn previous_anchor_boundary_uses_zip318_interval() {
        assert_eq!(previous_zip318_anchor_boundary(0), 0);
        assert_eq!(previous_zip318_anchor_boundary(255), 0);
        assert_eq!(previous_zip318_anchor_boundary(256), 256);
        assert_eq!(previous_zip318_anchor_boundary(511), 256);
    }

    #[test]
    fn canonical_denominations_are_zooko_125_greedy() {
        let parts = canonical_denominations(123_450_000);
        let total: u64 = parts.iter().map(|p| p.value_zat * u64::from(p.count)).sum();
        // 1.2345 ZEC → 1 + 0.2 + 0.02 + 0.01 + 0.002 + 0.002; residual 0.0005 abandoned
        assert_eq!(total, 123_400_000);
        assert_eq!(parts[0].value_zat, 100_000_000);
        assert_eq!(parts[0].count, 1);
        assert!(parts.iter().any(|p| p.value_zat == 20_000_000));
        assert!(parts.iter().any(|p| p.value_zat == 2_000_000));
    }

    #[test]
    fn power_of_ten_ladder_still_available() {
        let parts = canonical_power_of_ten_denominations(123_450_000);
        let total: u64 = parts.iter().map(|p| p.value_zat * u64::from(p.count)).sum();
        assert_eq!(total, 123_450_000);
        assert_eq!(parts[0].value_zat, 100_000_000);
    }

    #[test]
    fn zooko_round_amount_never_exceeds_balance() {
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        // ~2182.48296002 ZEC (Appendix A example starting balance after fee adjust)
        let balance = 218_248_296_002u64;
        for _ in 0..64 {
            if let Some(amount) = select_zooko_round_amount(balance, &mut rng) {
                assert!(amount <= balance);
                assert!(amount >= ZOOKO_RESIDUAL_ABANDON_ZAT);
                // All Appendix A buckets are multiples of 0.001 ZEC.
                assert_eq!(amount % ZOOKO_RESIDUAL_ABANDON_ZAT, 0);
            }
        }
        assert!(select_zooko_round_amount(50_000, &mut rng).is_none());
    }

    #[test]
    fn schedule_caps_same_denomination_per_anchor_bucket() {
        let denominations = vec![MigrationDenomination {
            value_zat: 10_000,
            count: 9,
        }];
        let schedule = schedule_canonical_transfers(&denominations, 1_000, 4);

        assert_eq!(schedule.len(), 9);
        assert_eq!(schedule[0].anchor_bucket_height, 1_024);
        assert_eq!(schedule[3].bucket_slot, 4);
        assert_eq!(schedule[4].anchor_bucket_height, 1_280);
        assert_eq!(schedule[8].anchor_bucket_height, 1_536);
    }

    #[test]
    fn schedule_validation_accepts_current_plan() {
        let plan = sample_plan(1_000);
        let schedule = build_schedule_from_plan(&plan, 1_000);
        let validation = validate_orchard_migration_schedule(&schedule, &plan, 1_024);

        assert!(validation.valid, "{:?}", validation.errors);
        assert_eq!(validation.expired_transfer_count, 0);
        assert_eq!(validation.stale_presigned_count, 0);
    }

    #[test]
    fn schedule_validation_rejects_source_note_mismatch() {
        let plan = sample_plan(1_000);
        let mut schedule = build_schedule_from_plan(&plan, 1_000);
        schedule.source_notes[0].value_zat = 11_000;

        let validation = validate_orchard_migration_schedule(&schedule, &plan, 1_024);

        assert!(!validation.valid);
        assert!(validation
            .errors
            .iter()
            .any(|error| error.contains("source notes")));
    }

    #[test]
    fn schedule_validation_detects_stale_presigned_window() {
        let plan = sample_plan(1_000);
        let mut schedule = build_schedule_from_plan(&plan, 1_000);
        let transfer = &mut schedule.transfers[0];
        transfer.status = MigrationTransferStatus::Presigned;
        transfer.presigned_tx_hex = Some("00".to_string());
        transfer.prepared_txid = Some("txid".to_string());
        transfer.source_nullifier_hex = Some("aa".repeat(32));
        transfer.prepared_at_height = Some(1_024);
        transfer.expires_at_height =
            Some(transfer.not_before_height + ZIP318_TRANSFER_EXPIRY_BLOCKS);
        let validation_height = transfer
            .not_before_height
            .saturating_add(ZIP318_TRANSFER_EXPIRY_BLOCKS)
            .saturating_add(1);

        let validation = validate_orchard_migration_schedule(&schedule, &plan, validation_height);

        assert!(validation.valid, "{:?}", validation.errors);
        assert_eq!(validation.expired_transfer_count, 2);
        assert_eq!(validation.stale_presigned_count, 1);
    }

    #[test]
    fn readiness_reports_split_required_before_prebuild() {
        let mut plan = sample_plan(1_000);
        plan.zip318.note_split_required = true;
        let schedule = build_schedule_from_plan(&plan, 1_000);

        let readiness = assess_orchard_migration_readiness(
            true,
            1_024,
            &plan,
            Some(&schedule),
            Some(0),
            Some(50),
        );

        assert_eq!(readiness.state, MigrationReadinessState::SplitRequired);
        assert!(readiness
            .blockers
            .iter()
            .any(|blocker| blocker.contains("note splitting")));
    }

    #[test]
    fn readiness_selects_next_eligible_transfer_by_height() {
        let plan = sample_plan(1_000);
        let schedule = build_schedule_from_plan(&plan, 1_000);

        let readiness = assess_orchard_migration_readiness(
            true,
            1_024,
            &plan,
            Some(&schedule),
            Some(0),
            Some(50),
        );

        assert_eq!(readiness.state, MigrationReadinessState::ReadyToPrebuild);
        assert_eq!(readiness.next_eligible_transfer.unwrap().sequence, 1);
    }

    #[test]
    fn readiness_waits_for_future_bucket() {
        let plan = sample_plan(1_000);
        let schedule = build_schedule_from_plan(&plan, 1_000);

        let readiness = assess_orchard_migration_readiness(
            true,
            1_023,
            &plan,
            Some(&schedule),
            Some(0),
            Some(50),
        );

        assert_eq!(readiness.state, MigrationReadinessState::WaitingForWindow);
        assert_eq!(readiness.next_waiting_transfer.unwrap().sequence, 1);
    }

    #[test]
    fn schedule_state_round_trips_with_new_presigned_fields() {
        let plan = sample_plan(1_000);
        let mut schedule = build_schedule_from_plan(&plan, 1_000);
        let transfer = &mut schedule.transfers[0];
        transfer.status = MigrationTransferStatus::Presigned;
        transfer.presigned_tx_hex = Some("deadbeef".to_string());
        transfer.prepared_txid = Some("prepared".to_string());
        transfer.source_nullifier_hex = Some("aa".repeat(32));
        transfer.prepared_at_height = Some(1_024);
        transfer.expires_at_height = Some(1_280);

        let serialized = serde_json::to_string(&schedule).unwrap();
        let decoded: MigrationSchedule = serde_json::from_str(&serialized).unwrap();

        let decoded_transfer = &decoded.transfers[0];
        assert_eq!(decoded_transfer.status, MigrationTransferStatus::Presigned);
        let expected_source = "aa".repeat(32);
        assert_eq!(
            decoded_transfer.source_nullifier_hex.as_deref(),
            Some(expected_source.as_str())
        );
        assert_eq!(decoded_transfer.prepared_at_height, Some(1_024));
        assert_eq!(decoded_transfer.expires_at_height, Some(1_280));
    }

    #[test]
    fn split_plan_adjusts_outputs_for_fee() {
        let (outputs, fee) = plan_orchard_note_split_outputs(110_000_000).expect("split plan");
        assert_eq!(outputs.first().copied(), Some(100_000_000));
        assert!(outputs.len() >= 2);
        assert!(fee > 0);
        assert!(outputs.iter().sum::<u64>() + fee <= 110_000_000);
    }

    #[test]
    fn note_requires_split_for_composite_values() {
        assert!(note_requires_canonical_split(110_000_000));
        assert!(!note_requires_canonical_split(100_000_000));
        // 0.2 ZEC is a single {1,2,5}×10^k bucket — no split required.
        assert!(!note_requires_canonical_split(20_000_000));
        // 0.3 ZEC = 0.2 + 0.1 → composite under the active ladder.
        assert!(note_requires_canonical_split(30_000_000));
    }

    #[test]
    fn splittable_split_gate_ignores_transfer_count_mismatch() {
        assert!(orchard_note_needs_splittable_split(110_000_000));
        assert!(!orchard_note_needs_splittable_split(100_000_000));
        assert!(!orchard_note_needs_splittable_split(10_000_000));
        // Single canonical bucket and residual-below-floor amounts are not split candidates.
        assert!(!note_requires_canonical_split(200_000));
        assert!(!note_requires_canonical_split(50_000));
        assert!(!orchard_note_needs_splittable_split(50_000));

        fn sample_note(value: u64, nullifier_byte: u8) -> crate::notes::SerializableOrchardNote {
            crate::notes::SerializableOrchardNote {
                note_bytes: vec![0u8; 180],
                value,
                address_bytes: vec![2u8; 43],
                nullifier_bytes: vec![nullifier_byte; 32],
                block_height: 1,
                txid: "abc".into(),
                spent: false,
                memo: vec![],
                orchard_incremental_witness_hex: None,
                orchard_witness_tip_height: None,
                ironwood_incremental_witness_hex: None,
                ironwood_witness_tip_height: None,
                rho_bytes: None,
                rseed_bytes: None,
                spent_in_txid: None,
                pool: crate::shielded_pool::ShieldedPool::Orchard,
            }
        }

        let notes = vec![
            sample_note(100_000_000, 1),
            sample_note(10_000_000, 2),
            // Residual / fee-dust leftover — not a split candidate under {1,2,5}×10^k.
            sample_note(50_000, 3),
        ];
        assert!(!orchard_wallet_needs_note_split(&notes));
    }

    #[test]
    fn presigned_broadcast_requires_open_bucket_and_presigned_fields() {
        let transfer = MigrationScheduledTransfer {
            sequence: 1,
            value_zat: 100_000_000,
            anchor_bucket_height: 1_024,
            not_before_height: 1_024,
            bucket_slot: 1,
            status: MigrationTransferStatus::Presigned,
            presigned_tx_hex: Some("deadbeef".to_string()),
            prepared_txid: Some("abc".to_string()),
            broadcast_txid: None,
            source_nullifier_hex: Some("aa".repeat(32)),
            prepared_at_height: Some(1_000),
            expires_at_height: Some(1_280),
        };

        assert!(presigned_transfer_broadcastable(&transfer, 1_023).is_err());
        assert!(presigned_transfer_broadcastable(&transfer, 1_024).is_ok());
        assert!(presigned_transfer_broadcastable(&transfer, 1_281).is_err());
    }
}
