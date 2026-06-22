use crate::error::TauriError;
use nozy::{
    estimate_transaction_fee_for_send, load_config, scan_notes_for_sending,
    transaction_history::{
        SentTransactionRecord, SentTransactionStorage, TransactionStatus,
    },
    WalletStorage, ZcashTransactionBuilder, ZebraClient,
};
use serde::{Deserialize, Serialize};
use tauri::command;

fn status_key(status: &TransactionStatus) -> &'static str {
    match status {
        TransactionStatus::Pending => "pending",
        TransactionStatus::Confirmed => "confirmed",
        TransactionStatus::Failed => "failed",
        TransactionStatus::Expired => "expired",
    }
}

fn tx_record_json(tx: &SentTransactionRecord) -> serde_json::Value {
    serde_json::json!({
        "txid": tx.txid,
        "recipient_address": tx.recipient_address,
        "recipient": tx.recipient_address,
        "amount_zatoshis": tx.amount_zatoshis,
        "amount_zec": tx.amount_zatoshis as f64 / 100_000_000.0,
        "fee_zatoshis": tx.fee_zatoshis,
        "fee_zec": tx.fee_zatoshis as f64 / 100_000_000.0,
        "memo": tx.memo.as_ref().and_then(|m| String::from_utf8(m.clone()).ok()),
        "status": status_key(&tx.status),
        "transaction_type": "sent",
        "type": "sent",
        "priority": tx.priority,
        "expiry_height": tx.expiry_height,
        "block_height": tx.block_height,
        "confirmations": tx.confirmations,
        "broadcast_at": tx.broadcast_at.map(|d| d.to_rfc3339()),
        "created_at": tx.created_at.to_rfc3339(),
        "timestamp": tx.created_at.timestamp(),
        "broadcast": tx.broadcast_at.is_some(),
        "spent_note_ids": tx.spent_note_ids,
        "speed_up_of_txid": tx.speed_up_of_txid,
    })
}

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
    #[serde(default)]
    pub priority: bool,
}

#[command]
pub async fn send_transaction(
    request: SendTransactionRequest,
) -> Result<SendTransactionResponse, TauriError> {
    if !request.recipient.starts_with("u1") || request.recipient.len() < 78 {
        return Ok(SendTransactionResponse {
            success: false,
            txid: None,
            message: "Invalid recipient address. Must be a valid shielded address (u1...)"
                .to_string(),
        });
    }

    if request.amount <= 0.0 || request.amount > 21_000_000.0 {
        return Ok(SendTransactionResponse {
            success: false,
            txid: None,
            message: "Invalid amount. Must be greater than 0 and less than 21,000,000 ZEC."
                .to_string(),
        });
    }

    let config = load_config();
    let zebra_url = request
        .zebra_url
        .unwrap_or_else(|| config.zebra_url.clone());

    let storage = WalletStorage::with_xdg_dir();
    let password = request.password.as_deref().unwrap_or("");

    let wallet = storage
        .load_wallet(password)
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

    let pilot = nozy::PilotSendOptions {
        priority: request.priority,
        expiry_delta_blocks: nozy::PILOT_EXPIRY_DELTA_BLOCKS,
    };
    let memo_preview = request
        .memo
        .as_ref()
        .map(|m| m.trim().as_bytes())
        .filter(|b| !b.is_empty());
    let fee_zatoshis =
        estimate_transaction_fee_for_send(&zebra_client, memo_preview, pilot.priority).await;

    let memo_bytes = request
        .memo
        .as_ref()
        .map(|m| m.trim().as_bytes().to_vec())
        .filter(|b| !b.is_empty());

    let mut tx_builder = ZcashTransactionBuilder::new();
    tx_builder.set_zebra_url(&zebra_url);
    tx_builder.enable_mainnet_broadcast();

    match tx_builder
        .build_and_broadcast_send_transaction(
            &zebra_client,
            &spendable_notes,
            &request.recipient,
            amount_zatoshis,
            fee_zatoshis,
            memo_bytes.as_deref(),
            pilot,
        )
        .await
    {
        Ok(transaction) => {
            let tx_storage =
                SentTransactionStorage::new().map_err(|e| TauriError::from(e.to_string()))?;

            let spent_note_ids: Vec<String> = spendable_notes
                .iter()
                .map(|note| hex::encode(note.orchard_note.nullifier.to_bytes()))
                .collect();

            let mut tx_record = SentTransactionRecord::new_pilot(
                transaction.txid.clone(),
                request.recipient.clone(),
                amount_zatoshis,
                fee_zatoshis,
                memo_bytes.clone(),
                spent_note_ids,
                pilot.priority,
                transaction.expiry_height,
            );
            tx_record.mark_broadcast();

            tx_storage
                .save_transaction(tx_record)
                .map_err(|e| TauriError::from(e.to_string()))?;

            Ok(SendTransactionResponse {
                success: true,
                txid: Some(transaction.txid.clone()),
                message: format!(
                    "Transaction broadcast successfully! TXID: {}",
                    transaction.txid
                ),
            })
        }
        Err(e) => Ok(SendTransactionResponse {
            success: false,
            txid: None,
            message: format!("Failed to send transaction: {}", e),
        }),
    }
}

