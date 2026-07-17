use axum::{extract::Json, http::StatusCode, response::Json as ResponseJson};
use serde::{Deserialize, Serialize};

use crate::handlers::{error_response_with_code, load_wallet_with_password};
use nozy::{
    execute_orchard_migration, execute_orchard_migration_broadcast, execute_orchard_note_split,
    fetch_pool_balances, ironwood::ironwood_software_send_available, ironwood_user_notices,
    is_ironwood_active, load_config, load_orchard_migration_schedule, load_wallet_notes,
    nu6_3_activation_height, plan_orchard_migration_at, plan_orchard_note_split_outputs,
    previous_zip318_anchor_boundary, safer_migration_status_snapshot,
    save_orchard_migration_plan_at, scan_notes_for_sending, shielded_pool::ShieldedPool,
    MigrationNetworkPrivacyOpts, MigrationReadinessState, ZebraClient,
    NU6_3_MAINNET_ACTIVATION_TARGET, NU6_3_TESTNET_ACTIVATION_TARGET,
};

#[derive(Debug, Serialize)]
pub struct IronwoodSaferMigrationResponse {
    pub network_privacy_allowed: bool,
    pub network_privacy_mode: Option<String>,
    pub zebra_url_local: bool,
    pub privacy_proxy_detected: bool,
    pub privacy_proxy_label: Option<String>,
    pub user_attested: bool,
    pub force_clearnet: bool,
    pub network_privacy_blockers: Vec<String>,
    pub network_privacy_warnings: Vec<String>,
    pub cover_bucket_height: u32,
    pub cover_local_transfers: usize,
    pub cover_k_max: u8,
    pub cover_thin: bool,
    pub cover_warnings: Vec<String>,
    pub cover_notes: Vec<String>,
    pub amount_timing_active: String,
    pub amount_timing_planned: String,
    pub amount_timing_notes: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct IronwoodStatusResponse {
    pub network: String,
    pub chain_tip: Option<u32>,
    pub activation_height: Option<u32>,
    pub activation_target_date: String,
    pub ironwood_active: bool,
    pub ironwood_rpc_detected: bool,
    pub orchard_chain_value_zec: Option<f64>,
    pub ironwood_chain_value_zec: Option<f64>,
    pub orchard_wallet_zat: u64,
    pub ironwood_wallet_zat: u64,
    pub ironwood_send_enabled: bool,
    pub wallet_ready: bool,
    pub migration_recommended: bool,
    pub migration_note_count: usize,
    pub migration_zat: u64,
    pub zip318_transfer_count: u32,
    pub zip318_note_split_required: bool,
    pub next_anchor_bucket_height: Option<u32>,
    pub migration_enabled: bool,
    pub blockers: Vec<String>,
    pub activation_notice: String,
    pub migration_privacy_warnings: Vec<String>,
    pub orchard_funds_at_risk: bool,
    pub safer_migration: IronwoodSaferMigrationResponse,
}

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

async fn chain_context(
) -> Result<(nozy::WalletConfig, u32, bool, u32), (StatusCode, ResponseJson<serde_json::Value>)> {
    let config = load_config();
    let is_testnet = config.network.eq_ignore_ascii_case("testnet");
    let zebra = ZebraClient::from_config(&config);
    let chain_tip = zebra.get_block_count().await.map_err(|e| {
        error_response_with_code(
            StatusCode::BAD_GATEWAY,
            format!("Zebra unreachable: {e}"),
            "ZEBRA_UNREACHABLE",
        )
    })?;
    let ironwood_active = is_ironwood_active(chain_tip, is_testnet);
    let activation = nu6_3_activation_height(is_testnet);
    let tip_for_plan = migration_schedule_tip(ironwood_active, chain_tip, activation);
    Ok((config, chain_tip, ironwood_active, tip_for_plan))
}

fn privacy_opts_from_config(config: &nozy::WalletConfig) -> MigrationNetworkPrivacyOpts {
    MigrationNetworkPrivacyOpts {
        attest_private_network: config.privacy_network.attest_private_network,
        force_clearnet: config.privacy_network.force_clearnet,
        broadcast_via_nym_mixnet: config.privacy_network.broadcast_via_nym_mixnet,
    }
}

pub async fn get_ironwood_status(
) -> Result<ResponseJson<IronwoodStatusResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    let config = load_config();
    let is_testnet = config.network == "testnet";
    let activation_height = nu6_3_activation_height(is_testnet);
    let activation_target_date = if is_testnet {
        NU6_3_TESTNET_ACTIVATION_TARGET
    } else {
        NU6_3_MAINNET_ACTIVATION_TARGET
    }
    .to_string();

