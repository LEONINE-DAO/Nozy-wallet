use crate::error::TauriError;
use crate::commands::ironwood::desktop_migration_readiness;
use nozy::{
    fetch_pool_balances, gather_sync_status, ironwood_user_notices, is_ironwood_active, load_config,
    load_orchard_migration_schedule, load_wallet_notes, max_serialized_witness_lag_blocks,
    nu6_3_activation_height, plan_orchard_migration_at, previous_zip318_anchor_boundary,
    safer_migration_status_snapshot, shielded_pool::ShieldedPool, MigrationNetworkPrivacyOpts,
    MigrationReadinessState, NU6_3_MAINNET_ACTIVATION_TARGET, NU6_3_TESTNET_ACTIVATION_TARGET,
    MAX_SEND_WITNESS_LAG_BLOCKS, ZebraClient,
};
use serde::{Deserialize, Serialize};
use tauri::command;

#[derive(Debug, Serialize)]
pub struct SyncStatusResponse {
    pub zebra_tip: Option<u32>,
    pub last_scan_height: Option<u32>,
    pub scan_gap_blocks: Option<u32>,
    pub witness_lag_blocks: u32,
    pub witness_fresh_for_send: bool,
    pub max_send_witness_lag_blocks: u32,
    pub lightwalletd_url: String,
    pub lwd_tip: Option<u64>,
    pub lwd_error: Option<String>,
    pub compact_max_height: Option<u64>,
    pub compact_db_exists: bool,
    pub message: String,
}

#[command]
pub async fn get_sync_status() -> Result<SyncStatusResponse, TauriError> {
    let config = load_config();
    let zebra_client = ZebraClient::from_config(&config);
    let snapshot = gather_sync_status(&zebra_client, &config).await;

    let zebra_tip = snapshot.zebra_tip;
    let last_scan = snapshot.last_scan_height;
    let scan_gap_blocks = match (zebra_tip, last_scan) {
        (Some(tip), Some(last)) if last > tip => Some(0),
        (Some(tip), Some(last)) => Some(tip.saturating_sub(last)),
        _ => None,
    };

    let notes = load_wallet_notes().unwrap_or_default();
    let witness_lag_blocks = zebra_tip
        .map(|tip| max_serialized_witness_lag_blocks(&notes, tip))
        .unwrap_or(0);
    let witness_fresh_for_send = witness_lag_blocks <= MAX_SEND_WITNESS_LAG_BLOCKS;

    let message = build_status_message(
        zebra_tip,
        last_scan,
        scan_gap_blocks,
        witness_lag_blocks,
        witness_fresh_for_send,
    );

    Ok(SyncStatusResponse {
        zebra_tip,
        last_scan_height: last_scan,
        scan_gap_blocks,
        witness_lag_blocks,
        witness_fresh_for_send,
        max_send_witness_lag_blocks: MAX_SEND_WITNESS_LAG_BLOCKS,
        lightwalletd_url: snapshot.lightwalletd_url,
        lwd_tip: snapshot.lwd_tip,
        lwd_error: snapshot.lwd_error,
        compact_max_height: snapshot.compact_max_height,
        compact_db_exists: snapshot.compact_db_exists,
        message,
    })
}

fn build_status_message(
    zebra_tip: Option<u32>,
    last_scan: Option<u32>,
    scan_gap: Option<u32>,
    witness_lag: u32,
    witness_fresh: bool,
) -> String {
    let mut parts = Vec::new();

    match (zebra_tip, last_scan, scan_gap) {
        (Some(tip), Some(last), _) if last > tip => {
            parts.push(format!(
                "Zebra node syncing — tip {tip} is behind wallet scan {last}; wait for node catch-up before sending"
            ));
        }
        (Some(tip), Some(_last), Some(gap)) if gap == 0 => {
            parts.push(format!("RPC scan caught up (height {tip})"));
        }
        (Some(tip), Some(last), Some(gap)) if gap > 0 => {
            parts.push(format!(
                "RPC scan {gap} blocks behind tip (last {last}, tip {tip}) — sync to tip before sending"
            ));
        }
        (Some(tip), None, _) => {
            parts.push(format!("Never scanned — tip {tip}; run sync"));
        }
        _ => parts.push("Zebra unreachable — check network settings".to_string()),
    }

    if witness_fresh {
        parts.push(format!("Orchard witness lag {witness_lag} blocks"));
    } else {
        parts.push(format!(
            "Orchard witness {witness_lag} blocks behind (max {MAX_SEND_WITNESS_LAG_BLOCKS}) — sync to tip"
        ));
    }

    parts.join(" · ")
}

