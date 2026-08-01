//! UniFFI surface for quiet Sapling legacy status / scan / shield-to-self.
//!
//! Same core path as CLI `nozy sapling`, Tauri, and `api-server` `/api/sapling/*`.
//! On-device proving still requires reachable Zebrad JSON-RPC + LWD compact SQLite.
//!
//! Generate Kotlin / Swift bindings with `uniffi-bindgen` (see README).

use std::path::Path;
use std::sync::OnceLock;

use tokio::runtime::Runtime;

fn runtime() -> &'static Runtime {
    static RT: OnceLock<Runtime> = OnceLock::new();
    RT.get_or_init(|| Runtime::new().expect("nozy-ffi Tokio runtime"))
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
#[uniffi(flat_error)]
pub enum NozyFfiError {
    #[error("{0}")]
    Message(String),
}

fn map_err(e: impl ToString) -> NozyFfiError {
    NozyFfiError::Message(e.to_string())
}

fn network_type(config: &nozy::WalletConfig) -> zcash_protocol::consensus::NetworkType {
    if config.network.eq_ignore_ascii_case("testnet") {
        zcash_protocol::consensus::NetworkType::Test
    } else {
        zcash_protocol::consensus::NetworkType::Main
    }
}

fn seed_from_mnemonic(mnemonic: &str) -> Result<Vec<u8>, NozyFfiError> {
    let wallet = nozy::HDWallet::from_mnemonic(mnemonic.trim()).map_err(map_err)?;
    Ok(wallet.get_mnemonic_object().to_seed("").to_vec())
}

#[derive(Clone, uniffi::Record)]
pub struct SaplingStatusFfi {
    pub unspent_notes: u64,
    pub with_rseed: u64,
    pub ready_to_shield: u64,
    pub unspent_zatoshis: u64,
    pub unspent_zec: f64,
    pub fee_zatoshis: u64,
    pub fee_zec: f64,
    pub has_legacy_balance: bool,
    pub message: String,
}

#[derive(Clone, uniffi::Record)]
pub struct SaplingScanResultFfi {
    pub blocks_scanned: u64,
    pub outputs_seen: u64,
    pub notes_discovered: u64,
    pub notes_marked_spent: u64,
    pub range_start: u64,
    pub range_end: u64,
    pub unspent_zatoshis: u64,
    pub unspent_notes: u64,
    pub message: String,
}

#[derive(Clone, uniffi::Record)]
pub struct SaplingShieldResultFfi {
    pub dry_run: bool,
    pub broadcast: bool,
    pub txid: Option<String>,
    pub shielded_value_zatoshis: Option<u64>,
    pub fee_zatoshis: u64,
    pub expiry_height: Option<u32>,
    pub candidate_notes: u64,
    pub candidate_zatoshis: u64,
    pub message: String,
}

/// Quiet legacy status from persisted Sapling notes under `wallet_data_dir`.
#[uniffi::export]
pub fn sapling_status(wallet_data_dir: String) -> Result<SaplingStatusFfi, NozyFfiError> {
    nozy::with_wallet_data_dir(Path::new(&wallet_data_dir), || {
        let notes = nozy::load_sapling_notes().unwrap_or_default();
        let unspent: Vec<_> = notes.iter().filter(|n| !n.spent).collect();
        let with_rseed = unspent
            .iter()
            .filter(|n| nozy::sapling_note_has_rseed(n))
            .count();
        let ready = unspent
            .iter()
            .filter(|n| nozy::sapling_note_ready_to_shield(n))
            .count();
        let bal = nozy::sapling_unspent_balance_zatoshis(&notes);
        let fee = nozy::sapling_shield_fee_zatoshis();
        let message = if ready > 0 {
            "Legacy funds ready to move into your shielded balance.".to_string()
        } else if with_rseed > 0 {
            "Legacy funds found — sync compact blocks, then move into shielded balance.".to_string()
        } else if bal > 0 {
            "Legacy notes need a rescan before they can be moved.".to_string()
        } else {
            "No legacy shielded balance.".to_string()
        };
        Ok(SaplingStatusFfi {
            unspent_notes: unspent.len() as u64,
            with_rseed: with_rseed as u64,
            ready_to_shield: ready as u64,
            unspent_zatoshis: bal,
            unspent_zec: bal as f64 / 100_000_000.0,
            fee_zatoshis: fee,
            fee_zec: fee as f64 / 100_000_000.0,
            has_legacy_balance: bal > 0,
            message,
        })
    })
}

