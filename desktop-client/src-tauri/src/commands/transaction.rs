use crate::error::TauriError;
use crate::session::load_session_wallet;
use nozy::{
    estimate_transaction_fee_for_send, load_config, mark_wallet_notes_spent_by_nullifier_hex,
    mark_wallet_notes_spent_from_spendables, scan_notes_for_sending, select_single_spend_note,
    sync_wallet_notes, WalletSyncOptions,
    transaction_history::{
        SentTransactionRecord, SentTransactionStorage, TransactionStatus,
    },
    ZcashTransactionBuilder, ZebraClient, load_wallet_notes,
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, command};

#[derive(Clone, Serialize)]
struct SendProgressEvent {
    stage: String,
    percent: u8,
    message: String,
}

fn emit_send_progress(app: &AppHandle, stage: &str, percent: u8, message: &str) {
    let _ = app.emit(
        "send-progress",
        SendProgressEvent {
            stage: stage.to_string(),
            percent,
            message: message.to_string(),
        },
    );
}

async fn repair_ironwood_witnesses_by_rescan(
    password: Option<&str>,
    zebra_url: &str,
) -> Result<bool, TauriError> {
    let notes = load_wallet_notes().map_err(|e| TauriError::from(e.to_string()))?;
    let min_ironwood_unspent = notes
        .iter()
        .filter(|n| !n.spent && matches!(n.pool, nozy::shielded_pool::ShieldedPool::Ironwood))
        .map(|n| n.block_height)
        .min();
    let Some(start_height) = min_ironwood_unspent.map(|h| h.saturating_sub(1)) else {
        return Ok(false);
    };

    let wallet = load_session_wallet(password)
        .await
        .map_err(|e| TauriError {
            message: e.message,
            code: e.code,
        })?;
    let sync_result = sync_wallet_notes(
        wallet,
        WalletSyncOptions {
            start_height: Some(start_height),
            end_height: None,
            scan_to_tip: true,
            zebra_url: Some(zebra_url.to_string()),
            ..WalletSyncOptions::default()
        },
    )
    .await;
    Ok(sync_result.is_ok())
}

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
    app: AppHandle,
    request: SendTransactionRequest,
) -> Result<SendTransactionResponse, TauriError> {
    emit_send_progress(
        &app,
        "Validating",
        5,
        "Checking recipient address and amount…",
    );

    let config = load_config();
    let recipient = nozy::input_validation::normalize_unified_address(&request.recipient);
    let expected_testnet = config.network.eq_ignore_ascii_case("testnet");
    let prefix_ok = if expected_testnet {
        recipient.starts_with("utest1")
    } else {
        recipient.starts_with("u1")
    };
    if !prefix_ok || recipient.len() < 78 {
        return Ok(SendTransactionResponse {
            success: false,
            txid: None,
            message: if expected_testnet {
                "Invalid recipient address. Testnet sends require a valid Orchard unified address (utest1...)."
                    .to_string()
            } else {
                "Invalid recipient address. Mainnet sends require a valid Orchard unified address (u1...)."
                    .to_string()
            },
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

    let zebra_url = request
        .zebra_url
        .unwrap_or_else(|| config.zebra_url.clone());

    emit_send_progress(&app, "Unlocking wallet", 15, "Loading wallet keys…");

    let wallet = load_session_wallet(request.password.as_deref())
        .await
        .map_err(|e| TauriError {
            message: e.message,
            code: e.code,
        })?;

    emit_send_progress(
        &app,
        "Selecting notes",
        30,
        "Gathering spendable Orchard notes…",
    );

    let spendable_notes = scan_notes_for_sending(wallet, &zebra_url)
        .await
        .map_err(|e| TauriError::from(e.to_string()))?;

    let amount_zatoshis = (request.amount * 100_000_000.0) as u64;
    let zebra_client = ZebraClient::new(zebra_url.clone());

    let pilot = nozy::PilotSendOptions {
        priority: request.priority | nozy::NOZY_WALLET_PRIORITY_FEE,
        expiry_delta_blocks: nozy::PILOT_EXPIRY_DELTA_BLOCKS,
    };
    let memo_preview = request
        .memo
        .as_ref()
        .map(|m| m.trim().as_bytes())
        .filter(|b| !b.is_empty());

    emit_send_progress(&app, "Estimating fee", 42, "Calculating ZIP-317 priority fee…");

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

    emit_send_progress(
        &app,
        "Building proof",
        58,
        "Generating zero-knowledge proof — this can take several minutes…",
    );

    let send_result = tx_builder
        .build_and_broadcast_send_transaction(
            &zebra_client,
            &spendable_notes,
            &recipient,
            amount_zatoshis,
            fee_zatoshis,
            memo_bytes.as_deref(),
            pilot,
        )
        .await;

    let send_result = if let Err(e) = &send_result {
        if e.to_string()
            .contains("Ironwood witness does not match z_gettreestate")
            && repair_ironwood_witnesses_by_rescan(request.password.as_deref(), &zebra_url)
                .await
                .unwrap_or(false)
        {
            emit_send_progress(
                &app,
                "Repairing witnesses",
                70,
                "Refreshing Ironwood witnesses and rebuilding the transaction…",
            );
            let wallet_retry = load_session_wallet(request.password.as_deref())
                .await
                .map_err(|e| TauriError {
                    message: e.message,
                    code: e.code,
                })?;
            let spendable_notes_retry = scan_notes_for_sending(wallet_retry, &zebra_url)
                .await
                .map_err(|e| TauriError::from(e.to_string()))?;
            emit_send_progress(
                &app,
                "Building proof",
                78,
                "Rebuilding shielded proof after witness repair…",
            );
            let retried = tx_builder
                .build_and_broadcast_send_transaction(
                    &zebra_client,
                    &spendable_notes_retry,
                    &recipient,
                    amount_zatoshis,
                    fee_zatoshis,
                    memo_bytes.as_deref(),
                    pilot,
                )
                .await;
            retried
        } else {
            send_result
        }
    } else {
        send_result
    };

    match send_result {
        Ok(transaction) => {
            emit_send_progress(
                &app,
                "Saving transaction",
                94,
                "Recording the sent transaction locally…",
            );
            let tx_storage =
                SentTransactionStorage::new().map_err(|e| TauriError::from(e.to_string()))?;

            let spent_note_ids = if let Some(nullifier_hex) = &transaction.spent_nullifier_hex {
                if let Err(e) = mark_wallet_notes_spent_by_nullifier_hex(
                    std::slice::from_ref(nullifier_hex),
                    Some(&transaction.txid),
                ) {
                    eprintln!("Warning: could not mark spent note locally: {e}");
                }
                vec![nullifier_hex.clone()]
            } else {
                let spent_note = select_single_spend_note(
                    &spendable_notes,
                    amount_zatoshis,
                    fee_zatoshis,
                )
                .map_err(|e| TauriError::from(e.to_string()))?;

                if let Err(e) = mark_wallet_notes_spent_from_spendables(
                    std::slice::from_ref(spent_note),
                    Some(&transaction.txid),
                ) {
                    eprintln!("Warning: could not mark spent note locally: {e}");
                }

                vec![hex::encode(spent_note.orchard_note.nullifier.to_bytes())]
            };

            let mut tx_record = SentTransactionRecord::new_pilot(
                transaction.txid.clone(),
                recipient.clone(),
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

            emit_send_progress(
                &app,
                "Complete",
                100,
                "Transaction broadcast successfully.",
            );

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
    let use_priority = priority.unwrap_or(nozy::NOZY_WALLET_PRIORITY_FEE);
    let fee_zatoshis =
        estimate_transaction_fee_for_send(&zebra_client, None, use_priority).await;
    let fee_zec = fee_zatoshis as f64 / 100_000_000.0;

    Ok(fee_zec)
}

#[command]
pub async fn get_transaction_history() -> Result<Vec<serde_json::Value>, TauriError> {
    use nozy::transaction_history::{
        collect_wallet_transaction_views, enrich_block_times_for_views,
        transaction_view_to_history_json,
    };

    let config = load_config();
    let current_height = config.last_scan_height.unwrap_or(0);
    let mut views = collect_wallet_transaction_views(current_height)
        .map_err(|e| TauriError::from(e.to_string()))?;

    let zebra_client = ZebraClient::new(config.zebra_url);
    enrich_block_times_for_views(&mut views, &zebra_client).await;

    Ok(views
        .iter()
        .map(transaction_view_to_history_json)
        .collect())
}

#[command]
pub async fn get_transaction(txid: String) -> Result<serde_json::Value, TauriError> {
    use nozy::transaction_history::{
        transaction_view_for_txid, transaction_view_to_history_json, SentTransactionStorage,
    };

    let config = load_config();
    let current_height = config.last_scan_height.unwrap_or(0);

    if let Some(view) = transaction_view_for_txid(&txid, current_height)
        .map_err(|e| TauriError::from(e.to_string()))?
    {
        return Ok(transaction_view_to_history_json(&view));
    }

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

    let wallet = load_session_wallet(request.password.as_deref())
        .await
        .map_err(|e| TauriError {
            message: e.message,
            code: e.code,
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
