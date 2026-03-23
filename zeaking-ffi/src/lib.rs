//! UniFFI surface for [`zeaking::lwd`] — same operations as Tauri commands and `api-server` LWD routes.
//!
//! Generate Kotlin / Swift bindings with `uniffi-bindgen` from the built library (see README).

use std::path::Path;
use std::sync::OnceLock;

use tokio::runtime::Runtime;

fn runtime() -> &'static Runtime {
    static RT: OnceLock<Runtime> = OnceLock::new();
    RT.get_or_init(|| Runtime::new().expect("zeaking-ffi Tokio runtime"))
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
#[uniffi(flat_error)]
pub enum ZeakingFfiError {
    #[error("{0}")]
    Message(String),
}

#[derive(Clone, uniffi::Record)]
pub struct LwdInfoFfi {
    pub version: String,
    pub chain_name: String,
    pub block_height: u64,
    pub estimated_height: u64,
}

#[derive(Clone, uniffi::Record)]
pub struct LwdSyncResultFfi {
    pub blocks_written: u64,
    pub range_start: u64,
    pub range_end: u64,
}

/// gRPC `GetLightdInfo` via lightwalletd (URL like `http://127.0.0.1:9067`).
#[uniffi::export]
pub fn lwd_get_info(lightwalletd_url: String) -> Result<LwdInfoFfi, ZeakingFfiError> {
    runtime().block_on(async move {
        let mut client = zeaking::lwd::connect_lightwalletd(&lightwalletd_url)
            .await
            .map_err(|e| ZeakingFfiError::Message(e.to_string()))?;
        use zeaking::lwd::proto::Empty;
        let info = client
            .get_lightd_info(Empty {})
            .await
            .map_err(|e| ZeakingFfiError::Message(format!("GetLightdInfo: {e}")))?
            .into_inner();
        Ok(LwdInfoFfi {
            version: info.version,
            chain_name: info.chain_name,
            block_height: info.block_height,
            estimated_height: info.estimated_height,
        })
    })
}

/// Chain tip height from lightwalletd (`GetLatestBlock`).
#[uniffi::export]
pub fn lwd_chain_tip(lightwalletd_url: String) -> Result<u64, ZeakingFfiError> {
    runtime().block_on(async move {
        let mut client = zeaking::lwd::connect_lightwalletd(&lightwalletd_url)
            .await
            .map_err(|e| ZeakingFfiError::Message(e.to_string()))?;
        zeaking::lwd::chain_tip_height(&mut client)
            .await
            .map_err(|e| ZeakingFfiError::Message(e.to_string()))
    })
}

/// Download inclusive `[start, end]` compact blocks into SQLite at `db_path`.
/// If `end` is `None`, syncs through the current chain tip.
#[uniffi::export]
pub fn lwd_sync_compact(
    lightwalletd_url: String,
    db_path: String,
    start: u64,
    end: Option<u64>,
) -> Result<LwdSyncResultFfi, ZeakingFfiError> {
    runtime().block_on(async move {
        let path = Path::new(&db_path);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let mut client = zeaking::lwd::connect_lightwalletd(&lightwalletd_url)
            .await
            .map_err(|e| ZeakingFfiError::Message(e.to_string()))?;
        let store = zeaking::lwd::LwdCompactStore::open(path)
            .map_err(|e| ZeakingFfiError::Message(e.to_string()))?;
        let range_end = match end {
            Some(e) => e,
            None => zeaking::lwd::chain_tip_height(&mut client)
                .await
                .map_err(|e| ZeakingFfiError::Message(e.to_string()))?,
        };
        let blocks_written =
            zeaking::lwd::sync_compact_range(&mut client, &store, start, range_end)
                .await
                .map_err(|e| ZeakingFfiError::Message(e.to_string()))?;
        Ok(LwdSyncResultFfi {
            blocks_written,
            range_start: start,
            range_end,
        })
    })
}

uniffi::setup_scaffolding!();