/// Scan LWD compact cache for Sapling notes belonging to this mnemonic.
#[uniffi::export]
pub fn sapling_scan(
    mnemonic: String,
    wallet_data_dir: String,
    compact_db_path: String,
    start_floor: Option<u64>,
    full: bool,
) -> Result<SaplingScanResultFfi, NozyFfiError> {
    let seed = seed_from_mnemonic(&mnemonic)?;
    nozy::with_wallet_data_dir(Path::new(&wallet_data_dir), || {
        let store = zeaking::lwd::LwdCompactStore::open(Path::new(&compact_db_path))
            .map_err(|e| map_err(format!("open compact store: {e}")))?;
        let (notes, scan) =
            nozy::scan_sapling_wallet_from_compact_store(&seed, &store, start_floor, full)
                .map_err(map_err)?;
        let unspent_zatoshis = nozy::sapling_unspent_balance_zatoshis(&notes);
        let unspent_notes = notes.iter().filter(|n| !n.spent).count();
        Ok(SaplingScanResultFfi {
            blocks_scanned: scan.blocks_scanned,
            outputs_seen: scan.outputs_seen,
            notes_discovered: scan.notes_discovered,
            notes_marked_spent: scan.notes_marked_spent,
            range_start: scan.range_start,
            range_end: scan.range_end,
            unspent_zatoshis,
            unspent_notes: unspent_notes as u64,
            message: format!(
                "Scanned {} block(s); {} legacy note(s) unspent.",
                scan.blocks_scanned, unspent_notes
            ),
        })
    })
}

/// Move legacy Sapling notes into this wallet's Orchard/Ironwood balance.
///
/// Requires reachable `zebra_url` (JSON-RPC) and `lightwalletd_url` for compact catch-up.
#[uniffi::export]
pub fn sapling_shield(
    mnemonic: String,
    wallet_data_dir: String,
    compact_db_path: String,
    zebra_url: String,
    lightwalletd_url: String,
    dry_run: bool,
    no_broadcast: bool,
) -> Result<SaplingShieldResultFfi, NozyFfiError> {
    let seed = seed_from_mnemonic(&mnemonic)?;
    nozy::with_wallet_data_dir(Path::new(&wallet_data_dir), || {
        runtime().block_on(shield_inner(
            &seed,
            Path::new(&compact_db_path),
            &zebra_url,
            &lightwalletd_url,
            dry_run,
            no_broadcast,
        ))
    })
}

