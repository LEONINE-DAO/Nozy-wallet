//! High-level vote operations shared by CLI and desktop (and later FFI / api-server).

use anyhow::{anyhow, bail, Context, Result};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::path::{Path, PathBuf};

use crate::config::Environment;
use crate::eligibility::{self, VotePhase};
use crate::sdk::{self, ActiveRound, NetworkKind};
use crate::urls;

pub const DEFAULT_WALLET_ID: &str = "nozy";

pub fn default_data_dir() -> Result<PathBuf> {
    Ok(
        directories::ProjectDirs::from("org", "LeonineDAO", "NozyVote")
            .context("resolve platform project dirs")?
            .data_dir()
            .to_path_buf(),
    )
}

pub fn ensure_data_dir(override_dir: Option<PathBuf>) -> Result<PathBuf> {
    let dir = match override_dir {
        Some(p) => p,
        None => default_data_dir()?,
    };
    std::fs::create_dir_all(&dir).with_context(|| format!("create data dir {}", dir.display()))?;
    Ok(dir)
}

pub fn parse_network(s: &str) -> Result<NetworkKind> {
    match s.to_ascii_lowercase().as_str() {
        "mainnet" | "main" => Ok(NetworkKind::Mainnet),
        "testnet" | "test" => Ok(NetworkKind::Testnet),
        other => bail!("unknown network {other} (mainnet|testnet)"),
    }
}

pub fn notes_path(data_dir: &Path) -> PathBuf {
    data_dir.join("vote-notes.json")
}

pub fn signing_request_path(data_dir: &Path, round_id: &str) -> PathBuf {
    data_dir.join(format!("signing-request-{round_id}.json"))
}

pub fn sig_path(data_dir: &Path, round_id: &str) -> PathBuf {
    data_dir.join(format!("delegation-sig-{round_id}.json"))
}

#[derive(Debug, Serialize)]
pub struct StatusSnapshot {
    pub helper_version: String,
    pub env: String,
    pub data_dir: String,
    pub sdk_enabled: bool,
    pub static_source: String,
    pub snapshot_utc: String,
    pub vote_start_utc: String,
    pub vote_end_utc: String,
    pub phase: String,
    pub phase_message: String,
    pub forum_url: String,
    pub notes_exported: bool,
    pub notes_count: Option<usize>,
    pub hotkey_ready: bool,
    pub signing_request_present: bool,
    pub sig_present: bool,
}

pub fn status_snapshot(
    env: Environment,
    static_source: Option<&str>,
    data_dir: &Path,
    network: NetworkKind,
) -> Result<StatusSnapshot> {
    let source = urls::static_source(env, static_source);
    let phase = eligibility::current_phase();
    let notes = notes_path(data_dir);
    let (notes_exported, notes_count) = if notes.is_file() {
        match sdk::load_note_export(&notes) {
            Ok(f) => (true, Some(f.notes.len())),
            Err(_) => (true, None),
        }
    } else {
        (false, None)
    };
    let hotkey_ready = sdk::hotkey_path(data_dir, network).is_file();
    let active = fetch_active_round_opt(env, static_source).ok();
    let active_rid = active.as_ref().and_then(|r| sdk::vote_round_id_hex(r).ok());
    let (signing_request_present, sig_present) = match &active_rid {
        Some(rid) => (
            signing_request_path(data_dir, rid).is_file(),
            sig_path(data_dir, rid).is_file(),
        ),
        None => (false, false),
    };

    let mut status = StatusSnapshot {
        helper_version: env!("CARGO_PKG_VERSION").to_string(),
        env: env.as_str().to_string(),
        data_dir: data_dir.display().to_string(),
        sdk_enabled: true,
        static_source: source,
        snapshot_utc: eligibility::SNAPSHOT_UTC.to_string(),
        vote_start_utc: eligibility::VOTE_START_UTC.to_string(),
        vote_end_utc: eligibility::VOTE_END_UTC.to_string(),
        phase: match phase {
            VotePhase::PreSnapshot => "pre_snapshot".into(),
            VotePhase::PreOpen => "pre_open".into(),
            VotePhase::Open => "open".into(),
            VotePhase::Closed => "closed".into(),
        },
        phase_message: eligibility::phase_message(phase),
        forum_url: eligibility::FORUM_URL.to_string(),
        notes_exported,
        notes_count,
        hotkey_ready,
        signing_request_present,
        sig_present,
    };
    if let Some(active) = active {
        apply_active_round(&mut status, &active, Utc::now());
    }
    Ok(status)
}

