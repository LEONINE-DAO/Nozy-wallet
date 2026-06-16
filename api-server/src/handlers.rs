use axum::{
    extract::{Json, Path, Query},
    http::StatusCode,
    response::Json as ResponseJson,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;
use tokio::sync::Mutex;

static WALLET_SYNC_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn wallet_sync_lock() -> &'static Mutex<()> {
    WALLET_SYNC_LOCK.get_or_init(|| Mutex::new(()))
}

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
    pub balance_zatoshis: u64,
    /// Total unspent notes in the persisted wallet cache (not just this scan).
    pub notes_found: usize,
    pub total_notes: usize,
    pub new_notes_in_scan: usize,
    pub last_scan_height: u32,
    pub chain_tip: u32,
    pub already_synced: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendTransactionRequest {
    pub recipient: String,
    pub amount: f64,
    pub memo: Option<String>,
    pub zebra_url: Option<String>,
    #[serde(default)]
    pub priority: bool,
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

    let pwd = password.unwrap_or_default();
    let wallet = storage
        .load_wallet(&pwd)
        .await
        .map_err(|e| format!("Failed to load wallet: {e}. Please check your password."))?;

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

    let mut wallet = nozy::HDWallet::new().map_err(|e| {
        error_response_with_code(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create wallet: {e}"),
            "WALLET_CREATE_FAILED",
        )
    })?;

    let password_for_save = payload.password.clone();

    if let Some(ref pwd) = payload.password {
        wallet.set_password(pwd).map_err(|e| {
            error_response_with_code(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to set password: {e}"),
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
                format!("Failed to save wallet: {e}"),
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

    let config = nozy::load_config();
    let network = if config.network == "testnet" {
        zcash_protocol::consensus::NetworkType::Test
    } else {
        zcash_protocol::consensus::NetworkType::Main
    };
    let address = wallet
        .generate_orchard_address(0, 0, network)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(serde_json::json!({
                    "error": format!("Failed to generate address: {}", e)
                })),
            )
        })?;

    Ok(ResponseJson(AddressResponse { address }))
}

pub async fn get_balance(
) -> Result<ResponseJson<BalanceResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    use nozy::{load_wallet_notes, wallet_unspent_balance_zatoshis};

    let notes = load_wallet_notes().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(serde_json::json!({
                "error": format!("Failed to read notes: {}", e)
            })),
        )
    })?;

    let total_zat = wallet_unspent_balance_zatoshis(&notes);

    Ok(ResponseJson(BalanceResponse {
        balance_zec: total_zat as f64 / 100_000_000.0,
        balance_zatoshis: total_zat,
    }))
}

pub async fn sync_wallet(
    Json(payload): Json<SyncRequest>,
) -> Result<ResponseJson<SyncResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    use nozy::{sync_wallet_notes, WalletSyncOptions};

    let _sync_guard = wallet_sync_lock().lock().await;

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

    let mut options = WalletSyncOptions::api_default();
    options.start_height = payload.start_height;
    options.end_height = payload.end_height;
    options.zebra_url = payload.zebra_url;

    match sync_wallet_notes(wallet, options).await {
        Ok(result) => {
            let balance_zec = result.balance_zatoshis as f64 / 100_000_000.0;
            let message = if result.already_synced {
                format!(
                    "Wallet already synced to tip (height {}). Cached balance: {balance_zec:.8} ZEC",
                    result.chain_tip
                )
            } else {
                format!(
                    "Sync completed for blocks {}-{}. Balance: {balance_zec:.8} ZEC",
                    result.scan_start, result.scan_end
                )
            };
            Ok(ResponseJson(SyncResponse {
                success: true,
                balance_zec,
                balance_zatoshis: result.balance_zatoshis,
                notes_found: result.unspent_notes,
                total_notes: result.total_notes,
                new_notes_in_scan: result.new_notes_in_scan,
                last_scan_height: result.last_scan_height,
                chain_tip: result.chain_tip,
                already_synced: result.already_synced,
                message,
            }))
        }
        Err(e) => {
            let status =
                StatusCode::from_u16(e.api_status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            Err((status, ResponseJson(e.to_api_json())))
        }
    }
}

