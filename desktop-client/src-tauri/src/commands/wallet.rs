use crate::error::TauriError;
use crate::session::{
    clear_session, is_session_unlocked, load_session_wallet, load_wallet_for_reveal,
    set_unlock_password,
};
use nozy::{
    active_profile_id, active_wallet_exists, configure_profile_network, create_new_profile,
    default_zebra_url_for_network, list_wallet_profiles, load_config,
    profile_connection_settings, profile_has_wallet, set_active_wallet_profile, HDWallet,
    WalletProfile, WalletStorage,
};
use serde::{Deserialize, Serialize};
use tauri::command;
use zcash_protocol::consensus::NetworkType;

#[derive(Debug, Serialize)]
pub struct WalletInfo {
    pub exists: bool,
    pub has_password: bool,
}

#[derive(Debug, Serialize)]
pub struct WalletStatus {
    pub exists: bool,
    pub unlocked: bool,
    pub has_password: bool,
    pub address: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UnlockWalletRequest {
    #[serde(default)]
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateWalletRequest {
    pub password: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RestoreWalletRequest {
    pub mnemonic: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WalletProfileInfo {
    pub id: String,
    pub name: String,
    pub created_at: u64,
    pub has_wallet: bool,
    pub is_active: bool,
    pub network: String,
    pub zebra_url: String,
}

#[derive(Debug, Deserialize)]
pub struct SwitchWalletProfileRequest {
    pub profile_id: String,
}

#[derive(Debug, Serialize)]
pub struct NetworkWalletStatus {
    pub network: String,
    pub zebra_url: String,
    pub active_profile: Option<WalletProfileInfo>,
    pub profiles: Vec<WalletProfileInfo>,
    pub suggested_testnet_profile_id: Option<String>,
    pub testnet_ready: bool,
}

#[derive(Debug, Deserialize)]
pub struct ConfigureNetworkWalletRequest {
    pub network: String,
    pub profile_id: Option<String>,
    pub zebra_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DesktopTestnetWalletRequest {
    pub name: Option<String>,
    pub password: Option<String>,
    pub mnemonic: Option<String>,
    pub rpc_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DesktopTestnetWalletResponse {
    pub profile: WalletProfileInfo,
    pub address: String,
    pub mnemonic: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyPasswordRequest {
    pub password: String,
}

async fn unlock_wallet_from_storage(
    supplied_password: &str,
) -> Result<(HDWallet, String), TauriError> {
    let storage = WalletStorage::with_xdg_dir();

    // Match CLI `load_wallet`: empty encryption password first.
    if let Ok(wallet) = storage.load_wallet("").await {
        return Ok((wallet, String::new()));
    }

    let trimmed = supplied_password.trim();
    if trimmed.is_empty() {
        return Err(TauriError {
            message: "This wallet requires a password. Enter the same password you use with the CLI."
                .to_string(),
            code: Some("AUTH_002".to_string()),
        });
    }

    storage
        .load_wallet(trimmed)
        .await
        .map(|wallet| (wallet, trimmed.to_string()))
        .map_err(|e| {
            let msg = e.to_string();
            TauriError {
                message: if msg.contains("Decryption failed") || msg.contains("Invalid password") {
                    "Incorrect password. If the CLI opens without asking for a password, leave this field blank and tap Unlock."
                        .to_string()
                } else {
                    format!("Failed to unlock wallet: {msg}")
                },
                code: Some("INVALID_PASSWORD".to_string()),
            }
        })
}

fn wallet_profile_info(profile: WalletProfile, active_id: Option<&str>) -> WalletProfileInfo {
    let connection = profile_connection_settings(&profile.id).unwrap_or_else(|_| {
        nozy::ProfileConnectionSettings {
            network: nozy::default_network_for_profile_name(&profile.name).to_string(),
            zebra_url: nozy::default_zebra_url_for_network(
                nozy::default_network_for_profile_name(&profile.name),
            )
            .to_string(),
            last_scan_height: profile.last_scan_height,
        }
    });
    WalletProfileInfo {
        has_wallet: profile_has_wallet(&profile.id),
        is_active: active_id == Some(profile.id.as_str()),
        id: profile.id,
        name: profile.name,
        created_at: profile.created_at,
        network: connection.network,
        zebra_url: connection.zebra_url,
    }
}

fn network_wallet_status() -> Result<NetworkWalletStatus, TauriError> {
    let config = load_config();
    let active_id = active_profile_id();
    let profiles: Vec<WalletProfileInfo> = list_wallet_profiles()
        .map_err(|e| TauriError::from(e.to_string()))?
        .into_iter()
        .map(|profile| wallet_profile_info(profile, active_id.as_deref()))
        .collect();
    let active_profile = active_id
        .as_deref()
        .and_then(|id| profiles.iter().find(|profile| profile.id == id))
        .cloned();
    let suggested_testnet_profile_id = profiles
        .iter()
        .find(|profile| {
            profile.has_wallet && profile.name.to_ascii_lowercase().contains("testnet")
        })
        .map(|profile| profile.id.clone());

    Ok(NetworkWalletStatus {
        testnet_ready: config.network.eq_ignore_ascii_case("testnet")
            && config.zebra_url.contains(":18232")
            && active_profile.as_ref().is_some_and(|profile| profile.has_wallet),
        network: config.network,
        zebra_url: config.zebra_url,
        active_profile,
        profiles,
        suggested_testnet_profile_id,
    })
}

fn configure_network(
    network: &str,
    profile_id: Option<&str>,
    zebra_url: Option<&str>,
) -> Result<(), TauriError> {
    let normalized_network = match network.to_ascii_lowercase().as_str() {
        "mainnet" | "main" => "mainnet",
        "testnet" | "test" => "testnet",
        other => {
            return Err(TauriError {
                message: format!("Unsupported network: {other}. Use mainnet or testnet."),
                code: Some("INVALID_NETWORK".to_string()),
            })
        }
    };

    clear_session();

    let target_profile_id = profile_id
        .map(str::to_string)
        .or_else(active_profile_id)
        .ok_or_else(|| TauriError {
            message: "Select a wallet profile before switching network.".to_string(),
            code: Some("INVALID_PROFILE".to_string()),
        })?;

    if active_profile_id().as_deref() != Some(target_profile_id.as_str()) {
        set_active_wallet_profile(&target_profile_id)
            .map_err(|e| TauriError::from(e.to_string()))?;
    }

    let resolved_url = zebra_url
        .filter(|url| !url.trim().is_empty())
        .map(|url| url.trim().to_string())
        .unwrap_or_else(|| default_zebra_url_for_network(normalized_network).to_string());

    configure_profile_network(
        &target_profile_id,
        normalized_network,
        &resolved_url,
        true,
    )
    .map_err(|e| TauriError::from(e.to_string()))
}

#[command]
pub async fn wallet_exists() -> Result<WalletInfo, TauriError> {
    let exists = active_wallet_exists();

    let has_password = if exists {
        WalletStorage::with_xdg_dir().requires_password().await
    } else {
        false
    };

    Ok(WalletInfo {
        exists,
        has_password,
    })
}

#[command]
pub async fn create_wallet(request: CreateWalletRequest) -> Result<String, TauriError> {
    clear_session();

    create_new_profile(request.name.as_deref())
        .map_err(|e| TauriError::from(e.to_string()))?;

    let mut wallet = HDWallet::new().map_err(|e| TauriError::from(e.to_string()))?;

    let password = request.password.as_deref().unwrap_or("");

    if !password.is_empty() {
        wallet
            .set_password(password)
            .map_err(|e| TauriError::from(e.to_string()))?;
    }

    let mnemonic = wallet.get_mnemonic();

    let storage = WalletStorage::with_xdg_dir();
    storage
        .save_wallet(&wallet, password)
        .await
        .map_err(|e| TauriError::from(e.to_string()))?;

    set_unlock_password(password.to_string());

    Ok(mnemonic)
}

#[command]
pub async fn restore_wallet(request: RestoreWalletRequest) -> Result<(), TauriError> {
    let words: Vec<&str> = request.mnemonic.split_whitespace().collect();
    if !matches!(words.len(), 12 | 15 | 18 | 21 | 24) {
        return Err(TauriError {
            message: "Invalid mnemonic format. Must be 12, 15, 18, 21, or 24 words.".to_string(),
            code: Some("INVALID_MNEMONIC".to_string()),
        });
    }

    clear_session();
    create_new_profile(None).map_err(|e| TauriError::from(e.to_string()))?;

    let wallet =
        HDWallet::from_mnemonic(&request.mnemonic).map_err(|e| TauriError::from(e.to_string()))?;

    let storage = WalletStorage::with_xdg_dir();
    storage
        .save_wallet(&wallet, &request.password)
        .await
        .map_err(|e| TauriError::from(e.to_string()))?;

    set_unlock_password(request.password.clone());

    Ok(())
}

#[command]
pub async fn unlock_wallet(request: UnlockWalletRequest) -> Result<WalletStatus, TauriError> {
    let (wallet, session_password) = unlock_wallet_from_storage(&request.password).await?;

    set_unlock_password(session_password.clone());

    let address = wallet
        .generate_orchard_address(0, 0, crate::network_from_config())
        .map_err(|e| TauriError::from(e.to_string()))?;

    let has_password = !session_password.is_empty();

    Ok(WalletStatus {
        exists: true,
        unlocked: true,
        has_password,
        address: Some(address),
    })
}

#[command]
pub fn lock_wallet() -> Result<(), TauriError> {
    clear_session();
    Ok(())
}

#[command]
pub async fn get_wallet_status() -> Result<WalletStatus, TauriError> {
    if !active_wallet_exists() {
        return Ok(WalletStatus {
            exists: false,
            unlocked: false,
            has_password: false,
            address: None,
        });
    }

    // When already unlocked, avoid a wasted empty-password decrypt (100k-iter KDF)
    // that previously blocked boot on the "Initializing Secure Environment..." screen.
    if is_session_unlocked() {
        let has_password = crate::session::session_password()
            .map(|p| !p.is_empty())
            .unwrap_or(false);
        match load_session_wallet(None).await {
            Ok(wallet) => {
                let address = wallet
                    .generate_orchard_address(0, 0, crate::network_from_config())
                    .ok();
                return Ok(WalletStatus {
                    exists: true,
                    unlocked: true,
                    has_password,
                    address,
                });
            }
            Err(_) => {
                clear_session();
            }
        }
    }

    let storage = WalletStorage::with_xdg_dir();
    let has_password = storage.requires_password().await;

    Ok(WalletStatus {
        exists: true,
        unlocked: false,
        has_password,
        address: None,
    })
}

#[command]
pub fn list_wallet_profiles_cmd() -> Result<Vec<WalletProfileInfo>, TauriError> {
    let profiles = list_wallet_profiles().map_err(|e| TauriError::from(e.to_string()))?;
    let active_id = active_profile_id();

    Ok(profiles
        .into_iter()
        .map(|profile: WalletProfile| wallet_profile_info(profile, active_id.as_deref()))
        .collect())
}

#[command]
pub async fn switch_wallet_profile(request: SwitchWalletProfileRequest) -> Result<(), TauriError> {
    clear_session();
    set_active_wallet_profile(&request.profile_id)
        .map_err(|e| TauriError::from(e.to_string()))?;
    Ok(())
}

#[command]
pub fn get_network_wallet_status() -> Result<NetworkWalletStatus, TauriError> {
    network_wallet_status()
}

#[command]
pub fn configure_network_wallet(
    request: ConfigureNetworkWalletRequest,
) -> Result<NetworkWalletStatus, TauriError> {
    configure_network(
        &request.network,
        request.profile_id.as_deref(),
        request.zebra_url.as_deref(),
    )?;
    network_wallet_status()
}

#[command]
pub async fn create_or_restore_testnet_wallet(
    request: DesktopTestnetWalletRequest,
) -> Result<DesktopTestnetWalletResponse, TauriError> {
    clear_session();
    let profile = create_new_profile(Some(
        request.name.as_deref().unwrap_or("Ironwood Testnet"),
    ))
    .map_err(|e| TauriError::from(e.to_string()))?;

    configure_network(
        "testnet",
        Some(&profile.id),
        request.rpc_url.as_deref().or(Some("http://127.0.0.1:18232")),
    )?;

    let wallet = if let Some(mnemonic) = request.mnemonic.as_deref() {
        let words: Vec<&str> = mnemonic.split_whitespace().collect();
        if !matches!(words.len(), 12 | 15 | 18 | 21 | 24) {
            return Err(TauriError {
                message: "Invalid mnemonic format. Must be 12, 15, 18, 21, or 24 words."
                    .to_string(),
                code: Some("INVALID_MNEMONIC".to_string()),
            });
        }
        HDWallet::from_mnemonic(mnemonic).map_err(|e| TauriError::from(e.to_string()))?
    } else {
        HDWallet::new().map_err(|e| TauriError::from(e.to_string()))?
    };
    let generated_mnemonic = if request.mnemonic.as_deref().is_some_and(|m| !m.trim().is_empty()) {
        None
    } else {
        Some(wallet.get_mnemonic())
    };

    let password = request.password.as_deref().unwrap_or("");
    let storage = WalletStorage::with_xdg_dir();
    storage
        .save_wallet(&wallet, password)
        .await
        .map_err(|e| TauriError::from(e.to_string()))?;
    set_unlock_password(password.to_string());

    let address = wallet
        .generate_orchard_address(0, 0, NetworkType::Test)
        .map_err(|e| TauriError::from(e.to_string()))?;
    let active_id = active_profile_id();

    Ok(DesktopTestnetWalletResponse {
        profile: wallet_profile_info(profile, active_id.as_deref()),
        address,
        mnemonic: generated_mnemonic,
    })
}

#[command]
pub async fn change_password(request: ChangePasswordRequest) -> Result<(), TauriError> {
    let storage = WalletStorage::with_xdg_dir();
    let mut wallet = storage
        .load_wallet(&request.current_password)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            TauriError {
                message: if msg.contains("Decryption failed") || msg.contains("Invalid password") {
                    "Current password is incorrect.".to_string()
                } else {
                    format!("Failed to verify current password: {msg}")
                },
                code: Some("INVALID_PASSWORD".to_string()),
            }
        })?;

    wallet
        .set_password(&request.new_password)
        .map_err(|e| TauriError::from(e.to_string()))?;

    storage
        .save_wallet(&wallet, &request.new_password)
        .await
        .map_err(|e| TauriError::from(e.to_string()))?;

    set_unlock_password(request.new_password);
    Ok(())
}

#[command]
pub async fn get_mnemonic(request: VerifyPasswordRequest) -> Result<String, TauriError> {
    let wallet = load_wallet_for_reveal(&request.password).await?;
    Ok(wallet.get_mnemonic())
}

#[command]
pub async fn get_private_key(request: VerifyPasswordRequest) -> Result<String, TauriError> {
    let wallet = load_wallet_for_reveal(&request.password).await?;
    let key = wallet
        .derive_key("m/44'/133'/0'/0/0")
        .map_err(|e| TauriError::from(e.to_string()))?;
    Ok(hex::encode(key.private_key().to_bytes()))
}
