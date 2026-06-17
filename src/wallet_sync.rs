//! Unified wallet note sync: scan → merge → persist `notes.json` → balance.
//!
//! See [`docs/rfcs/WALLET_SYNC_UNIFIED_ARCHITECTURE.md`](../docs/rfcs/WALLET_SYNC_UNIFIED_ARCHITECTURE.md).

use crate::config::{load_config, update_last_scan_height, WalletConfig};
use crate::error::NozyError;
use crate::hd_wallet::HDWallet;
use crate::notes::{
    load_wallet_notes, merge_scanned_notes, save_wallet_notes, wallet_unspent_balance_zatoshis,
    NoteScanner,
};
use crate::zebra_integration::ZebraClient;
use serde::{Deserialize, Serialize};

/// Phase of [`sync_wallet_notes`] where a failure occurred (for API integrators).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WalletSyncPhase {
    Connect,
    ResolveRange,
    LoadNotes,
    Scan,
    Persist,
    Checkpoint,
}

impl WalletSyncPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Connect => "connect",
            Self::ResolveRange => "resolve_range",
            Self::LoadNotes => "load_notes",
            Self::Scan => "scan",
            Self::Persist => "persist",
            Self::Checkpoint => "checkpoint",
        }
    }

    fn error_code(self) -> &'static str {
        match self {
            Self::Connect => "SYNC_CONNECT_FAILED",
            Self::ResolveRange => "SYNC_RANGE_FAILED",
            Self::LoadNotes => "SYNC_LOAD_NOTES_FAILED",
            Self::Scan => "SYNC_SCAN_FAILED",
            Self::Persist => "SYNC_PERSIST_FAILED",
            Self::Checkpoint => "SYNC_CHECKPOINT_FAILED",
        }
    }
}

/// Structured sync failure with scan context for HTTP clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletSyncError {
    pub phase: WalletSyncPhase,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scan_start: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scan_end: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_tip: Option<u32>,
}

impl WalletSyncError {
    pub fn is_zebra_unavailable(&self) -> bool {
        crate::cli_helpers::is_zebra_unavailable_error(&self.message)
    }

    pub fn api_status_code(&self) -> u16 {
        if self.is_zebra_unavailable() {
            503
        } else {
            500
        }
    }

    pub fn api_code(&self) -> &str {
        if self.phase == WalletSyncPhase::Connect {
            crate::cli_helpers::zebra_connect_api_code(&self.message)
        } else if self.is_zebra_unavailable() {
            "ZEBRA_UNAVAILABLE"
        } else {
            self.phase.error_code()
        }
    }

    pub fn to_api_json(&self) -> serde_json::Value {
        serde_json::json!({
            "error": self.message,
            "code": self.api_code(),
            "phase": self.phase.as_str(),
            "block_height": self.block_height,
            "scan_start": self.scan_start,
            "scan_end": self.scan_end,
            "chain_tip": self.chain_tip,
        })
    }

    fn with_range(
        phase: WalletSyncPhase,
        source: NozyError,
        block_height: Option<u32>,
        scan_start: Option<u32>,
        scan_end: Option<u32>,
        chain_tip: Option<u32>,
    ) -> Self {
        Self {
            phase,
            message: source.to_string(),
            block_height: block_height.or_else(|| source.scan_block_height()),
            scan_start,
            scan_end,
            chain_tip,
        }
    }
}

impl std::fmt::Display for WalletSyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for WalletSyncError {}

struct SyncRangeContext {
    scan_start: u32,
    scan_end: u32,
    chain_tip: u32,
}

impl SyncRangeContext {
    fn from_range(range: &ScanRange) -> Self {
        Self {
            scan_start: range.scan_start,
            scan_end: range.scan_end,
            chain_tip: range.chain_tip,
        }
    }