pub async fn send_transaction(
    Json(payload): Json<SendTransactionRequestWrapper>,
) -> Result<ResponseJson<SendTransactionResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    use nozy::cli_helpers::{
        cached_unspent_balance_zatoshis, format_insufficient_funds_message,
        is_insufficient_funds_error, is_zebra_unavailable_error, scan_notes_for_sending,
    };
    use nozy::ZcashTransactionBuilder;
    use nozy::{estimate_orchard_send_fee_zatoshis, load_config, ZebraClient};

    let config = load_config();
    let zebra_url = payload
        .request
        .zebra_url
        .unwrap_or_else(|| config.zebra_url.clone());

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

    if let Err(e) = nozy::input_validation::validate_zcash_address(&payload.request.recipient) {
        return Ok(ResponseJson(SendTransactionResponse {
            success: false,
            txid: None,
            message: format!("Invalid recipient address: {e}"),
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
            message: "Invalid amount. Must be greater than 0 and less than 21,000,000 ZEC."
                .to_string(),
        }));
    }

    let memo_bytes_opt = payload
        .request
        .memo
        .as_ref()
        .map(|m| m.trim().as_bytes().to_vec())
        .filter(|b| !b.is_empty());

    let amount_zatoshis = (payload.request.amount * 100_000_000.0) as u64;

    let pilot = nozy::PilotSendOptions {
        priority: payload.request.priority,
        expiry_delta_blocks: nozy::PILOT_EXPIRY_DELTA_BLOCKS,
    };
    let fee_zatoshis =
        estimate_orchard_send_fee_zatoshis(memo_bytes_opt.as_deref(), pilot.priority);

    let cached_balance = cached_unspent_balance_zatoshis().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(serde_json::json!({
                "error": format!("Failed to read cached balance: {e}")
            })),
        )
    })?;

    let total_needed = amount_zatoshis.saturating_add(fee_zatoshis);
    if cached_balance < total_needed {
        return Ok(ResponseJson(SendTransactionResponse {
            success: false,
            txid: None,
            message: format_insufficient_funds_message(
                cached_balance,
                amount_zatoshis,
                fee_zatoshis,
            ),
        }));
    }

    let spendable_notes = scan_notes_for_sending(wallet, &zebra_url)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if is_zebra_unavailable_error(&msg) {
                error_response_with_code(
                    StatusCode::SERVICE_UNAVAILABLE,
                    format!("Zebra node unavailable during note scan: {msg}"),
                    "ZEBRA_UNAVAILABLE",
                )
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ResponseJson(serde_json::json!({
                        "error": format!("Failed to scan notes: {msg}")
                    })),
                )
            }
        })?;

    let zebra_client = ZebraClient::new(zebra_url.clone());

    let tip_height = zebra_client.get_best_block_height().await.map_err(|e| {
        let msg = e.to_string();
        if is_zebra_unavailable_error(&msg) {
            error_response_with_code(
                StatusCode::SERVICE_UNAVAILABLE,
                format!("Zebra node unavailable: {msg}"),
                "ZEBRA_UNAVAILABLE",
            )
        } else {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(serde_json::json!({
                    "error": format!("Failed to read chain tip: {msg}")
                })),
            )
        }
    })?;
    let expiry_height = tip_height.saturating_add(pilot.expiry_delta_blocks);

    let mut tx_builder = ZcashTransactionBuilder::new();
    tx_builder.set_zebra_url(&zebra_url);
    tx_builder.enable_mainnet_broadcast();

    let transaction = match tx_builder
        .build_send_transaction(
            &zebra_client,
            &spendable_notes,
            &payload.request.recipient,
            amount_zatoshis,
            fee_zatoshis,
            memo_bytes_opt.as_deref(),
            pilot,
        )
        .await
    {
        Ok(tx) => tx,
        Err(e) => {
            let msg = e.to_string();
            if is_insufficient_funds_error(&msg) {
                return Ok(ResponseJson(SendTransactionResponse {
                    success: false,
                    txid: None,
                    message: msg,
                }));
            }
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(serde_json::json!({
                    "error": format!("Failed to build transaction: {msg}")
                })),
            ));
        }
    };

    match tx_builder
        .broadcast_transaction(&zebra_client, &transaction)
        .await
    {
        Ok(network_txid) => {
            use nozy::transaction_history::{SentTransactionRecord, SentTransactionStorage};
            if let Ok(tx_storage) = SentTransactionStorage::new() {
                let spent_note_ids: Vec<String> = spendable_notes
                    .iter()
                    .map(|note| hex::encode(note.orchard_note.nullifier.to_bytes()))
                    .collect();

                let _ = nozy::mark_wallet_notes_spent_from_spendables(&spendable_notes);

                let mut tx_record = SentTransactionRecord::new_pilot(
                    network_txid.clone(),
                    payload.request.recipient.clone(),
                    amount_zatoshis,
                    fee_zatoshis,
                    memo_bytes_opt.clone(),
                    spent_note_ids,
                    pilot.priority,
                    expiry_height,
                );
                tx_record.mark_broadcast();
                let _ = tx_storage.save_transaction(tx_record);
            }

            Ok(ResponseJson(SendTransactionResponse {
                success: true,
                txid: Some(network_txid.clone()),
                message: format!("Transaction sent successfully! TXID: {network_txid}"),
            }))
        }
        Err(e) => Ok(ResponseJson(SendTransactionResponse {
            success: false,
            txid: Some(transaction.txid.clone()),
            message: format!("Failed to broadcast transaction: {e}"),
        })),
    }
}

