use axum::{
    extract::{Json, Path, Query},
    response::Json as ResponseJson,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(dead_code)]
pub fn error_response(
    status: StatusCode,
    message: impl Into<String>,
) -> (StatusCode, ResponseJson<serde_json::Value>) {
    (
        status,
        ResponseJson(serde_json::json!({
            "error": message.into()
        })),
    )
}

pub fn error_response_with_code(
    status: StatusCode,
    message: impl Into<String>,
    code: impl Into<String>,
) -> (StatusCode, ResponseJson<serde_json::Value>) {
    (
        status,
        ResponseJson(serde_json::json!({
            "error": message.into(),
            "code": code.into()
        })),
    )
}

#[allow(dead_code)]
pub fn error_response_with_details(
    status: StatusCode,
    message: impl Into<String>,
    details: serde_json::Value,
) -> (StatusCode, ResponseJson<serde_json::Value>) {
    (
        status,
        ResponseJson(serde_json::json!({
            "error": message.into(),
            "details": details
        })),
    )
}

fn validate_address(address: &str) -> bool {
    address.starts_with("u1") && address.len() >= 78 && address.len() <= 100
}

fn validate_amount(amount: f64) -> bool {
    amount > 0.0 && amount <= 21_000_000.0 
}

fn validate_mnemonic(mnemonic: &str) -> bool {
    let words: Vec<&str> = mnemonic.split_whitespace().collect();
    matches!(words.len(), 12 | 15 | 18 | 21 | 24)
}

fn validate_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}

