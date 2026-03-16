use crate::error::TauriError;
use nozy::{HDWallet, WalletStorage, paths::get_wallet_data_dir};
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
pub struct CreateWalletRequest {
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RestoreWalletRequest {
    pub mnemonic: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct UnlockWalletRequest {
    pub password: String,
}

#[command]
pub async fn wallet_exists() -> Result<WalletInfo, TauriError> {
    let wallet_path = get_wallet_data_dir().join("wallet.dat");
    let exists = wallet_path.exists();
    
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
pub async fn create_wallet(
    request: CreateWalletRequest,
) -> Result<String, TauriError> {
    let wallet_path = get_wallet_data_dir().join("wallet.dat");
    
    if wallet_path.exists() {
        return Err(TauriError {
            message: "A wallet already exists! To create a new wallet, please delete the existing one first or restore from your seed phrase.".to_string(),
            code: Some("WALLET_EXISTS".to_string()),
        });
    }
    
    let mut wallet = HDWallet::new()
        .map_err(|e| TauriError::from(e.to_string()))?;
    
    let password = request.password.as_deref().unwrap_or("");
    
    if !password.is_empty() {
        wallet.set_password(password)
            .map_err(|e| TauriError::from(e.to_string()))?;
    }
    
    let mnemonic = wallet.get_mnemonic();
    
    let storage = WalletStorage::with_xdg_dir();
    storage.save_wallet(&wallet, password)
        .await
        .map_err(|e| TauriError::from(e.to_string()))?;
    
    Ok(mnemonic)
}

#[command]
pub async fn restore_wallet(
    request: RestoreWalletRequest,
) -> Result<(), TauriError> {
    let words: Vec<&str> = request.mnemonic.split_whitespace().collect();
    if !matches!(words.len(), 12 | 15 | 18 | 21 | 24) {
        return Err(TauriError {
            message: "Invalid mnemonic format. Must be 12, 15, 18, 21, or 24 words.".to_string(),
            code: Some("INVALID_MNEMONIC".to_string()),
        });
    }
    
    let wallet = HDWallet::from_mnemonic(&request.mnemonic)
        .map_err(|e| TauriError::from(e.to_string()))?;
    
    let storage = WalletStorage::with_xdg_dir();
    storage.save_wallet(&wallet, &request.password)
        .await
        .map_err(|e| TauriError::from(e.to_string()))?;
    
    Ok(())
}

#[command]
pub async fn unlock_wallet(
    request: UnlockWalletRequest,
) -> Result<WalletStatus, TauriError> {
    let storage = WalletStorage::with_xdg_dir();
    let wallet = storage.load_wallet(&request.password)
        .await
        .map_err(|e| TauriError {
            message: format!("Failed to unlock wallet: {}", e),
            code: Some("INVALID_PASSWORD".to_string()),
        })?;
    
    let address = wallet.generate_orchard_address(0, 0)
        .map_err(|e| TauriError::from(e.to_string()))?;
    
    Ok(WalletStatus {
        exists: true,
        unlocked: true,
        has_password: !request.password.is_empty(),
        address: Some(address),
    })
}

#[command]
pub async fn get_wallet_status() -> Result<WalletStatus, TauriError> {
    let wallet_path = get_wallet_data_dir().join("wallet.dat");
    
    if !wallet_path.exists() {
        return Ok(WalletStatus {
            exists: false,
            unlocked: false,
            has_password: false,
            address: None,
        });
    }
    
    let storage = WalletStorage::with_xdg_dir();
    
    let unlocked = storage.load_wallet("").await.is_ok();
    
    let address = if unlocked {
        if let Ok(wallet) = storage.load_wallet("").await {
            wallet.generate_orchard_address(0, 0).ok()
        } else {
            None
        }
    } else {
        None
    };
    
    let has_password = !unlocked;
    
    Ok(WalletStatus {
        exists: true,
        unlocked,
        has_password,
        address,
    })
}