/// Replace the fallback NU7 calendar when Valar has a live round.
///
/// The August 2026 constants stay for offline status. An active round's
/// snapshot height and `vote_end_time` are what the wallet should show.
pub fn apply_active_round(status: &mut StatusSnapshot, active: &ActiveRound, now: DateTime<Utc>) {
    let Some(end) = DateTime::<Utc>::from_timestamp(active.vote_end_time as i64, 0) else {
        return;
    };
    let open = active.status == 1 && now <= end;
    let title = active
        .title
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .unwrap_or("Coinholder poll");
    status.snapshot_utc = format!("height {}", active.snapshot_height);
    status.vote_start_utc = "live".to_string();
    status.vote_end_utc = end.to_rfc3339();
    status.forum_url = String::new();
    if open {
        status.phase = "open".to_string();
        status.phase_message = format!(
            "{title} is open. Weight is spendable Ironwood at snapshot height {}. Delegate, then cast on this screen. Closes {}.",
            active.snapshot_height,
            end.format("%Y-%m-%d %H:%M UTC")
        );
    } else {
        status.phase = "closed".to_string();
        status.phase_message = format!("{title} is closed. Tallies: {}", eligibility::TALLY_URL);
    }
}

fn fetch_active_round_opt(env: Environment, static_source: Option<&str>) -> Result<ActiveRound> {
    let (_resolved, servers) = sdk::resolve_vote_server_urls(env, static_source)?;
    sdk::fetch_active_round(&servers)
}

pub fn fetch_active(env: Environment, static_source: Option<&str>) -> Result<ActiveRound> {
    fetch_active_round_opt(env, static_source)
}

#[derive(Debug, Serialize)]
pub struct PrepareResult {
    pub round_id: String,
    pub snapshot_height: u64,
    pub note_count: usize,
    pub bundle_count: u32,
    pub eligible_weight_zat: u64,
    pub notes_path: String,
}

/// Hotkey + create_round + import notes from `notes_path`.
pub fn prepare_round(
    env: Environment,
    static_source: Option<&str>,
    data_dir: &Path,
    network: NetworkKind,
    wallet_id: &str,
    notes_file: &Path,
) -> Result<PrepareResult> {
    let _hotkey = sdk::init_or_load_hotkey(data_dir, network)?;
    let (_resolved, servers) = sdk::resolve_vote_server_urls(env, static_source)?;
    let active = sdk::fetch_active_round(&servers)?;
    let db = sdk::open_voting_db(data_dir, wallet_id)?;
    let params = sdk::create_round_from_active(&db, network, &active)?;
    let export = sdk::load_note_export(notes_file)?;
    let notes = sdk::exported_to_note_infos(&export)?;
    let witnesses = sdk::exported_to_witnesses(&export)?;
    if notes.is_empty() {
        bail!("note export contains no notes");
    }
    let layout = sdk::ensure_bundles_and_witnesses(&db, &params.vote_round_id, &notes, &witnesses)?;
    Ok(PrepareResult {
        round_id: params.vote_round_id,
        snapshot_height: params.snapshot_height,
        note_count: notes.len(),
        bundle_count: layout.bundle_count,
        eligible_weight_zat: layout.eligible_weight,
        notes_path: notes_file.display().to_string(),
    })
}

#[derive(Debug, Serialize)]
pub struct DelegatePrepareResult {
    pub round_id: String,
    pub note_count: usize,
    pub bundle_count: u32,
    pub eligible_weight_zat: u64,
    pub signing_request_path: String,
}