    let zebra_client = ZebraClient::from_config(&config);
    let chain_tip = zebra_client.get_block_count().await.ok();
    let ironwood_active = chain_tip
        .map(|tip| is_ironwood_active(tip, is_testnet))
        .unwrap_or(false);

    let mut orchard_chain_value_zec = None;
    let mut ironwood_chain_value_zec = None;
    let mut ironwood_rpc_detected = false;
    if let Ok(pools) = fetch_pool_balances(&zebra_client).await {
        for pool in pools {
            match pool.pool {
                ShieldedPool::Orchard => orchard_chain_value_zec = Some(pool.chain_value_zec),
                ShieldedPool::Ironwood => {
                    ironwood_chain_value_zec = Some(pool.chain_value_zec);
                    ironwood_rpc_detected = true;
                }
            }
        }
    }

    let notes = load_wallet_notes().unwrap_or_default();
    let orchard_wallet_zat = notes
        .iter()
        .filter(|n| !n.spent && n.pool == ShieldedPool::Orchard)
        .map(|n| n.value)
        .sum();
    let ironwood_wallet_zat = notes
        .iter()
        .filter(|n| !n.spent && n.pool == ShieldedPool::Ironwood)
        .map(|n| n.value)
        .sum();

    let migration_schedule_tip = if ironwood_active {
        chain_tip.unwrap_or(0)
    } else {
        activation_height
            .map(|height| height.saturating_sub(1))
            .unwrap_or_else(|| chain_tip.unwrap_or(0))
    };

    let plan = plan_orchard_migration_at(ironwood_active, migration_schedule_tip).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(serde_json::json!({
                "error": format!("Failed to plan migration: {e}")
            })),
        )
    })?;

    let mut blockers = Vec::new();
    if chain_tip.is_none() {
        blockers.push("Zebra RPC is unreachable from API settings.".to_string());
    }
    if activation_height.is_none() {
        blockers
            .push("NU6.3 activation height is not configured for this network yet.".to_string());
    } else if !ironwood_active {
        blockers.push(format!(
            "Ironwood activates at height {} (target {}). Until then Orchard works normally; \
             after activation only migration turnstiles can spend Orchard notes.",
            activation_height.unwrap(),
            activation_target_date
        ));
    }
    if !ironwood_rpc_detected {
        blockers
            .push("Connected Zebra RPC does not expose the Ironwood value pool yet.".to_string());
    }
    if ironwood_active && orchard_wallet_zat > 0 {
        blockers.push(
            "Orchard notes remain — run Ironwood Plan → Migrate → Broadcast from the app."
                .to_string(),
        );
    }
    if ironwood_active && ironwood_wallet_zat == 0 && orchard_wallet_zat == 0 {
        blockers.push(
            "No unspent shielded notes — receive ZEC or sync to tip to index Ironwood outputs."
                .to_string(),
        );
    }

    let tip_for_cover = chain_tip.unwrap_or(migration_schedule_tip);
    let bucket = previous_zip318_anchor_boundary(tip_for_cover);
    let local_in_bucket = load_orchard_migration_schedule()
        .ok()
        .flatten()
        .map(|schedule| {
            schedule
                .transfers
                .iter()
                .filter(|t| t.anchor_bucket_height == bucket)
                .count()
        })
        .unwrap_or(0);

    let safer = safer_migration_status_snapshot(
        &config.zebra_url,
        tip_for_cover,
        local_in_bucket,
        &privacy_opts_from_config(&config),
    )
    .await;

    if !safer.network_privacy_allowed {
        blockers.push(
            "Safer migration Priority 1: broadcast would be blocked until local Zebrad, Tor/I2P, Nym mixnet broadcast helper, or Nym/Tor attestation."
                .to_string(),
        );
    }
    for warning in &safer.cover_warnings {
        blockers.push(format!("Cover traffic: {warning}"));
    }

    let ironwood_send_enabled =
        ironwood_software_send_available(ironwood_active, ironwood_wallet_zat);
    let wallet_ready = ironwood_active
        && ironwood_wallet_zat > 0
        && orchard_wallet_zat == 0
        && blockers.is_empty();
    let notices = ironwood_user_notices(ironwood_active, orchard_wallet_zat);

    Ok(ResponseJson(IronwoodStatusResponse {
        network: config.network,
        chain_tip,
        activation_height,
        activation_target_date,
        ironwood_active,
        ironwood_rpc_detected,
        orchard_chain_value_zec,
        ironwood_chain_value_zec,
        orchard_wallet_zat,
        ironwood_wallet_zat,
        ironwood_send_enabled,
        wallet_ready,
        migration_recommended: plan.orchard_notes_to_migrate > 0 && ironwood_active,
        migration_note_count: plan.orchard_notes_to_migrate,
        migration_zat: plan.total_zatoshis,
        zip318_transfer_count: plan.zip318.total_transfer_count,
        zip318_note_split_required: plan.zip318.note_split_required,
        next_anchor_bucket_height: chain_tip.and(plan.zip318.next_anchor_bucket_height),
        migration_enabled: true,
        blockers,
        activation_notice: notices.activation_notice,
        migration_privacy_warnings: notices.migration_privacy_warnings,
        orchard_funds_at_risk: notices.orchard_funds_at_risk,
        safer_migration: IronwoodSaferMigrationResponse {
            network_privacy_allowed: safer.network_privacy_allowed,
            network_privacy_mode: safer.network_privacy_mode,
            zebra_url_local: safer.zebra_url_local,
            privacy_proxy_detected: safer.privacy_proxy_detected,
            privacy_proxy_label: safer.privacy_proxy_label,
            user_attested: safer.user_attested,
            force_clearnet: safer.force_clearnet,
            network_privacy_blockers: safer.network_privacy_blockers,
            network_privacy_warnings: safer.network_privacy_warnings,
            cover_bucket_height: safer.cover_bucket_height,
            cover_local_transfers: safer.cover_local_transfers,
            cover_k_max: safer.cover_k_max,
            cover_thin: safer.cover_thin,
            cover_warnings: safer.cover_warnings,
            cover_notes: safer.cover_notes,
            amount_timing_active: safer.amount_timing_active,
            amount_timing_planned: safer.amount_timing_planned,
            amount_timing_notes: safer.amount_timing_notes,
        },
    }))
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

