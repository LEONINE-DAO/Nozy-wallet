//! lightwalletd endpoints for Chrome/Edge extensions and web UI (`zeaking::lwd`).

use axum::{extract::Query, http::StatusCode, response::Json as ResponseJson};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::handlers::error_response_with_code;

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

#[derive(Debug, Deserialize)]
pub struct LwdSyncToTipBody {
    pub lightwalletd_url: Option<String>,
    pub db_path: Option<String>,
    /// Lower bound for first height (default `1`). Use wallet birthday / activation when shrinking range.
    pub start_floor: Option<u64>,
    /// Metadata checkpoint interval (default `32`). Minimum effective value is `1`.
    pub persist_progress_every: Option<u64>,
}

fn zeaking_status(e: &zeaking::ZeakingError) -> StatusCode {
    match e {
        zeaking::ZeakingError::Grpc(_) => StatusCode::BAD_GATEWAY,
        zeaking::ZeakingError::Storage(_) => StatusCode::INTERNAL_SERVER_ERROR,
        zeaking::ZeakingError::InvalidOperation(_) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn zeaking_error_code(e: &zeaking::ZeakingError) -> &'static str {
    match e {
        zeaking::ZeakingError::Grpc(_) => "LWD_GRPC",
        zeaking::ZeakingError::Storage(_) => "LWD_STORAGE",
        zeaking::ZeakingError::InvalidOperation(_) => "LWD_INVALID",
        zeaking::ZeakingError::Network(_) => "LWD_NETWORK",
        zeaking::ZeakingError::NotFound(_) => "LWD_NOT_FOUND",
        zeaking::ZeakingError::Serialization(_) => "LWD_SERIALIZATION",
        zeaking::ZeakingError::Io(_) => "LWD_IO",
    }
}

fn zeaking_err_response(e: zeaking::ZeakingError) -> (StatusCode, ResponseJson<serde_json::Value>) {
    let status = zeaking_status(&e);
    let code = zeaking_error_code(&e);
    let msg = e.to_string();
    error_response_with_code(status, msg, code)
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
    compact_db_path_opt(body.db_path.as_ref())
}

fn compact_db_path_opt(db_path: Option<&String>) -> PathBuf {
    db_path
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
        .map_err(zeaking_err_response)?;
    use zeaking::lwd::proto::Empty;
    let info = client
        .get_lightd_info(Empty {})
        .await
        .map_err(|e| {
            zeaking_err_response(zeaking::ZeakingError::Grpc(format!("GetLightdInfo: {e}")))
        })?
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
        .map_err(zeaking_err_response)?;
    let tip = zeaking::lwd::chain_tip_height(&mut client)
        .await
        .map_err(zeaking_err_response)?;
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
        .map_err(zeaking_err_response)?;
    let store = zeaking::lwd::LwdCompactStore::open(&db).map_err(zeaking_err_response)?;
    let end = if let Some(e) = body.end {
        e
    } else {
        zeaking::lwd::chain_tip_height(&mut client)
            .await
            .map_err(zeaking_err_response)?
    };
    let opts = zeaking::lwd::SyncCompactOptions {
        resume_from_store: body.resume.unwrap_or(false),
        ..Default::default()
    };
    let stats =
        zeaking::lwd::sync_compact_range_with_options(&mut client, &store, body.start, end, opts)
            .await
            .map_err(zeaking_err_response)?;
    Ok(ResponseJson(serde_json::json!({
        "blocks_written": stats.blocks_written,
        "range_start_requested": stats.range_start_requested,
        "range_start_effective": stats.range_start_effective,
        "range_end": stats.range_end,
        "db_path": db.to_string_lossy(),
    })))
}

/// POST `/api/lwd/sync/compact-to-tip` — sync from next missing height through lightwalletd tip (resume-safe).
pub async fn lwd_sync_compact_to_tip(
    axum::Json(body): axum::Json<LwdSyncToTipBody>,
) -> Result<ResponseJson<serde_json::Value>, (StatusCode, ResponseJson<serde_json::Value>)> {
    let url = body
        .lightwalletd_url
        .clone()
        .or_else(|| std::env::var("LIGHTWALLETD_GRPC").ok())
        .unwrap_or_else(|| "http://127.0.0.1:9067".to_string());
    let db = compact_db_path_opt(body.db_path.as_ref());
    if let Some(parent) = db.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let mut client = zeaking::lwd::connect_lightwalletd(&url)
        .await
        .map_err(zeaking_err_response)?;
    let store = zeaking::lwd::LwdCompactStore::open(&db).map_err(zeaking_err_response)?;
    let tip_opts = zeaking::lwd::SyncCompactToTipOptions {
        start_floor: body.start_floor,
        persist_progress_every: body
            .persist_progress_every
            .unwrap_or_else(|| {
                zeaking::lwd::SyncCompactToTipOptions::default().persist_progress_every
            })
            .max(1),
    };
    let stats = zeaking::lwd::sync_compact_to_tip_with_options(&mut client, &store, tip_opts)
        .await
        .map_err(zeaking_err_response)?;
    // Quiet Sapling scan when wallet unlocks with empty password (parity with CLI).
    let mut sapling_scan: Option<serde_json::Value> = None;
    if let Ok((wallet, _)) = crate::handlers::load_wallet_with_password(Some(String::new())).await {
        let seed = wallet.get_mnemonic_object().to_seed("");
        if let Ok((notes, scan)) =
            nozy::scan_sapling_wallet_from_compact_store(&seed, &store, None, false)
        {
            sapling_scan = Some(serde_json::json!({
                "notes_discovered": scan.notes_discovered,
                "notes_marked_spent": scan.notes_marked_spent,
                "unspent_zatoshis": nozy::sapling_unspent_balance_zatoshis(&notes),
            }));
        }
    }
    Ok(ResponseJson(serde_json::json!({
        "chain_tip": stats.chain_tip,
        "already_at_tip": stats.already_at_tip,
        "blocks_written": stats.blocks_written,
        "range_start_requested": stats.range_start_requested,
        "range_start_effective": stats.range_start_effective,
        "range_end": stats.range_end,
        "db_path": db.to_string_lossy(),
        "sapling_scan": sapling_scan,
    })))
}

/// Max inclusive block span for one compact HTTP pull (extension batches).
const COMPACT_HTTP_MAX_SPAN: u64 = 500;

#[derive(Debug, Deserialize)]
pub struct LwdCompactBlocksQuery {
    pub start: u64,
    pub end: u64,
    pub lightwalletd_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LwdTreeStateQuery {
    pub height: Option<u64>,
    pub lightwalletd_url: Option<String>,
}

fn lwd_uri_opt(lightwalletd_url: &Option<String>) -> String {
    lightwalletd_url
        .clone()
        .or_else(|| std::env::var("LIGHTWALLETD_GRPC").ok())
        .unwrap_or_else(|| "http://127.0.0.1:9067".to_string())
}

/// GET `/api/lwd/compact/tip` — chain tip from lightwalletd (no wallet DB).
pub async fn lwd_compact_tip(
    Query(q): Query<LwdUrlQuery>,
) -> Result<ResponseJson<serde_json::Value>, (StatusCode, ResponseJson<serde_json::Value>)> {
    let url = lwd_uri(&q);
    let mut client = zeaking::lwd::connect_lightwalletd(&url)
        .await
        .map_err(zeaking_err_response)?;
    let tip = zeaking::lwd::chain_tip_height(&mut client)
        .await
        .map_err(zeaking_err_response)?;
    Ok(ResponseJson(serde_json::json!({ "height": tip })))
}

/// GET `/api/lwd/compact/treestate?height=` — LWD treestate for tracker init (no wallet DB).
pub async fn lwd_compact_treestate(
    Query(q): Query<LwdTreeStateQuery>,
) -> Result<ResponseJson<serde_json::Value>, (StatusCode, ResponseJson<serde_json::Value>)> {
    let url = lwd_uri_opt(&q.lightwalletd_url);
    let mut client = zeaking::lwd::connect_lightwalletd(&url)
        .await
        .map_err(zeaking_err_response)?;
    let ts = if let Some(h) = q.height {
        zeaking::lwd::get_tree_state(&mut client, h, None)
            .await
            .map_err(zeaking_err_response)?
    } else {
        zeaking::lwd::get_latest_tree_state(&mut client)
            .await
            .map_err(zeaking_err_response)?
    };
    Ok(ResponseJson(serde_json::json!({
        "network": ts.network,
        "height": ts.height,
        "hash": ts.hash,
        "time": ts.time,
        "sapling_tree": ts.sapling_tree,
        "orchard_tree": ts.orchard_tree,
        // WASM tracker expects getblock-style finalState hex when present.
        "orchard": { "commitments": { "finalState": ts.orchard_tree } },
        "ironwood": { "commitments": { "finalState": ts.orchard_tree } },
    })))
}

/// GET `/api/lwd/compact/blocks?start=&end=` — stream decode to scan JSON (no SQLite, no keys).
pub async fn lwd_compact_blocks(
    Query(q): Query<LwdCompactBlocksQuery>,
) -> Result<ResponseJson<serde_json::Value>, (StatusCode, ResponseJson<serde_json::Value>)> {
    if q.end < q.start {
        return Err(error_response_with_code(
            StatusCode::BAD_REQUEST,
            "end must be >= start",
            "LWD_INVALID",
        ));
    }
    if q.end - q.start > COMPACT_HTTP_MAX_SPAN {
        return Err(error_response_with_code(
            StatusCode::BAD_REQUEST,
            format!(
                "range too large (max {} blocks inclusive span)",
                COMPACT_HTTP_MAX_SPAN
            ),
            "LWD_INVALID",
        ));
    }
    let url = lwd_uri_opt(&q.lightwalletd_url);
    // Force local LWD on the VPS — never let clients redirect this proxy at a third party.
    let url = if url.contains("127.0.0.1") || url.contains("localhost") {
        url
    } else {
        std::env::var("LIGHTWALLETD_GRPC").unwrap_or_else(|_| "http://127.0.0.1:9067".to_string())
    };
    let mut client = zeaking::lwd::connect_lightwalletd(&url)
        .await
        .map_err(zeaking_err_response)?;
    let blocks = zeaking::lwd::fetch_compact_scan_blocks(&mut client, q.start, q.end)
        .await
        .map_err(zeaking_err_response)?;
    Ok(ResponseJson(serde_json::json!({
        "start": q.start,
        "end": q.end,
        "count": blocks.len(),
        "blocks": blocks,
    })))
}