async fn shield_inner(
    seed: &[u8],
    compact_db: &Path,
    zebra_url: &str,
    lwd_url: &str,
    dry_run: bool,
    no_broadcast: bool,
) -> Result<SaplingShieldResultFfi, NozyFfiError> {
    let mut notes = nozy::load_sapling_notes().unwrap_or_default();
    let fee = nozy::sapling_shield_fee_zatoshis();
    let candidates: Vec<_> = notes
        .iter()
        .filter(|n| !n.spent && nozy::sapling_note_has_rseed(n))
        .collect();
    let candidate_zatoshis: u64 = candidates.iter().map(|n| n.value).sum();
    let candidate_notes = candidates.len() as u64;

    if dry_run {
        return Ok(SaplingShieldResultFfi {
            dry_run: true,
            broadcast: false,
            txid: None,
            shielded_value_zatoshis: None,
            fee_zatoshis: fee,
            expiry_height: None,
            candidate_notes,
            candidate_zatoshis,
            message: format!(
                "Dry run: {candidate_notes} note(s), {:.8} ZEC (fee ~{:.8}).",
                candidate_zatoshis as f64 / 100_000_000.0,
                fee as f64 / 100_000_000.0
            ),
        });
    }

    if candidates.is_empty() {
        return Err(NozyFfiError::Message(
            "No reconstructible legacy notes — sync compact blocks and scan first.".to_string(),
        ));
    }

    if let Some(parent) = compact_db.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let lwd = if lwd_url.trim().is_empty() {
        std::env::var("LIGHTWALLETD_GRPC").unwrap_or_else(|_| "http://127.0.0.1:9067".to_string())
    } else {
        lwd_url.trim().to_string()
    };

    if let Ok(mut client) = zeaking::lwd::connect_lightwalletd(&lwd).await {
        if let Ok(store) = zeaking::lwd::LwdCompactStore::open(compact_db) {
            let _ = zeaking::lwd::sync_compact_to_tip_with_options(
                &mut client,
                &store,
                zeaking::lwd::SyncCompactToTipOptions::default(),
            )
            .await;
            let _ = nozy::scan_sapling_wallet_from_compact_store(seed, &store, None, false);
            notes = nozy::load_sapling_notes().unwrap_or_default();
        }
    }

    let store = zeaking::lwd::LwdCompactStore::open(compact_db)
        .map_err(|e| map_err(format!("open compact store: {e}")))?;
    let mut config = nozy::load_config();
    let zebra_override = zebra_url.trim();
    if !zebra_override.is_empty() {
        config = config.with_zebra_url_override(Some(zebra_override.to_string()));
        config.ensure_trusted_zebra_url(zebra_override);
    }
    let zebra = nozy::ZebraClient::from_config(&config);
    let keys = nozy::derive_sapling_account_keys(seed, 0, 0).map_err(map_err)?;
    let expiry = nozy::fee_policy::PilotSendOptions::for_send().expiry_delta_blocks;
    let network = network_type(&config);

    let built = nozy::build_sapling_shield_to_self(
        &zebra,
        &store,
        seed,
        &keys.extsk,
        &mut notes,
        network,
        expiry,
    )
    .await
    .map_err(map_err)?;
    nozy::save_sapling_notes(&notes).map_err(map_err)?;

    if no_broadcast {
        return Ok(SaplingShieldResultFfi {
            dry_run: false,
            broadcast: false,
            txid: Some(built.txid.clone()),
            shielded_value_zatoshis: Some(built.shielded_value_zatoshis),
            fee_zatoshis: built.fee_zatoshis,
            expiry_height: Some(built.expiry_height),
            candidate_notes,
            candidate_zatoshis,
            message: format!(
                "Built (not broadcast). TXID {}. Move {:.8} ZEC after fee.",
                built.txid,
                built.shielded_value_zatoshis as f64 / 100_000_000.0
            ),
        });
    }

    let txid = zebra
        .broadcast_transaction_bytes(&built.raw_transaction)
        .await
        .map_err(map_err)?;
    nozy::save_sapling_notes(&notes).map_err(map_err)?;

    Ok(SaplingShieldResultFfi {
        dry_run: false,
        broadcast: true,
        txid: Some(txid.clone()),
        shielded_value_zatoshis: Some(built.shielded_value_zatoshis),
        fee_zatoshis: built.fee_zatoshis,
        expiry_height: Some(built.expiry_height),
        candidate_notes,
        candidate_zatoshis,
        message: format!(
            "Broadcast {}. Moved {:.8} ZEC into shielded balance.",
            txid,
            built.shielded_value_zatoshis as f64 / 100_000_000.0
        ),
    })
}

uniffi::setup_scaffolding!();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_empty_dir_has_no_legacy_balance() {
        let dir = std::env::temp_dir().join(format!("nozy-ffi-status-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let status = sapling_status(dir.to_string_lossy().into_owned()).expect("status");
        assert!(!status.has_legacy_balance);
        assert_eq!(status.unspent_zatoshis, 0);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn scan_rejects_bad_mnemonic() {
        let dir = std::env::temp_dir().join(format!("nozy-ffi-scan-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let db = dir.join("lwd_compact.sqlite");
        let err = sapling_scan(
            "not a real mnemonic".into(),
            dir.to_string_lossy().into_owned(),
            db.to_string_lossy().into_owned(),
            None,
            false,
        );
        assert!(err.is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
