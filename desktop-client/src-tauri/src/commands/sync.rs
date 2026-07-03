use crate::error::TauriError;
use crate::session::load_session_wallet;
use nozy::{
    load_config, sync_wallet_notes, wallet_balance_snapshot, HDWallet,
    WalletSyncOptions,
};
use serde::{Deserialize, Serialize};
use tauri::command;

#[derive(Debug, Serialize)]
pub struct BalanceResponse {
    /// Spendable balance (confirmed minus pending outbound sends).
    pub balance: f64,
    /// Confirmed shielded balance from note cache (legacy field name).
    pub verified_balance: f64,
    pub confirmed_zec: f64,
    pub pending_zec: f64,
    pub available_zec: f64,
    pub unspent_note_count: usize,
}

#[derive(Debug, Serialize)]
pub struct SyncResponse {
    pub success: bool,
    pub balance_zec: f64,
    pub notes_found: usize,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_scan_height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_tip: Option<u32>,
    pub already_synced: bool,
}

#[derive(Debug, Deserialize)]
pub struct SyncRequest {
    pub start_height: Option<u32>,
    pub end_height: Option<u32>,
    pub zebra_url: Option<String>,
    pub password: Option<String>,
}

fn zats_to_zec(zat: u64) -> f64 {
    zat as f64 / 100_000_000.0
}

async fn load_wallet(password: Option<&str>) -> Result<HDWallet, TauriError> {
    load_session_wallet(password).await
}

#[command]
pub async fn get_balance() -> Result<BalanceResponse, TauriError> {
    let snapshot = wallet_balance_snapshot().map_err(TauriError::from)?;
    Ok(BalanceResponse {
        balance: zats_to_zec(snapshot.available_zatoshis),
        verified_balance: zats_to_zec(snapshot.confirmed_zatoshis),
        confirmed_zec: zats_to_zec(snapshot.confirmed_zatoshis),
        pending_zec: zats_to_zec(snapshot.pending_zatoshis),
        available_zec: zats_to_zec(snapshot.available_zatoshis),
        unspent_note_count: snapshot.unspent_note_count,
    })
}

#[command]
pub async fn sync_wallet(request: SyncRequest) -> Result<SyncResponse, TauriError> {
    let wallet = load_wallet(request.password.as_deref()).await?;

    let scan_to_tip = request.end_height.is_none();
    let options = WalletSyncOptions {
        start_height: request.start_height,
        end_height: request.end_height,
        scan_to_tip,
        zebra_url: request.zebra_url.or_else(|| Some(load_config().zebra_url)),
        ..Default::default()
    };

    match sync_wallet_notes(wallet, options).await {
        Ok(result) => {
            let balance_zec = result.balance_zatoshis as f64 / 100_000_000.0;
            let message = if result.already_synced {
                format!(
                    "Wallet synced to tip (height {}). Cached balance: {balance_zec:.8} ZEC",
                    result.chain_tip
                )
            } else if result.blocks_scanned == 0 {
                format!(
                    "Witness catch-up in progress (height {}). Repeat sync until ready for send. Balance: {balance_zec:.8} ZEC",
                    result.chain_tip
                )
            } else {
                format!(
                    "Sync completed for blocks {}-{}. Balance: {balance_zec:.8} ZEC",
                    result.scan_start, result.scan_end
                )
            };
            Ok(SyncResponse {
                success: true,
                balance_zec,
                notes_found: result.unspent_notes,
                message,
                last_scan_height: Some(result.last_scan_height),
                chain_tip: Some(result.chain_tip),
                already_synced: result.already_synced,
            })
        }
        Err(e) => Err(TauriError {
            message: e.to_string(),
            code: Some(e.api_code().to_string()),
        }),
    }
}
