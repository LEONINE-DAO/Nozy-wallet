use crate::error::{TauriError, TauriResult};
use nozy::{load_config, save_config, ZebraClient};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigResponse {
    pub zebra_url: String,
    pub theme: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetZebraUrlRequest {
    pub url: String,
}

#[tauri::command]
pub async fn get_config() -> TauriResult<ConfigResponse> {
    let config = load_config();
    Ok(ConfigResponse {
        zebra_url: config.zebra_url,
        theme: "dark".to_string(), // Default theme
    })
}

#[tauri::command]
pub async fn set_zebra_url(request: SetZebraUrlRequest) -> TauriResult<()> {
    let mut config = load_config();
    config.zebra_url = request.url;
    save_config(&config)
        .map_err(|e| TauriError::Config(format!("Failed to save config: {}", e)))?;
    Ok(())
}

#[tauri::command]
pub async fn test_zebra_connection(
    request: Option<serde_json::Value>,
) -> TauriResult<String> {
    let zebra_url = if let Some(req) = request {
        req.get("zebra_url")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    } else {
        None
    };

    let mut config = load_config();
    if let Some(url) = zebra_url {
        config.zebra_url = url;
    }

    let client = ZebraClient::from_config(&config);
    
    match client.get_block_count().await {
        Ok(height) => Ok(format!("✅ Connected successfully! Current block height: {}", height)),
        Err(e) => Err(TauriError::Network(format!("Failed to connect: {}", e))),
    }
}