fn validate_theme(theme: &str) -> bool {
    theme == "dark" || theme == "light"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletInfo {
    pub exists: bool,
    pub has_password: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddressResponse {
    pub address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceResponse {
    pub balance_zec: f64,
    pub balance_zatoshis: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncResponse {
    pub success: bool,
    pub balance_zec: f64,
    pub notes_found: usize,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendTransactionRequest {
    pub recipient: String,
    pub amount: f64,
    pub memo: Option<String>,
    pub zebra_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendTransactionResponse {
    pub success: bool,
    pub txid: Option<String>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigResponse {
    pub zebra_url: String,
    pub network: String,
    pub last_scan_height: Option<u32>,
    pub theme: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProvingStatusResponse {
    pub spend_params: bool,
    pub output_params: bool,
    pub spend_vk: bool,
    pub output_vk: bool,
    pub can_prove: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateWalletRequest {
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RestoreWalletRequest {
    pub mnemonic: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct UnlockWalletRequest {
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct GenerateAddressRequest {
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SyncRequest {
    pub start_height: Option<u32>,
    pub end_height: Option<u32>,
    pub zebra_url: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SendTransactionRequestWrapper {
    #[serde(flatten)]
    pub request: SendTransactionRequest,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SetZebraUrlRequest {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct SetThemeRequest {
    pub theme: String,
}

#[derive(Debug, Deserialize)]
pub struct TestZebraRequest {
    pub zebra_url: Option<String>,
}

async fn load_wallet_with_password(
    password: Option<String>,
) -> Result<(nozy::HDWallet, nozy::WalletStorage), String> {
    use nozy::paths::get_wallet_data_dir;
    let storage = nozy::WalletStorage::with_xdg_dir();
    let wallet_path = get_wallet_data_dir().join("wallet.dat");

    if !wallet_path.exists() {
        return Err("No wallet found. Please create or restore a wallet first.".to_string());
    }

    let pwd = password.unwrap_or_else(|| "".to_string());
    let wallet = storage
        .load_wallet(&pwd)
        .await
        .map_err(|e| format!("Failed to load wallet: {}. Please check your password.", e))?;

    Ok((wallet, storage))
}

pub async fn check_wallet_exists() -> ResponseJson<WalletInfo> {
    use nozy::paths::get_wallet_data_dir;
    let wallet_path = get_wallet_data_dir().join("wallet.dat");
    let exists = wallet_path.exists();

    let has_password = if exists {
        let storage = nozy::WalletStorage::with_xdg_dir();
        storage.load_wallet("").await.is_err()
    } else {
        false
    };

    ResponseJson(WalletInfo {
        exists,
        has_password,
    })
}

pub async fn create_wallet(
    Json(payload): Json<CreateWalletRequest>,
) -> Result<ResponseJson<String>, (StatusCode, ResponseJson<serde_json::Value>)> {
    use nozy::paths::get_wallet_data_dir;
    let wallet_path = get_wallet_data_dir().join("wallet.dat");
    if wallet_path.exists() {
        return Err(error_response_with_code(
            StatusCode::BAD_REQUEST,
            "A wallet already exists! To create a new wallet, please delete the existing one first or restore from your seed phrase.",
            "WALLET_EXISTS",
        ));
    }

    let mut wallet = nozy::HDWallet::new()
        .map_err(|e| {
            error_response_with_code(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to create wallet: {}", e),
                "WALLET_CREATE_FAILED",
            )
        })?;

    let password_for_save = payload.password.clone();
    
    if let Some(ref pwd) = payload.password {
        wallet.set_password(pwd).map_err(|e| {
            error_response_with_code(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to set password: {}", e),
                "PASSWORD_SET_FAILED",
            )
        })?;
    }

    let storage = nozy::WalletStorage::with_xdg_dir();
    storage
        .save_wallet(&wallet, password_for_save.as_deref().unwrap_or(""))
        .await
        .map_err(|e| {
            error_response_with_code(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to save wallet: {}", e),
                "WALLET_SAVE_FAILED",
            )
        })?;

    // SECURITY: Never return full mnemonic in API responses
    // Only return masked version for security
    // Note: Consider removing mnemonic endpoints entirely for production
    use nozy::safe_display::display_mnemonic_safe;
    Ok(ResponseJson(display_mnemonic_safe(&wallet.get_mnemonic())))
}

pub async fn restore_wallet(
    Json(payload): Json<RestoreWalletRequest>,
) -> Result<ResponseJson<serde_json::Value>, (StatusCode, ResponseJson<serde_json::Value>)> {
    if !validate_mnemonic(&payload.mnemonic) {
        return Err(error_response_with_code(
            StatusCode::BAD_REQUEST,
            "Invalid mnemonic format. Must be 12, 15, 18, 21, or 24 words.",
            "INVALID_MNEMONIC",
        ));
    }
    
    if payload.password.len() > 256 {
        return Err((
            StatusCode::BAD_REQUEST,
            ResponseJson(serde_json::json!({
                "error": "Password is too long (max 256 characters)."
            })),
        ));
    }
    
    let wallet = nozy::HDWallet::from_mnemonic(&payload.mnemonic).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            ResponseJson(serde_json::json!({
                "error": format!("Invalid mnemonic: {}", e)
            })),
        )
    })?;

    let storage = nozy::WalletStorage::with_xdg_dir();
    storage
        .save_wallet(&wallet, &payload.password)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(serde_json::json!({
                    "error": format!("Failed to save wallet: {}", e)
                })),
            )
        })?;

    Ok(ResponseJson(serde_json::json!({"success": true})))
}

pub async fn unlock_wallet(
    Json(payload): Json<UnlockWalletRequest>,
) -> Result<ResponseJson<String>, (StatusCode, ResponseJson<serde_json::Value>)> {
    let (wallet, _storage) = load_wallet_with_password(Some(payload.password))
        .await
        .map_err(|e| {
            (
                StatusCode::UNAUTHORIZED,
                ResponseJson(serde_json::json!({
                    "error": e
                })),
            )
        })?;

    // SECURITY: Never return full mnemonic in API responses
    // Only return masked version for security
    // Note: Consider removing mnemonic endpoints entirely for production
    use nozy::safe_display::display_mnemonic_safe;
    Ok(ResponseJson(display_mnemonic_safe(&wallet.get_mnemonic())))
}

pub async fn generate_address(
    Json(payload): Json<GenerateAddressRequest>,
) -> Result<ResponseJson<AddressResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    let (wallet, _storage) = load_wallet_with_password(payload.password)
        .await
        .map_err(|e| {
            (
                StatusCode::UNAUTHORIZED,
                ResponseJson(serde_json::json!({
                    "error": e
                })),
            )
        })?;

    let address = wallet.generate_orchard_address(0, 0).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(serde_json::json!({
                "error": format!("Failed to generate address: {}", e)
            })),
        )
    })?;

    Ok(ResponseJson(AddressResponse { address }))
}

pub async fn get_balance() -> Result<ResponseJson<BalanceResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    use std::fs;

    use nozy::paths::get_wallet_data_dir;
    let notes_path = get_wallet_data_dir().join("notes.json");
    if !notes_path.exists() {
        return Ok(ResponseJson(BalanceResponse {
            balance_zec: 0.0,
            balance_zatoshis: 0,
        }));
    }

    let content = fs::read_to_string(notes_path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(serde_json::json!({
                "error": format!("Failed to read notes: {}", e)
            })),
        )
    })?;

    let parsed: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(serde_json::json!({
                "error": format!("Failed to parse notes: {}", e)
            })),
        )
    })?;

    let total_zat: u64 = parsed
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|n| n.get("value").and_then(|v| v.as_u64()))
        .sum();

    Ok(ResponseJson(BalanceResponse {
        balance_zec: total_zat as f64 / 100_000_000.0,
        balance_zatoshis: total_zat,
    }))
}

