use crate::error::NozyResult;
use crate::hd_wallet::HDWallet;
use crate::wallet_sync::{sync_wallet_notes, WalletSyncOptions, WalletSyncResult};
use crate::zebra_integration::ZebraClient;
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
        let options = WalletSyncOptions::api_default();
        let result = sync_wallet_notes(self.wallet.clone(), options).await?;

        Ok(SyncResult {
            notes_found: result.new_notes_in_scan,
            blocks_scanned: result.blocks_scanned as usize,
            new_balance: result.balance_zatoshis,
            last_scanned_height: result.last_scan_height,
        })
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
                            println!(
                                "🔄 Background sync: Found {} new notes in {} blocks",
                                result.notes_found, result.blocks_scanned
                            );
                        }
                    }
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

// Keep WalletSyncResult available for callers migrating off this wrapper.
pub type UnifiedSyncResult = WalletSyncResult;