pub async fn ironwood_plan_save(
) -> Result<ResponseJson<IronwoodPlanSaveResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    let (_config, _chain_tip, ironwood_active, tip_for_plan) = chain_context().await?;
    let plan = plan_orchard_migration_at(ironwood_active, tip_for_plan).map_err(|e| {
        error_response_with_code(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to plan migration: {e}"),
            "IRONWOOD_PLAN_FAILED",
        )
    })?;
    let (schedule, path) =
        save_orchard_migration_plan_at(ironwood_active, tip_for_plan).map_err(|e| {
            error_response_with_code(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to save migration plan: {e}"),
                "IRONWOOD_PLAN_SAVE_FAILED",
            )
        })?;

    Ok(ResponseJson(IronwoodPlanSaveResponse {
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
    }))
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

pub async fn ironwood_split(
    Json(payload): Json<IronwoodSplitRequest>,
) -> Result<ResponseJson<IronwoodSplitResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    let (config, _chain_tip, ironwood_active, _tip) = chain_context().await?;
    if !ironwood_active {
        return Err(error_response_with_code(
            StatusCode::BAD_REQUEST,
            "Ironwood (NU6.3) is not active yet. Note splitting opens after activation.",
            "IRONWOOD_INACTIVE",
        ));
    }

    let (wallet, _) = load_wallet_with_password(payload.password)
        .await
        .map_err(|e| error_response_with_code(StatusCode::UNAUTHORIZED, e, "INVALID_PASSWORD"))?;
    let spendable = scan_notes_for_sending(wallet, &config.zebra_url)
        .await
        .map_err(|e| {
            error_response_with_code(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to scan notes: {e}"),
                "NOTE_SCAN_FAILED",
            )
        })?;

    if payload.dry_run {
        let spend_note = spendable
            .iter()
            .filter(|note| !note.orchard_note.spent)
            .filter(|note| nozy::ironwood::note_requires_canonical_split(note.orchard_note.value))
            .max_by_key(|note| note.orchard_note.value)
            .ok_or_else(|| {
                error_response_with_code(
                    StatusCode::BAD_REQUEST,
                    "No Orchard note requires canonical splitting.",
                    "SPLIT_NOT_REQUIRED",
                )
            })?;
        let (outputs, fee) = plan_orchard_note_split_outputs(spend_note.orchard_note.value)
            .map_err(|e| {
                error_response_with_code(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to plan split: {e}"),
                    "SPLIT_PLAN_FAILED",
                )
            })?;
        return Ok(ResponseJson(IronwoodSplitResponse {
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
        }));
    }

    let result = execute_orchard_note_split(&config.zebra_url, ironwood_active, &spendable, true)
        .await
        .map_err(|e| {
            error_response_with_code(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Split failed: {e}"),
                "SPLIT_FAILED",
            )
        })?;

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

    Ok(ResponseJson(IronwoodSplitResponse {
        dry_run: false,
        source_value_zat: result.source_value_zat,
        source_nullifier_hex: result.source_nullifier_hex,
        fee_zat: result.fee_zat,
        output_values_zat: result.output_values_zat,
        txid: Some(result.txid),
        note_split_still_required: result.note_split_still_required,
        message,
    }))
}