pub async fn sync_wallet(
    Json(payload): Json<SyncRequest>,
) -> Result<ResponseJson<SyncResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    use nozy::{NoteScanner, ZebraClient, load_config, update_last_scan_height};

    let config = load_config();
    let zebra_url = payload.zebra_url.unwrap_or_else(|| config.zebra_url.clone());

    let (wallet, _storage) = load_wallet_with_password(payload.password)
        .await
        .map_err(|e| {
            (
                StatusCode::UNAUTHORIZED,
                ResponseJson(serde_json::json!({
                    "error": e
                })),
            )
        })?;

    let zebra_client = ZebraClient::new(zebra_url.clone());
    let effective_start = payload.start_height.or(config.last_scan_height);

    let mut note_scanner = NoteScanner::new(wallet, zebra_client.clone());

    match note_scanner.scan_notes(effective_start, payload.end_height).await {
        Ok((result, _spendable_notes)) => {
            use std::fs;
            use nozy::paths::get_wallet_data_dir;
            let notes_dir = get_wallet_data_dir();
            if !notes_dir.exists() {
                let _ = fs::create_dir_all(&notes_dir);
            }
            let notes_path = notes_dir.join("notes.json");
            if let Ok(serialized) = serde_json::to_string_pretty(&result.notes) {
                let _ = fs::write(&notes_path, serialized);
            }

            if let Some(end) = payload.end_height {
                let _ = update_last_scan_height(end);
            } else {
                if let Ok(block_count) = zebra_client.get_block_count().await {
                    let _ = update_last_scan_height(block_count);
                }
            }

            let balance_zec = result.total_balance as f64 / 100_000_000.0;

            Ok(ResponseJson(SyncResponse {
                success: true,
                balance_zec,
                notes_found: result.notes.len(),
                message: format!("Sync completed. Balance: {:.8} ZEC", balance_zec),
            }))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(serde_json::json!({
                "error": format!("Sync failed: {}", e)
            })),
        )),
    }
}