/// Build delegation PCZT + write signing request for `nozy` / desktop to sign.
pub fn prepare_delegation(
    env: Environment,
    static_source: Option<&str>,
    data_dir: &Path,
    network: NetworkKind,
    wallet_id: &str,
    notes_file: &Path,
    round_id: Option<&str>,
) -> Result<DelegatePrepareResult> {
    use zcash_voting::prelude::{
        DelegationKeys, LightwalletdBranchIdProvider, NoopProgressReporter,
    };

    let (_resolved, servers) = sdk::resolve_vote_server_urls(env, static_source)?;
    let active = sdk::fetch_active_round(&servers)?;
    let params = sdk::active_to_round_params(&active)?;
    let rid = round_id
        .map(|s| s.to_string())
        .unwrap_or_else(|| params.vote_round_id.clone());
    if rid != params.vote_round_id {
        bail!(
            "round_id {rid} does not match active round {}",
            params.vote_round_id
        );
    }

    let export = sdk::load_note_export(notes_file)?;
    let notes = sdk::exported_to_note_infos(&export)?;
    let hotkey = sdk::init_or_load_hotkey(data_dir, network)?;
    let db = sdk::open_voting_db(data_dir, wallet_id)?;
    let _ = sdk::create_round_from_active(&db, network, &active)?;
    let witnesses = sdk::exported_to_witnesses(&export)?;
    let layout = sdk::ensure_bundles_and_witnesses(&db, &rid, &notes, &witnesses)?;

    let fvk = hex::decode(export.orchard_fvk_hex.trim())
        .context("decode orchard_fvk_hex from note export")?;
    let seed_fp: [u8; 32] = hex::decode(export.seed_fingerprint_hex.trim())
        .context("decode seed_fingerprint_hex")?
        .try_into()
        .map_err(|_| anyhow!("seed_fingerprint must be 32 bytes"))?;

    let keys = DelegationKeys::with_voting_hotkey(
        fvk,
        &hotkey,
        seed_fp,
        export.account_index,
        active.title.clone().unwrap_or_else(|| "nu7".into()),
    )
    .map_err(|e| anyhow!("DelegationKeys: {e}"))?;

    let branch = LightwalletdBranchIdProvider::for_height(network.to_sdk(), params.snapshot_height)
        .map_err(|e| anyhow!("branch id at snapshot: {e}"))?;
    let progress = NoopProgressReporter;
    let _setup = zcash_voting::delegate::setup(&db, &rid, 0, &notes, &keys, &branch, &progress)
        .map_err(|e| anyhow!("delegation setup: {e}"))?;

    let signing = zcash_voting::delegate::signing_request(&db, &rid, 0, &keys)
        .map_err(|e| anyhow!("signing_request: {e}"))?;
    let req_path = signing_request_path(data_dir, &rid);
    sdk::write_signing_request(&req_path, network, &rid, 0, &signing)?;

    Ok(DelegatePrepareResult {
        round_id: rid,
        note_count: notes.len(),
        bundle_count: layout.bundle_count,
        eligible_weight_zat: layout.eligible_weight,
        signing_request_path: req_path.display().to_string(),
    })
}

#[derive(Debug, Serialize)]
pub struct DelegateFinishResult {
    pub round_id: String,
    pub tx_hash: String,
    pub van_leaf_position: Option<u32>,
    pub confirmed: bool,
}

/// PIR + ZKP1 + submit delegation (+ optional confirm).
pub fn finish_delegation(
    env: Environment,
    static_source: Option<&str>,
    data_dir: &Path,
    network: NetworkKind,
    wallet_id: &str,
    notes_file: &Path,
    sig_file: &Path,
    wait: bool,
) -> Result<DelegateFinishResult> {
    use zcash_voting::prelude::DelegationKeys;

    let (resolved, servers) = sdk::resolve_vote_server_urls(env, static_source)?;
    let active = sdk::fetch_active_round(&servers)?;
    let params = sdk::active_to_round_params(&active)?;
    let rid = params.vote_round_id.clone();

    let sig = sdk::load_sig_file(sig_file)?;
    if sig.round_id != rid {
        bail!("sig round_id {} != active round {rid}", sig.round_id);
    }

    let export = sdk::load_note_export(notes_file)?;
    let notes = sdk::exported_to_note_infos(&export)?;
    let hotkey = sdk::init_or_load_hotkey(data_dir, network)?;
    let db = sdk::open_voting_db(data_dir, wallet_id)?;
    let _ = sdk::create_round_from_active(&db, network, &active)?;

    let fvk = hex::decode(export.orchard_fvk_hex.trim()).context("decode orchard_fvk_hex")?;
    let seed_fp: [u8; 32] = hex::decode(export.seed_fingerprint_hex.trim())?
        .try_into()
        .map_err(|_| anyhow!("seed_fingerprint must be 32 bytes"))?;
    let keys = DelegationKeys::with_voting_hotkey(
        fvk,
        &hotkey,
        seed_fp,
        export.account_index,
        active.title.clone().unwrap_or_else(|| "nu7".into()),
    )
    .map_err(|e| anyhow!("DelegationKeys: {e}"))?;

    let pir = sdk::connect_pir_client(&resolved)?;
    sdk::pir_and_prove(&db, &rid, &notes, &keys, &pir, network)?;
    let tx_hash = sdk::submit_delegation(&db, &rid, &servers, &sig)?;

    let mut van_leaf_position = None;
    let mut confirmed = false;
    if wait {
        let conf_json = sdk::wait_tx_confirm(&servers, &tx_hash)?;
        match sdk::confirm_delegation_from_tx(&db, &rid, 0, &tx_hash, &conf_json) {
            Ok(conf) => {
                van_leaf_position = Some(conf.van_leaf_position);
                confirmed = true;
            }
            Err(e) => {
                // Still return the tx hash so the desktop can persist it and cast
                // with --delegation-tx after the chain indexes the leaf.
                eprintln!("warning: delegation tx {tx_hash} submitted but VAN confirm failed: {e}");
            }
        }
    }

    Ok(DelegateFinishResult {
        round_id: rid,
        tx_hash,
        van_leaf_position,
        confirmed,
    })
}