pub async fn estimate_fee(
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<ResponseJson<serde_json::Value>, (StatusCode, ResponseJson<serde_json::Value>)> {
    let priority = params
        .get("priority")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);
    let fee_zatoshis = nozy::estimate_orchard_send_fee_zatoshis(None, priority);
    let fee_zec = fee_zatoshis as f64 / 100_000_000.0;

    Ok(ResponseJson(serde_json::json!({
        "fee_zatoshis": fee_zatoshis,
        "fee_zec": fee_zec,
        "priority": priority,
        "expiry_delta_blocks": nozy::PILOT_EXPIRY_DELTA_BLOCKS,
        "fee_source": "zip317_client",
        "estimated_at": chrono::Utc::now().to_rfc3339()
    })))
}

pub async fn get_config(
) -> Result<ResponseJson<ConfigResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
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
    let url = payload
        .zebra_url
        .unwrap_or_else(|| config.zebra_url.clone());

    let client = ZebraClient::new(url.clone());
    match client.get_block_count().await {
        Ok(block_count) => Ok(ResponseJson(format!(
            "✅ Connected to Zebra node at {url}\nBlock height: {block_count}"
        ))),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            ResponseJson(serde_json::json!({
                "error": format!("❌ Failed to connect: {}", e)
            })),
        )),
    }
}

// ========== Zero-knowledge endpoints (no wallet, no password) ==========
// Used when client holds keys and does scan/sign locally; server only provides
// chain data and broadcast. See docs/SERVER_PRIVACY_ZERO_KNOWLEDGE.md.

#[derive(Debug, Serialize, Deserialize)]
pub struct ChainBlockCountResponse {
    pub block_count: u32,
}

pub async fn chain_block_count(
) -> Result<ResponseJson<ChainBlockCountResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    use nozy::{load_config, ZebraClient};

    let config = load_config();
    let client = ZebraClient::from_config(&config);
    let block_count = client.get_block_count().await.map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            ResponseJson(serde_json::json!({
                "error": format!("Failed to get block count from chain: {}", e)
            })),
        )
    })?;
    Ok(ResponseJson(ChainBlockCountResponse { block_count }))
}

pub async fn chain_block(
    Path(height): Path<u32>,
) -> Result<ResponseJson<serde_json::Value>, (StatusCode, ResponseJson<serde_json::Value>)> {
    use nozy::{load_config, ZebraClient};

    let config = load_config();
    let client = ZebraClient::from_config(&config);
    let block_data = client.get_block(height).await.map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            ResponseJson(serde_json::json!({
                "error": format!("Failed to get block at height {}: {}", height, e)
            })),
        )
    })?;
    Ok(ResponseJson(
        serde_json::to_value(block_data).unwrap_or(serde_json::Value::Null),
    ))
}