pub async fn send_transaction(
    Json(payload): Json<SendTransactionRequestWrapper>,
) -> Result<ResponseJson<SendTransactionResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    use nozy::{ZebraClient, load_config};
    use nozy::cli_helpers::scan_notes_for_sending;
    use nozy::ZcashTransactionBuilder;

    let config = load_config();
    let zebra_url = payload.request.zebra_url.unwrap_or_else(|| config.zebra_url.clone());

    let (wallet, _storage) = load_wallet_with_password(payload.password)
        .await
        .map_err(|e| {
            (
                StatusCode::UNAUTHORIZED,
                ResponseJson(serde_json::json!({
                    "error": e
                })),
            )
        })?;

    if !validate_address(&payload.request.recipient) {
        return Ok(ResponseJson(SendTransactionResponse {
            success: false,
            txid: None,
            message: "Invalid recipient address. Must be a valid shielded address (u1...)".to_string(),
        }));
    }
    
    if payload.request.recipient.starts_with("t1") {
        return Ok(ResponseJson(SendTransactionResponse {
            success: false,
            txid: None,
            message: "Transparent addresses (t1) are not supported. Please use a shielded address (u1...)".to_string(),
        }));
    }
    
    if !validate_amount(payload.request.amount) {
        return Ok(ResponseJson(SendTransactionResponse {
            success: false,
            txid: None,
            message: "Invalid amount. Must be greater than 0 and less than 21,000,000 ZEC.".to_string(),
        }));
    }

    let spendable_notes = scan_notes_for_sending(wallet, &zebra_url)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(serde_json::json!({
                    "error": format!("Failed to scan notes: {}", e)
                })),
            )
        })?;

    let memo_bytes_opt = payload
        .request
        .memo
        .as_ref()
        .map(|m| m.trim().as_bytes().to_vec())
        .filter(|b| !b.is_empty());

    let amount_zatoshis = (payload.request.amount * 100_000_000.0) as u64;
    let zebra_client = ZebraClient::new(zebra_url.clone());
    
    let fee_zatoshis = nozy::cli_helpers::estimate_transaction_fee(&zebra_client).await;

    let mut tx_builder = ZcashTransactionBuilder::new();
    tx_builder.set_zebra_url(&zebra_url);
    tx_builder.enable_mainnet_broadcast();

    let transaction = tx_builder
        .build_send_transaction(
            &zebra_client,
            &spendable_notes,
            &payload.request.recipient,
            amount_zatoshis,
            fee_zatoshis,
            memo_bytes_opt.as_deref(),
        )
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(serde_json::json!({
                    "error": format!("Failed to build transaction: {}", e)
                })),
            )
        })?;

    match tx_builder.broadcast_transaction(&zebra_client, &transaction).await {
        Ok(network_txid) => {
            use nozy::transaction_history::{SentTransactionStorage, SentTransactionRecord};
            if let Ok(tx_storage) = SentTransactionStorage::new() {
                let spent_note_ids: Vec<String> = spendable_notes.iter()
                    .map(|note| hex::encode(note.orchard_note.nullifier.to_bytes()))
                    .collect();
                
                let mut tx_record = SentTransactionRecord::new(
                    network_txid.clone(),
                    payload.request.recipient.clone(),
                    amount_zatoshis,
                    fee_zatoshis,
                    memo_bytes_opt.clone(),
                    spent_note_ids,
                );
                tx_record.mark_broadcast();
                let _ = tx_storage.save_transaction(tx_record);
            }
            
            Ok(ResponseJson(SendTransactionResponse {
                success: true,
                txid: Some(network_txid.clone()),
                message: format!("Transaction sent successfully! TXID: {}", network_txid),
            }))
        },
        Err(e) => Ok(ResponseJson(SendTransactionResponse {
            success: false,
            txid: Some(transaction.txid.clone()), 
            message: format!("Failed to broadcast transaction: {}", e),
        })),
    }
}

pub async fn estimate_fee() -> Result<ResponseJson<serde_json::Value>, (StatusCode, ResponseJson<serde_json::Value>)> {
    use nozy::{ZebraClient, load_config};
    
    let config = load_config();
    let zebra_client = ZebraClient::new(config.zebra_url);
    
    let fee_zatoshis = nozy::cli_helpers::estimate_transaction_fee(&zebra_client).await;
    let fee_zec = fee_zatoshis as f64 / 100_000_000.0;
    
    Ok(ResponseJson(serde_json::json!({
        "fee_zatoshis": fee_zatoshis,
        "fee_zec": fee_zec,
        "estimated_at": chrono::Utc::now().to_rfc3339()
    })))
}

pub async fn get_config() -> Result<ResponseJson<ConfigResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    use nozy::load_config;

    let config = load_config();
    Ok(ResponseJson(ConfigResponse {
        zebra_url: config.zebra_url,
        network: config.network,
        last_scan_height: config.last_scan_height,
        theme: config.theme,
    }))
}

