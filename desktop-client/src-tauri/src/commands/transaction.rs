use crate::error::TauriError;
use nozy::{HDWallet, WalletStorage, ZebraClient, ZcashTransactionBuilder, load_config, estimate_transaction_fee, scan_notes_for_sending, transaction_history::{SentTransactionStorage, SentTransactionRecord}};
use serde::{Deserialize, Serialize};
use tauri::command;

#[derive(Debug, Serialize)]
pub struct SendTransactionResponse {
    pub success: bool,
    pub txid: Option<String>,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct SendTransactionRequest {
    pub recipient: String,
    pub amount: f64,
    pub memo: Option<String>,
    pub zebra_url: Option<String>,
    pub password: Option<String>,
}

#[command]
pub async fn send_transaction(
    request: SendTransactionRequest,
) -> Result<SendTransactionResponse, TauriError> {
    if !request.recipient.starts_with("u1") || request.recipient.len() < 78 {
        return Ok(SendTransactionResponse {
            success: false,
            txid: None,
            message: "Invalid recipient address. Must be a valid shielded address (u1...)".to_string(),
        });
    }
    
    if request.amount <= 0.0 || request.amount > 21_000_000.0 {
        return Ok(SendTransactionResponse {
            success: false,
            txid: None,
            message: "Invalid amount. Must be greater than 0 and less than 21,000,000 ZEC.".to_string(),
        });
    }
    
    let config = load_config();
    let zebra_url = request.zebra_url.unwrap_or_else(|| config.zebra_url.clone());
    
    let storage = WalletStorage::with_xdg_dir();
    let password = request.password.as_deref().unwrap_or("");
    
    let wallet = storage.load_wallet(password)
        .await
        .map_err(|e| TauriError {
            message: format!("Failed to load wallet: {}", e),
            code: Some("WALLET_NOT_FOUND".to_string()),
        })?;
    
    let spendable_notes = scan_notes_for_sending(wallet, &zebra_url)
        .await
        .map_err(|e| TauriError::from(e.to_string()))?;
    
    let amount_zatoshis = (request.amount * 100_000_000.0) as u64;
    let zebra_client = ZebraClient::new(zebra_url.clone());
    
    let fee_zatoshis = estimate_transaction_fee(&zebra_client).await;
    
    let memo_bytes = request.memo
        .as_ref()
        .map(|m| m.trim().as_bytes().to_vec())
        .filter(|b| !b.is_empty());
    
    let mut tx_builder = ZcashTransactionBuilder::new();
    tx_builder.set_zebra_url(&zebra_url);
    tx_builder.enable_mainnet_broadcast();
    
    let transaction = tx_builder
        .build_send_transaction(
            &zebra_client,
            &spendable_notes,
            &request.recipient,
            amount_zatoshis,
            fee_zatoshis,
            memo_bytes.as_deref(),
        )
        .await
        .map_err(|e| TauriError::from(e.to_string()))?;
    
    match tx_builder.broadcast_transaction(&zebra_client, &transaction).await {
        Ok(network_txid) => {
            let tx_storage = SentTransactionStorage::new()
                .map_err(|e| TauriError::from(e.to_string()))?;
            
            let spent_note_ids: Vec<String> = spendable_notes.iter()
                .map(|note| hex::encode(note.orchard_note.nullifier.to_bytes()))
                .collect();
            
            let mut tx_record = SentTransactionRecord::new(
                network_txid.clone(),
                request.recipient.clone(),
                amount_zatoshis,
                fee_zatoshis,
                memo_bytes.clone(),
                spent_note_ids,
            );
            tx_record.mark_broadcast();
            
            tx_storage.save_transaction(tx_record)
                .map_err(|e| TauriError::from(e.to_string()))?;
            
            Ok(SendTransactionResponse {
                success: true,
                txid: Some(network_txid.clone()),
                message: format!("Transaction broadcast successfully! TXID: {}", network_txid),
            })
        }
        Err(e) => Ok(SendTransactionResponse {
            success: false,
            txid: None,
            message: format!("Failed to broadcast transaction: {}", e),
        }),
    }
}

#[command]
pub async fn estimate_fee(
    zebra_url: Option<String>,
) -> Result<f64, TauriError> {
    let config = load_config();
    let zebra_url = zebra_url.unwrap_or_else(|| config.zebra_url.clone());
    let zebra_client = ZebraClient::new(zebra_url);
    
    let fee_zatoshis = estimate_transaction_fee(&zebra_client).await;
    let fee_zec = fee_zatoshis as f64 / 100_000_000.0;
    
    Ok(fee_zec)
}

#[command]
pub async fn get_transaction_history() -> Result<Vec<serde_json::Value>, TauriError> {
    use nozy::transaction_history::SentTransactionStorage;
    
    let tx_storage = SentTransactionStorage::new()
        .map_err(|e| TauriError::from(e.to_string()))?;
    
    let transactions = tx_storage.get_all_transactions();
    
    let json_transactions: Vec<serde_json::Value> = transactions
        .iter()
        .map(|tx| serde_json::json!({
            "txid": tx.txid.clone(),
            "recipient": tx.recipient_address.clone(),
            "amount_zec": tx.amount_zatoshis as f64 / 100_000_000.0,
            "fee_zec": tx.fee_zatoshis as f64 / 100_000_000.0,
            "memo": tx.memo.as_ref().and_then(|m| String::from_utf8(m.clone()).ok()),
            "broadcast": tx.broadcast_at.is_some(),
            "confirmations": tx.confirmations,
            "timestamp": tx.created_at.timestamp(),
        }))
        .collect();
    
    Ok(json_transactions)
}

#[command]
pub async fn get_transaction(
    txid: String,
) -> Result<serde_json::Value, TauriError> {
    use nozy::transaction_history::SentTransactionStorage;
    
    let tx_storage = SentTransactionStorage::new()
        .map_err(|e| TauriError::from(e.to_string()))?;
    
    let transaction = tx_storage.get_transaction(&txid)
        .ok_or_else(|| TauriError {
            message: format!("Transaction not found: {}", txid),
            code: Some("TRANSACTION_NOT_FOUND".to_string()),
        })?;
    
    Ok(serde_json::json!({
        "txid": transaction.txid.clone(),
        "recipient": transaction.recipient_address.clone(),
        "amount_zec": transaction.amount_zatoshis as f64 / 100_000_000.0,
        "fee_zec": transaction.fee_zatoshis as f64 / 100_000_000.0,
        "memo": transaction.memo.as_ref().and_then(|m| String::from_utf8(m.clone()).ok()),
        "broadcast": transaction.broadcast_at.is_some(),
        "confirmations": transaction.confirmations,
        "timestamp": transaction.created_at.timestamp(),
        "spent_notes": transaction.spent_note_ids.clone(),
    }))
}

