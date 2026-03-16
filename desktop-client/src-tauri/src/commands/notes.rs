use crate::error::TauriError;
use nozy::paths::get_wallet_data_dir;
use serde::Serialize;
use tauri::command;
use std::fs;

#[derive(Debug, Serialize)]
pub struct NotesResponse {
    pub notes: Vec<serde_json::Value>,
    pub total_balance_zec: f64,
    pub count: usize,
}

#[command]
pub async fn get_notes() -> Result<NotesResponse, TauriError> {
    let notes_path = get_wallet_data_dir().join("notes.json");
    
    if !notes_path.exists() {
        return Ok(NotesResponse {
            notes: vec![],
            total_balance_zec: 0.0,
            count: 0,
        });
    }
    
    let content = fs::read_to_string(&notes_path)
        .map_err(|e| TauriError::from(e.to_string()))?;
    
    let notes: Vec<serde_json::Value> = serde_json::from_str(&content)
        .map_err(|e| TauriError::from(e.to_string()))?;
    
    let total_zat: u64 = notes
        .iter()
        .filter_map(|n| n.get("value").and_then(|v| v.as_u64()))
        .sum();
    
    let total_balance_zec = total_zat as f64 / 100_000_000.0;
    
    Ok(NotesResponse {
        notes: notes.clone(),
        total_balance_zec,
        count: notes.len(),
    })
}