pub async fn set_zebra_url(
    Json(payload): Json<SetZebraUrlRequest>,
) -> Result<ResponseJson<serde_json::Value>, (StatusCode, ResponseJson<serde_json::Value>)> {
    use nozy::{load_config, save_config};

    if !validate_url(&payload.url) {
        return Err(error_response_with_code(
            StatusCode::BAD_REQUEST,
            "Invalid URL format. Must start with http:// or https://",
            "INVALID_URL",
        ));
    }
    
    if payload.url.len() > 2048 {
        return Err(error_response_with_code(
            StatusCode::BAD_REQUEST,
            "URL is too long (max 2048 characters)",
            "URL_TOO_LONG",
        ));
    }

    let mut config = load_config();
    config.zebra_url = payload.url;
    save_config(&config).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(serde_json::json!({
                "error": format!("Failed to save config: {}", e)
            })),
        )
    })?;

    Ok(ResponseJson(serde_json::json!({"success": true})))
}

pub async fn set_theme(
    Json(payload): Json<SetThemeRequest>,
) -> Result<ResponseJson<serde_json::Value>, (StatusCode, ResponseJson<serde_json::Value>)> {
    use nozy::{load_config, save_config};

    if !validate_theme(&payload.theme) {
        return Err(error_response_with_code(
            StatusCode::BAD_REQUEST,
            "Theme must be 'dark' or 'light'",
            "INVALID_THEME",
        ));
    }

    let mut config = load_config();
    config.theme = payload.theme;
    save_config(&config).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(serde_json::json!({
                "error": format!("Failed to save theme: {}", e)
            })),
        )
    })?;

    Ok(ResponseJson(serde_json::json!({"success": true})))
}

pub async fn test_zebra_connection(
    Json(payload): Json<TestZebraRequest>,
) -> Result<ResponseJson<String>, (StatusCode, ResponseJson<serde_json::Value>)> {
    use nozy::{load_config, ZebraClient};

    let config = load_config();
    let url = payload.zebra_url.unwrap_or_else(|| config.zebra_url.clone());

    let client = ZebraClient::new(url.clone());
    match client.get_block_count().await {
        Ok(block_count) => Ok(ResponseJson(format!(
            "✅ Connected to Zebra node at {}\nBlock height: {}",
            url, block_count
        ))),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            ResponseJson(serde_json::json!({
                "error": format!("❌ Failed to connect: {}", e)
            })),
        )),
    }
}

pub async fn check_proving_status() -> Result<ResponseJson<ProvingStatusResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    use nozy::orchard_tx::OrchardTransactionBuilder;

    let builder = OrchardTransactionBuilder::new_async(false)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(serde_json::json!({
                    "error": format!("Failed to initialize builder: {}", e)
                })),
            )
        })?;

    let status = builder.get_proving_status();

    Ok(ResponseJson(ProvingStatusResponse {
        spend_params: status.spend_params,
        output_params: status.output_params,
        spend_vk: status.spend_vk,
        output_vk: status.output_vk,
        can_prove: status.can_prove,
    }))
}

pub async fn download_proving_parameters() -> Result<ResponseJson<String>, (StatusCode, ResponseJson<serde_json::Value>)> {
    use nozy::orchard_tx::OrchardTransactionBuilder;

    let mut builder = OrchardTransactionBuilder::new_async(true)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(serde_json::json!({
                    "error": format!("Failed to initialize builder: {}", e)
                })),
            )
        })?;

    builder.download_parameters().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(serde_json::json!({
                "error": format!("Failed to download parameters: {}", e)
            })),
        )
    })?;

    Ok(ResponseJson("✅ Proving parameters downloaded successfully!".to_string()))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionHistoryResponse {
    pub transactions: Vec<serde_json::Value>,
    pub total: usize,
}

#[derive(Debug, Deserialize)]
pub struct TransactionQueryParams {
    pub status: Option<String>,
    pub min_amount: Option<f64>,
    pub max_amount: Option<f64>,
    pub recipient: Option<String>,
}

