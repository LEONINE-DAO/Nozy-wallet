//! Privacy-network config, send-egress badge, and Nym helper status for desktop/CLI parity.
//! Used by the browser extension (and mobile) so Chrome never embeds the Nym SDK.

use axum::{
    extract::{Json, Query},
    http::StatusCode,
    response::Json as ResponseJson,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::handlers::error_response_with_code;

#[derive(Debug, Serialize)]
pub struct PrivacyNetworkResponse {
    pub tor_enabled: bool,
    pub tor_proxy: String,
    pub i2p_enabled: bool,
    pub i2p_proxy: String,
    pub preferred_network: String,
    pub require_privacy_network: bool,
    pub broadcast_via_nym_mixnet: bool,
    pub sync_via_nym_dvpn: bool,
    pub attest_private_network: bool,
    pub force_clearnet: bool,
}

#[derive(Debug, Deserialize)]
pub struct PrivacyNetworkUpdate {
    pub tor_enabled: Option<bool>,
    pub i2p_enabled: Option<bool>,
    pub preferred_network: Option<String>,
    pub require_privacy_network: Option<bool>,
    pub broadcast_via_nym_mixnet: Option<bool>,
    pub sync_via_nym_dvpn: Option<bool>,
    pub attest_private_network: Option<bool>,
    pub force_clearnet: Option<bool>,
}

fn snapshot(config: &nozy::WalletConfig) -> PrivacyNetworkResponse {
    let p = &config.privacy_network;
    PrivacyNetworkResponse {
        tor_enabled: p.tor_enabled,
        tor_proxy: p.tor_proxy.clone(),
        i2p_enabled: p.i2p_enabled,
        i2p_proxy: p.i2p_proxy.clone(),
        preferred_network: p.preferred_network.clone(),
        require_privacy_network: p.require_privacy_network,
        broadcast_via_nym_mixnet: p.broadcast_via_nym_mixnet,
        sync_via_nym_dvpn: p.sync_via_nym_dvpn,
        attest_private_network: p.attest_private_network,
        force_clearnet: p.force_clearnet,
    }
}

/// GET `/api/config/privacy-network`
pub async fn get_privacy_network(
) -> Result<ResponseJson<PrivacyNetworkResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    let config = nozy::load_config();
    Ok(ResponseJson(snapshot(&config)))
}

/// POST `/api/config/privacy-network`
pub async fn set_privacy_network(
    Json(body): Json<PrivacyNetworkUpdate>,
) -> Result<ResponseJson<PrivacyNetworkResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    let mut config = nozy::load_config();
    if let Some(v) = body.tor_enabled {
        config.privacy_network.tor_enabled = v;
    }
    if let Some(v) = body.i2p_enabled {
        config.privacy_network.i2p_enabled = v;
    }
    if let Some(v) = body.preferred_network {
        let t = v.trim().to_ascii_lowercase();
        if t != "tor" && t != "i2p" && t != "none" {
            return Err(error_response_with_code(
                StatusCode::BAD_REQUEST,
                "preferred_network must be tor, i2p, or none",
                "INVALID_PREFERRED_NETWORK",
            ));
        }
        config.privacy_network.preferred_network = t;
    }
    if let Some(v) = body.require_privacy_network {
        config.privacy_network.require_privacy_network = v;
    }
    if let Some(v) = body.broadcast_via_nym_mixnet {
        config.privacy_network.broadcast_via_nym_mixnet = v;
    }
    if let Some(v) = body.sync_via_nym_dvpn {
        config.privacy_network.sync_via_nym_dvpn = v;
    }
    if let Some(v) = body.attest_private_network {
        config.privacy_network.attest_private_network = v;
    }
    if let Some(v) = body.force_clearnet {
        config.privacy_network.force_clearnet = v;
    }
    nozy::save_config(&config).map_err(|e| {
        error_response_with_code(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to save privacy-network config: {e}"),
            "CONFIG_SAVE",
        )
    })?;
    Ok(ResponseJson(snapshot(&config)))
}

