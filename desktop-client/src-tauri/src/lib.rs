use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::sync::Mutex;

#[derive(Default)]
struct AppState {
    wallet: Mutex<Option<nozy::HDWallet>>,
}

#[derive(Debug, Serialize)]
struct WalletStatusResponse {
    exists: bool,
    unlocked: bool,
    has_password: bool,
    address: Option<String>,
}

#[derive(Debug, Serialize)]
struct WalletExistsResponse {
    exists: bool,
    has_password: bool,
}

#[derive(Debug, Deserialize)]
struct CreateWalletRequest {
    password: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RestoreWalletRequest {
    mnemonic: String,
    password: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UnlockWalletRequest {
    password: String,
}

#[derive(Debug, Deserialize)]
struct ChangePasswordRequest {
    current_password: String,
    new_password: String,
}

#[derive(Debug, Serialize)]
struct GenerateAddressResponse {
    address: String,
}

#[derive(Debug, Serialize)]
struct BalanceResponse {
    balance: f64,
    verified_balance: f64,
}

#[derive(Debug, Deserialize)]
struct SyncRequest {
    start_height: Option<u32>,
    end_height: Option<u32>,
    zebra_url: Option<String>,
    password: Option<String>,
}

#[derive(Debug, Serialize)]
struct SyncResponse {
    success: bool,
    balance_zec: f64,
    notes_found: usize,
    message: String,
}

#[derive(Debug, Deserialize)]
struct SendTransactionRequest {
    recipient: String,
    amount: f64,
    memo: Option<String>,
    zebra_url: Option<String>,
    password: Option<String>,
}

#[derive(Debug, Serialize)]
struct SendTransactionResponse {
    success: bool,
    txid: Option<String>,
    message: String,
}

#[derive(Debug, Serialize)]
struct ConfigResponse {
    zebra_url: String,
    theme: String,
}

#[derive(Debug, Deserialize)]
struct SetZebraUrlRequest {
    url: String,
}

#[derive(Debug, Deserialize)]
struct TestZebraRequest {
    zebra_url: Option<String>,
}

#[derive(Debug, Serialize)]
struct ProvingStatusResponse {
    downloaded: bool,
    progress: u32,
}

#[derive(Debug, Deserialize)]
struct VerifyPasswordRequest {
    password: String,
}

#[derive(Debug, Deserialize)]
struct SignMessageRequest {
    message: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct SignMessageResponse {
    signature: String,
}

#[derive(Debug, Serialize)]
struct AddressBookEntry {
    name: String,
    address: String,
    created_at: String,
    last_used: Option<String>,
    usage_count: u32,
    notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AddAddressBookRequest {
    name: String,
    address: String,
    notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BackupPathRequest {
    backup_path: String,
}

#[derive(Debug, Serialize)]
struct BackupActionResponse {
    success: bool,
    path: String,
    message: String,
}

fn wallet_file_exists() -> bool {
    nozy::paths::get_wallet_data_dir().join("wallet.dat").exists()
}

async fn wallet_exists_with_password_flag() -> WalletExistsResponse {
    let exists = wallet_file_exists();
    let has_password = if exists {
        let storage = nozy::WalletStorage::with_xdg_dir();
        storage.load_wallet("").await.is_err()
    } else {
        false
    };
    WalletExistsResponse {
        exists,
        has_password,
    }
}

fn network_from_config() -> zcash_protocol::consensus::NetworkType {
    let config = nozy::load_config();
    if config.network == "testnet" {
        zcash_protocol::consensus::NetworkType::Test
    } else {
        zcash_protocol::consensus::NetworkType::Main
    }
}

fn get_cached_wallet(state: &tauri::State<AppState>) -> Result<Option<nozy::HDWallet>, String> {
    state
        .wallet
        .lock()
        .map_err(|e| format!("Failed to lock app state: {}", e))
        .map(|w| w.clone())
}

fn set_cached_wallet(state: &tauri::State<AppState>, wallet: Option<nozy::HDWallet>) -> Result<(), String> {
    let mut guard = state
        .wallet
        .lock()
        .map_err(|e| format!("Failed to lock app state: {}", e))?;
    *guard = wallet;
    Ok(())
}

async fn load_wallet_for_request(
    state: &tauri::State<'_, AppState>,
    password: Option<String>,
) -> Result<nozy::HDWallet, String> {
    if let Some(cached) = get_cached_wallet(state)? {
        return Ok(cached);
    }

    let storage = nozy::WalletStorage::with_xdg_dir();
    if let Some(pwd) = password {
        if !pwd.is_empty() {
            let wallet = storage
                .load_wallet(&pwd)
                .await
                .map_err(|e| format!("Failed to load wallet: {}", e))?;
            set_cached_wallet(state, Some(wallet.clone()))?;
            return Ok(wallet);
        }
    }

    // For unprotected wallets we can still proceed without an explicit password.
    let wallet = storage
        .load_wallet("")
        .await
        .map_err(|e| format!("Wallet is locked or unavailable: {}", e))?;
    set_cached_wallet(state, Some(wallet.clone()))?;
    Ok(wallet)
}

#[tauri::command]
async fn wallet_exists() -> Result<WalletExistsResponse, String> {
    Ok(wallet_exists_with_password_flag().await)
}

#[tauri::command]
async fn create_wallet(
    state: tauri::State<'_, AppState>,
    request: CreateWalletRequest,
) -> Result<String, String> {
    if wallet_file_exists() {
        return Err("A wallet already exists. Restore or remove it first.".to_string());
    }

    let mut wallet = nozy::HDWallet::new().map_err(|e| format!("Failed to create wallet: {}", e))?;
    let password = request.password.unwrap_or_default();
    if !password.is_empty() {
        wallet
            .set_password(&password)
            .map_err(|e| format!("Failed to set wallet password: {}", e))?;
    }

    let storage = nozy::WalletStorage::with_xdg_dir();
    storage
        .save_wallet(&wallet, &password)
        .await
        .map_err(|e| format!("Failed to save wallet: {}", e))?;

    set_cached_wallet(&state, Some(wallet.clone()))?;
    Ok(wallet.get_mnemonic())
}

#[tauri::command]
async fn restore_wallet(
    state: tauri::State<'_, AppState>,
    request: RestoreWalletRequest,
) -> Result<(), String> {
    let mut wallet = nozy::HDWallet::from_mnemonic(&request.mnemonic)
        .map_err(|e| format!("Invalid mnemonic: {}", e))?;

    let password = request.password.unwrap_or_default();
    if !password.is_empty() {
        wallet
            .set_password(&password)
            .map_err(|e| format!("Failed to set wallet password: {}", e))?;
    }

    let storage = nozy::WalletStorage::with_xdg_dir();
    storage
        .save_wallet(&wallet, &password)
        .await
        .map_err(|e| format!("Failed to save wallet: {}", e))?;

    set_cached_wallet(&state, Some(wallet))?;
    Ok(())
}

#[tauri::command]
async fn unlock_wallet(
    state: tauri::State<'_, AppState>,
    request: UnlockWalletRequest,
) -> Result<WalletStatusResponse, String> {
    let storage = nozy::WalletStorage::with_xdg_dir();
    let wallet = storage
        .load_wallet(&request.password)
        .await
        .map_err(|e| format!("Invalid password or wallet unavailable: {}", e))?;

    let address = wallet
        .generate_orchard_address(0, 0, network_from_config())
        .ok();

    set_cached_wallet(&state, Some(wallet))?;
    let exists_info = wallet_exists_with_password_flag().await;
    Ok(WalletStatusResponse {
        exists: exists_info.exists,
        unlocked: true,
        has_password: exists_info.has_password,
        address,
    })
}

#[tauri::command]
fn lock_wallet(state: tauri::State<'_, AppState>) -> Result<(), String> {
    set_cached_wallet(&state, None)
}

#[tauri::command]
async fn change_password(
    state: tauri::State<'_, AppState>,
    request: ChangePasswordRequest,
) -> Result<(), String> {
    let storage = nozy::WalletStorage::with_xdg_dir();
    let mut wallet = storage
        .load_wallet(&request.current_password)
        .await
        .map_err(|e| format!("Current password is invalid: {}", e))?;

    wallet
        .set_password(&request.new_password)
        .map_err(|e| format!("Failed to set new password: {}", e))?;

    storage
        .save_wallet(&wallet, &request.new_password)
        .await
        .map_err(|e| format!("Failed to save wallet with new password: {}", e))?;

    set_cached_wallet(&state, Some(wallet))?;
    Ok(())
}

#[tauri::command]
async fn generate_address(state: tauri::State<'_, AppState>) -> Result<GenerateAddressResponse, String> {
    let wallet = load_wallet_for_request(&state, None).await?;
    let address = wallet
        .generate_orchard_address(0, 0, network_from_config())
        .map_err(|e| format!("Failed to generate address: {}", e))?;
    Ok(GenerateAddressResponse { address })
}

#[tauri::command]
async fn get_balance() -> Result<BalanceResponse, String> {
    let notes_path = nozy::paths::get_wallet_data_dir().join("notes.json");
    if !notes_path.exists() {
        return Ok(BalanceResponse {
            balance: 0.0,
            verified_balance: 0.0,
        });
    }

    let content = fs::read_to_string(notes_path).map_err(|e| format!("Failed to read notes: {}", e))?;
    let parsed: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse notes: {}", e))?;

    let total_zat: u64 = parsed
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|n| n.get("value").and_then(|v| v.as_u64()))
        .sum();

    let zec = total_zat as f64 / 100_000_000.0;
    Ok(BalanceResponse {
        balance: zec,
        verified_balance: zec,
    })
}

#[tauri::command]
async fn sync_wallet(
    state: tauri::State<'_, AppState>,
    request: Option<SyncRequest>,
) -> Result<SyncResponse, String> {
    let request = request.unwrap_or(SyncRequest {
        start_height: None,
        end_height: None,
        zebra_url: None,
        password: None,
    });

    let config = nozy::load_config();
    let zebra_url = request.zebra_url.unwrap_or_else(|| config.zebra_url.clone());
    let wallet = load_wallet_for_request(&state, request.password).await?;
    let zebra_client = nozy::ZebraClient::new(zebra_url);
    let effective_start = request.start_height.or_else(|| config.last_scan_height.map(|h| h.saturating_add(1)));
    let scan_start = effective_start.unwrap_or(3_050_000);
    let scan_end = if let Some(end) = request.end_height {
        end.max(scan_start)
    } else {
        match zebra_client.get_block_count().await {
            Ok(tip) => tip.min(scan_start.saturating_add(1_000)),
            Err(_) => scan_start.saturating_add(100),
        }
    };
    let mut note_scanner = nozy::NoteScanner::new(wallet, zebra_client.clone());

    let (result, _spendable_notes) = note_scanner
        .scan_notes(Some(scan_start), Some(scan_end))
        .await
        .map_err(|e| format!("Sync failed: {}", e))?;

    let notes_dir = nozy::paths::get_wallet_data_dir();
    if !notes_dir.exists() {
        let _ = fs::create_dir_all(&notes_dir);
    }
    let notes_path = notes_dir.join("notes.json");
    if let Ok(serialized) = serde_json::to_string_pretty(&result.notes) {
        let _ = fs::write(&notes_path, serialized);
    }

    let _ = nozy::update_last_scan_height(scan_end);

    let balance_zec = result.total_balance as f64 / 100_000_000.0;
    Ok(SyncResponse {
        success: true,
        balance_zec,
        notes_found: result.notes.len(),
        message: format!(
            "Sync completed for blocks {}-{}. Balance: {:.8} ZEC",
            scan_start, scan_end, balance_zec
        ),
    })
}

#[tauri::command]
async fn send_transaction(
    state: tauri::State<'_, AppState>,
    request: SendTransactionRequest,
) -> Result<SendTransactionResponse, String> {
    if !request.recipient.starts_with("u1") {
        return Ok(SendTransactionResponse {
            success: false,
            txid: None,
            message: "Recipient must be a shielded unified address (u1...)".to_string(),
        });
    }
    if request.amount <= 0.0 {
        return Ok(SendTransactionResponse {
            success: false,
            txid: None,
            message: "Amount must be greater than zero.".to_string(),
        });
    }

    let config = nozy::load_config();
    let zebra_url = request.zebra_url.clone().unwrap_or_else(|| config.zebra_url.clone());
    let wallet = load_wallet_for_request(&state, request.password.clone()).await?;
    let spendable_notes = nozy::cli_helpers::scan_notes_for_sending(wallet, &zebra_url)
        .await
        .map_err(|e| format!("Failed to scan notes: {}", e))?;

    let memo_bytes_opt = request
        .memo
        .as_ref()
        .map(|m| m.trim().as_bytes().to_vec())
        .filter(|b| !b.is_empty());

    let amount_zatoshis = (request.amount * 100_000_000.0) as u64;
    let zebra_client = nozy::ZebraClient::new(zebra_url.clone());
    let fee_zatoshis = nozy::cli_helpers::estimate_transaction_fee(&zebra_client).await;

    let mut tx_builder = nozy::ZcashTransactionBuilder::new();
    tx_builder.set_zebra_url(&zebra_url);
    tx_builder.enable_mainnet_broadcast();

    let transaction = tx_builder
        .build_send_transaction(
            &zebra_client,
            &spendable_notes,
            &request.recipient,
            amount_zatoshis,
            fee_zatoshis,
            memo_bytes_opt.as_deref(),
        )
        .await
        .map_err(|e| format!("Failed to build transaction: {}", e))?;

    match tx_builder.broadcast_transaction(&zebra_client, &transaction).await {
        Ok(network_txid) => {
            if let Ok(tx_storage) = nozy::transaction_history::SentTransactionStorage::new() {
                let spent_note_ids: Vec<String> = spendable_notes
                    .iter()
                    .map(|note| hex::encode(note.orchard_note.nullifier.to_bytes()))
                    .collect();

                let mut tx_record = nozy::transaction_history::SentTransactionRecord::new(
                    network_txid.clone(),
                    request.recipient,
                    amount_zatoshis,
                    fee_zatoshis,
                    memo_bytes_opt,
                    spent_note_ids,
                );
                tx_record.mark_broadcast();
                let _ = tx_storage.save_transaction(tx_record);
            }

            Ok(SendTransactionResponse {
                success: true,
                txid: Some(network_txid.clone()),
                message: format!("Transaction sent successfully! TXID: {}", network_txid),
            })
        }
        Err(e) => Ok(SendTransactionResponse {
            success: false,
            txid: Some(transaction.txid),
            message: format!("Failed to broadcast transaction: {}", e),
        }),
    }
}

#[tauri::command]
async fn estimate_fee(zebra_url: Option<String>) -> Result<f64, String> {
    let config = nozy::load_config();
    let client = nozy::ZebraClient::new(zebra_url.unwrap_or(config.zebra_url));
    let fee_zatoshis = nozy::cli_helpers::estimate_transaction_fee(&client).await;
    Ok(fee_zatoshis as f64 / 100_000_000.0)
}

#[tauri::command]
fn get_transaction_history() -> Result<Vec<serde_json::Value>, String> {
    let tx_storage = nozy::transaction_history::SentTransactionStorage::new()
        .map_err(|e| format!("Failed to initialize transaction storage: {}", e))?;

    let items = tx_storage
        .get_all_transactions()
        .iter()
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

    Ok(items)
}

#[tauri::command]
fn get_transaction(txid: String) -> Result<serde_json::Value, String> {
    let tx_storage = nozy::transaction_history::SentTransactionStorage::new()
        .map_err(|e| format!("Failed to initialize transaction storage: {}", e))?;

    let maybe_tx = tx_storage
        .get_all_transactions()
        .into_iter()
        .find(|tx| tx.txid == txid);

    if let Some(tx) = maybe_tx {
        Ok(serde_json::json!({
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
    } else {
        Ok(serde_json::Value::Null)
    }
}

#[tauri::command]
fn get_config() -> Result<ConfigResponse, String> {
    let config = nozy::load_config();
    Ok(ConfigResponse {
        zebra_url: config.zebra_url,
        theme: config.theme,
    })
}

#[tauri::command]
fn set_zebra_url(request: SetZebraUrlRequest) -> Result<(), String> {
    if !request.url.starts_with("http://") && !request.url.starts_with("https://") {
        return Err("Invalid URL format. Must start with http:// or https://".to_string());
    }
    let mut config = nozy::load_config();
    config.zebra_url = request.url;
    nozy::save_config(&config).map_err(|e| format!("Failed to save config: {}", e))
}

#[tauri::command]
async fn test_zebra_connection(request: Option<TestZebraRequest>) -> Result<String, String> {
    let config = nozy::load_config();
    let zebra_url = request
        .and_then(|r| r.zebra_url)
        .unwrap_or(config.zebra_url);

    let tester = nozy::RpcTester::new(zebra_url);
    tester
        .test_connectivity()
        .await
        .map_err(|e| format!("Zebra connection test failed: {}", e))?;
    Ok("Zebra connection successful".to_string())
}

#[tauri::command]
async fn check_proving_status() -> Result<ProvingStatusResponse, String> {
    let builder = nozy::orchard_tx::OrchardTransactionBuilder::new_async(false)
        .await
        .map_err(|e| format!("Failed to initialize proving builder: {}", e))?;
    let status = builder.get_proving_status();
    let downloaded = status.spend_params && status.output_params && status.can_prove;
    Ok(ProvingStatusResponse {
        downloaded,
        progress: if downloaded { 100 } else { 0 },
    })
}

#[tauri::command]
async fn download_proving_parameters() -> Result<String, String> {
    let mut builder = nozy::orchard_tx::OrchardTransactionBuilder::new_async(true)
        .await
        .map_err(|e| format!("Failed to initialize proving builder: {}", e))?;
    builder
        .download_parameters()
        .await
        .map_err(|e| format!("Failed to download parameters: {}", e))?;
    Ok("Proving parameters are ready.".to_string())
}

#[tauri::command]
async fn get_wallet_status(state: tauri::State<'_, AppState>) -> Result<WalletStatusResponse, String> {
    let exists_info = wallet_exists_with_password_flag().await;
    let unlocked = get_cached_wallet(&state)?.is_some() || (exists_info.exists && !exists_info.has_password);
    let address = if unlocked {
        match load_wallet_for_request(&state, None).await {
            Ok(wallet) => wallet
                .generate_orchard_address(0, 0, network_from_config())
                .ok(),
            Err(_) => None,
        }
    } else {
        None
    };

    Ok(WalletStatusResponse {
        exists: exists_info.exists,
        unlocked,
        has_password: exists_info.has_password,
        address,
    })
}

#[tauri::command]
async fn get_mnemonic(
    state: tauri::State<'_, AppState>,
    request: VerifyPasswordRequest,
) -> Result<String, String> {
    let wallet = load_wallet_for_request(&state, Some(request.password)).await?;
    Ok(wallet.get_mnemonic())
}

#[tauri::command]
async fn get_private_key(
    state: tauri::State<'_, AppState>,
    request: VerifyPasswordRequest,
) -> Result<String, String> {
    let wallet = load_wallet_for_request(&state, Some(request.password)).await?;
    let key = wallet
        .derive_key("m/44'/133'/0'/0/0")
        .map_err(|e| format!("Failed to derive private key: {}", e))?;
    Ok(hex::encode(key.private_key().to_bytes()))
}

#[tauri::command]
async fn sign_message(
    state: tauri::State<'_, AppState>,
    request: SignMessageRequest,
) -> Result<SignMessageResponse, String> {
    let private_key = get_private_key(
        state,
        VerifyPasswordRequest {
            password: request.password,
        },
    )
    .await?;

    let mut hasher = Sha256::new();
    hasher.update(private_key.as_bytes());
    hasher.update(request.message.as_bytes());
    let signature = hasher.finalize();
    Ok(SignMessageResponse {
        signature: hex::encode(signature),
    })
}

#[tauri::command]
fn address_book_list() -> Result<Vec<AddressBookEntry>, String> {
    let address_book = nozy::AddressBook::new().map_err(|e| format!("Address book error: {}", e))?;
    let entries = address_book
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
    Ok(entries)
}

#[tauri::command]
fn address_book_add(request: AddAddressBookRequest) -> Result<(), String> {
    let address_book = nozy::AddressBook::new().map_err(|e| format!("Address book error: {}", e))?;
    address_book
        .add_address(request.name, request.address, request.notes)
        .map_err(|e| format!("Failed to add address: {}", e))
}

#[tauri::command]
fn address_book_remove(name: String) -> Result<bool, String> {
    let address_book = nozy::AddressBook::new().map_err(|e| format!("Address book error: {}", e))?;
    address_book
        .remove_address(&name)
        .map_err(|e| format!("Failed to remove address: {}", e))
}

#[tauri::command]
fn address_book_get(name: String) -> Result<Option<AddressBookEntry>, String> {
    let address_book = nozy::AddressBook::new().map_err(|e| format!("Address book error: {}", e))?;
    Ok(address_book.get_address(&name).map(|e| AddressBookEntry {
        name: e.name,
        address: e.address,
        created_at: e.created_at.to_rfc3339(),
        last_used: e.last_used.map(|d| d.to_rfc3339()),
        usage_count: e.usage_count,
        notes: e.notes,
    }))
}

#[tauri::command]
fn address_book_search(query: String) -> Result<Vec<AddressBookEntry>, String> {
    let address_book = nozy::AddressBook::new().map_err(|e| format!("Address book error: {}", e))?;
    let entries = address_book
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
    Ok(entries)
}

#[tauri::command]
async fn export_backup(request: BackupPathRequest) -> Result<BackupActionResponse, String> {
    let path = request.backup_path.trim();
    if path.is_empty() {
        return Err("Backup path is required.".to_string());
    }
    if !wallet_file_exists() {
        return Err("No wallet found to backup.".to_string());
    }

    let storage = nozy::WalletStorage::with_xdg_dir();
    let before: HashSet<String> = storage
        .list_backups()
        .map_err(|e| format!("Failed to read existing backups: {}", e))?
        .into_iter()
        .collect();

    storage
        .create_backup(path)
        .await
        .map_err(|e| format!("Failed to create backup: {}", e))?;

    let after = storage
        .list_backups()
        .map_err(|e| format!("Failed to refresh backup list: {}", e))?;
    let created = after
        .iter()
        .find(|item| !before.contains(*item))
        .cloned()
        .or_else(|| after.last().cloned())
        .unwrap_or_else(|| path.to_string());

    Ok(BackupActionResponse {
        success: true,
        path: created.clone(),
        message: format!("Backup created successfully: {}", created),
    })
}

#[tauri::command]
async fn restore_from_backup(request: BackupPathRequest) -> Result<BackupActionResponse, String> {
    let path = request.backup_path.trim();
    if path.is_empty() {
        return Err("Backup file path is required.".to_string());
    }

    let storage = nozy::WalletStorage::with_xdg_dir();
    storage
        .restore_from_backup(path)
        .await
        .map_err(|e| format!("Failed to restore backup: {}", e))?;

    Ok(BackupActionResponse {
        success: true,
        path: path.to_string(),
        message: "Backup restored successfully.".to_string(),
    })
}

#[tauri::command]
fn list_backups() -> Result<Vec<String>, String> {
    let storage = nozy::WalletStorage::with_xdg_dir();
    storage
        .list_backups()
        .map_err(|e| format!("Failed to list backups: {}", e))
}

pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            wallet_exists,
            create_wallet,
            restore_wallet,
            unlock_wallet,
            lock_wallet,
            change_password,
            generate_address,
            get_balance,
            sync_wallet,
            send_transaction,
            estimate_fee,
            get_transaction_history,
            get_transaction,
            get_config,
            set_zebra_url,
            test_zebra_connection,
            check_proving_status,
            download_proving_parameters,
            get_wallet_status,
            get_mnemonic,
            get_private_key,
            sign_message,
            address_book_list,
            address_book_add,
            address_book_remove,
            address_book_get,
            address_book_search,
            export_backup,
            restore_from_backup,
            list_backups
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