pub async fn get_transaction_history(
    Query(params): Query<TransactionQueryParams>,
) -> Result<ResponseJson<TransactionHistoryResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    use nozy::transaction_history::SentTransactionStorage;
    
    let tx_storage = SentTransactionStorage::new()
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(serde_json::json!({
                    "error": format!("Failed to initialize transaction storage: {}", e)
                })),
            )
        })?;
    
    let all_txs = tx_storage.get_all_transactions();
    
    let filtered: Vec<_> = all_txs.iter()
        .filter(|tx| {
            if let Some(ref status) = params.status {
                if format!("{:?}", tx.status).to_lowercase() != status.to_lowercase() {
                    return false;
                }
            }
            if let Some(min) = params.min_amount {
                let amount_zec = tx.amount_zatoshis as f64 / 100_000_000.0;
                if amount_zec < min {
                    return false;
                }
            }
            if let Some(max) = params.max_amount {
                let amount_zec = tx.amount_zatoshis as f64 / 100_000_000.0;
                if amount_zec > max {
                    return false;
                }
            }
            if let Some(ref recipient) = params.recipient {
                if !tx.recipient_address.contains(recipient) {
                    return false;
                }
            }
            true
        })
        .map(|tx| serde_json::json!({
            "txid": tx.txid,
            "status": format!("{:?}", tx.status),
            "amount_zatoshis": tx.amount_zatoshis,
            "amount_zec": tx.amount_zatoshis as f64 / 100_000_000.0,
            "fee_zatoshis": tx.fee_zatoshis,
            "fee_zec": tx.fee_zatoshis as f64 / 100_000_000.0,
            "recipient": tx.recipient_address,
            "block_height": tx.block_height,
            "confirmations": tx.confirmations,
            "broadcast_at": tx.broadcast_at.map(|d| d.to_rfc3339()),
            "memo": tx.memo.as_ref().and_then(|m| String::from_utf8(m.clone()).ok())
        }))
        .collect();
    
    Ok(ResponseJson(TransactionHistoryResponse {
        transactions: filtered.clone(),
        total: filtered.len(),
    }))
}

pub async fn get_transaction(
    Path(txid): Path<String>,
) -> Result<ResponseJson<serde_json::Value>, (StatusCode, ResponseJson<serde_json::Value>)> {
    use nozy::transaction_history::SentTransactionStorage;
    
    let tx_storage = SentTransactionStorage::new()
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(serde_json::json!({
                    "error": format!("Failed to initialize transaction storage: {}", e)
                })),
            )
        })?;
    
    let all_txs = tx_storage.get_all_transactions();
    let tx = all_txs.iter().find(|t| t.txid == txid);
    
    match tx {
        Some(t) => Ok(ResponseJson(serde_json::json!({
            "txid": t.txid,
            "status": format!("{:?}", t.status),
            "amount_zatoshis": t.amount_zatoshis,
            "amount_zec": t.amount_zatoshis as f64 / 100_000_000.0,
            "fee_zatoshis": t.fee_zatoshis,
            "fee_zec": t.fee_zatoshis as f64 / 100_000_000.0,
            "recipient": t.recipient_address,
            "block_height": t.block_height,
            "confirmations": t.confirmations,
            "broadcast_at": t.broadcast_at.map(|d| d.to_rfc3339()),
            "memo": t.memo.as_ref().and_then(|m| String::from_utf8(m.clone()).ok()),
            "spent_note_ids": t.spent_note_ids
        }))),
        None => Err((
            StatusCode::NOT_FOUND,
            ResponseJson(serde_json::json!({
                "error": format!("Transaction {} not found", txid)
            })),
        )),
    }
}