#[derive(Debug, Serialize)]
pub struct OrchardPoolStatsResponse {
    pub chain_value_zec: f64,
    pub chain_value_zat: u64,
    pub monitored: bool,
    pub block_height: u32,
}

#[command]
pub async fn get_orchard_pool_stats() -> Result<OrchardPoolStatsResponse, TauriError> {
    let config = load_config();
    let zebra_client = ZebraClient::from_config(&config);
    let stats = zebra_client
        .get_orchard_pool_stats()
        .await
        .map_err(TauriError::from)?;

    Ok(OrchardPoolStatsResponse {
        chain_value_zec: stats.chain_value_zec,
        chain_value_zat: stats.chain_value_zat,
        monitored: stats.monitored,
        block_height: stats.block_height,
    })
}

#[derive(Debug, Deserialize, Default)]
pub struct IronwoodStatusRequest {
    #[serde(default)]
    pub attest_private_network: bool,
    #[serde(default)]
    pub force_clearnet: bool,
}

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
pub struct IronwoodDesktopStatusResponse {
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
    pub readiness_state: String,
    pub ready_to_prebuild: bool,
    pub ready_to_broadcast: bool,
    pub blockers: Vec<String>,
    pub activation_notice: String,
    pub migration_privacy_warnings: Vec<String>,
    pub orchard_funds_at_risk: bool,
    pub safer_migration: IronwoodSaferMigrationResponse,
}

#[command]
pub async fn get_ironwood_status(
    request: Option<IronwoodStatusRequest>,
) -> Result<IronwoodDesktopStatusResponse, TauriError> {
    let request = request.unwrap_or_default();
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
    let plan = plan_orchard_migration_at(ironwood_active, migration_schedule_tip)
        .map_err(TauriError::from)?;
    let mut blockers = Vec::new();
    if chain_tip.is_none() {
        blockers.push("Zebra RPC is unreachable from desktop settings.".to_string());
    }
    match activation_height {
        None => blockers.push(
            "NU6.3 activation height is not configured for this network yet.".to_string(),
        ),
        Some(height) if !ironwood_active => blockers.push(format!(
            "Ironwood activates at height {height} (target {activation_target_date}). Until then Orchard works normally; \
             after activation only migration turnstiles can spend Orchard notes."
        )),
        Some(_) => {}
    }
    if !ironwood_rpc_detected {
        blockers
            .push("Connected Zebra RPC does not expose the Ironwood value pool yet.".to_string());
    }
    if ironwood_active && orchard_wallet_zat > 0 {
        blockers.push(
            "Orchard notes remain — use Plan, Migrate, and Broadcast on the Ironwood tab \
             (or CLI `nozy ironwood …`)."
                .to_string(),
        );
    }
    if plan.zip318.note_split_required {
        blockers.push(
            "ZIP 318 note splitting is required before Migrate. Run `nozy ironwood split` in the CLI, then return here."
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
        &MigrationNetworkPrivacyOpts {
            attest_private_network: request.attest_private_network,
            force_clearnet: request.force_clearnet,
            broadcast_via_nym_mixnet: config.privacy_network.broadcast_via_nym_mixnet,
        },
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
        nozy::ironwood::ironwood_software_send_available(ironwood_active, ironwood_wallet_zat);
    let wallet_ready = ironwood_active
        && ironwood_wallet_zat > 0
        && orchard_wallet_zat == 0
        && blockers.is_empty();

    let tip_for_readiness = chain_tip.unwrap_or(migration_schedule_tip);
    let (readiness_state, readiness_blockers) =
        match desktop_migration_readiness(ironwood_active, tip_for_readiness, migration_schedule_tip)
        {
            Ok((state, rb)) => (state, rb),
            Err(_) => (MigrationReadinessState::Blocked, Vec::new()),
        };
    for b in readiness_blockers {
        if !blockers.iter().any(|existing| existing == &b) {
            blockers.push(b);
        }
    }

    let ready_to_prebuild = readiness_state == MigrationReadinessState::ReadyToPrebuild;
    let ready_to_broadcast = matches!(
        readiness_state,
        MigrationReadinessState::ReadyToBroadcast
            | MigrationReadinessState::PresignedWaitingForBroadcast
    );
    // Migration actions available once Ironwood is live on-chain (or RPC shows the pool on testnet).
    let migration_enabled = ironwood_active || (is_testnet && ironwood_rpc_detected);
    let notices = ironwood_user_notices(ironwood_active, orchard_wallet_zat);

    Ok(IronwoodDesktopStatusResponse {
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
        migration_enabled,
        readiness_state: readiness_state.label().to_string(),
        ready_to_prebuild,
        ready_to_broadcast,
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
    })
}
