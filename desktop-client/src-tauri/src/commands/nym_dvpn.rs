//! Nym dVPN compact-sync readiness + probe (subprocess helper; issue #146 / C6).

use crate::error::TauriError;
use nozy::{load_config, save_config};
use serde::{Deserialize, Serialize};
use tauri::command;

#[derive(Debug, Serialize)]
pub struct NymDvpnSyncStatusResponse {
    pub requested: bool,
    pub lwd_url: String,
    pub lwd_url_local: bool,
    pub would_use_dvpn: bool,
    pub helper_ok: bool,
    pub helper_path: Option<String>,
    pub helper_error: Option<String>,
    pub mnemonic_env_ok: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SetSyncViaNymDvpnRequest {
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct RunNymDvpnSyncProbeRequest {
    pub lightwalletd_url: Option<String>,
    pub blocks: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct NymDvpnSyncProbeResponse {
    pub ok: bool,
    pub exit_code: Option<i32>,
    pub helper_path: String,
    pub lwd_url: String,
    pub blocks: u64,
    pub stdout_tail: String,
    pub stderr_tail: String,
    pub timed_out: bool,
}

#[command]
pub async fn get_nym_dvpn_sync_status(
    lightwalletd_url: Option<String>,
) -> Result<NymDvpnSyncStatusResponse, TauriError> {
    let config = load_config();
    let r = nozy::nym_dvpn_sync::assess_dvpn_sync_readiness(
        config.privacy_network.sync_via_nym_dvpn,
        lightwalletd_url.as_deref(),
    );
    Ok(NymDvpnSyncStatusResponse {
        requested: r.requested,
        lwd_url: r.lwd_url,
        lwd_url_local: r.lwd_url_local,
        would_use_dvpn: r.would_use_dvpn,
        helper_ok: r.helper_ok,
        helper_path: r.helper_path,
        helper_error: r.helper_error,
        mnemonic_env_ok: r.mnemonic_env_ok,
        notes: r.notes,
    })
}

#[command]
pub async fn set_sync_via_nym_dvpn(request: SetSyncViaNymDvpnRequest) -> Result<(), TauriError> {
    let mut config = load_config();
    config.privacy_network.sync_via_nym_dvpn = request.enabled;
    save_config(&config).map_err(|e| TauriError::from(e.to_string()))?;
    Ok(())
}

#[command]
pub async fn run_nym_dvpn_sync_probe(
    request: RunNymDvpnSyncProbeRequest,
) -> Result<NymDvpnSyncProbeResponse, TauriError> {
    let config = load_config();
    let r = nozy::nym_dvpn_sync::run_dvpn_sync_probe(
        config.privacy_network.sync_via_nym_dvpn,
        request.lightwalletd_url.as_deref(),
        request.blocks,
    )
    .await
    .map_err(|e| TauriError {
        message: e.to_string(),
        code: Some("NYM_DVPN".to_string()),
    })?;
    Ok(NymDvpnSyncProbeResponse {
        ok: r.ok,
        exit_code: r.exit_code,
        helper_path: r.helper_path,
        lwd_url: r.lwd_url,
        blocks: r.blocks,
        stdout_tail: r.stdout_tail,
        stderr_tail: r.stderr_tail,
        timed_out: r.timed_out,
    })
}