/// GET `/api/privacy/send-egress`
pub async fn get_send_egress(
) -> Result<ResponseJson<nozy::SendEgressSnapshot>, (StatusCode, ResponseJson<serde_json::Value>)> {
    let config = nozy::load_config();
    Ok(ResponseJson(nozy::assess_send_egress(&config)))
}

/// GET `/api/privacy/nym-mixnet`
pub async fn get_nym_mixnet() -> Result<
    ResponseJson<nozy::nym_mixnet_broadcast::MixnetBroadcastReadiness>,
    (StatusCode, ResponseJson<serde_json::Value>),
> {
    let config = nozy::load_config();
    Ok(ResponseJson(
        nozy::nym_mixnet_broadcast::assess_mixnet_broadcast_readiness(
            &config.zebra_url,
            config.privacy_network.broadcast_via_nym_mixnet,
        ),
    ))
}

/// GET `/api/privacy/nym-dvpn?lightwalletd_url=`
pub async fn get_nym_dvpn(
    Query(q): Query<HashMap<String, String>>,
) -> Result<
    ResponseJson<nozy::nym_dvpn_sync::NymDvpnSyncReadiness>,
    (StatusCode, ResponseJson<serde_json::Value>),
> {
    let config = nozy::load_config();
    let lwd = q.get("lightwalletd_url").map(|s| s.as_str());
    Ok(ResponseJson(
        nozy::nym_dvpn_sync::assess_dvpn_sync_readiness(
            config.privacy_network.sync_via_nym_dvpn,
            lwd,
        ),
    ))
}

#[derive(Debug, Deserialize)]
pub struct SetNymDvpnBody {
    pub enabled: bool,
}

/// POST `/api/privacy/nym-dvpn`
pub async fn set_nym_dvpn(
    Json(body): Json<SetNymDvpnBody>,
) -> Result<
    ResponseJson<nozy::nym_dvpn_sync::NymDvpnSyncReadiness>,
    (StatusCode, ResponseJson<serde_json::Value>),
> {
    let mut config = nozy::load_config();
    config.privacy_network.sync_via_nym_dvpn = body.enabled;
    nozy::save_config(&config).map_err(|e| {
        error_response_with_code(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to save dVPN flag: {e}"),
            "CONFIG_SAVE",
        )
    })?;
    Ok(ResponseJson(
        nozy::nym_dvpn_sync::assess_dvpn_sync_readiness(
            config.privacy_network.sync_via_nym_dvpn,
            None,
        ),
    ))
}

#[derive(Debug, Deserialize)]
pub struct NymDvpnProbeBody {
    pub lightwalletd_url: Option<String>,
    pub blocks: Option<u64>,
}

/// GET `/api/privacy/nym-vpn-app`
///
/// Consumer NymVPN OS app (not mixnet sendraw / dVPN sync). Used to gate the
/// in-wallet browser: Chrome cannot start the VPN; the companion can see it.
pub async fn get_nym_vpn_app() -> Result<
    ResponseJson<nozy::nym_vpn_app::NymVpnAppStatus>,
    (StatusCode, ResponseJson<serde_json::Value>),
> {
    Ok(ResponseJson(nozy::nym_vpn_app::probe_nym_vpn_app().await))
}

/// POST `/api/privacy/nym-dvpn/probe`
pub async fn probe_nym_dvpn(
    Json(body): Json<NymDvpnProbeBody>,
) -> Result<
    ResponseJson<nozy::nym_dvpn_sync::NymDvpnSyncProbeResult>,
    (StatusCode, ResponseJson<serde_json::Value>),
> {
    let config = nozy::load_config();
    let result = nozy::nym_dvpn_sync::run_dvpn_sync_probe(
        config.privacy_network.sync_via_nym_dvpn,
        body.lightwalletd_url.as_deref(),
        body.blocks,
    )
    .await
    .map_err(|e| error_response_with_code(StatusCode::BAD_GATEWAY, e.to_string(), "NYM_DVPN"))?;
    Ok(ResponseJson(result))
}
