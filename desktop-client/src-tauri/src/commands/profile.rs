use crate::error::TauriError;
use crate::session::load_session_wallet;
use nozy::{load_config, save_config, WalletRole};
use serde::{Deserialize, Serialize};
use tauri::command;

#[derive(Debug, Serialize)]
pub struct ProfileResponse {
    pub role: String,
    pub orchard_account: u32,
    pub business_display_name: Option<String>,
    pub linked_zns_name: Option<String>,
    pub linked_zns_display: Option<String>,
    pub receive_address: Option<String>,
    pub business_address: Option<String>,
    pub personal_address: Option<String>,
    pub network: String,
}

#[derive(Debug, Deserialize)]
pub struct GetProfileRequest {
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub password: Option<String>,
    pub role: Option<String>,
    pub business_display_name: Option<String>,
}

fn profile_from_config(
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

#[command]
pub async fn get_wallet_profile(request: GetProfileRequest) -> Result<ProfileResponse, TauriError> {
    let config = load_config();
    if request.password.is_none() {
        return Ok(profile_from_config(&config, None, None, None));
    }

    let wallet = load_session_wallet(request.password.as_deref())
        .await
        .map_err(|e| TauriError {
            message: e.message,
            code: e.code,
        })?;
    let net = crate::network_from_config();
    let personal = wallet
        .generate_orchard_address(0, 0, net)
        .map_err(|e| TauriError::from(e.to_string()))?;
    let business = wallet
        .generate_orchard_address(1, 0, net)
        .map_err(|e| TauriError::from(e.to_string()))?;
    let receive = if config.active_role == WalletRole::Business {
        business.clone()
    } else {
        personal.clone()
    };
    Ok(profile_from_config(
        &config,
        Some(receive),
        Some(business),
        Some(personal),
    ))
}

#[command]
pub async fn update_wallet_profile(
    request: UpdateProfileRequest,
) -> Result<ProfileResponse, TauriError> {
    let mut config = load_config();
    if let Some(role_raw) = request.role.as_deref() {
        let role = WalletRole::parse(role_raw)
            .ok_or_else(|| TauriError::from("role must be \"personal\" or \"business\""))?;
        config.active_role = role;
    }
    if let Some(name) = request.business_display_name {
        let trimmed = name.trim().to_string();
        config.business_display_name = if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        };
    }
    save_config(&config).map_err(|e| TauriError::from(e.to_string()))?;

    get_wallet_profile(GetProfileRequest {
        password: request.password,
    })
    .await
}

#[derive(Debug, Deserialize)]
pub struct LinkZnsRequest {
    pub name: String,
    /// Address from indexer resolve (client-side). Must match Business UA.
    pub resolved_address: String,
    pub password: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LinkZnsResponse {
    pub linked: bool,
    pub name: String,
    pub display: String,
    pub address: String,
    pub business_address: String,
}

#[command]
pub async fn get_zns_link() -> Result<serde_json::Value, TauriError> {
    let config = load_config();
    Ok(serde_json::json!({
        "linked": config.linked_zns_name.is_some(),
        "name": config.linked_zns_name,
        "display": config.linked_zns_display(),
    }))
}

#[command]
pub async fn link_zns_name(request: LinkZnsRequest) -> Result<LinkZnsResponse, TauriError> {
    let name = request.name.trim().to_ascii_lowercase();
    let name = name
        .strip_suffix(".zcash")
        .or_else(|| name.strip_suffix(".zec"))
        .unwrap_or(&name)
        .to_string();
    if name.is_empty() {
        return Err(TauriError::from("Enter a Zcash name."));
    }

    let resolved_address = request
        .resolved_address
        .replace([' ', '\n', '\t'], "");
    if resolved_address.is_empty() {
        return Err(TauriError::from("Resolve the name before linking."));
    }

    let wallet = load_session_wallet(request.password.as_deref())
        .await
        .map_err(|e| TauriError {
            message: e.message,
            code: e.code,
        })?;
    let business_address = wallet
        .generate_orchard_address(1, 0, crate::network_from_config())
        .map_err(|e| TauriError::from(e.to_string()))?;

    if resolved_address != business_address {
        return Err(TauriError::from(
            "Name resolves to a different address than your Business UA. Point the name at your Business receive address on zcashnames.com, then link again.",
        ));
    }

    let mut config = load_config();
    config.linked_zns_name = Some(name.clone());
    config.active_role = WalletRole::Business;
    save_config(&config).map_err(|e| TauriError::from(e.to_string()))?;

    Ok(LinkZnsResponse {
        linked: true,
        name: name.clone(),
        display: format!("{name}.zcash"),
        address: business_address.clone(),
        business_address,
    })
}

#[command]
pub async fn unlink_zns_name() -> Result<serde_json::Value, TauriError> {
    let mut config = load_config();
    config.linked_zns_name = None;
    save_config(&config).map_err(|e| TauriError::from(e.to_string()))?;
    Ok(serde_json::json!({ "linked": false, "name": null }))
}
