//! UniFFI surface for quiet Sapling legacy status / scan / shield-to-self,
//! plus NU7 coinholder vote **export/sign** (seed stays on device).
//!
//! Prepare / delegate-finish / cast need `zcash_voting` and cannot link here
//! beside `zeaking` (sqlite conflict) — use desktop Vote tab or `nozy-vote` CLI.
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

fn parse_network_label(
    network: &str,
) -> Result<zcash_protocol::consensus::NetworkType, NozyFfiError> {
    match network.trim().to_ascii_lowercase().as_str() {
        "" | "main" | "mainnet" => Ok(zcash_protocol::consensus::NetworkType::Main),
        "test" | "testnet" => Ok(zcash_protocol::consensus::NetworkType::Test),
        other => Err(NozyFfiError::Message(format!(
            "unknown network {other} (mainnet|testnet)"
        ))),
    }
}

fn seed_from_mnemonic(mnemonic: &str) -> Result<Vec<u8>, NozyFfiError> {
    let wallet = nozy::HDWallet::from_mnemonic(mnemonic.trim()).map_err(map_err)?;
    Ok(wallet.get_mnemonic_object().to_seed("").to_vec())
}

fn wallet_from_mnemonic(mnemonic: &str) -> Result<nozy::HDWallet, NozyFfiError> {
    nozy::HDWallet::from_mnemonic(mnemonic.trim()).map_err(map_err)
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

#[derive(Clone, uniffi::Record)]
pub struct VoteCalendarFfi {
    pub snapshot_utc: String,
    pub vote_start_utc: String,
    pub vote_end_utc: String,
    pub forum_url: String,
    pub tally_url: String,
    pub message: String,
}

#[derive(Clone, uniffi::Record)]
pub struct VoteNotesExportFfi {
    pub format: String,
    pub network: String,
    pub note_count: u64,
    pub total_value_zat: u64,
    pub seed_fingerprint_hex: String,
    /// Full `nozy-vote-notes-v1` JSON for `nozy-vote import-notes` / desktop handoff.
    pub notes_json: String,
    pub message: String,
}

#[derive(Clone, uniffi::Record)]
pub struct VoteDelegationSigFfi {
    pub format: String,
    pub round_id: String,
    pub bundle_index: u32,
    pub sighash_hex: String,
    pub spend_auth_sig_hex: String,
    /// Full `nozy-vote-delegation-sig-v1` JSON for `delegate-finish`.
    pub sig_json: String,
    pub message: String,
}

/// Static NU7 coinholder vote calendar (no network / no SDK).
#[uniffi::export]
pub fn vote_calendar_info() -> VoteCalendarFfi {
    VoteCalendarFfi {
        snapshot_utc: "2026-08-24T19:00:00Z".into(),
        vote_start_utc: "2026-08-25T00:00:00Z".into(),
        vote_end_utc: "2026-09-14T19:00:00Z".into(),
        forum_url: "https://forum.zcashcommunity.com/t/nu7-coinholder-vote/56912".into(),
        tally_url: "https://tally.valargroup.org".into(),
        message: "Eligible weight = spendable Ironwood notes at snapshot. \
Prepare/cast on desktop or nozy-vote CLI; this FFI only exports notes and signs delegation."
            .into(),
    }
}

/// Export unspent Ironwood notes (+ witnesses) as `nozy-vote-notes-v1` JSON.
///
/// Requires synced Ironwood notes under `wallet_data_dir`. Prepare/cast stay on desktop/`nozy-vote`.
#[uniffi::export]
pub fn vote_export_notes(
    mnemonic: String,
    wallet_data_dir: String,
    network: String,
) -> Result<VoteNotesExportFfi, NozyFfiError> {
    let wallet = wallet_from_mnemonic(&mnemonic)?;
    let net = if network.trim().is_empty() {
        let config = nozy::load_config();
        network_type(&config)
    } else {
        parse_network_label(&network)?
    };
    nozy::with_wallet_data_dir(Path::new(&wallet_data_dir), || {
        let file = nozy::build_ironwood_vote_notes(&wallet, net).map_err(map_err)?;
        let total_value_zat: u64 = file.notes.iter().map(|n| n.value).sum();
        let notes_json = serde_json::to_string_pretty(&file)
            .map_err(|e| map_err(format!("serialize notes json: {e}")))?;
        Ok(VoteNotesExportFfi {
            format: file.format.clone(),
            network: file.network.clone(),
            note_count: file.notes.len() as u64,
            total_value_zat,
            seed_fingerprint_hex: file.seed_fingerprint_hex.clone(),
            notes_json,
            message: format!(
                "Exported {} Ironwood note(s). Share notes_json to desktop/nozy-vote for import.",
                file.notes.len()
            ),
        })
    })
}

/// Sign a Valar delegation PCZT request (`nozy-vote-delegation-sign-v1` JSON).
#[uniffi::export]
pub fn vote_sign_delegation(
    mnemonic: String,
    request_json: String,
) -> Result<VoteDelegationSigFfi, NozyFfiError> {
    let wallet = wallet_from_mnemonic(&mnemonic)?;
    let sig =
        nozy::sign_delegation_request_json(&wallet, request_json.as_bytes()).map_err(map_err)?;
    let sig_json = serde_json::to_string_pretty(&sig)
        .map_err(|e| map_err(format!("serialize sig json: {e}")))?;
    Ok(VoteDelegationSigFfi {
        format: sig.format.clone(),
        round_id: sig.round_id.clone(),
        bundle_index: sig.bundle_index,
        sighash_hex: sig.sighash_hex.clone(),
        spend_auth_sig_hex: sig.spend_auth_sig_hex.clone(),
        sig_json,
        message: "Delegation signed. Share sig_json to desktop/nozy-vote for delegate-finish."
            .into(),
    })
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

    #[test]
    fn vote_calendar_has_snapshot() {
        let c = vote_calendar_info();
        assert!(c.snapshot_utc.contains("2026-08-24"));
    }

    #[test]
    fn vote_sign_rejects_bad_json() {
        let err = vote_sign_delegation(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
                .into(),
            "{not json".into(),
        );
        assert!(err.is_err());
    }

    #[test]
    fn vote_export_rejects_bad_mnemonic() {
        let dir = std::env::temp_dir().join(format!("nozy-ffi-vote-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let err = vote_export_notes(
            "not a real mnemonic".into(),
            dir.to_string_lossy().into_owned(),
            "mainnet".into(),
        );
        assert!(err.is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
