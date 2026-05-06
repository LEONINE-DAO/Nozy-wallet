use crate::commands::wallet::get_unlocked_wallet;
use crate::error::{TauriError, TauriResult};
use nozy::{load_config, NoteScanner, ZebraClient};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceResponse {
    pub balance: f64,
    pub verified_balance: f64,
}

#[tauri::command]
pub async fn get_balance() -> TauriResult<BalanceResponse> {
    let wallet = get_unlocked_wallet()?;
    let config = load_config();
    let zebra_client = ZebraClient::from_config(&config);
    
    let mut note_scanner = NoteScanner::new(wallet.clone(), zebra_client.clone());
    
    // Get current block height
    let tip_height = zebra_client
        .get_block_count()
        .await
        .map_err(|e| TauriError::Network(format!("Failed to get block height: {}", e)))?;
    
    // Scan for notes (use last 10k blocks for quick scan)
    let start_height = tip_height.saturating_sub(10_000);
    
    let (scan_result, _spendable) = note_scanner
        .scan_notes(Some(start_height), Some(tip_height))
        .await
        .map_err(|e| TauriError::Network(format!("Failed to scan notes: {}", e)))?;
    
    // Orchard-only product balance (Orchard notes in scan result)
    let total_balance_zatoshis = scan_result.total_balance;
    let total_balance_zec = total_balance_zatoshis as f64 / 100_000_000.0;
    
    // For verified balance, we'd need to check confirmation status
    // For now, use total balance as verified
    Ok(BalanceResponse {
        balance: total_balance_zec,
        verified_balance: total_balance_zec,
    })
}
