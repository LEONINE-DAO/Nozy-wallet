//! lightwalletd + `zeaking::lwd` — compact block sync for Zebrad stacks (Chrome/Edge extension can use companion HTTP API).

use crate::error::TauriError;
use nozy::paths::get_wallet_data_dir;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::command;
use zeaking::lwd::proto::Empty;

#[derive(Debug, Serialize)]
pub struct LwdInfoResponse {
    pub version: String,
    pub chain_name: String,
    pub block_height: u64,
    pub estimated_height: u64,
}

#[derive(Debug, Serialize)]
pub struct LwdSyncResponse {
    pub blocks_written: u64,
    /// Height the client asked for (same as request `start`).
    pub range_start_requested: u64,
    /// First height actually fetched after optional resume skip.
    pub range_start_effective: u64,
    pub range_end: u64,
}

#[derive(Debug, Serialize)]
pub struct LwdSyncToTipResponse {
    pub chain_tip: u64,
    pub already_at_tip: bool,
    pub blocks_written: u64,
    pub range_start_requested: u64,
    pub range_start_effective: u64,
    pub range_end: u64,
}

#[derive(Debug, Deserialize)]
pub struct LwdSyncRequest {
    pub start: u64,
    pub end: Option<u64>,
    pub lightwalletd_url: Option<String>,
    pub db_path: Option<String>,
    /// Skip compact heights already present in the SQLite store.
    pub resume: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct LwdSyncToTipRequest {
    pub lightwalletd_url: Option<String>,
    pub db_path: Option<String>,
    pub start_floor: Option<u64>,
    pub persist_progress_every: Option<u64>,
}

fn lwd_url(o: Option<String>) -> String {
    o.or_else(|| std::env::var("LIGHTWALLETD_GRPC").ok())
        .unwrap_or_else(|| "http://127.0.0.1:9067".to_string())
}

fn zeaking_err(e: zeaking::ZeakingError) -> TauriError {
    let code = match &e {
        zeaking::ZeakingError::Grpc(_) => Some("LWD_GRPC".to_string()),
        zeaking::ZeakingError::Storage(_) => Some("LWD_STORAGE".to_string()),
        zeaking::ZeakingError::InvalidOperation(_) => Some("LWD_INVALID".to_string()),
        _ => Some("ZEAKING".to_string()),
    };
    TauriError {
        message: e.to_string(),
        code,
    }
}

fn compact_db_path(request: &LwdSyncRequest) -> PathBuf {
    compact_db_path_opt(request.db_path.as_ref())
}

fn compact_db_path_opt(db_path: Option<&String>) -> PathBuf {
    db_path
        .map(PathBuf::from)
        .unwrap_or_else(|| get_wallet_data_dir().join("lwd_compact.sqlite"))
}

/// `GetLightdInfo` from lightwalletd (chain name, tip height, etc.).
#[command]
pub async fn lwd_get_info(lightwalletd_url: Option<String>) -> Result<LwdInfoResponse, TauriError> {
    let url = lwd_url(lightwalletd_url);
    let mut client = zeaking::lwd::connect_lightwalletd(&url)
        .await
        .map_err(zeaking_err)?;
    let info = client
        .get_lightd_info(Empty {})
        .await
        .map_err(|e| TauriError {
            message: format!("gRPC GetLightdInfo: {e}"),
            code: Some("LWD_GRPC".to_string()),
        })?
        .into_inner();
    Ok(LwdInfoResponse {
        version: info.version,
        chain_name: info.chain_name,
        block_height: info.block_height,
        estimated_height: info.estimated_height,
    })
}

/// Best-chain tip height from lightwalletd.
#[command]
pub async fn lwd_chain_tip(lightwalletd_url: Option<String>) -> Result<u64, TauriError> {
    let url = lwd_url(lightwalletd_url);
    let mut client = zeaking::lwd::connect_lightwalletd(&url)
        .await
        .map_err(zeaking_err)?;
    zeaking::lwd::chain_tip_height(&mut client)
        .await
        .map_err(zeaking_err)
}

/// Stream compact blocks `[start, end]` (default `end` = lightwalletd tip) into SQLite.
#[command]
pub async fn lwd_sync_compact(request: LwdSyncRequest) -> Result<LwdSyncResponse, TauriError> {
    let url = lwd_url(request.lightwalletd_url.clone());
    let db = compact_db_path(&request);
    if let Some(parent) = db.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let mut client = zeaking::lwd::connect_lightwalletd(&url)
        .await
        .map_err(zeaking_err)?;
    let store = zeaking::lwd::LwdCompactStore::open(&db).map_err(zeaking_err)?;
    let end = if let Some(e) = request.end {
        e
    } else {
        zeaking::lwd::chain_tip_height(&mut client)
            .await
            .map_err(zeaking_err)?
    };
    let opts = zeaking::lwd::SyncCompactOptions {
        resume_from_store: request.resume.unwrap_or(false),
        ..Default::default()
    };
    let stats = zeaking::lwd::sync_compact_range_with_options(
        &mut client,
        &store,
        request.start,
        end,
        opts,
    )
    .await
    .map_err(zeaking_err)?;
    Ok(LwdSyncResponse {
        blocks_written: stats.blocks_written,
        range_start_requested: stats.range_start_requested,
        range_start_effective: stats.range_start_effective,
        range_end: stats.range_end,
    })
}

/// Sync compact blocks from the next missing height through lightwalletd tip (resume-safe).
#[command]
pub async fn lwd_sync_compact_to_tip(
    request: LwdSyncToTipRequest,
) -> Result<LwdSyncToTipResponse, TauriError> {
    let url = lwd_url(request.lightwalletd_url.clone());
    let db = compact_db_path_opt(request.db_path.as_ref());
    if let Some(parent) = db.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let mut client = zeaking::lwd::connect_lightwalletd(&url)
        .await
        .map_err(zeaking_err)?;
    let store = zeaking::lwd::LwdCompactStore::open(&db).map_err(zeaking_err)?;
    let tip_opts = zeaking::lwd::SyncCompactToTipOptions {
        start_floor: request.start_floor,
        persist_progress_every: request
            .persist_progress_every
            .unwrap_or_else(|| zeaking::lwd::SyncCompactToTipOptions::default().persist_progress_every)
            .max(1),
    };
    let stats = zeaking::lwd::sync_compact_to_tip_with_options(&mut client, &store, tip_opts)
        .await
        .map_err(zeaking_err)?;
    Ok(LwdSyncToTipResponse {
        chain_tip: stats.chain_tip,
        already_at_tip: stats.already_at_tip,
        blocks_written: stats.blocks_written,
        range_start_requested: stats.range_start_requested,
        range_start_effective: stats.range_start_effective,
        range_end: stats.range_end,
    })
}
