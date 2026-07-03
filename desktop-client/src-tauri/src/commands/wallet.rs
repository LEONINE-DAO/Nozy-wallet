use crate::error::TauriError;
use crate::session::{
    clear_session, is_session_unlocked, load_session_wallet, load_wallet_for_reveal,
    set_unlock_password,
};
use nozy::{
    active_profile_id, create_new_profile, list_wallet_profiles, profile_has_wallet,
    set_active_wallet_profile, active_wallet_exists, HDWallet, WalletProfile, WalletStorage,
};
use serde::{Deserialize, Serialize};
use tauri::command;

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

#[derive(Debug, Serialize)]
pub struct WalletProfileInfo {
    pub id: String,
    pub name: String,
    pub created_at: u64,
    pub has_wallet: bool,
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct SwitchWalletProfileRequest {
    pub profile_id: String,
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

#[command]
pub async fn wallet_exists() -> Result<WalletInfo, TauriError> {
    let exists = active_wallet_exists();

    let has_password = if exists {
        let storage = WalletStorage::with_xdg_dir();
        storage.load_wallet("").await.is_err()
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

    let has_password = WalletStorage::with_xdg_dir()
        .load_wallet("")
        .await
        .is_err();

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

    let storage = WalletStorage::with_xdg_dir();
    let has_password = storage.load_wallet("").await.is_err();

    if is_session_unlocked() {
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
        .map(|profile: WalletProfile| WalletProfileInfo {
            has_wallet: profile_has_wallet(&profile.id),
            is_active: active_id.as_deref() == Some(profile.id.as_str()),
            id: profile.id,
            name: profile.name,
            created_at: profile.created_at,
        })
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