#[derive(Debug, Deserialize, Default)]
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

pub async fn ironwood_migrate(
    Json(payload): Json<IronwoodMigrateRequest>,
) -> Result<ResponseJson<IronwoodMigrateResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    let (config, _chain_tip, ironwood_active, _tip_for_plan) = chain_context().await?;
    let (wallet, _) = load_wallet_with_password(payload.password)
        .await
        .map_err(|e| error_response_with_code(StatusCode::UNAUTHORIZED, e, "INVALID_PASSWORD"))?;
    let spendable = scan_notes_for_sending(wallet, &config.zebra_url)
        .await
        .map_err(|e| {
            error_response_with_code(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to scan notes: {e}"),
                "NOTE_SCAN_FAILED",
            )
        })?;
    let result = execute_orchard_migration(&config.zebra_url, ironwood_active, &spendable)
        .await
        .map_err(|e| {
            error_response_with_code(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Migrate failed: {e}"),
                "MIGRATE_FAILED",
            )
        })?;

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
                "ZIP 318 note splitting is required before migrate. Use Split, then retry."
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

    Ok(ResponseJson(IronwoodMigrateResponse {
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
    }))
}

#[derive(Debug, Deserialize, Default)]
pub struct IronwoodBroadcastRequest {
    pub password: Option<String>,
    #[serde(default)]
    pub attest_private_network: Option<bool>,
    #[serde(default)]
    pub force_clearnet: Option<bool>,
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

pub async fn ironwood_broadcast(
    Json(payload): Json<IronwoodBroadcastRequest>,
) -> Result<ResponseJson<IronwoodBroadcastResponse>, (StatusCode, ResponseJson<serde_json::Value>)>
{
    let (config, _chain_tip, ironwood_active, _tip) = chain_context().await?;
    let _ = load_wallet_with_password(payload.password)
        .await
        .map_err(|e| error_response_with_code(StatusCode::UNAUTHORIZED, e, "INVALID_PASSWORD"))?;

    let network_privacy = MigrationNetworkPrivacyOpts {
        attest_private_network: payload
            .attest_private_network
            .unwrap_or(config.privacy_network.attest_private_network),
        force_clearnet: payload
            .force_clearnet
            .unwrap_or(config.privacy_network.force_clearnet),
        broadcast_via_nym_mixnet: config.privacy_network.broadcast_via_nym_mixnet,
    };

    let result = execute_orchard_migration_broadcast(
        &config.zebra_url,
        ironwood_active,
        payload.dry_run,
        payload.wait_confirm,
        &network_privacy,
    )
    .await
    .map_err(|e| {
        error_response_with_code(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Broadcast failed: {e}"),
            "BROADCAST_FAILED",
        )
    })?;

    let message = if payload.dry_run {
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
        format!("Broadcast turnstile #{} ({})", result.sequence, result.txid)
    } else {
        format!(
            "Broadcast incomplete for #{}: {}",
            result.sequence,
            result.blockers.first().cloned().unwrap_or_default()
        )
    };

    Ok(ResponseJson(IronwoodBroadcastResponse {
        readiness_state: result.readiness_state.label().to_string(),
        sequence: result.sequence,
        txid: result.txid,
        broadcast_at_height: result.broadcast_at_height,
        schedule_path: result.schedule_path.display().to_string(),
        confirmed: result.confirmed,
        blockers: result.blockers,
        message,
    }))
}
