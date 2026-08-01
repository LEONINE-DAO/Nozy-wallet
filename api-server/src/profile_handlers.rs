//! Personal / Business wallet role (Orchard account 0 / 1).

use axum::{extract::Json, http::StatusCode, response::Json as ResponseJson};
use serde::{Deserialize, Serialize};

use crate::handlers::{error_response, load_wallet_with_password};

#[derive(Debug, Serialize)]
pub struct ProfileResponse {
    pub role: String,
    pub orchard_account: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub business_display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linked_zns_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linked_zns_display: Option<String>,
    /// Unified address for the active role (requires wallet unlock / password).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receive_address: Option<String>,
    /// Business (account 1) UA when password provided — used for ZNS link.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub business_address: Option<String>,
    /// Personal (account 0) UA when password provided.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub personal_address: Option<String>,
    pub network: String,
}

#[derive(Debug, Deserialize)]
pub struct GetProfileQuery {
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub password: Option<String>,
    /// `"personal"` or `"business"`.
    pub role: Option<String>,
    pub business_display_name: Option<String>,
}

fn network_type(config: &nozy::WalletConfig) -> zcash_protocol::consensus::NetworkType {
    if config.network == "testnet" {
        zcash_protocol::consensus::NetworkType::Test
    } else {
        zcash_protocol::consensus::NetworkType::Main
    }
}

fn build_profile_response(
    config: &nozy::WalletConfig,
    receive_address: Option<String>,
    business_address: Option<String>,
    personal_address: Option<String>,
) -> ProfileResponse {
    ProfileResponse {
        role: config.active_role.as_str().to_string(),
        orchard_account: config.active_orchard_account(),
        business_display_name: config.business_display_name.clone(),
        linked_zns_name: config.linked_zns_name.clone(),
        linked_zns_display: config.linked_zns_display(),
        receive_address,
        business_address,
        personal_address,
        network: config.network.clone(),
    }
}

/// GET `/api/profile` — optional `?password=` to include receive UAs.
pub async fn get_profile(
    axum::extract::Query(query): axum::extract::Query<GetProfileQuery>,
) -> Result<ResponseJson<ProfileResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    let config = nozy::load_config();
    if query.password.is_none() {
        return Ok(ResponseJson(build_profile_response(
            &config, None, None, None,
        )));
    }

    let (wallet, _storage) = load_wallet_with_password(query.password)
        .await
        .map_err(|e| error_response(StatusCode::UNAUTHORIZED, e))?;

    let net = network_type(&config);
    let personal = wallet
        .generate_orchard_address(0, 0, net)
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let business = wallet
        .generate_orchard_address(1, 0, net)
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let receive = if config.active_role == nozy::WalletRole::Business {
        business.clone()
    } else {
        personal.clone()
    };

    Ok(ResponseJson(build_profile_response(
        &config,
        Some(receive),
        Some(business),
        Some(personal),
    )))
}

/// POST `/api/profile` — set role and/or business display name.
pub async fn update_profile(
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<ResponseJson<ProfileResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    let mut config = nozy::load_config();

    if let Some(role_raw) = payload.role.as_deref() {
        let role = nozy::WalletRole::parse(role_raw).ok_or_else(|| {
            error_response(
                StatusCode::BAD_REQUEST,
                "role must be \"personal\" or \"business\"",
            )
        })?;
        config.active_role = role;
    }

    if let Some(name) = payload.business_display_name {
        let trimmed = name.trim().to_string();
        config.business_display_name = if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        };
    }

    nozy::save_config(&config).map_err(|e| {
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to save profile: {e}"),
        )
    })?;

    // Re-read after save; optionally attach addresses when password given.
    let config = nozy::load_config();
    if payload.password.is_none() {
        return Ok(ResponseJson(build_profile_response(
            &config, None, None, None,
        )));
    }

    let (wallet, _storage) = load_wallet_with_password(payload.password)
        .await
        .map_err(|e| error_response(StatusCode::UNAUTHORIZED, e))?;
    let net = network_type(&config);
    let personal = wallet
        .generate_orchard_address(0, 0, net)
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let business = wallet
        .generate_orchard_address(1, 0, net)
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let receive = if config.active_role == nozy::WalletRole::Business {
        business.clone()
    } else {
        personal.clone()
    };

    Ok(ResponseJson(build_profile_response(
        &config,
        Some(receive),
        Some(business),
        Some(personal),
    )))
}
