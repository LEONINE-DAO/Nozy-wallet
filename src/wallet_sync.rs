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
use crate::orchard_tx::refresh_cached_witnesses_to_tip;
use crate::send_readiness::{max_serialized_witness_lag_blocks, MAX_SEND_WITNESS_LAG_BLOCKS};
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
    /// How the Zebra client reached (or attempted) the node — useful when connect fails.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_mode: Option<String>,
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
            "connection_mode": self.connection_mode,
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
        Self::with_context(
            phase,
            source,
            block_height,
            scan_start,
            scan_end,
            chain_tip,
            None,
        )
    }

    fn with_connect(source: NozyError, connection_mode: &str) -> Self {
        Self::with_context(
            WalletSyncPhase::Connect,
            source,
            None,
            None,
            None,
            None,
            Some(connection_mode.to_string()),
        )
    }

    fn with_context(
        phase: WalletSyncPhase,
        source: NozyError,
        block_height: Option<u32>,
        scan_start: Option<u32>,
        scan_end: Option<u32>,
        chain_tip: Option<u32>,
        connection_mode: Option<String>,
    ) -> Self {
        Self {
            phase,
            message: source.to_string(),
            block_height: block_height.or_else(|| source.scan_block_height()),
            scan_start,
            scan_end,
            chain_tip,
            connection_mode,
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

fn default_first_scan_start(config: &WalletConfig) -> u32 {
    if config.network == "testnet" {
        1
    } else {
        MAINNET_DEFAULT_SCAN_START
    }
}

/// When `notes.json` is empty, scan full history to tip instead of trusting
/// `last_scan_height` (avoids "already synced" with zero balance).
pub(crate) fn apply_empty_cache_backfill(
    range: &mut ScanRange,
    config: &WalletConfig,
    options: &WalletSyncOptions,
    notes_before: &[crate::notes::SerializableOrchardNote],
) {
    if !notes_before.is_empty() {
        return;
    }
    // User-scoped windows must not expand; plain `sync --to-tip` with an empty cache still needs
    // a full backfill (otherwise only the incremental tail is scanned and balance stays zero).
    if options.start_height.is_some() || options.end_height.is_some() {
        return;
    }
    range.scan_start = default_first_scan_start(config);
    range.scan_end = range.chain_tip;
}

/// Highest known height from cached notes (discovery block or witness tip).
pub(crate) fn notes_cache_checkpoint_height(
    notes: &[crate::notes::SerializableOrchardNote],
) -> Option<u32> {
    notes
        .iter()
        .map(|n| {
            n.block_height
                .max(n.orchard_witness_tip_height.unwrap_or(0))
                .max(n.ironwood_witness_tip_height.unwrap_or(0))
        })
        .max()
}

fn note_has_pool_witness(note: &crate::notes::SerializableOrchardNote) -> bool {
    note.witness_hex_for_pool()
        .is_some_and(|hex| !hex.is_empty())
}

/// Earliest unspent note that still needs a pool witness rebuilt via RPC scan.
pub(crate) fn unwitnessed_unspent_floor(
    notes: &[crate::notes::SerializableOrchardNote],
) -> Option<u32> {
    notes
        .iter()
        .filter(|n| !n.spent && !note_has_pool_witness(n))
        .map(|n| n.block_height)
        .min()
}

/// When notes exist but `last_scan_height` is missing, resume after the note cache
/// instead of re-scanning from the network default floor (testnet height 1).
///
/// Unspent notes without a persisted pool witness must be rescanned from their
/// discovery height — witness refresh cannot bootstrap from an empty hex blob.
pub(crate) fn apply_cached_notes_resume(
    range: &mut ScanRange,
    config: &WalletConfig,
    options: &WalletSyncOptions,
    notes_before: &[crate::notes::SerializableOrchardNote],
) {
    if options.start_height.is_some() || notes_before.is_empty() {
        return;
    }

    if let Some(floor) = unwitnessed_unspent_floor(notes_before) {
        if floor < range.scan_start {
            range.scan_start = floor;
        }
        // Bound each round so witness rebuild can persist progress and the UI can update.
        let batch = options.incremental_batch.max(1);
        if options.end_height.is_none() {
            range.scan_end = range.scan_start.saturating_add(batch).min(range.chain_tip);
        }
        return;
    }

    if config.last_scan_height.is_some() {
        return;
    }
    let Some(checkpoint) = notes_cache_checkpoint_height(notes_before) else {
        return;
    };
    let resume = checkpoint.saturating_add(1);
    if resume > range.scan_start {
        range.scan_start = resume;
    }
    if !options.scan_to_tip && options.end_height.is_none() {
        let batch = options.incremental_batch.max(1);
        range.scan_end = range.scan_start.saturating_add(batch).min(range.chain_tip);
    } else if options.scan_to_tip && options.end_height.is_none() {
        range.scan_end = range.chain_tip;
    }
}

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
    let effective_start = options
        .start_height
        .or_else(|| config.last_scan_height.map(|h| h.saturating_add(1)));

    let scan_start = effective_start.unwrap_or_else(|| default_first_scan_start(config));

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
    let config = load_config();
    let zebra_client = ZebraClient::from_config_with_url(&config, options.zebra_url.as_deref());
    let connection_mode = zebra_client.connection_mode().as_str();
    let chain_tip = zebra_client
        .get_block_count()
        .await
        .map_err(|e| WalletSyncError::with_connect(e, connection_mode))?;
    let mut range = resolve_scan_range(&config, &options, chain_tip).map_err(|e| {
        WalletSyncError::with_range(
            WalletSyncPhase::ResolveRange,
            e,
            None,
            None,
            None,
            Some(chain_tip),
        )
    })?;

    let notes_before = load_wallet_notes().map_err(|e| {
        WalletSyncError::with_range(
            WalletSyncPhase::LoadNotes,
            e,
            None,
            None,
            None,
            Some(chain_tip),
        )
    })?;
    apply_empty_cache_backfill(&mut range, &config, &options, &notes_before);
    apply_cached_notes_resume(&mut range, &config, &options, &notes_before);

    let ctx = SyncRangeContext::from_range(&range);
    let (scan_start, scan_end, chain_tip_opt) = ctx.scan_fields();
    let total_before = notes_before.len();

    // Block scan caught up but Orchard witnesses may still lag — refresh witnesses to tip.
    if range.scan_start > range.scan_end {
        return finish_caught_up_sync(&zebra_client, notes_before, &range, &config, 0).await;
    }

    let notes_path = crate::paths::get_wallet_data_dir().join("notes.json");
    let mut note_scanner = if notes_path.exists() {
        NoteScanner::with_index_file(wallet, zebra_client.clone(), &notes_path).map_err(|e| {
            WalletSyncError::with_range(
                WalletSyncPhase::LoadNotes,
                e,
                None,
                scan_start,
                scan_end,
                chain_tip_opt,
            )
        })?
    } else {
        NoteScanner::new(wallet, zebra_client.clone())
    };
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
    if cached_notes.is_empty() && total_before > 0 {
        return Err(WalletSyncError::with_range(
            WalletSyncPhase::Persist,
            NozyError::Storage(
                "sync would overwrite a non-empty note cache with zero notes; refusing to persist. \
                 Re-run with `nozy sync --start-height 3050000 --to-tip` to rebuild the cache."
                    .to_string(),
            ),
            None,
            scan_start,
            scan_end,
            chain_tip_opt,
        ));
    }
    refresh_and_persist_witnesses(&zebra_client, &mut cached_notes, range.scan_end)
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
    let _ = crate::wallet_profiles::touch_active_profile_scan_height(range.scan_end);

    let new_notes_in_scan = cached_notes.len().saturating_sub(total_before);
    let blocks_scanned = range
        .scan_end
        .saturating_sub(range.scan_start)
        .saturating_add(1);

    cached_notes = reload_wallet_notes()?;

    let balance_zatoshis = wallet_unspent_balance_zatoshis(&cached_notes);
    let unspent_notes = cached_notes.iter().filter(|n| !n.spent).count();

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

async fn refresh_and_persist_witnesses(
    zebra_client: &ZebraClient,
    notes: &mut [crate::notes::SerializableOrchardNote],
    target_height: u32,
) -> NozyResult<u32> {
    let orchard_refreshed =
        refresh_cached_witnesses_to_tip(zebra_client, notes, target_height).await?;
    let ironwood_refreshed = crate::ironwood_tx::refresh_ironwood_cached_witnesses_to_tip(
        zebra_client,
        notes,
        target_height,
    )
    .await?;
    Ok(orchard_refreshed + ironwood_refreshed)
}

/// Target height for witness catch-up when RPC scan is already at tip.
///
/// Large witness lag is advanced in [`DEFAULT_INCREMENTAL_BATCH`] chunks so API sync
/// calls stay bounded instead of fetching thousands of blocks in one request.
fn witness_catchup_target_height(
    notes: &[crate::notes::SerializableOrchardNote],
    chain_tip: u32,
) -> u32 {
    let lag = max_serialized_witness_lag_blocks(notes, chain_tip);
    if lag == 0 {
        return chain_tip;
    }
    let batch = DEFAULT_INCREMENTAL_BATCH.max(1);
    if lag <= batch {
        return chain_tip;
    }
    let min_stored = notes
        .iter()
        .filter(|n| {
            !n.spent
                && n.orchard_incremental_witness_hex
                    .as_ref()
                    .is_some_and(|h| !h.is_empty())
        })
        .map(|n| n.orchard_witness_tip_height.unwrap_or(0))
        .min()
        .unwrap_or(0);
    min_stored.saturating_add(batch).min(chain_tip)
}

/// When RPC scan is caught up, advance Orchard witnesses on cached notes and report send readiness.
async fn finish_caught_up_sync(
    zebra_client: &ZebraClient,
    notes_before: Vec<crate::notes::SerializableOrchardNote>,
    range: &ScanRange,
    config: &WalletConfig,
    blocks_scanned: u32,
) -> Result<WalletSyncResult, WalletSyncError> {
    let witness_lag_before = max_serialized_witness_lag_blocks(&notes_before, range.chain_tip);
    let mut notes_mut = notes_before;
    if witness_lag_before > 0 {
        let witness_target = witness_catchup_target_height(&notes_mut, range.chain_tip);
        refresh_and_persist_witnesses(zebra_client, &mut notes_mut, witness_target)
            .await
            .map_err(|e| {
                WalletSyncError::with_range(
                    WalletSyncPhase::Scan,
                    e,
                    None,
                    None,
                    None,
                    Some(range.chain_tip),
                )
            })?;
        save_wallet_notes(&notes_mut).map_err(|e| {
            WalletSyncError::with_range(
                WalletSyncPhase::Persist,
                e,
                None,
                None,
                None,
                Some(range.chain_tip),
            )
        })?;
    }

    let notes_after_repair = reload_wallet_notes()?;
    let last_scan_height = config.last_scan_height.unwrap_or(range.chain_tip);
    let witness_lag_after = max_serialized_witness_lag_blocks(&notes_after_repair, range.chain_tip);
    let scan_caught_up = last_scan_height >= range.chain_tip;
    let already_synced = scan_caught_up && witness_lag_after <= MAX_SEND_WITNESS_LAG_BLOCKS;

    Ok(WalletSyncResult {
        balance_zatoshis: wallet_unspent_balance_zatoshis(&notes_after_repair),
        unspent_notes: notes_after_repair.iter().filter(|n| !n.spent).count(),
        total_notes: notes_after_repair.len(),
        new_notes_in_scan: 0,
        scan_start: range.scan_start,
        scan_end: range.scan_end,
        chain_tip: range.chain_tip,
        blocks_scanned,
        last_scan_height,
        already_synced,
    })
}

fn reload_wallet_notes() -> Result<Vec<crate::notes::SerializableOrchardNote>, WalletSyncError> {
    load_wallet_notes().map_err(|e| {
        WalletSyncError::with_range(WalletSyncPhase::LoadNotes, e, None, None, None, None)
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
    fn witness_catchup_batches_large_lag() {
        use crate::notes::SerializableOrchardNote;

        let notes = vec![SerializableOrchardNote {
            note_bytes: vec![1],
            value: 250_000,
            address_bytes: vec![0; 43],
            block_height: 3_379_050,
            nullifier_bytes: vec![2; 32],
            txid: "abc".to_string(),
            spent: false,
            memo: vec![],
            orchard_incremental_witness_hex: Some("ab".to_string()),
            orchard_witness_tip_height: Some(3_379_050),
            ironwood_incremental_witness_hex: None,
            ironwood_witness_tip_height: None,
            rho_bytes: None,
            rseed_bytes: None,
            spent_in_txid: None,
            pool: crate::shielded_pool::ShieldedPool::Orchard,
        }];
        let chain_tip = 3_389_822;
        let target = witness_catchup_target_height(&notes, chain_tip);
        assert_eq!(target, 3_379_050 + DEFAULT_INCREMENTAL_BATCH);
        assert!(target < chain_tip);
    }

    #[test]
    fn witness_catchup_reaches_tip_when_lag_within_batch() {
        use crate::notes::SerializableOrchardNote;

        let notes = vec![SerializableOrchardNote {
            note_bytes: vec![1],
            value: 250_000,
            address_bytes: vec![0; 43],
            block_height: 3_389_500,
            nullifier_bytes: vec![2; 32],
            txid: "abc".to_string(),
            spent: false,
            memo: vec![],
            orchard_incremental_witness_hex: Some("ab".to_string()),
            orchard_witness_tip_height: Some(3_389_500),
            ironwood_incremental_witness_hex: None,
            ironwood_witness_tip_height: None,
            rho_bytes: None,
            rseed_bytes: None,
            spent_in_txid: None,
            pool: crate::shielded_pool::ShieldedPool::Orchard,
        }];
        let chain_tip = 3_389_822;
        assert_eq!(witness_catchup_target_height(&notes, chain_tip), chain_tip);
    }

    #[test]
    fn empty_cache_backfills_when_checkpoint_at_tip() {
        let config = test_config(Some(3_380_000), "mainnet");
        let opts = WalletSyncOptions::default();
        let mut range = resolve_scan_range(&config, &opts, 3_380_000).unwrap();
        assert!(range.scan_start > range.scan_end);
        apply_empty_cache_backfill(&mut range, &config, &opts, &[]);
        assert_eq!(range.scan_start, MAINNET_DEFAULT_SCAN_START);
        assert_eq!(range.scan_end, 3_380_000);
    }

    #[test]
    fn empty_cache_to_tip_backfills_from_mainnet_default() {
        let config = test_config(Some(3_380_000), "mainnet");
        let opts = WalletSyncOptions::to_tip();
        let mut range = resolve_scan_range(&config, &opts, 3_380_000).unwrap();
        assert_eq!(range.scan_end, 3_380_000);
        apply_empty_cache_backfill(&mut range, &config, &opts, &[]);
        assert_eq!(range.scan_start, MAINNET_DEFAULT_SCAN_START);
        assert_eq!(range.scan_end, 3_380_000);
    }

    #[test]
    fn empty_cache_backfill_skipped_when_notes_exist() {
        use crate::notes::SerializableOrchardNote;

        let config = test_config(Some(3_380_000), "mainnet");
        let opts = WalletSyncOptions::default();
        let mut range = resolve_scan_range(&config, &opts, 3_380_000).unwrap();
        let cached = vec![SerializableOrchardNote {
            note_bytes: vec![1],
            value: 250_000,
            address_bytes: vec![0; 43],
            block_height: 3_100_000,
            nullifier_bytes: vec![2; 32],
            txid: "abc".to_string(),
            spent: false,
            memo: vec![],
            orchard_incremental_witness_hex: None,
            orchard_witness_tip_height: None,
            ironwood_incremental_witness_hex: None,
            ironwood_witness_tip_height: None,
            rho_bytes: None,
            rseed_bytes: None,
            spent_in_txid: None,
            pool: crate::shielded_pool::ShieldedPool::Orchard,
        }];
        apply_empty_cache_backfill(&mut range, &config, &opts, &cached);
        assert!(range.scan_start > range.scan_end);
    }

    #[test]
    fn cached_notes_resume_when_last_scan_missing() {
        use crate::notes::SerializableOrchardNote;

        let config = test_config(None, "testnet");
        let opts = WalletSyncOptions::to_tip();
        let mut range = resolve_scan_range(&config, &opts, 4_159_000).unwrap();
        assert_eq!(range.scan_start, 1);
        let cached = vec![SerializableOrchardNote {
            note_bytes: vec![1],
            value: 250_000,
            address_bytes: vec![0; 43],
            block_height: 4_100_000,
            nullifier_bytes: vec![2; 32],
            txid: "abc".to_string(),
            spent: false,
            memo: vec![],
            orchard_incremental_witness_hex: None,
            orchard_witness_tip_height: Some(4_120_000),
            ironwood_incremental_witness_hex: None,
            ironwood_witness_tip_height: None,
            rho_bytes: None,
            rseed_bytes: None,
            spent_in_txid: None,
            pool: crate::shielded_pool::ShieldedPool::Orchard,
        }];
        apply_cached_notes_resume(&mut range, &config, &opts, &cached);
        assert_eq!(range.scan_start, 4_120_001);
        assert_eq!(range.scan_end, 4_159_000);
    }

    #[test]
    fn cached_notes_rescan_unwitnessed_unspent_even_with_checkpoint() {
        use crate::notes::SerializableOrchardNote;

        let config = test_config(Some(4_159_000), "testnet");
        let opts = WalletSyncOptions::to_tip();
        let mut range = resolve_scan_range(&config, &opts, 4_159_100).unwrap();
        assert_eq!(range.scan_start, 4_159_001);
        let cached = vec![SerializableOrchardNote {
            note_bytes: vec![1],
            value: 250_000,
            address_bytes: vec![0; 43],
            block_height: 4_143_641,
            nullifier_bytes: vec![2; 32],
            txid: "abc".to_string(),
            spent: false,
            memo: vec![],
            orchard_incremental_witness_hex: None,
            orchard_witness_tip_height: None,
            ironwood_incremental_witness_hex: None,
            ironwood_witness_tip_height: None,
            rho_bytes: None,
            rseed_bytes: None,
            spent_in_txid: None,
            pool: crate::shielded_pool::ShieldedPool::Ironwood,
        }];
        apply_cached_notes_resume(&mut range, &config, &opts, &cached);
        assert_eq!(range.scan_start, 4_143_641);
        assert_eq!(range.scan_end, 4_144_641);
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
            ironwood_incremental_witness_hex: None,
            ironwood_witness_tip_height: None,
            rho_bytes: None,
            rseed_bytes: None,
            spent_in_txid: None,
            pool: crate::shielded_pool::ShieldedPool::Orchard,
        }];

        let scanned: Vec<SerializableOrchardNote> = vec![];
        merge_scanned_notes(&mut cached, &scanned);
        assert_eq!(cached.len(), 1);
        assert_eq!(cached[0].value, 250_000);
    }
}