#[command]
pub async fn estimate_fee(
    zebra_url: Option<String>,
    priority: Option<bool>,
) -> Result<f64, TauriError> {
    let config = load_config();
    let zebra_url = zebra_url.unwrap_or_else(|| config.zebra_url.clone());
    let zebra_client = ZebraClient::new(zebra_url);
    let priority = priority.unwrap_or(false);

    let fee_zatoshis = estimate_transaction_fee_for_send(&zebra_client, None, priority).await;
    let fee_zec = fee_zatoshis as f64 / 100_000_000.0;

    Ok(fee_zec)
}

#[command]
pub async fn get_transaction_history() -> Result<Vec<serde_json::Value>, TauriError> {
    use nozy::transaction_history::{
        collect_wallet_transaction_views, transaction_view_to_history_json,
    };

    let views = collect_wallet_transaction_views(0).map_err(|e| TauriError::from(e.to_string()))?;
    Ok(views
        .iter()
        .map(transaction_view_to_history_json)
        .collect())
}

#[command]
pub async fn get_transaction(txid: String) -> Result<serde_json::Value, TauriError> {
    use nozy::transaction_history::SentTransactionStorage;

    let tx_storage = SentTransactionStorage::new().map_err(|e| TauriError::from(e.to_string()))?;

    let transaction = tx_storage
        .get_transaction(&txid)
        .ok_or_else(|| TauriError {
            message: format!("Transaction not found: {}", txid),
            code: Some("TRANSACTION_NOT_FOUND".to_string()),
        })?;

    Ok(tx_record_json(&transaction))
}

#[derive(Debug, Deserialize)]
pub struct SpeedUpTransactionRequest {
    pub original_txid: String,
    pub password: Option<String>,
    pub zebra_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SpeedUpTransactionResponse {
    pub success: bool,
    pub txid: Option<String>,
    pub original_txid: String,
    pub message: String,
}

#[command]
pub async fn speed_up_transaction(
    request: SpeedUpTransactionRequest,
) -> Result<SpeedUpTransactionResponse, TauriError> {
    let config = load_config();
    let zebra_url = request
        .zebra_url
        .unwrap_or_else(|| config.zebra_url.clone());

    let storage = WalletStorage::with_xdg_dir();
    let password = request.password.as_deref().unwrap_or("");

    let wallet = storage
        .load_wallet(password)
        .await
        .map_err(|e| TauriError {
            message: format!("Failed to load wallet: {}", e),
            code: Some("WALLET_NOT_FOUND".to_string()),
        })?;

    match nozy::speed_up_transaction(wallet, &zebra_url, &request.original_txid).await {
        Ok(new_txid) => Ok(SpeedUpTransactionResponse {
            success: true,
            txid: Some(new_txid.clone()),
            original_txid: request.original_txid.clone(),
            message: format!(
                "Speed-up transaction broadcast. New TXID: {} (replaces {})",
                new_txid, request.original_txid
            ),
        }),
        Err(e) => Ok(SpeedUpTransactionResponse {
            success: false,
            txid: None,
            original_txid: request.original_txid.clone(),
            message: e.to_string(),
        }),
    }
}

#[derive(Debug, Serialize)]
pub struct CheckConfirmationsResponse {
    pub pending_updated: usize,
    pub expired_updated: usize,
    pub confirmations_updated: usize,
}

#[command]
pub async fn check_transaction_confirmations(
    zebra_url: Option<String>,
) -> Result<CheckConfirmationsResponse, TauriError> {
    let config = load_config();
    let zebra_url = zebra_url.unwrap_or_else(|| config.zebra_url.clone());
    let zebra_client = ZebraClient::new(zebra_url);

    let tx_storage = SentTransactionStorage::new().map_err(|e| TauriError::from(e.to_string()))?;

    let pending_updated = tx_storage
        .check_all_pending_transactions(&zebra_client)
        .await
        .unwrap_or(0);
    let expired_updated = tx_storage
        .check_expired_pending_transactions(&zebra_client)
        .await
        .unwrap_or(0);
    let confirmations_updated = tx_storage
        .update_confirmations(&zebra_client)
        .await
        .unwrap_or(0);

    Ok(CheckConfirmationsResponse {
        pending_updated,
        expired_updated,
        confirmations_updated,
    })
}
