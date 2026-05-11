//! lightwalletd endpoints for Chrome/Edge extensions and web UI (`zeaking::lwd`).

use axum::{extract::Query, http::StatusCode, response::Json as ResponseJson};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::handlers::error_response;

#[derive(Debug, Deserialize)]
pub struct LwdUrlQuery {
    pub lightwalletd_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LwdSyncBody {
    pub start: u64,
    pub end: Option<u64>,
    pub lightwalletd_url: Option<String>,
    pub db_path: Option<String>,
    /// When `true`, skip heights already in the compact DB (`max(height)+1` .. `end`).
    pub resume: Option<bool>,
}

fn zeaking_status(e: &zeaking::ZeakingError) -> StatusCode {
    match e {
        zeaking::ZeakingError::Grpc(_) => StatusCode::BAD_GATEWAY,
        zeaking::ZeakingError::Storage(_) => StatusCode::INTERNAL_SERVER_ERROR,
        zeaking::ZeakingError::InvalidOperation(_) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn lwd_uri(q: &LwdUrlQuery) -> String {
    q.lightwalletd_url
        .clone()
        .or_else(|| std::env::var("LIGHTWALLETD_GRPC").ok())
        .unwrap_or_else(|| "http://127.0.0.1:9067".to_string())
}

fn lwd_uri_from_body(body: &LwdSyncBody) -> String {
    body.lightwalletd_url
        .clone()
        .or_else(|| std::env::var("LIGHTWALLETD_GRPC").ok())
        .unwrap_or_else(|| "http://127.0.0.1:9067".to_string())
}

fn compact_db_path(body: &LwdSyncBody) -> PathBuf {
    body.db_path
        .as_ref()
        .map(PathBuf::from)
        .or_else(|| std::env::var("NOZY_LWD_DB").ok().map(PathBuf::from))
        .unwrap_or_else(|| nozy::paths::get_wallet_data_dir().join("lwd_compact.sqlite"))
}

/// GET `/api/lwd/info?lightwalletd_url=...`
pub async fn lwd_info(
    Query(q): Query<HashMap<String, String>>,
) -> Result<ResponseJson<serde_json::Value>, (StatusCode, ResponseJson<serde_json::Value>)> {
    let q = LwdUrlQuery {
        lightwalletd_url: q.get("lightwalletd_url").cloned(),
    };
    let url = lwd_uri(&q);
    let mut client = zeaking::lwd::connect_lightwalletd(&url)
        .await
        .map_err(|e| error_response(StatusCode::BAD_GATEWAY, e.to_string()))?;
    use zeaking::lwd::proto::Empty;
    let info = client
        .get_lightd_info(Empty {})
        .await
        .map_err(|e| error_response(StatusCode::BAD_GATEWAY, format!("gRPC: {e}")))?
        .into_inner();
    Ok(ResponseJson(serde_json::json!({
        "version": info.version,
        "chain_name": info.chain_name,
        "block_height": info.block_height,
        "estimated_height": info.estimated_height,
    })))
}

/// GET `/api/lwd/chain-tip?lightwalletd_url=...`
pub async fn lwd_chain_tip(
    Query(q): Query<HashMap<String, String>>,
) -> Result<ResponseJson<serde_json::Value>, (StatusCode, ResponseJson<serde_json::Value>)> {
    let q = LwdUrlQuery {
        lightwalletd_url: q.get("lightwalletd_url").cloned(),
    };
    let url = lwd_uri(&q);
    let mut client = zeaking::lwd::connect_lightwalletd(&url)
        .await
        .map_err(|e| error_response(StatusCode::BAD_GATEWAY, e.to_string()))?;
    let tip = zeaking::lwd::chain_tip_height(&mut client)
        .await
        .map_err(|e| error_response(StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(ResponseJson(serde_json::json!({ "chain_tip": tip })))
}

/// POST `/api/lwd/sync/compact` JSON body: `{ "start", "end"?, "lightwalletd_url"?, "db_path"?, "resume"? }`
pub async fn lwd_sync_compact(
    axum::Json(body): axum::Json<LwdSyncBody>,
) -> Result<ResponseJson<serde_json::Value>, (StatusCode, ResponseJson<serde_json::Value>)> {
    let url = lwd_uri_from_body(&body);
    let db = compact_db_path(&body);
    if let Some(parent) = db.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let mut client = zeaking::lwd::connect_lightwalletd(&url)
        .await
        .map_err(|e| {
            let status = zeaking_status(&e);
            error_response(status, e.to_string())
        })?;
    let store = zeaking::lwd::LwdCompactStore::open(&db).map_err(|e| {
        let status = zeaking_status(&e);
        error_response(status, e.to_string())
    })?;
    let end = if let Some(e) = body.end {
        e
    } else {
        zeaking::lwd::chain_tip_height(&mut client)
            .await
            .map_err(|e| error_response(zeaking_status(&e), e.to_string()))?
    };
    let opts = zeaking::lwd::SyncCompactOptions {
        resume_from_store: body.resume.unwrap_or(false),
        ..Default::default()
    };
    let stats =
        zeaking::lwd::sync_compact_range_with_options(&mut client, &store, body.start, end, opts)
            .await
            .map_err(|e| error_response(zeaking_status(&e), e.to_string()))?;
    Ok(ResponseJson(serde_json::json!({
        "blocks_written": stats.blocks_written,
        "range_start_requested": stats.range_start_requested,
        "range_start_effective": stats.range_start_effective,
        "range_end": stats.range_end,
        "db_path": db.to_string_lossy(),
    })))
}
