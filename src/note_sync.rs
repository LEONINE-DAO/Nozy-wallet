use crate::error::NozyResult;
use crate::zebra_integration::ZebraClient;
use crate::notes::{NoteScanner, SerializableOrchardNote};
use crate::hd_wallet::HDWallet;
use crate::config::{load_config, update_last_scan_height};
use crate::paths::get_wallet_data_dir;
use std::fs;
use tokio::time::{interval, Duration};

pub struct NoteSyncManager {
    wallet: HDWallet,
    zebra_client: ZebraClient,
    sync_interval: Duration,
}

impl NoteSyncManager {
    pub fn new(wallet: HDWallet, zebra_client: ZebraClient, sync_interval_secs: u64) -> Self {
        Self {
            wallet,
            zebra_client,
            sync_interval: Duration::from_secs(sync_interval_secs),
        }
    }

    pub async fn sync_once(&self) -> NozyResult<SyncResult> {
        let config = load_config();
        let start_height = if let Some(last_height) = config.last_scan_height {
            last_height + 1
        } else {
            3050000
        };

        let tip_height = self.zebra_client.get_block_count().await?;
        
        let end_height = (start_height + 1000).min(tip_height);
        
        if start_height > end_height {
            return Ok(SyncResult {
                notes_found: 0,
                blocks_scanned: 0,
                new_balance: 0,
                last_scanned_height: tip_height,
            });
        }

        let mut scanner = NoteScanner::new(self.wallet.clone(), self.zebra_client.clone());
        
        match scanner.scan_notes(Some(start_height), Some(end_height)).await {
            Ok((result, _spendable_notes)) => {
                let notes_dir = get_wallet_data_dir();
                let notes_path = notes_dir.join("notes.json");
                
                let mut existing_notes: Vec<SerializableOrchardNote> = if notes_path.exists() {
                    if let Ok(content) = fs::read_to_string(&notes_path) {
                        serde_json::from_str(&content).unwrap_or_default()
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                };
                
                let existing_nullifiers: std::collections::HashSet<Vec<u8>> = existing_notes.iter()
                    .map(|n| n.nullifier_bytes.clone())
                    .collect();
                
                let mut new_count = 0;
                for new_note in &result.notes {
                    if !existing_nullifiers.contains(&new_note.nullifier_bytes) {
                        existing_notes.push(new_note.clone());
                        new_count += 1;
                    }
                }
                
                if let Ok(serialized) = serde_json::to_string_pretty(&existing_notes) {
                    let _ = fs::write(&notes_path, serialized);
                }
                
                let _ = update_last_scan_height(end_height);
                
                let new_balance: u64 = existing_notes.iter()
                    .map(|n| n.value)
                    .sum();
                
                Ok(SyncResult {
                    notes_found: new_count,
                    blocks_scanned: (end_height - start_height + 1) as usize,
                    new_balance,
                    last_scanned_height: end_height,
                })
            },
            Err(e) => Err(e),
        }
    }

    pub fn start_background_sync(&self) -> tokio::task::JoinHandle<()> {
        let wallet = self.wallet.clone();
        let zebra_client = self.zebra_client.clone();
        let interval_duration = self.sync_interval;

        tokio::spawn(async move {
            let manager = NoteSyncManager::new(wallet, zebra_client, interval_duration.as_secs());
            let mut interval = interval(interval_duration);
            
            loop {
                interval.tick().await;
                
                match manager.sync_once().await {
                    Ok(result) => {
                        if result.notes_found > 0 {
                            println!("🔄 Background sync: Found {} new notes in {} blocks", 
                                result.notes_found, result.blocks_scanned);
                        }
                    },
                    Err(e) => {
                        eprintln!("⚠️  Background sync error: {}", e);
                    }
                }
            }
        })
    }
}

#[derive(Debug, Clone)]
pub struct SyncResult {
    pub notes_found: usize,
    pub blocks_scanned: usize,
    pub new_balance: u64,
    pub last_scanned_height: u32,
}

