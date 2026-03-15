use crate::error::TauriResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProvingStatusResponse {
    pub downloaded: bool,
    pub progress: f64,
}

#[tauri::command]
pub async fn check_proving_status() -> TauriResult<ProvingStatusResponse> {
    // Orchard uses Halo 2 which doesn't require external proving parameters
    // The proving system is built-in, so it's always "ready"
    Ok(ProvingStatusResponse {
        downloaded: true,
        progress: 100.0,
    })
}

#[tauri::command]
pub async fn download_proving_parameters() -> TauriResult<String> {
    // Orchard uses Halo 2 proving system which doesn't require external parameters
    // The proving system is built-in and ready to use
    Ok("Proving system ready - no parameters needed for Orchard Halo 2".to_string())
}