#[derive(Debug, Deserialize)]
pub struct BroadcastRawRequest {
    pub raw_transaction_hex: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BroadcastRawResponse {
    pub success: bool,
    pub txid: Option<String>,
    pub message: String,
}

pub async fn broadcast_raw_transaction(
    Json(payload): Json<BroadcastRawRequest>,
) -> Result<ResponseJson<BroadcastRawResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    use nozy::{load_config, ZebraClient};

    let hex_str = payload.raw_transaction_hex.trim();
    if hex_str.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            ResponseJson(serde_json::json!({
                "error": "raw_transaction_hex is required and must be non-empty"
            })),
        ));
    }
    if hex_str.len() % 2 != 0 || !hex_str.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err((
            StatusCode::BAD_REQUEST,
            ResponseJson(serde_json::json!({
                "error": "raw_transaction_hex must be valid hex"
            })),
        ));
    }

    let config = load_config();
    let client = ZebraClient::from_config(&config);
    match client.broadcast_transaction(hex_str).await {
        Ok(txid) => Ok(ResponseJson(BroadcastRawResponse {
            success: true,
            txid: Some(txid),
            message: "Transaction broadcast successfully".to_string(),
        })),
        Err(e) => Err((
            StatusCode::BAD_GATEWAY,
            ResponseJson(serde_json::json!({
                "error": format!("Broadcast failed: {}", e)
            })),
        )),
    }
}

pub async fn check_proving_status(
) -> Result<ResponseJson<ProvingStatusResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
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

pub async fn download_proving_parameters(
) -> Result<ResponseJson<String>, (StatusCode, ResponseJson<serde_json::Value>)> {
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

    Ok(ResponseJson(
        "✅ Proving parameters downloaded successfully!".to_string(),
    ))
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
) -> Result<ResponseJson<TransactionHistoryResponse>, (StatusCode, ResponseJson<serde_json::Value>)>
{
    use nozy::transaction_history::SentTransactionStorage;

    let tx_storage = SentTransactionStorage::new().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(serde_json::json!({
                "error": format!("Failed to initialize transaction storage: {}", e)
            })),
        )
    })?;

    let all_txs = tx_storage.get_all_transactions();

    let filtered: Vec<_> = all_txs
        .iter()
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
        .map(|tx| {
            serde_json::json!({
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
            })
        })
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

    let tx_storage = SentTransactionStorage::new().map_err(|e| {
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

pub async fn check_transaction_confirmations(
) -> Result<ResponseJson<serde_json::Value>, (StatusCode, ResponseJson<serde_json::Value>)> {
    use nozy::{load_config, transaction_history::SentTransactionStorage, ZebraClient};

    let config = load_config();
    let zebra_client = ZebraClient::new(config.zebra_url);
    let tx_storage = SentTransactionStorage::new().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(serde_json::json!({
                "error": format!("Failed to initialize transaction storage: {}", e)
            })),
        )
    })?;

    let updated_pending = tx_storage
        .check_all_pending_transactions(&zebra_client)
        .await
        .unwrap_or(0);
    let expired_updated = tx_storage
        .check_expired_pending_transactions(&zebra_client)
        .await
        .unwrap_or(0);
    let updated_confirmations = tx_storage
        .update_confirmations(&zebra_client)
        .await
        .unwrap_or(0);

    Ok(ResponseJson(serde_json::json!({
        "pending_updated": updated_pending,
        "expired_updated": expired_updated,
        "confirmations_updated": updated_confirmations
    })))
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

pub async fn speed_up_transaction(
    Json(payload): Json<SpeedUpTransactionRequest>,
) -> Result<ResponseJson<SpeedUpTransactionResponse>, (StatusCode, ResponseJson<serde_json::Value>)>
{
    use nozy::{load_config, speed_up_transaction as core_speed_up};

    let config = load_config();
    let zebra_url = payload
        .zebra_url
        .unwrap_or_else(|| config.zebra_url.clone());

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

    match core_speed_up(wallet, &zebra_url, &payload.original_txid).await {
        Ok(new_txid) => Ok(ResponseJson(SpeedUpTransactionResponse {
            success: true,
            txid: Some(new_txid.clone()),
            original_txid: payload.original_txid.clone(),
            message: format!(
                "Speed-up transaction broadcast. New TXID: {new_txid} (replaces {})",
                payload.original_txid
            ),
        })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            ResponseJson(serde_json::json!({
                "error": e.to_string()
            })),
        )),
    }
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

