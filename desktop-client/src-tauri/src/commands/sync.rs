use crate::error::TauriError;
use nozy::{
    load_config, paths::get_wallet_data_dir, update_last_scan_height, NoteScanner, WalletStorage,
    ZebraClient,
};
use serde::{Deserialize, Serialize};
use std::fs;
use tauri::command;

#[derive(Debug, Serialize)]
pub struct SyncResponse {
    pub success: bool,
    pub balance_zec: f64,
    pub notes_found: usize,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct SyncRequest {
    pub start_height: Option<u32>,
    pub end_height: Option<u32>,
    pub zebra_url: Option<String>,
    pub password: Option<String>,
}

#[command]
pub async fn sync_wallet(request: SyncRequest) -> Result<SyncResponse, TauriError> {
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

    let zebra_client = ZebraClient::new(zebra_url.clone());
    let effective_start = request
        .start_height
        .or_else(|| config.last_scan_height.map(|h| h.saturating_add(1)));
    let scan_start = effective_start.unwrap_or(3_050_000);
    let scan_end = if let Some(end) = request.end_height {
        end.max(scan_start)
    } else {
        match zebra_client.get_block_count().await {
            Ok(tip) => tip.min(scan_start.saturating_add(1_000)),
            Err(_) => scan_start.saturating_add(100),
        }
    };

    let mut note_scanner = NoteScanner::new(wallet, zebra_client.clone());

    match note_scanner
        .scan_notes(Some(scan_start), Some(scan_end))
        .await
    {
        Ok((result, _spendable_notes, _sapling)) => {
            let notes_dir = get_wallet_data_dir();
            if !notes_dir.exists() {
                let _ = fs::create_dir_all(&notes_dir);
            }
            let notes_path = notes_dir.join("notes.json");
            if let Ok(serialized) = serde_json::to_string_pretty(&result.notes) {
                let _ = fs::write(&notes_path, serialized);
            }

            let _ = update_last_scan_height(scan_end);

            let balance_zec =
                (result.total_balance + result.sapling_total_balance) as f64 / 100_000_000.0;

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
        Err(e) => Err(TauriError {
            message: format!("Sync failed: {}", e),
            code: Some("SYNC_FAILED".to_string()),
        }),
    }
}

#[command]
pub async fn get_balance() -> Result<SyncResponse, TauriError> {
    let notes_path = get_wallet_data_dir().join("notes.json");

    if !notes_path.exists() {
        return Ok(SyncResponse {
            success: true,
            balance_zec: 0.0,
            notes_found: 0,
            message: "No notes found".to_string(),
        });
    }

    let content = fs::read_to_string(&notes_path).map_err(|e| TauriError::from(e.to_string()))?;

    let parsed: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| TauriError::from(e.to_string()))?;

    let total_zat: u64 = parsed
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|n| n.get("value").and_then(|v| v.as_u64()))
        .sum();

    let balance_zec = total_zat as f64 / 100_000_000.0;

    Ok(SyncResponse {
        success: true,
        balance_zec,
        notes_found: parsed.as_array().map(|a| a.len()).unwrap_or(0),
        message: format!("Balance: {:.8} ZEC", balance_zec),
    })
}