pub async fn check_transaction_confirmations() -> Result<ResponseJson<serde_json::Value>, (StatusCode, ResponseJson<serde_json::Value>)> {
    use nozy::{ZebraClient, load_config, transaction_history::SentTransactionStorage};
    
    let config = load_config();
    let zebra_client = ZebraClient::new(config.zebra_url);
    let tx_storage = SentTransactionStorage::new()
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(serde_json::json!({
                    "error": format!("Failed to initialize transaction storage: {}", e)
                })),
            )
        })?;
    
    let updated_pending = tx_storage.check_all_pending_transactions(&zebra_client).await
        .unwrap_or(0);
    let updated_confirmations = tx_storage.update_confirmations(&zebra_client).await
        .unwrap_or(0);
    
    Ok(ResponseJson(serde_json::json!({
        "pending_updated": updated_pending,
        "confirmations_updated": updated_confirmations
    })))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddressBookEntry {
    pub name: String,
    pub address: String,
    pub created_at: String,
    pub last_used: Option<String>,
    pub usage_count: u32,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddAddressRequest {
    pub name: String,
    pub address: String,
    pub notes: Option<String>,
}

pub async fn list_address_book() -> Result<ResponseJson<Vec<AddressBookEntry>>, (StatusCode, ResponseJson<serde_json::Value>)> {
    use nozy::AddressBook;
    
    let address_book = AddressBook::new()
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(serde_json::json!({
                    "error": format!("Failed to initialize address book: {}", e)
                })),
            )
        })?;
    
    let entries: Vec<AddressBookEntry> = address_book.list_addresses()
        .iter()
        .map(|e| AddressBookEntry {
            name: e.name.clone(),
            address: e.address.clone(),
            created_at: e.created_at.to_rfc3339(),
            last_used: e.last_used.map(|d| d.to_rfc3339()),
            usage_count: e.usage_count,
            notes: e.notes.clone(),
        })
        .collect();
    
    Ok(ResponseJson(entries))
}

pub async fn add_address_book_entry(
    Json(payload): Json<AddAddressRequest>,
) -> Result<ResponseJson<serde_json::Value>, (StatusCode, ResponseJson<serde_json::Value>)> {
    use nozy::AddressBook;
    
    let address_book = AddressBook::new()
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(serde_json::json!({
                    "error": format!("Failed to initialize address book: {}", e)
                })),
            )
        })?;
    
    address_book.add_address(payload.name.clone(), payload.address.clone(), payload.notes.clone())
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                ResponseJson(serde_json::json!({
                    "error": format!("Failed to add address: {}", e)
                })),
            )
        })?;
    
    Ok(ResponseJson(serde_json::json!({
        "success": true,
        "message": format!("Address '{}' added successfully", payload.name)
    })))
}

pub async fn remove_address_book_entry(
    Path(name): Path<String>,
) -> Result<ResponseJson<serde_json::Value>, (StatusCode, ResponseJson<serde_json::Value>)> {
    use nozy::AddressBook;
    
    let address_book = AddressBook::new()
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(serde_json::json!({
                    "error": format!("Failed to initialize address book: {}", e)
                })),
            )
        })?;
    
    let removed = address_book.remove_address(&name)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(serde_json::json!({
                    "error": format!("Failed to remove address: {}", e)
                })),
            )
        })?;
    
    if removed {
        Ok(ResponseJson(serde_json::json!({
            "success": true,
            "message": format!("Address '{}' removed successfully", name)
        })))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            ResponseJson(serde_json::json!({
                "error": format!("Address '{}' not found", name)
            })),
        ))
    }
}

pub async fn search_address_book(
    Query(params): Query<HashMap<String, String>>,
) -> Result<ResponseJson<Vec<AddressBookEntry>>, (StatusCode, ResponseJson<serde_json::Value>)> {
    use nozy::AddressBook;
    
    let address_book = AddressBook::new()
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(serde_json::json!({
                    "error": format!("Failed to initialize address book: {}", e)
                })),
            )
        })?;
    
    let query = params.get("q").cloned().unwrap_or_default();
    let entries: Vec<AddressBookEntry> = address_book.search_addresses(&query)
        .iter()
        .map(|e| AddressBookEntry {
            name: e.name.clone(),
            address: e.address.clone(),
            created_at: e.created_at.to_rfc3339(),
            last_used: e.last_used.map(|d| d.to_rfc3339()),
            usage_count: e.usage_count,
            notes: e.notes.clone(),
        })
        .collect();
    
    Ok(ResponseJson(entries))
}

