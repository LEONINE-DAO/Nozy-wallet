use crate::commands::wallet::get_unlocked_wallet;
use crate::error::{TauriError, TauriResult};
use nozy::{load_config, NoteScanner, ZebraClient};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncRequest {
    pub start_height: Option<u32>,
    pub end_height: Option<u32>,
    pub zebra_url: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncResponse {
    pub success: bool,
    pub message: String,
    pub notes_found: usize,
    pub balance: f64,
}

#[tauri::command]
pub async fn sync_wallet(request: SyncRequest) -> TauriResult<SyncResponse> {
    let wallet = get_unlocked_wallet()?;
    
    let mut config = load_config();
    if let Some(url) = request.zebra_url {
        config.zebra_url = url;
    }
    
    let zebra_client = ZebraClient::from_config(&config);
    
    // Get current block height
    let tip_height = zebra_client
        .get_block_count()
        .await
        .map_err(|e| TauriError::Network(format!("Failed to get block height: {}", e)))?;
    
    let start_height = request.start_height.unwrap_or_else(|| {
        // Default to last 10k blocks for quick sync
        tip_height.saturating_sub(10_000)
    });
    
    let end_height = request.end_height.unwrap_or(tip_height);
    
    let mut note_scanner = NoteScanner::new(wallet, zebra_client.clone());
    
    let (scan_result, _spendable) = note_scanner
        .scan_notes(Some(start_height), Some(end_height))
        .await
        .map_err(|e| TauriError::Network(format!("Failed to sync wallet: {}", e)))?;
    
    let balance_zec = scan_result.total_balance as f64 / 100_000_000.0;
    
    Ok(SyncResponse {
        success: true,
        message: format!(
            "Sync complete! Scanned blocks {} to {}. Found {} notes.",
            start_height, end_height, scan_result.notes.len()
        ),
        notes_found: scan_result.notes.len(),
        balance: balance_zec,
    })
}
