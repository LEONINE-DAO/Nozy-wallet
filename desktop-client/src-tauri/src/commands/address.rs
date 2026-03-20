use crate::error::TauriError;
use nozy::{HDWallet, WalletStorage};
use serde::{Deserialize, Serialize};
use tauri::command;

#[derive(Debug, Serialize)]
pub struct AddressResponse {
    pub address: String,
}

#[derive(Debug, Deserialize)]
pub struct GenerateAddressRequest {
    pub password: Option<String>,
    pub account: Option<u32>,
    pub index: Option<u32>,
}

#[command]
pub async fn generate_address(
    request: GenerateAddressRequest,
) -> Result<AddressResponse, TauriError> {
    let storage = WalletStorage::with_xdg_dir();
    let password = request.password.as_deref().unwrap_or("");
    
    let wallet = storage.load_wallet(password)
        .await
        .map_err(|e| TauriError {
            message: format!("Failed to load wallet: {}", e),
            code: Some("WALLET_NOT_FOUND".to_string()),
        })?;
    
    let account = request.account.unwrap_or(0);
    let index = request.index.unwrap_or(0);
    
    let address = wallet.generate_orchard_address(account, index, crate::network_from_config())
        .map_err(|e| TauriError::from(e.to_string()))?;
    
    Ok(AddressResponse { address })
}

