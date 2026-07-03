use crate::error::TauriError;
use crate::session::{load_session_wallet, verify_wallet_password};
use nozy::active_wallet_exists;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::command;

#[derive(Debug, Deserialize)]
pub struct SignMessageRequest {
    pub message: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct SignMessageResponse {
    pub signature: String,
}

#[command]
pub async fn sign_message(request: SignMessageRequest) -> Result<SignMessageResponse, TauriError> {
    if !active_wallet_exists() {
        return Err(TauriError {
            message: "No wallet found.".to_string(),
            code: Some("WALLET_NOT_FOUND".to_string()),
        });
    }

    let wallet = load_session_wallet(Some(&request.password)).await?;
    verify_wallet_password(&wallet, &request.password)?;

    // Match browser-extension wasm-core `sign_message` (seed + message SHA-256).
    let seed_bytes = wallet.get_mnemonic_object().to_seed("");
    let mut hasher = Sha256::new();
    hasher.update(&seed_bytes);
    hasher.update(request.message.as_bytes());
    let signature = hasher.finalize();

    Ok(SignMessageResponse {
        signature: hex::encode(signature),
    })
}
