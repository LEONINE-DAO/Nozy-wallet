use crate::commands::wallet::get_unlocked_wallet;
use crate::error::{TauriError, TauriResult};
use nozy::cli_helpers::{build_and_broadcast_transaction, scan_notes_for_sending};
use nozy::{load_config, ZebraClient};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SendTransactionRequest {
    pub recipient: String,
    pub amount: f64,
    pub memo: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendTransactionResponse {
    pub success: bool,
    pub txid: Option<String>,
    pub message: String,
}

#[tauri::command]
pub async fn send_transaction(
    request: SendTransactionRequest,
) -> TauriResult<SendTransactionResponse> {
    let wallet = get_unlocked_wallet()?;
    
    let config = load_config();
    let zebra_client = ZebraClient::from_config(&config);
    
    let amount_zatoshis = (request.amount * 100_000_000.0) as u64;
    
    let spendable_notes = scan_notes_for_sending(wallet.clone(), &config.zebra_url)
        .await
        .map_err(|e| TauriError::Network(format!("Failed to scan notes: {}", e)))?;
    
    if spendable_notes.is_empty() {
        return Err(TauriError::Wallet(
            "No spendable notes found. Please sync your wallet first.".to_string(),
        ));
    }
    
    let total_available: u64 = spendable_notes.iter().map(|note| note.orchard_note.value).sum();
    
    if total_available < amount_zatoshis {
        return Err(TauriError::Wallet(format!(
            "Insufficient balance. Available: {:.8} ZEC, Required: {:.8} ZEC",
            total_available as f64 / 100_000_000.0,
            request.amount
        )));
    }
    
    let memo_bytes = request.memo.as_ref().map(|m| m.as_bytes().to_vec());
    
    match build_and_broadcast_transaction(
        &zebra_client,
        &spendable_notes,
        &request.recipient,
        amount_zatoshis,
        None, 
        memo_bytes.as_deref(),
        true, 
        &config.zebra_url,
    )
    .await
    {
        Ok(txid) => {
            Ok(SendTransactionResponse {
                success: true,
                txid: Some(txid),
                message: "Transaction broadcast successfully!".to_string(),
            })
        }
        Err(e) => Err(TauriError::InvalidOperation(format!("Failed to send transaction: {}", e))),
    }
}

#[tauri::command]
pub async fn estimate_fee(zebra_url: Option<String>) -> TauriResult<f64> {
    let mut config = load_config();
    if let Some(url) = zebra_url {
        config.zebra_url = url;
    }
    
    let zebra_client = ZebraClient::from_config(&config);
    
    let fee_zatoshis = nozy::cli_helpers::estimate_transaction_fee(&zebra_client).await;
    let fee_zec = fee_zatoshis as f64 / 100_000_000.0;
    
    Ok(fee_zec)
}

#[tauri::command]
pub async fn get_transaction_history() -> TauriResult<Vec<serde_json::Value>> {
    // TODO: Implement transaction history retrieval
    // This would need to read from transaction_history storage
    Ok(Vec::new())
}

#[tauri::command]
pub async fn get_transaction(_txid: String) -> TauriResult<serde_json::Value> {
    // TODO: Implement transaction retrieval
    // This would query Zebra for transaction details
    Err(TauriError::InvalidOperation("Not yet implemented".to_string()))
}
