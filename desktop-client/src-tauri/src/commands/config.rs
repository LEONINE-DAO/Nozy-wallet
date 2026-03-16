use crate::error::TauriError;
use nozy::{load_config, save_config, ZebraClient};
use serde::{Deserialize, Serialize};
use tauri::command;

#[derive(Debug, Serialize)]
pub struct ConfigResponse {
    pub zebra_url: String,
    pub last_scan_height: Option<u32>,
    pub theme: Option<String>,
}

#[command]
pub async fn get_config() -> Result<ConfigResponse, TauriError> {
    let config = load_config();
    
    Ok(ConfigResponse {
        zebra_url: config.zebra_url,
        last_scan_height: config.last_scan_height,
        theme: None, 
    })
}

#[derive(Debug, Deserialize)]
pub struct SetZebraUrlRequest {
    pub url: String,
}

#[command]
pub async fn set_zebra_url(
    request: SetZebraUrlRequest,
) -> Result<(), TauriError> {
    let mut config = load_config();
    config.zebra_url = request.url.clone();
    save_config(&config)
        .map_err(|e| TauriError::from(e.to_string()))?;
    
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct TestZebraConnectionRequest {
    pub zebra_url: Option<String>,
}

#[command]
pub async fn test_zebra_connection(
    request: TestZebraConnectionRequest,
) -> Result<String, TauriError> {
    let config = load_config();
    let zebra_url = request.zebra_url.unwrap_or_else(|| config.zebra_url.clone());
    
    let zebra_client = ZebraClient::new(zebra_url.clone());
    
    match zebra_client.get_block_count().await {
        Ok(block_count) => Ok(format!("✅ Connected to Zebra at {}. Current block height: {}", zebra_url, block_count)),
        Err(e) => Err(TauriError {
            message: format!("Failed to connect to Zebra at {}: {}", zebra_url, e),
            code: Some("CONNECTION_FAILED".to_string()),
        }),
    }
}

