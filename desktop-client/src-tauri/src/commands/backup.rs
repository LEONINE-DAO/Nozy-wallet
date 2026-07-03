use crate::error::TauriError;
use nozy::WalletStorage;
use serde::{Deserialize, Serialize};
use tauri::command;

#[derive(Debug, Deserialize)]
pub struct BackupPathRequest {
    pub backup_path: String,
}

#[derive(Debug, Serialize)]
pub struct BackupActionResponse {
    pub success: bool,
    pub path: String,
    pub message: String,
}

#[command]
pub async fn export_backup(request: BackupPathRequest) -> Result<BackupActionResponse, TauriError> {
    let path = request.backup_path.trim();
    if path.is_empty() {
        return Err(TauriError {
            message: "Backup path is required.".to_string(),
            code: Some("INVALID_INPUT".to_string()),
        });
    }
    if !nozy::active_wallet_exists() {
        return Err(TauriError {
            message: "No wallet found to backup.".to_string(),
            code: Some("WALLET_NOT_FOUND".to_string()),
        });
    }

    let storage = WalletStorage::with_xdg_dir();
    storage
        .create_backup(path)
        .await
        .map_err(|e| TauriError::from(e.to_string()))?;

    let backups = storage
        .list_backups()
        .map_err(|e| TauriError::from(e.to_string()))?;
    let backup_file = backups
        .into_iter()
        .max()
        .unwrap_or_else(|| path.to_string());

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
    let path = request.backup_path.trim();
    if path.is_empty() {
        return Err(TauriError {
            message: "Backup file path is required.".to_string(),
            code: Some("INVALID_INPUT".to_string()),
        });
    }

    crate::session::clear_session();

    let storage = WalletStorage::with_xdg_dir();
    storage
        .restore_from_backup(path)
        .await
        .map_err(|e| TauriError::from(e.to_string()))?;

    Ok(BackupActionResponse {
        success: true,
        path: path.to_string(),
        message: format!("Wallet restored from {path}. Unlock again to continue."),
    })
}

#[command]
pub async fn list_backups() -> Result<Vec<String>, TauriError> {
    let storage = WalletStorage::with_xdg_dir();
    storage
        .list_backups()
        .map_err(|e| TauriError::from(e.to_string()))
}
