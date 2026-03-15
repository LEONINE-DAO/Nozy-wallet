use crate::commands::wallet::get_unlocked_wallet;
use crate::error::{TauriError, TauriResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateAddressResponse {
    pub address: String,
}

#[tauri::command]
pub async fn generate_address() -> TauriResult<GenerateAddressResponse> {
    let wallet = get_unlocked_wallet()?;
    
    let address = wallet
        .generate_orchard_address(0, 0)
        .map_err(|e| TauriError::Wallet(format!("Failed to generate address: {}", e)))?;

    Ok(GenerateAddressResponse { address })
}
