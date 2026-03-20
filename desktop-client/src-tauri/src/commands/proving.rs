use crate::error::TauriError;
use nozy::paths::get_wallet_data_dir;
use nozy::proving::OrchardProvingManager;
use serde::Serialize;
use tauri::command;

#[derive(Debug, Serialize)]
pub struct ProvingStatusResponse {
    pub spend_params: bool,
    pub output_params: bool,
    pub spend_vk: bool,
    pub output_vk: bool,
    pub can_prove: bool,
    pub message: String,
}

#[command]
pub async fn check_proving_status() -> Result<ProvingStatusResponse, TauriError> {
    let params_dir = get_wallet_data_dir().join("orchard_params");
    let mut manager = OrchardProvingManager::new(params_dir);

    manager
        .initialize()
        .await
        .map_err(|e| TauriError::from(e.to_string()))?;

    let status = manager.get_status();

    Ok(ProvingStatusResponse {
        spend_params: status.spend_params,
        output_params: status.output_params,
        spend_vk: status.spend_vk,
        output_vk: status.output_vk,
        can_prove: status.can_prove,
        message: status.status_message(),
    })
}

#[command]
pub async fn download_proving_parameters() -> Result<String, TauriError> {
    let params_dir = get_wallet_data_dir().join("orchard_params");
    let mut manager = OrchardProvingManager::new(params_dir);

    manager
        .initialize()
        .await
        .map_err(|e| TauriError::from(e.to_string()))?;

    manager
        .download_parameters()
        .await
        .map_err(|e| TauriError::from(e.to_string()))?;

    Ok("Proving parameters downloaded successfully".to_string())
}