#[derive(Debug, Serialize)]
pub struct CastResult {
    pub round_id: String,
    pub proposal_count: usize,
}

pub fn cast_votes(
    env: Environment,
    static_source: Option<&str>,
    data_dir: &Path,
    network: NetworkKind,
    wallet_id: &str,
    choices: &[String],
    delegation_tx: Option<&str>,
    single_share: bool,
    wait: bool,
) -> Result<CastResult> {
    let (_resolved, servers) = sdk::resolve_vote_server_urls(env, static_source)?;
    let active = sdk::fetch_active_round(&servers)?;
    let rid = sdk::vote_round_id_hex(&active)?;
    let db = sdk::open_voting_db(data_dir, wallet_id)?;
    let _ = sdk::create_round_from_active(&db, network, &active)?;
    let hotkey = sdk::init_or_load_hotkey(data_dir, network)?;

    if let Some(tx) = delegation_tx {
        let conf_json = sdk::wait_tx_confirm(&servers, tx)?;
        let _ = sdk::confirm_delegation_from_tx(&db, &rid, 0, tx, &conf_json)?;
    }

    let drafts = sdk::drafts_from_choices(&active, choices, single_share)?;
    let count = drafts.len();
    sdk::cast_ballots(&db, &active, &servers, &hotkey, &drafts, !wait)?;
    Ok(CastResult {
        round_id: rid,
        proposal_count: count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sdk::ActiveRound;

    fn sample_round(status: u32, vote_end_time: u64) -> ActiveRound {
        ActiveRound {
            vote_round_id: "abc".into(),
            snapshot_height: 3_578_540,
            snapshot_blockhash: None,
            vote_end_time,
            nullifier_imt_root: "n".into(),
            nc_root: "r".into(),
            status,
            ea_pk: "k".into(),
            title: Some("NU7.1 Poll".into()),
            description: None,
            proposals: Vec::new(),
        }
    }

    fn closed_status() -> StatusSnapshot {
        StatusSnapshot {
            helper_version: "test".into(),
            env: "prod".into(),
            data_dir: ".".into(),
            sdk_enabled: true,
            static_source: "pin".into(),
            snapshot_utc: eligibility::SNAPSHOT_UTC.into(),
            vote_start_utc: eligibility::VOTE_START_UTC.into(),
            vote_end_utc: eligibility::VOTE_END_UTC.into(),
            phase: "closed".into(),
            phase_message: "Voting window has ended.".into(),
            forum_url: eligibility::FORUM_URL.into(),
            notes_exported: false,
            notes_count: None,
            hotkey_ready: false,
            signing_request_present: false,
            sig_present: false,
        }
    }

    #[test]
    fn active_round_replaces_stale_nu7_calendar() {
        let mut status = closed_status();
        let now = DateTime::parse_from_rfc3339("2026-11-18T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        apply_active_round(&mut status, &sample_round(1, 1_796_093_940), now);
        assert_eq!(status.phase, "open");
        assert_eq!(status.snapshot_utc, "height 3578540");
        assert!(status.vote_end_utc.starts_with("2026-12-01T02:59:00"));
        assert!(status.phase_message.contains("NU7.1 Poll"));
        assert!(status.phase_message.contains("3578540"));
        assert!(status.forum_url.is_empty());
    }

    #[test]
    fn ended_active_round_stays_closed() {
        let mut status = closed_status();
        let now = DateTime::parse_from_rfc3339("2026-12-01T03:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        apply_active_round(&mut status, &sample_round(1, 1_796_093_940), now);
        assert_eq!(status.phase, "closed");
        assert!(status.phase_message.contains("closed"));
    }
}