    fn scan_fields(&self) -> (Option<u32>, Option<u32>, Option<u32>) {
        (
            Some(self.scan_start),
            Some(self.scan_end),
            Some(self.chain_tip),
        )
    }
}

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
    /// True when the wallet is already caught up to chain tip (no blocks scanned).
    pub already_synced: bool,
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
) -> Result<WalletSyncResult, WalletSyncError> {
    let mut config = load_config();
    if let Some(url) = options.zebra_url.clone() {
        config.zebra_url = url;
    }

    let zebra_client = ZebraClient::from_config(&config);
    let chain_tip = zebra_client.get_block_count().await.map_err(|e| {
        WalletSyncError::with_range(WalletSyncPhase::Connect, e, None, None, None, None)
    })?;
    let range = resolve_scan_range(&config, &options, chain_tip).map_err(|e| {
        WalletSyncError::with_range(
            WalletSyncPhase::ResolveRange,
            e,
            None,
            None,
            None,
            Some(chain_tip),
        )
    })?;
    let ctx = SyncRangeContext::from_range(&range);
    let (scan_start, scan_end, chain_tip_opt) = ctx.scan_fields();

    let notes_before = load_wallet_notes().map_err(|e| {
        WalletSyncError::with_range(
            WalletSyncPhase::LoadNotes,
            e,
            None,
            scan_start,
            scan_end,
            chain_tip_opt,
        )
    })?;
    let total_before = notes_before.len();
    let balance_before = wallet_unspent_balance_zatoshis(&notes_before);

    // Already caught up: return cached balance without re-scanning or rewriting notes.json.
    if range.scan_start > range.scan_end {
        let last_scan_height = config.last_scan_height.unwrap_or(range.chain_tip);
        return Ok(WalletSyncResult {
            balance_zatoshis: balance_before,
            unspent_notes: notes_before.iter().filter(|n| !n.spent).count(),
            total_notes: notes_before.len(),
            new_notes_in_scan: 0,
            scan_start: range.scan_start,
            scan_end: range.scan_end,
            chain_tip: range.chain_tip,
            blocks_scanned: 0,
            last_scan_height,
            already_synced: true,
        });
    }

    let mut note_scanner = NoteScanner::new(wallet, zebra_client);
    let (scan_result, _spendable) = note_scanner
        .scan_notes(Some(range.scan_start), Some(range.scan_end))
        .await
        .map_err(|e| {
            WalletSyncError::with_range(
                WalletSyncPhase::Scan,
                e,
                None,
                scan_start,
                scan_end,
                chain_tip_opt,
            )
        })?;

    let mut cached_notes = notes_before;
    merge_scanned_notes(&mut cached_notes, &scan_result.notes);
    save_wallet_notes(&cached_notes).map_err(|e| {
        WalletSyncError::with_range(
            WalletSyncPhase::Persist,
            e,
            None,
            scan_start,
            scan_end,
            chain_tip_opt,
        )
    })?;

    update_last_scan_height(range.scan_end).map_err(|e| {
        WalletSyncError::with_range(
            WalletSyncPhase::Checkpoint,
            e,
            None,
            scan_start,
            scan_end,
            chain_tip_opt,
        )
    })?;

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
        already_synced: false,
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
    fn already_at_tip_yields_empty_scan_range() {
        let config = test_config(Some(3_100_000), "mainnet");
        let opts = WalletSyncOptions::default();
        let range = resolve_scan_range(&config, &opts, 3_050_000).unwrap();
        assert!(range.scan_start > range.scan_end);
    }

    #[test]
    fn scan_error_includes_block_height_in_api_json() {
        let err = WalletSyncError::with_range(
            WalletSyncPhase::Scan,
            NozyError::ScanAtBlock {
                height: 3_050_123,
                detail: "witness append failed".to_string(),
            },
            None,
            Some(3_050_000),
            Some(3_051_000),
            Some(3_060_000),
        );
        let json = err.to_api_json();
        assert_eq!(json["phase"], "scan");
        assert_eq!(json["block_height"], 3_050_123);
        assert_eq!(json["code"], "SYNC_SCAN_FAILED");
        assert_eq!(json["scan_start"], 3_050_000);
    }

    #[test]
    fn merge_preserves_cached_notes_across_incremental_scans() {
        use crate::notes::{merge_scanned_notes, SerializableOrchardNote};

        let mut cached = vec![SerializableOrchardNote {
            note_bytes: vec![1],
            value: 250_000,
            address_bytes: vec![0; 43],
            nullifier_bytes: vec![1; 32],
            block_height: 3_050_500,
            txid: "abc".to_string(),
            spent: false,
            memo: vec![],
            orchard_incremental_witness_hex: None,
            orchard_witness_tip_height: None,
            rho_bytes: None,
            rseed_bytes: None,
        }];

        let scanned: Vec<SerializableOrchardNote> = vec![];
        merge_scanned_notes(&mut cached, &scanned);
        assert_eq!(cached.len(), 1);
        assert_eq!(cached[0].value, 250_000);
    }
}
