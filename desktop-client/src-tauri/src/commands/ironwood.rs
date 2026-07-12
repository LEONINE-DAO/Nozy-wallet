//! Desktop Ironwood migration actions (plan / split / migrate / broadcast).
//! Thin wrappers around `nozy::ironwood` — same path as CLI.

use crate::error::TauriError;
use crate::session::load_session_wallet;
use nozy::{
    assess_orchard_migration_readiness, execute_orchard_migration,
    execute_orchard_migration_broadcast, execute_orchard_note_split, is_ironwood_active,
    load_config, load_orchard_migration_schedule, load_wallet_notes,
    max_serialized_witness_lag_blocks, plan_orchard_migration_at, plan_orchard_note_split_outputs,
    save_orchard_migration_plan_at, scan_notes_for_sending, MigrationNetworkPrivacyOpts,
    MigrationReadinessState, MAX_SEND_WITNESS_LAG_BLOCKS, ZebraClient,
};
use serde::{Deserialize, Serialize};
use tauri::command;

fn migration_schedule_tip(
    ironwood_active: bool,
    chain_tip: u32,
    activation_height: Option<u32>,
) -> u32 {
    if ironwood_active {
        chain_tip
    } else {
        activation_height
            .map(|height| height.saturating_sub(1))
            .unwrap_or(chain_tip)
    }
}

async fn chain_context() -> Result<(nozy::WalletConfig, u32, bool, u32), TauriError> {
    let config = load_config();
    let is_testnet = config.network.eq_ignore_ascii_case("testnet");
    let zebra = ZebraClient::from_config(&config);
    let chain_tip = zebra.get_block_count().await.map_err(TauriError::from)?;
    let ironwood_active = is_ironwood_active(chain_tip, is_testnet);
    let activation = nozy::nu6_3_activation_height(is_testnet);
    let tip_for_plan = migration_schedule_tip(ironwood_active, chain_tip, activation);
    Ok((config, chain_tip, ironwood_active, tip_for_plan))
}

