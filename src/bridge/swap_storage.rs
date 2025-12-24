// Swap Storage
// Persists swap history and state

use crate::error::{NozyError, NozyResult};
use crate::paths::get_wallet_data_dir;
use crate::swap::types::{SwapStatus, SwapDirection};
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredSwap {
    pub swap_id: String,
    pub direction: SwapDirection,
    pub amount: f64,
    pub status: SwapStatus,
    pub from_address: String,
    pub to_address: String,
    pub created_at: u64,
    pub completed_at: Option<u64>,
    pub txid: Option<String>,
}

pub struct SwapStorage {
    swaps_file: PathBuf,
}

impl SwapStorage {
    pub fn new() -> NozyResult<Self> {
        let data_dir = get_wallet_data_dir();
        if !data_dir.exists() {
            fs::create_dir_all(&data_dir)
                .map_err(|e| NozyError::Storage(format!("Failed to create directory: {}", e)))?;
        }
        
        Ok(Self {
            swaps_file: data_dir.join("swaps.json"),
        })
    }
    
    pub fn load_swaps(&self) -> NozyResult<Vec<StoredSwap>> {
        if !self.swaps_file.exists() {
            return Ok(Vec::new());
        }
        
        let content = fs::read_to_string(&self.swaps_file)
            .map_err(|e| NozyError::Storage(format!("Failed to read swaps: {}", e)))?;
        
        let swaps: Vec<StoredSwap> = serde_json::from_str(&content)
            .map_err(|e| NozyError::Storage(format!("Failed to parse swaps: {}", e)))?;
        
        Ok(swaps)
    }
    
    pub fn save_swaps(&self, swaps: &[StoredSwap]) -> NozyResult<()> {
        let content = serde_json::to_string_pretty(swaps)
            .map_err(|e| NozyError::Storage(format!("Failed to serialize swaps: {}", e)))?;
        
        fs::write(&self.swaps_file, content)
            .map_err(|e| NozyError::Storage(format!("Failed to save swaps: {}", e)))?;
        
        Ok(())
    }
    
    pub fn add_swap(&self, swap: StoredSwap) -> NozyResult<()> {
        let mut swaps = self.load_swaps()?;
        swaps.push(swap);
        self.save_swaps(&swaps)?;
        Ok(())
    }
    
    pub fn update_swap(&self, swap_id: &str, status: SwapStatus, txid: Option<String>) -> NozyResult<()> {
        let mut swaps = self.load_swaps()?;
        
        if let Some(swap) = swaps.iter_mut().find(|s| s.swap_id == swap_id) {
            swap.status = status.clone();
            if let Some(txid) = txid {
                swap.txid = Some(txid);
            }
            if matches!(status, SwapStatus::Completed | SwapStatus::Failed | SwapStatus::Refunded) {
                swap.completed_at = Some(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                        .as_secs()
                );
            }
        } else {
            return Err(NozyError::Storage(format!("Swap {} not found", swap_id)));
        }
        
        self.save_swaps(&swaps)?;
        Ok(())
    }
    
    pub fn get_swap(&self, swap_id: &str) -> NozyResult<Option<StoredSwap>> {
        let swaps = self.load_swaps()?;
        Ok(swaps.into_iter().find(|s| s.swap_id == swap_id))
    }
    
    pub fn list_swaps(&self) -> NozyResult<Vec<StoredSwap>> {
        self.load_swaps()
    }
}
