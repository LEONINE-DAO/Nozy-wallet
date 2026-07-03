use crate::error::TauriError;
use crate::session::load_session_wallet;
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
    let wallet = load_session_wallet(request.password.as_deref())
        .await
        .map_err(|e| TauriError {
            message: e.message,
            code: e.code,
        })?;

    let account = request.account.unwrap_or(0);
    let index = request.index.unwrap_or(0);

    let address = wallet
        .generate_orchard_address(account, index, crate::network_from_config())
        .map_err(|e| TauriError::from(e.to_string()))?;

    Ok(AddressResponse { address })
}