#[derive(Debug, Serialize)]
pub struct IronwoodPlanSaveResponse {
    pub orchard_notes_to_migrate: usize,
    pub total_zatoshis: u64,
    pub transfer_count: u32,
    pub note_split_required: bool,
    pub next_anchor_bucket_height: Option<u32>,
    pub schedule_path: String,
    pub ironwood_active: bool,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct IronwoodMigrateRequest {
    pub password: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct IronwoodMigrateResponse {
    pub readiness_state: String,
    pub orchard_notes_to_migrate: usize,
    pub total_zatoshis: u64,
    pub total_transfer_count: u32,
    pub schedule_path: Option<String>,
    pub prepared_txid: Option<String>,
    pub prepared_sequence: Option<u32>,
    pub prepared_value_zat: Option<u64>,
    pub prepared_at_height: Option<u32>,
    pub expires_at_height: Option<u32>,
    pub blockers: Vec<String>,
    pub message: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct IronwoodBroadcastRequest {
    pub password: Option<String>,
    #[serde(default)]
    pub attest_private_network: bool,
    #[serde(default)]
    pub force_clearnet: bool,
    #[serde(default)]
    pub dry_run: bool,
    #[serde(default)]
    pub wait_confirm: bool,
}

#[derive(Debug, Serialize)]
pub struct IronwoodBroadcastResponse {
    pub readiness_state: String,
    pub sequence: u32,
    pub txid: String,
    pub broadcast_at_height: u32,
    pub schedule_path: String,
    pub confirmed: bool,
    pub blockers: Vec<String>,
    pub message: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct IronwoodSplitRequest {
    pub password: Option<String>,
    #[serde(default)]
    pub dry_run: bool,
}

#[derive(Debug, Serialize)]
pub struct IronwoodSplitResponse {
    pub dry_run: bool,
    pub source_value_zat: u64,
    pub source_nullifier_hex: String,
    pub fee_zat: u64,
    pub output_values_zat: Vec<u64>,
    pub txid: Option<String>,
    pub note_split_still_required: bool,
    pub message: String,
}

#[command]
pub async fn ironwood_plan_save() -> Result<IronwoodPlanSaveResponse, TauriError> {
    let (_config, _chain_tip, ironwood_active, tip_for_plan) = chain_context().await?;
    let plan = plan_orchard_migration_at(ironwood_active, tip_for_plan).map_err(TauriError::from)?;
    let (schedule, path) =
        save_orchard_migration_plan_at(ironwood_active, tip_for_plan).map_err(TauriError::from)?;

    Ok(IronwoodPlanSaveResponse {
        orchard_notes_to_migrate: plan.orchard_notes_to_migrate,
        total_zatoshis: plan.total_zatoshis,
        transfer_count: schedule.transfers.len() as u32,
        note_split_required: plan.zip318.note_split_required,
        next_anchor_bucket_height: plan.zip318.next_anchor_bucket_height,
        schedule_path: path.display().to_string(),
        ironwood_active,
        message: format!(
            "Saved ZIP 318 schedule with {} transfer(s).",
            schedule.transfers.len()
        ),
    })
}

#[command]
pub async fn ironwood_split(
    request: IronwoodSplitRequest,
) -> Result<IronwoodSplitResponse, TauriError> {
    let (config, _chain_tip, ironwood_active, _tip) = chain_context().await?;
    if !ironwood_active {
        return Err(TauriError::from(nozy::NozyError::InvalidOperation(
            "Ironwood (NU6.3) is not active yet. Note splitting opens after activation."
                .to_string(),
        )));
    }

    let wallet = load_session_wallet(request.password.as_deref()).await?;
    let spendable = scan_notes_for_sending(wallet, &config.zebra_url)
        .await
        .map_err(TauriError::from)?;

    if request.dry_run {
        let spend_note = spendable
            .iter()
            .filter(|note| !note.orchard_note.spent)
            .filter(|note| {
                nozy::ironwood::note_requires_canonical_split(note.orchard_note.value)
            })
            .max_by_key(|note| note.orchard_note.value)
            .ok_or_else(|| {
                TauriError::from(nozy::NozyError::InvalidOperation(
                    "No Orchard note requires canonical splitting.".to_string(),
                ))
            })?;
        let (outputs, fee) = plan_orchard_note_split_outputs(spend_note.orchard_note.value)
            .map_err(TauriError::from)?;
        return Ok(IronwoodSplitResponse {
            dry_run: true,
            source_value_zat: spend_note.orchard_note.value,
            source_nullifier_hex: hex::encode(spend_note.orchard_note.nullifier.to_bytes()),
            fee_zat: fee,
            output_values_zat: outputs.clone(),
            txid: None,
            note_split_still_required: true,
            message: format!(
                "Dry run — would split {} zat into {} output(s) (fee {} zat).",
                spend_note.orchard_note.value,
                outputs.len(),
                fee
            ),
        });
    }

    let result = execute_orchard_note_split(&config.zebra_url, ironwood_active, &spendable, true)
        .await
        .map_err(TauriError::from)?;

    let message = if result.note_split_still_required {
        format!(
            "Split broadcast ({}). Sync to tip, then split again if still required.",
            result.txid
        )
    } else {
        format!(
            "Split broadcast ({}). Sync to tip, then Plan → Migrate → Broadcast.",
            result.txid
        )
    };

    Ok(IronwoodSplitResponse {
        dry_run: false,
        source_value_zat: result.source_value_zat,
        source_nullifier_hex: result.source_nullifier_hex,
        fee_zat: result.fee_zat,
        output_values_zat: result.output_values_zat,
        txid: Some(result.txid),
        note_split_still_required: result.note_split_still_required,
        message,
    })
}

#[command]
pub async fn ironwood_migrate(
    request: IronwoodMigrateRequest,
) -> Result<IronwoodMigrateResponse, TauriError> {
    let (config, _chain_tip, ironwood_active, _tip_for_plan) = chain_context().await?;
    let wallet = load_session_wallet(request.password.as_deref()).await?;
    let spendable = scan_notes_for_sending(wallet, &config.zebra_url)
        .await
        .map_err(TauriError::from)?;
    let result = execute_orchard_migration(&config.zebra_url, ironwood_active, &spendable)
        .await
        .map_err(TauriError::from)?;

    let message = if let Some(ref prepared) = result.prepared {
        format!(
            "Prebuilt turnstile #{} ({} zat). Broadcast when the ZIP 318 window opens.",
            prepared.sequence, prepared.value_zat
        )
    } else {
        match result.readiness_state {
            MigrationReadinessState::NoOrchardNotes => {
                "No Orchard notes require Ironwood migration.".to_string()
            }
            MigrationReadinessState::SplitRequired => {
                "ZIP 318 note splitting is required before migrate. Use Split on this tab, then retry."
                    .to_string()
            }
            MigrationReadinessState::WaitingForWindow => {
                "Waiting for the next ZIP 318 anchor bucket before prebuild.".to_string()
            }
            MigrationReadinessState::PresignedWaitingForBroadcast
            | MigrationReadinessState::ReadyToBroadcast => {
                "A turnstile is already presigned — use Broadcast.".to_string()
            }
            other => format!("Migrate did not prebuild ({})", other.label()),
        }
    };

    Ok(IronwoodMigrateResponse {
        readiness_state: result.readiness_state.label().to_string(),
        orchard_notes_to_migrate: result.orchard_notes_to_migrate,
        total_zatoshis: result.total_zatoshis,
        total_transfer_count: result.total_transfer_count,
        schedule_path: result
            .schedule_path
            .as_ref()
            .map(|p| p.display().to_string()),
        prepared_txid: result.prepared.as_ref().map(|p| p.txid.clone()),
        prepared_sequence: result.prepared.as_ref().map(|p| p.sequence),
        prepared_value_zat: result.prepared.as_ref().map(|p| p.value_zat),
        prepared_at_height: result.prepared.as_ref().map(|p| p.prepared_at_height),
        expires_at_height: result.prepared.as_ref().map(|p| p.expires_at_height),
        blockers: result.blockers,
        message,
    })
}

#[command]
pub async fn ironwood_broadcast(
    request: IronwoodBroadcastRequest,
) -> Result<IronwoodBroadcastResponse, TauriError> {
    let (config, _chain_tip, ironwood_active, _tip) = chain_context().await?;
    let _ = load_session_wallet(request.password.as_deref()).await?;

    let network_privacy = MigrationNetworkPrivacyOpts {
        attest_private_network: request.attest_private_network,
        force_clearnet: request.force_clearnet,
        broadcast_via_nym_mixnet: config.privacy_network.broadcast_via_nym_mixnet,
    };

    let result = execute_orchard_migration_broadcast(
        &config.zebra_url,
        ironwood_active,
        request.dry_run,
        request.wait_confirm,
        &network_privacy,
    )
    .await
    .map_err(TauriError::from)?;

    let message = if request.dry_run {
        format!(
            "Dry run — would broadcast turnstile #{} ({})",
            result.sequence, result.txid
        )
    } else if result.confirmed {
        format!(
            "Broadcast and confirmed turnstile #{} ({})",
            result.sequence, result.txid
        )
    } else if result.blockers.is_empty() {
        format!(
            "Broadcast turnstile #{} ({})",
            result.sequence, result.txid
        )
    } else {
        format!(
            "Broadcast incomplete for #{}: {}",
            result.sequence,
            result.blockers.first().cloned().unwrap_or_default()
        )
    };

    Ok(IronwoodBroadcastResponse {
        readiness_state: result.readiness_state.label().to_string(),
        sequence: result.sequence,
        txid: result.txid,
        broadcast_at_height: result.broadcast_at_height,
        schedule_path: result.schedule_path.display().to_string(),
        confirmed: result.confirmed,
        blockers: result.blockers,
        message,
    })
}

/// Shared readiness snapshot for status UI gating.
pub fn desktop_migration_readiness(
    ironwood_active: bool,
    chain_tip: u32,
    tip_for_plan: u32,
) -> Result<(MigrationReadinessState, Vec<String>), TauriError> {
    let plan = plan_orchard_migration_at(ironwood_active, tip_for_plan).map_err(TauriError::from)?;
    let schedule = load_orchard_migration_schedule().map_err(TauriError::from)?;
    let notes = load_wallet_notes().unwrap_or_default();
    let witness_lag = max_serialized_witness_lag_blocks(&notes, chain_tip);
    let report = assess_orchard_migration_readiness(
        ironwood_active,
        chain_tip,
        &plan,
        schedule.as_ref(),
        Some(witness_lag),
        Some(MAX_SEND_WITNESS_LAG_BLOCKS),
    );
    Ok((report.state, report.blockers))
}
