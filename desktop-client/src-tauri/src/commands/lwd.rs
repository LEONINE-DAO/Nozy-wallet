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
    pub range_start: u64,
    pub range_end: u64,
}

#[derive(Debug, Deserialize)]
pub struct LwdSyncRequest {
    pub start: u64,
    pub end: Option<u64>,
    pub lightwalletd_url: Option<String>,
    pub db_path: Option<String>,
}

fn lwd_url(o: Option<String>) -> String {
    o.or_else(|| std::env::var("LIGHTWALLETD_GRPC").ok())
        .unwrap_or_else(|| "http://127.0.0.1:9067".to_string())
}

fn zeaking_err(e: zeaking::ZeakingError) -> TauriError {
    TauriError {
        message: e.to_string(),
        code: Some("ZEAKING".to_string()),
    }
}

fn compact_db_path(request: &LwdSyncRequest) -> PathBuf {
    request
        .db_path
        .as_ref()
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
    let n = zeaking::lwd::sync_compact_range(&mut client, &store, request.start, end)
        .await
        .map_err(zeaking_err)?;
    Ok(LwdSyncResponse {
        blocks_written: n,
        range_start: request.start,
        range_end: end,
    })
}