pub async fn list_address_book(
) -> Result<ResponseJson<Vec<AddressBookEntry>>, (StatusCode, ResponseJson<serde_json::Value>)> {
    use nozy::AddressBook;

    let address_book = AddressBook::new().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(serde_json::json!({
                "error": format!("Failed to initialize address book: {}", e)
            })),
        )
    })?;

    let entries: Vec<AddressBookEntry> = address_book
        .list_addresses()
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

    let address_book = AddressBook::new().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(serde_json::json!({
                "error": format!("Failed to initialize address book: {}", e)
            })),
        )
    })?;

    address_book
        .add_address(
            payload.name.clone(),
            payload.address.clone(),
            payload.notes.clone(),
        )
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

    let address_book = AddressBook::new().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(serde_json::json!({
                "error": format!("Failed to initialize address book: {}", e)
            })),
        )
    })?;

    let removed = address_book.remove_address(&name).map_err(|e| {
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

    let address_book = AddressBook::new().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(serde_json::json!({
                "error": format!("Failed to initialize address book: {}", e)
            })),
        )
    })?;

    let query = params.get("q").cloned().unwrap_or_default();
    let entries: Vec<AddressBookEntry> = address_book
        .search_addresses(&query)
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

pub async fn get_wallet_status(
) -> Result<ResponseJson<WalletStatusResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    use nozy::{
        load_config, load_wallet_notes, transaction_history::SentTransactionStorage,
        wallet_unspent_balance_zatoshis, ZebraClient,
    };

    let config = load_config();
    let zebra_client = ZebraClient::new(config.zebra_url.clone());

    let balance_zatoshis = load_wallet_notes()
        .map(|notes| wallet_unspent_balance_zatoshis(&notes))
        .unwrap_or(0);

    let (pending_count, total_count) = if let Ok(tx_storage) = SentTransactionStorage::new() {
        let pending = tx_storage.get_pending_transactions();
        let all = tx_storage.get_all_transactions();
        (pending.len(), all.len())
    } else {
        (0, 0)
    };

    let current_height = zebra_client.get_block_count().await.ok();
    let blocks_behind =
        if let (Some(current), Some(last)) = (current_height, config.last_scan_height) {
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

#[derive(Debug, Serialize, Deserialize)]
pub struct WebEntitlements {
    pub has_active_subscription: bool,
    pub includes_nym_privacy: bool,
    pub includes_encrypted_backup: bool,
    pub can_host_node: bool,
    pub vault_features_enabled: bool,
    pub phase1_watch_only: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebMeResponse {
    pub user_id: String,
    pub plan: String,
    pub entitlements: WebEntitlements,
}

pub async fn web_me(
) -> Result<ResponseJson<WebMeResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    let has_subscription = std::env::var("NOZY_WEB_SUB_ACTIVE")
        .ok()
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    Ok(ResponseJson(WebMeResponse {
        user_id: "local-dev-user".to_string(),
        plan: if has_subscription {
            "nozy-private-standard".to_string()
        } else {
            "free-watch-only".to_string()
        },
        entitlements: WebEntitlements {
            has_active_subscription: has_subscription,
            includes_nym_privacy: has_subscription,
            includes_encrypted_backup: has_subscription,
            can_host_node: false,
            vault_features_enabled: false,
            phase1_watch_only: true,
        },
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebReadStateResponse {
    pub balance_zec: f64,
    pub balance_zatoshis: u64,
    pub last_sync_height: Option<u32>,
    pub current_block_height: Option<u32>,
    pub blocks_behind: Option<u32>,
    pub recent_transactions: Vec<serde_json::Value>,
}

pub async fn web_read_state(
) -> Result<ResponseJson<WebReadStateResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    use nozy::{
        load_config, load_wallet_notes, transaction_history::SentTransactionStorage,
        wallet_unspent_balance_zatoshis, ZebraClient,
    };

    let config = load_config();
    let zebra_client = ZebraClient::from_config(&config);

    let balance_zatoshis = load_wallet_notes()
        .map(|notes| wallet_unspent_balance_zatoshis(&notes))
        .unwrap_or(0);

    let txs: Vec<serde_json::Value> = if let Ok(tx_storage) = SentTransactionStorage::new() {
        let all = tx_storage.get_all_transactions();
        all.iter()
            .take(10)
            .map(|t| {
                serde_json::json!({
                    "txid": t.txid,
                    "status": format!("{:?}", t.status),
                    "amount_zatoshis": t.amount_zatoshis,
                    "amount_zec": t.amount_zatoshis as f64 / 100_000_000.0,
                    "recipient": t.recipient_address,
                    "confirmations": t.confirmations,
                    "broadcast_at": t.broadcast_at.map(|d| d.to_rfc3339())
                })
            })
            .collect()
    } else {
        Vec::new()
    };

    let current_height = zebra_client.get_block_count().await.ok();
    let blocks_behind =
        if let (Some(current), Some(last)) = (current_height, config.last_scan_height) {
            Some(current.saturating_sub(last))
        } else {
            None
        };

    Ok(ResponseJson(WebReadStateResponse {
        balance_zec: balance_zatoshis as f64 / 100_000_000.0,
        balance_zatoshis,
        last_sync_height: config.last_scan_height,
        current_block_height: current_height,
        blocks_behind,
        recent_transactions: txs,
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebPrivacyStatusResponse {
    pub strict_mode: bool,
    pub zebra_url: String,
    pub is_local_endpoint: bool,
    pub privacy_route_active: bool,
    pub selected_route: Option<String>,
    pub remote_rpc_allowed: bool,
    pub message: String,
    pub remediation_steps: Vec<String>,
}

pub async fn web_privacy_status(
) -> Result<ResponseJson<WebPrivacyStatusResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    use nozy::load_config;
    use nozy::privacy_network::proxy::ProxyConfig;

    let config = load_config();
    let zebra_url = config.zebra_url.clone();
    let strict_mode = config.privacy_network.require_privacy_network;
    let is_local_endpoint = zebra_url.contains("127.0.0.1") || zebra_url.contains("localhost");
    let force_privacy_route_active = std::env::var("NOZY_WEB_FORCE_PRIVACY_ROUTE_ACTIVE")
        .ok()
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if is_local_endpoint {
        return Ok(ResponseJson(WebPrivacyStatusResponse {
            strict_mode,
            zebra_url,
            is_local_endpoint: true,
            privacy_route_active: false,
            selected_route: None,
            remote_rpc_allowed: true,
            message: "Local Zebra RPC endpoint in use.".to_string(),
            remediation_steps: vec![
                "Local endpoint detected. No additional privacy route is required for localhost RPC.".to_string(),
            ],
        }));
    }

    if force_privacy_route_active {
        return Ok(ResponseJson(WebPrivacyStatusResponse {
            strict_mode,
            zebra_url,
            is_local_endpoint: false,
            privacy_route_active: true,
            selected_route: Some("DevSimulated".to_string()),
            remote_rpc_allowed: true,
            message: "Remote RPC treated as privacy-routed (development override).".to_string(),
            remediation_steps: vec![
                "Development override active via NOZY_WEB_FORCE_PRIVACY_ROUTE_ACTIVE.".to_string(),
                "For production validation, disable override and run with real Tor/I2P."
                    .to_string(),
            ],
        }));
    }

    let proxy = ProxyConfig::auto_detect().await;
    if proxy.enabled {
        return Ok(ResponseJson(WebPrivacyStatusResponse {
            strict_mode,
            zebra_url,
            is_local_endpoint: false,
            privacy_route_active: true,
            selected_route: Some(format!("{:?}", proxy.network)),
            remote_rpc_allowed: true,
            message: "Remote RPC protected by privacy route.".to_string(),
            remediation_steps: vec![
                "Privacy route is active. Continue using this endpoint.".to_string(),
                "If connectivity fails, verify local Tor/I2P service health and proxy settings."
                    .to_string(),
            ],
        }));
    }

    let remote_rpc_allowed = !strict_mode;
    let message = if remote_rpc_allowed {
        "Remote RPC has no privacy route (allowed because strict mode is disabled).".to_string()
    } else {
        "Remote RPC blocked: strict privacy mode requires Tor/I2P route.".to_string()
    };
    let remediation_steps = if remote_rpc_allowed {
        vec![
            "Enable Tor or I2P to improve metadata privacy for remote RPC.".to_string(),
            "Or switch Zebra RPC to localhost (127.0.0.1/localhost).".to_string(),
        ]
    } else {
        vec![
            "Start Tor (default socks5://127.0.0.1:9050) or I2P (default http://127.0.0.1:4444)."
                .to_string(),
            "Verify privacy route status in settings and retry.".to_string(),
            "If available, switch Zebra RPC to localhost (127.0.0.1/localhost).".to_string(),
            "Disable strict mode only if you explicitly accept metadata leak risk.".to_string(),
        ]
    };

    Ok(ResponseJson(WebPrivacyStatusResponse {
        strict_mode,
        zebra_url,
        is_local_endpoint: false,
        privacy_route_active: false,
        selected_route: None,
        remote_rpc_allowed,
        message,
        remediation_steps,
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebNodeStatusResponse {
    pub backend: String,
    pub protocol: String,
    pub zebra_url: String,
    pub connected: bool,
    pub block_height: Option<u32>,
    pub error: Option<String>,
}

pub async fn web_node_status(
) -> Result<ResponseJson<WebNodeStatusResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    use nozy::load_config;
    use nozy::{BackendKind, Protocol, ZebraClient};

    let config = load_config();
    let zebra_url =
        if matches!(config.backend, BackendKind::Crosslink) && !config.crosslink_url.is_empty() {
            config.crosslink_url.clone()
        } else {
            config.zebra_url.clone()
        };

    let backend = match config.backend {
        BackendKind::Zebra => "zebra",
        BackendKind::Crosslink => "crosslink",
    }
    .to_string();

    let protocol = match config.protocol {
        Protocol::JsonRpc => "jsonrpc",
        Protocol::Grpc => "grpc",
    }
    .to_string();

    let client = ZebraClient::from_config(&config);
    match client.get_block_count().await {
        Ok(block_height) => Ok(ResponseJson(WebNodeStatusResponse {
            backend,
            protocol,
            zebra_url,
            connected: true,
            block_height: Some(block_height),
            error: None,
        })),
        Err(e) => Ok(ResponseJson(WebNodeStatusResponse {
            backend,
            protocol,
            zebra_url,
            connected: false,
            block_height: None,
            error: Some(e.to_string()),
        })),
    }
}

#[derive(Debug, Serialize)]
pub struct NotesResponse {
    pub notes: Vec<serde_json::Value>,
    pub total: usize,
    pub total_balance_zatoshis: u64,
    pub total_balance_zec: f64,
}

pub async fn get_notes(
) -> Result<ResponseJson<NotesResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    use nozy::{load_wallet_notes, wallet_unspent_balance_zatoshis};

    let notes = load_wallet_notes().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(serde_json::json!({
                "error": format!("Failed to read notes: {}", e)
            })),
        )
    })?;

    let total_balance_zatoshis = wallet_unspent_balance_zatoshis(&notes);
    let total_balance_zec = total_balance_zatoshis as f64 / 100_000_000.0;

    let notes_json: Vec<serde_json::Value> = notes
        .iter()
        .map(|n| {
            serde_json::json!({
                "value": n.value,
                "value_zec": n.value as f64 / 100_000_000.0,
                "block_height": n.block_height,
                "txid": n.txid,
                "spent": n.spent,
                "memo": String::from_utf8(n.memo.clone()).unwrap_or_default()
            })
        })
        .collect();

    Ok(ResponseJson(NotesResponse {
        notes: notes_json,
        total: notes.len(),
        total_balance_zatoshis,
        total_balance_zec,
    }))
}
