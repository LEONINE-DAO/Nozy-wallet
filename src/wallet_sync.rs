//! Unified wallet note sync: scan → merge → persist `notes.json` → balance.
//!
//! See [`docs/rfcs/WALLET_SYNC_UNIFIED_ARCHITECTURE.md`](../docs/rfcs/WALLET_SYNC_UNIFIED_ARCHITECTURE.md).

use crate::config::{load_config, update_last_scan_height, WalletConfig};
use crate::error::{NozyError, NozyResult};
use crate::hd_wallet::HDWallet;
use crate::notes::{
    load_wallet_notes, merge_scanned_notes, save_wallet_notes, wallet_unspent_balance_zatoshis,
    NoteScanner,
};
use crate::zebra_integration::ZebraClient;

/// Default Orchard-heavy mainnet scan floor when no `last_scan_height` exists.
pub const MAINNET_DEFAULT_SCAN_START: u32 = 3_050_000;

/// Default incremental scan window when not scanning to tip.
pub const DEFAULT_INCREMENTAL_BATCH: u32 = 1_000;

#[derive(Debug, Clone)]
pub struct WalletSyncOptions {
    pub start_height: Option<u32>,
    pub end_height: Option<u32>,
    /// Scan through chain tip when `end_height` is unset (CLI `--to-tip`).
    pub scan_to_tip: bool,
    /// Max blocks per incremental sync when not scanning to tip.
    pub incremental_batch: u32,
    pub zebra_url: Option<String>,
}

impl Default for WalletSyncOptions {
    fn default() -> Self {
        Self {
            start_height: None,
            end_height: None,
            scan_to_tip: false,
            incremental_batch: DEFAULT_INCREMENTAL_BATCH,
            zebra_url: None,
        }
    }
}

impl WalletSyncOptions {
    /// API-server style: incremental chunk from `last_scan_height + 1`.
    pub fn api_default() -> Self {
        Self::default()
    }

    /// CLI `--to-tip`.
    pub fn to_tip() -> Self {
        Self {
            scan_to_tip: true,
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanRange {
    pub scan_start: u32,
    pub scan_end: u32,
    pub chain_tip: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalletSyncResult {
    pub balance_zatoshis: u64,
    pub unspent_notes: usize,
    pub total_notes: usize,
    pub new_notes_in_scan: usize,
    pub scan_start: u32,
    pub scan_end: u32,
    pub chain_tip: u32,
    pub blocks_scanned: u32,
    pub last_scan_height: u32,
}

/// Resolve inclusive scan bounds from config + options (no network I/O).
pub fn resolve_scan_range(
    config: &WalletConfig,
    options: &WalletSyncOptions,
    chain_tip: u32,
) -> Result<ScanRange, NozyError> {
    let default_first_scan_start = if config.network == "testnet" {
        1
    } else {
        MAINNET_DEFAULT_SCAN_START
    };

    let effective_start = options
        .start_height
        .or_else(|| config.last_scan_height.map(|h| h.saturating_add(1)));

    let scan_start = effective_start.unwrap_or(default_first_scan_start);

    let batch = options.incremental_batch.max(1);
    let requested_end = if let Some(end) = options.end_height {
        end.max(scan_start)
    } else if options.scan_to_tip || options.start_height.is_some() {
        chain_tip
    } else {
        scan_start.saturating_add(batch)
    };

    let scan_end = requested_end.min(chain_tip);

    if scan_start > scan_end {
        return Err(NozyError::InvalidOperation(format!(
            "start height ({scan_start}) is above effective end height ({scan_end}, zebrad tip {chain_tip}). \
             Wait for zebrad to pass your wallet birthday, use a snapshot, or pass a lower start height."
        )));
    }

    Ok(ScanRange {
        scan_start,
        scan_end,
        chain_tip,
    })
}

/// Scan Orchard notes over RPC, merge into cached `notes.json`, update `last_scan_height`.
pub async fn sync_wallet_notes(
    wallet: HDWallet,
    options: WalletSyncOptions,
) -> NozyResult<WalletSyncResult> {
    let mut config = load_config();
    if let Some(url) = options.zebra_url.clone() {
        config.zebra_url = url;
    }

    let zebra_client = ZebraClient::from_config(&config);
    let chain_tip = zebra_client.get_block_count().await?;
    let range = resolve_scan_range(&config, &options, chain_tip)?;

    let notes_before = load_wallet_notes()?;
    let total_before = notes_before.len();

    let mut note_scanner = NoteScanner::new(wallet, zebra_client);
    let (scan_result, _spendable) = note_scanner
        .scan_notes(Some(range.scan_start), Some(range.scan_end))
        .await?;

    let mut cached_notes = notes_before;
    merge_scanned_notes(&mut cached_notes, &scan_result.notes);
    save_wallet_notes(&cached_notes)?;

    let _ = update_last_scan_height(range.scan_end);

    let balance_zatoshis = wallet_unspent_balance_zatoshis(&cached_notes);
    let unspent_notes = cached_notes.iter().filter(|n| !n.spent).count();
    let new_notes_in_scan = cached_notes.len().saturating_sub(total_before);
    let blocks_scanned = range
        .scan_end
        .saturating_sub(range.scan_start)
        .saturating_add(1);

    Ok(WalletSyncResult {
        balance_zatoshis,
        unspent_notes,
        total_notes: cached_notes.len(),
        new_notes_in_scan,
        scan_start: range.scan_start,
        scan_end: range.scan_end,
        chain_tip: range.chain_tip,
        blocks_scanned,
        last_scan_height: range.scan_end,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::WalletConfig;

    fn test_config(last_scan: Option<u32>, network: &str) -> WalletConfig {
        WalletConfig {
            last_scan_height: last_scan,
            network: network.to_string(),
            ..WalletConfig::default()
        }
    }

    #[test]
    fn incremental_caps_at_batch_size() {
        let config = test_config(Some(3_050_000), "mainnet");
        let opts = WalletSyncOptions::default();
        let range = resolve_scan_range(&config, &opts, 3_060_000).unwrap();
        assert_eq!(range.scan_start, 3_050_001);
        assert_eq!(range.scan_end, 3_051_001);
    }

    #[test]
    fn incremental_respects_chain_tip() {
        let config = test_config(Some(3_050_000), "mainnet");
        let opts = WalletSyncOptions::default();
        let range = resolve_scan_range(&config, &opts, 3_050_500).unwrap();
        assert_eq!(range.scan_end, 3_050_500);
    }

    #[test]
    fn scan_to_tip_reaches_chain_tip() {
        let config = test_config(Some(3_050_000), "mainnet");
        let opts = WalletSyncOptions::to_tip();
        let range = resolve_scan_range(&config, &opts, 3_060_000).unwrap();
        assert_eq!(range.scan_end, 3_060_000);
    }

    #[test]
    fn rejects_start_above_end() {
        let config = test_config(Some(3_100_000), "mainnet");
        let opts = WalletSyncOptions::default();
        assert!(resolve_scan_range(&config, &opts, 3_050_000).is_err());
    }
}
