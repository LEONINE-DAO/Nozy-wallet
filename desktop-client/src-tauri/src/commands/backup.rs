use crate::error::TauriError;
use crate::session::{clear_session, load_wallet_for_migrate};
use nozy::{resolve_allowlisted_user_path, WalletStorage};
use serde::{Deserialize, Serialize};
use tauri::command;

#[derive(Debug, Deserialize)]
pub struct BackupPathRequest {
    pub backup_path: String,
    /// Required for restore when the wallet is password-protected (F-11).
    pub password: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BackupActionResponse {
    pub success: bool,
    pub path: String,
    pub message: String,
}

#[command]
pub async fn export_backup(request: BackupPathRequest) -> Result<BackupActionResponse, TauriError> {
    let path = resolve_allowlisted_user_path(&request.backup_path).map_err(|message| {
        TauriError {
            message,
            code: Some("PATH_DENIED".to_string()),
        }
    })?;
    if !nozy::active_wallet_exists() {
        return Err(TauriError {
            message: "No wallet found to backup.".to_string(),
            code: Some("WALLET_NOT_FOUND".to_string()),
        });
    }

    let storage = WalletStorage::with_xdg_dir();
    storage
        .create_backup(path.to_string_lossy().as_ref())
        .await
        .map_err(|e| TauriError::from(e.to_string()))?;

    let backups = storage
        .list_backups()
        .map_err(|e| TauriError::from(e.to_string()))?;
    let backup_file = backups
        .into_iter()
        .max()
        .unwrap_or_else(|| path.display().to_string());

    Ok(BackupActionResponse {
        success: true,
        path: backup_file.clone(),
        message: format!("Wallet backup created at {backup_file}"),
    })
}

#[command]
pub async fn restore_from_backup(
    request: BackupPathRequest,
) -> Result<BackupActionResponse, TauriError> {
    let path = resolve_allowlisted_user_path(&request.backup_path).map_err(|message| {
        TauriError {
            message,
            code: Some("PATH_DENIED".to_string()),
        }
    })?;

    // F-11: require step-up auth before overwriting an existing wallet.dat.
    if nozy::active_wallet_exists() {
        let _ = load_wallet_for_migrate(request.password.as_deref()).await?;
    }

    clear_session();

    let storage = WalletStorage::with_xdg_dir();
    storage
        .restore_from_backup(path.to_string_lossy().as_ref())
        .await
        .map_err(|e| TauriError::from(e.to_string()))?;

    Ok(BackupActionResponse {
        success: true,
        path: path.display().to_string(),
        message: format!(
            "Wallet restored from {}. Unlock again to continue.",
            path.display()
        ),
    })
}

#[command]
pub async fn list_backups() -> Result<Vec<String>, TauriError> {
    let storage = WalletStorage::with_xdg_dir();
    storage
        .list_backups()
        .map_err(|e| TauriError::from(e.to_string()))
}