#[derive(Debug, Serialize)]
pub struct WalletStatusResponse {
    pub balance_zec: f64,
    pub balance_zatoshis: u64,
    pub pending_transactions: usize,
    pub total_transactions: usize,
    pub last_sync_height: Option<u32>,
    pub current_block_height: Option<u32>,
    pub blocks_behind: Option<u32>,
}

pub async fn get_wallet_status() -> Result<ResponseJson<WalletStatusResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    use nozy::{load_config, ZebraClient, transaction_history::SentTransactionStorage};
    use std::fs;
    use nozy::paths::get_wallet_data_dir;
    
    let config = load_config();
    let zebra_client = ZebraClient::new(config.zebra_url.clone());
    
    let notes_path = get_wallet_data_dir().join("notes.json");
    let balance_zatoshis = if notes_path.exists() {
        if let Ok(content) = fs::read_to_string(&notes_path) {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) {
                parsed.as_array()
                    .unwrap_or(&vec![])
                    .iter()
                    .filter_map(|n| n.get("value").and_then(|v| v.as_u64()))
                    .sum::<u64>()
            } else {
                0
            }
        } else {
            0
        }
    } else {
        0
    };
    
    let (pending_count, total_count) = if let Ok(tx_storage) = SentTransactionStorage::new() {
        let pending = tx_storage.get_pending_transactions();
        let all = tx_storage.get_all_transactions();
        (pending.len(), all.len())
    } else {
        (0, 0)
    };
    
    let current_height = zebra_client.get_block_count().await.ok();
    let blocks_behind = if let (Some(current), Some(last)) = (current_height, config.last_scan_height) {
        Some(current.saturating_sub(last))
    } else {
        None
    };
    
    Ok(ResponseJson(WalletStatusResponse {
        balance_zec: balance_zatoshis as f64 / 100_000_000.0,
        balance_zatoshis,
        pending_transactions: pending_count,
        total_transactions: total_count,
        last_sync_height: config.last_scan_height,
        current_block_height: current_height,
        blocks_behind,
    }))
}

#[derive(Debug, Serialize)]
pub struct NotesResponse {
    pub notes: Vec<serde_json::Value>,
    pub total: usize,
    pub total_balance_zatoshis: u64,
    pub total_balance_zec: f64,
}

pub async fn get_notes() -> Result<ResponseJson<NotesResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    use std::fs;
    use nozy::paths::get_wallet_data_dir;
    use nozy::SerializableOrchardNote;
    
    let notes_path = get_wallet_data_dir().join("notes.json");
    
    if !notes_path.exists() {
        return Ok(ResponseJson(NotesResponse {
            notes: vec![],
            total: 0,
            total_balance_zatoshis: 0,
            total_balance_zec: 0.0,
        }));
    }
    
    let content = fs::read_to_string(&notes_path)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(serde_json::json!({
                    "error": format!("Failed to read notes: {}", e)
                })),
            )
        })?;
    
    let notes: Vec<SerializableOrchardNote> = serde_json::from_str(&content)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(serde_json::json!({
                    "error": format!("Failed to parse notes: {}", e)
                })),
            )
        })?;
    
    let total_balance_zatoshis: u64 = notes.iter().map(|n| n.value).sum();
    let total_balance_zec = total_balance_zatoshis as f64 / 100_000_000.0;
    
    let notes_json: Vec<serde_json::Value> = notes.iter()
        .map(|n| serde_json::json!({
            "value": n.value,
            "value_zec": n.value as f64 / 100_000_000.0,
            "block_height": n.block_height,
            "txid": n.txid,
            "spent": n.spent,
            "memo": String::from_utf8(n.memo.clone()).unwrap_or_default()
        }))
        .collect();
    
    Ok(ResponseJson(NotesResponse {
        notes: notes_json,
        total: notes.len(),
        total_balance_zatoshis,
        total_balance_zec,
    }))
}

