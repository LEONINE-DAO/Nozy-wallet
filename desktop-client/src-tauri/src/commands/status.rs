use crate::error::TauriError;
use nozy::{
    gather_sync_status, load_config, load_wallet_notes, max_serialized_witness_lag_blocks,
    MAX_SEND_WITNESS_LAG_BLOCKS, ZebraClient,
};
use serde::Serialize;
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
