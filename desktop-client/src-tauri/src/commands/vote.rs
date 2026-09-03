//! Desktop NU7 coinholder vote (Valar Shielded Vote).
//!
//! Export/sign use `nozy` in-process. SDK steps shell out to the `nozy-vote`
//! helper — `zcash_voting` cannot link beside `zeaking` (sqlite `links` conflict).
//! Tracking: https://github.com/LEONINE-DAO/Nozy-wallet/issues/273

use crate::error::TauriError;
use crate::session::load_session_wallet;
use nozy::{load_config, sign_delegation_request};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::command;

fn map_err(msg: impl Into<String>) -> TauriError {
    TauriError {
        message: msg.into(),
        code: Some("VOTE_ERROR".into()),
    }
}

fn default_vote_data_dir() -> Result<PathBuf, TauriError> {
    directories::ProjectDirs::from("org", "LeonineDAO", "NozyVote")
        .map(|d| d.data_dir().to_path_buf())
        .ok_or_else(|| map_err("resolve NozyVote data dir"))
}

fn ensure_vote_data_dir() -> Result<PathBuf, TauriError> {
    let dir = default_vote_data_dir()?;
    std::fs::create_dir_all(&dir).map_err(|e| map_err(format!("create vote data dir: {e}")))?;
    Ok(dir)
}

fn notes_path(data_dir: &Path) -> PathBuf {
    data_dir.join("vote-notes.json")
}

fn delegation_tx_path(data_dir: &Path) -> PathBuf {
    data_dir.join("last-delegation-tx.txt")
}

fn save_delegation_tx(data_dir: &Path, tx_hash: &str) -> Result<(), TauriError> {
    let tx = tx_hash.trim();
    if tx.len() < 16 {
        return Ok(());
    }
    std::fs::write(delegation_tx_path(data_dir), tx)
        .map_err(|e| map_err(format!("persist delegation tx: {e}")))
}

fn load_delegation_tx(data_dir: &Path) -> Option<String> {
    let raw = std::fs::read_to_string(delegation_tx_path(data_dir)).ok()?;
    let tx = raw.trim().to_string();
    if tx.len() >= 16 {
        Some(tx)
    } else {
        None
    }
}

fn signing_request_path(data_dir: &Path, round_id: &str) -> PathBuf {
    data_dir.join(format!("signing-request-{round_id}.json"))
}

fn sig_path(data_dir: &Path, round_id: &str) -> PathBuf {
    data_dir.join(format!("delegation-sig-{round_id}.json"))
}

fn network_label() -> String {
    let config = load_config();
    if config.network.eq_ignore_ascii_case("testnet") {
        "testnet".into()
    } else {
        "mainnet".into()
    }
}

fn consensus_network() -> zcash_protocol::consensus::NetworkType {
    let config = load_config();
    if config.network.eq_ignore_ascii_case("testnet") {
        zcash_protocol::consensus::NetworkType::Test
    } else {
        zcash_protocol::consensus::NetworkType::Main
    }
}

/// Locate `nozy-vote` / `nozy-vote.exe` for sidecar calls.
fn resolve_nozy_vote_bin() -> Result<PathBuf, TauriError> {
    if let Ok(p) = std::env::var("NOZY_VOTE_BIN") {
        let path = PathBuf::from(p);
        if path.is_file() {
            return Ok(path);
        }
        return Err(map_err(format!(
            "NOZY_VOTE_BIN set but not a file: {}",
            path.display()
        )));
    }

    let exe_name = if cfg!(windows) {
        "nozy-vote.exe"
    } else {
        "nozy-vote"
    };

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let sibling = dir.join(exe_name);
            if sibling.is_file() {
                return Ok(sibling);
            }
        }
    }

    // Dev checkout: tools/nozy-vote/target/release
    let mut cursor = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for _ in 0..4 {
        let candidate = cursor
            .join("tools")
            .join("nozy-vote")
            .join("target")
            .join("release")
            .join(exe_name);
        if candidate.is_file() {
            return Ok(candidate);
        }
        if !cursor.pop() {
            break;
        }
    }

    Err(map_err(
        "nozy-vote binary not found. Build tools/nozy-vote (`cargo build --release`) \
         or set NOZY_VOTE_BIN to the helper path. (Desktop cannot link zcash_voting \
         beside zeaking due to sqlite version conflict.)",
    ))
}

fn run_nozy_vote(args: &[&str]) -> Result<String, TauriError> {
    let bin = resolve_nozy_vote_bin().map_err(|e| {
        
        e
    })?;
    let data_dir = ensure_vote_data_dir()?;
    
    let started = std::time::Instant::now();
    let output = Command::new(&bin)
        .args(args)
        .arg("--data-dir")
        .arg(&data_dir)
        .output()
        .map_err(|e| map_err(format!("spawn {}: {e}", bin.display())))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let elapsed_ms = started.elapsed().as_millis();
    if !output.status.success() {
        let detail = if !stderr.trim().is_empty() {
            stderr.clone()
        } else {
            stdout.clone()
        };
        
        return Err(map_err(format!(
            "nozy-vote {} failed: {}",
            args.join(" "),
            detail.trim()
        )));
    }
    
    Ok(stdout)
}

async fn run_blocking<T, F>(f: F) -> Result<T, TauriError>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, TauriError> + Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| map_err(format!("vote task join: {e}")))?
}

#[derive(Debug, Deserialize)]
pub struct VoteEnvRequest {
    #[serde(default = "default_env")]
    pub env: String,
}

fn default_env() -> String {
    "prod".into()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VoteStatusResponse {
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

#[command]
pub async fn vote_status(request: VoteEnvRequest) -> Result<VoteStatusResponse, TauriError> {
    let env = request.env.clone();
    run_blocking(move || {
        let out = run_nozy_vote(&["--env", &env, "status", "--json"])?;
        serde_json::from_str(&out).map_err(|e| map_err(format!("decode status json: {e}\n{out}")))
    })
    .await
}

#[command]
pub async fn vote_active(request: VoteEnvRequest) -> Result<Value, TauriError> {
    let env = request.env.clone();
    run_blocking(move || {
        let out = run_nozy_vote(&["--env", &env, "active", "--json"])?;
        serde_json::from_str(&out).map_err(|e| map_err(format!("decode active json: {e}\n{out}")))
    })
    .await
}

#[derive(Debug, Serialize)]
pub struct VoteExportNotesResponse {
    pub notes_path: String,
    pub note_count: usize,
    pub total_value_zat: u64,
    pub message: String,
}

#[command]
pub async fn vote_export_notes(
    password: Option<String>,
) -> Result<VoteExportNotesResponse, TauriError> {
    let wallet = load_session_wallet(password.as_deref()).await?;
    let network = consensus_network();
    let data_dir = ensure_vote_data_dir()?;
    let out = notes_path(&data_dir);

    // Snapshot-anchored witnesses are required for Valar `store_witnesses` (nc_root match).
    let (snapshot_height, nc_root_hex) = {
        let env = "prod".to_string();
        let active: Result<Value, TauriError> = run_blocking(move || {
            let out = run_nozy_vote(&["--env", &env, "active", "--json"])?;
            serde_json::from_str(&out).map_err(|e| map_err(format!("decode active json: {e}")))
        })
        .await;
        match active {
            Ok(v) => {
                let height = v
                    .get("snapshot_height")
                    .and_then(|h| h.as_u64())
                    .map(|h| h as u32)
                    .unwrap_or(nozy::NU7_SNAPSHOT_HEIGHT_MAINNET);
                let nc = v
                    .get("nc_root")
                    .and_then(|x| x.as_str())
                    .and_then(|b64| {
                        use base64::Engine;
                        base64::engine::general_purpose::STANDARD
                            .decode(b64)
                            .ok()
                            .map(hex::encode)
                    });
                (height, nc)
            }
            Err(_) => (nozy::NU7_SNAPSHOT_HEIGHT_MAINNET, None),
        }
    };

    

    // Skip multi-hour Zebrad rebuild when a prior export already matches Valar nc_root.
    if let (Some(nc), true) = (nc_root_hex.as_ref(), out.is_file()) {
        if let Ok(bytes) = std::fs::read(&out) {
            if let Ok(existing) = serde_json::from_slice::<Value>(&bytes) {
                let snap_ok = existing
                    .get("snapshot_height")
                    .and_then(|h| h.as_u64())
                    .map(|h| h as u32)
                    == Some(snapshot_height);
                let notes = existing.get("notes").and_then(|n| n.as_array());
                let roots_ok = notes
                    .map(|arr| {
                        !arr.is_empty()
                            && arr.iter().all(|n| {
                                n.get("root_hex")
                                    .and_then(|r| r.as_str())
                                    .map(|r| r.eq_ignore_ascii_case(nc))
                                    .unwrap_or(false)
                            })
                    })
                    .unwrap_or(false);
                if snap_ok && roots_ok {
                    let note_count = notes.map(|a| a.len()).unwrap_or(0);
                    let total_value_zat = notes
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|n| n.get("value").and_then(|v| v.as_u64()))
                                .sum::<u64>()
                        })
                        .unwrap_or(0);
                    
                    let ballot_hint = if total_value_zat < 12_500_000 {
                        format!(
                            " Note: {total_value_zat} zat is below one ballot (12_500_000 zat / 0.125 ZEC) — Prepare will fail until Ironwood balance at snapshot is increased."
                        )
                    } else {
                        String::new()
                    };
                    return Ok(VoteExportNotesResponse {
                        notes_path: out.display().to_string(),
                        note_count,
                        total_value_zat,
                        message: format!(
                            "Snapshot witnesses already valid for height {snapshot_height} (skipped rebuild).{ballot_hint}"
                        ),
                    });
                }
            }
        }
    }

    let config = load_config();
    let zebra = nozy::ZebraClient::from_config_with_url(&config, Some(&config.zebra_url));
    let file = match nozy::export_ironwood_vote_notes_at_snapshot(
        &wallet,
        network,
        &zebra,
        snapshot_height,
        &out,
    )
    .await
    {
        Ok(f) => f,
        Err(e) => {
            
            return Err(TauriError::from(e));
        }
    };
    let total_value_zat = file.notes.iter().map(|n| n.value).sum();
    let root0 = file
        .notes
        .first()
        .map(|n| n.root_hex.chars().take(16).collect::<String>());
    let root_matches_nc = nc_root_hex
        .as_ref()
        .map(|nc| {
            file.notes
                .iter()
                .all(|n| n.root_hex.eq_ignore_ascii_case(nc))
        })
        .unwrap_or(false);
    
    let ballot_hint = if total_value_zat < 12_500_000 {
        format!(
            " Warning: {total_value_zat} zat < one ballot (0.125 ZEC) — you cannot delegate until you hold more Ironwood at the snapshot."
        )
    } else {
        String::new()
    };
    Ok(VoteExportNotesResponse {
        notes_path: out.display().to_string(),
        note_count: file.notes.len(),
        total_value_zat,
        message: format!(
            "Exported {} Ironwood note(s) at snapshot {} for voting.{}",
            file.notes.len(),
            snapshot_height,
            ballot_hint
        ),
    })
}

#[derive(Debug, Deserialize)]
pub struct VotePrepareRequest {
    #[serde(default = "default_env")]
    pub env: String,
}

#[derive(Debug, Serialize)]
pub struct VotePrepareResult {
    pub round_id: String,
    pub message: String,
    pub stdout: String,
}

#[command]
pub async fn vote_prepare(request: VotePrepareRequest) -> Result<VotePrepareResult, TauriError> {
    let env = request.env.clone();
    let network = network_label();
    run_blocking(move || {
        let data_dir = ensure_vote_data_dir()?;
        let notes = notes_path(&data_dir);
        if !notes.is_file() {
            return Err(map_err("export Ironwood notes first (Vote → Export)"));
        }
        let notes_s = notes.display().to_string();
        let _ = run_nozy_vote(&["--env", &env, "hotkey-init", "--network", &network])?;
        let init_out = run_nozy_vote(&["--env", &env, "init-round", "--network", &network])?;
        let import_out = run_nozy_vote(&[
            "--env",
            &env,
            "import-notes",
            "--network",
            &network,
            "--file",
            &notes_s,
        ])
        .map_err(|e| {
            
            e
        })?;
        
        let round_id = extract_round_id(&init_out).unwrap_or_default();
        Ok(VotePrepareResult {
            round_id,
            message: "Prepared voting hotkey, round, and imported notes.".into(),
            stdout: format!("{init_out}\n{import_out}"),
        })
    })
    .await
}

fn extract_round_id(stdout: &str) -> Option<String> {
    for line in stdout.lines() {
        if let Some(rest) = line.trim().strip_prefix("round_id:") {
            return Some(rest.trim().to_string());
        }
        if let Some(rest) = line.trim().strip_prefix("round_id: ") {
            return Some(rest.trim().to_string());
        }
    }
    // "  round_id: HEX"
    for line in stdout.lines() {
        if line.contains("round_id:") {
            return line.split("round_id:").nth(1).map(|s| s.trim().to_string());
        }
    }
    None
}

#[derive(Debug, Deserialize)]
pub struct VoteDelegateRequest {
    #[serde(default = "default_env")]
    pub env: String,
}

#[derive(Debug, Serialize)]
pub struct VoteDelegatePrepareResult {
    pub message: String,
    pub stdout: String,
}

#[command]
pub async fn vote_delegate(
    request: VoteDelegateRequest,
) -> Result<VoteDelegatePrepareResult, TauriError> {
    let env = request.env.clone();
    let network = network_label();
    run_blocking(move || {
        let data_dir = ensure_vote_data_dir()?;
        let notes = notes_path(&data_dir);
        if !notes.is_file() {
            return Err(map_err("export Ironwood notes first"));
        }
        let notes_s = notes.display().to_string();
        let stdout = run_nozy_vote(&[
            "--env",
            &env,
            "delegate",
            "--network",
            &network,
            "--notes-file",
            &notes_s,
        ])?;
        Ok(VoteDelegatePrepareResult {
            message: "Delegation signing request written.".into(),
            stdout,
        })
    })
    .await
}

#[derive(Debug, Deserialize)]
pub struct VoteSignRequest {
    pub password: Option<String>,
    #[serde(default = "default_env")]
    pub env: String,
}

#[derive(Debug, Serialize)]
pub struct VoteSignResponse {
    pub round_id: String,
    pub sig_path: String,
    pub message: String,
}

#[command]
pub async fn vote_sign_delegation(
    request: VoteSignRequest,
) -> Result<VoteSignResponse, TauriError> {
    let env = request.env.clone();
    let wallet = load_session_wallet(request.password.as_deref()).await?;
    let data_dir = ensure_vote_data_dir()?;
    let active_json = run_blocking({
        let env = env.clone();
        move || run_nozy_vote(&["--env", &env, "active", "--json"])
    })
    .await?;
    let active: Value =
        serde_json::from_str(&active_json).map_err(|e| map_err(format!("decode active: {e}")))?;
    let rid = active
        .get("vote_round_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| map_err("active round missing vote_round_id"))?
        .to_string();
    // Prefer hex form from signing-request filename: scan data dir
    let req_path =
        find_signing_request(&data_dir)?.unwrap_or_else(|| signing_request_path(&data_dir, &rid));
    if !req_path.is_file() {
        return Err(map_err(
            "No signing request yet — run Prepare / Delegate first.",
        ));
    }
    let round_from_file = req_path
        .file_stem()
        .and_then(|s| s.to_str())
        .and_then(|s| s.strip_prefix("signing-request-"))
        .unwrap_or(&rid)
        .to_string();
    let out = sig_path(&data_dir, &round_from_file);
    let _sig = sign_delegation_request(&wallet, &req_path, &out).map_err(TauriError::from)?;
    Ok(VoteSignResponse {
        round_id: round_from_file,
        sig_path: out.display().to_string(),
        message: "Delegation signed with this wallet.".into(),
    })
}

fn find_signing_request(data_dir: &Path) -> Result<Option<PathBuf>, TauriError> {
    let mut best: Option<PathBuf> = None;
    let entries =
        std::fs::read_dir(data_dir).map_err(|e| map_err(format!("read vote data dir: {e}")))?;
    for ent in entries.flatten() {
        let name = ent.file_name();
        let name = name.to_string_lossy();
        if name.starts_with("signing-request-") && name.ends_with(".json") {
            best = Some(ent.path());
        }
    }
    Ok(best)
}

#[derive(Debug, Deserialize)]
pub struct VoteDelegateFinishRequest {
    #[serde(default = "default_env")]
    pub env: String,
    #[serde(default = "default_true")]
    pub wait: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize)]
pub struct VoteDelegateFinishResult {
    pub tx_hash: String,
    pub confirmed: bool,
    pub stdout: String,
}

#[command]
pub async fn vote_delegate_finish(
    request: VoteDelegateFinishRequest,
) -> Result<VoteDelegateFinishResult, TauriError> {
    let env = request.env.clone();
    let wait = request.wait;
    let network = network_label();
    run_blocking(move || {
        let data_dir = ensure_vote_data_dir()?;
        let notes = notes_path(&data_dir);
        let sig = find_sig_file(&data_dir)?.ok_or_else(|| map_err("sign the delegation first"))?;
        let notes_s = notes.display().to_string();
        let sig_s = sig.display().to_string();
        let mut args = vec![
            "--env".to_string(),
            env.clone(),
            "delegate-finish".into(),
            "--network".into(),
            network,
            "--notes-file".into(),
            notes_s,
            "--sig".into(),
            sig_s,
        ];
        if !wait {
            args.push("--no-wait".into());
        }
        let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let stdout = run_nozy_vote(&arg_refs)?;
        let tx_hash = extract_tx_hash(&stdout).unwrap_or_default();
        let confirmed = stdout.to_lowercase().contains("confirmed");
        if !tx_hash.is_empty() {
            let _ = save_delegation_tx(&data_dir, &tx_hash);
            
        }
        Ok(VoteDelegateFinishResult {
            tx_hash,
            confirmed,
            stdout,
        })
    })
    .await
}

fn find_sig_file(data_dir: &Path) -> Result<Option<PathBuf>, TauriError> {
    let mut best: Option<PathBuf> = None;
    let entries =
        std::fs::read_dir(data_dir).map_err(|e| map_err(format!("read vote data dir: {e}")))?;
    for ent in entries.flatten() {
        let name = ent.file_name();
        let name = name.to_string_lossy();
        if name.starts_with("delegation-sig-") && name.ends_with(".json") {
            best = Some(ent.path());
        }
    }
    Ok(best)
}

fn extract_tx_hash(stdout: &str) -> Option<String> {
    for line in stdout.lines() {
        if let Some(idx) = line.find("tx ") {
            let rest = line[idx + 3..].trim();
            let token = rest.split_whitespace().next()?;
            if token.len() >= 16 {
                return Some(token.to_string());
            }
        }
    }
    None
}

#[derive(Debug, Deserialize)]
pub struct VoteCastRequest {
    #[serde(default = "default_env")]
    pub env: String,
    pub choices: HashMap<String, u32>,
    pub delegation_tx: Option<String>,
    #[serde(default)]
    pub single_share: bool,
    #[serde(default = "default_true")]
    pub wait: bool,
}

#[derive(Debug, Serialize)]
pub struct VoteCastResult {
    pub proposal_count: usize,
    pub stdout: String,
}

#[command]
pub async fn vote_cast(request: VoteCastRequest) -> Result<VoteCastResult, TauriError> {
    let env = request.env.clone();
    let network = network_label();
    let mut choice_strs: Vec<String> = request
        .choices
        .iter()
        .map(|(pid, opt)| format!("{pid}={opt}"))
        .collect();
    choice_strs.sort();
    let choices_joined = choice_strs.join(",");
    let proposal_count = choice_strs.len();
    let data_dir = match ensure_vote_data_dir() {
        Ok(d) => d,
        Err(e) => return Err(e),
    };
    let persisted = load_delegation_tx(&data_dir);
    let delegation_tx = request
        .delegation_tx
        .clone()
        .filter(|s| s.trim().len() >= 16)
        .or(persisted);
    let single_share = request.single_share;
    let wait = request.wait;
    
    if delegation_tx.is_none() {
        
        return Err(map_err(
            "No confirmed delegation yet. Complete step 5 (Submit delegation) first, \
             wait until it says confirmed, then cast again."
                .to_string(),
        ));
    }
    run_blocking(move || {
        let mut args = vec![
            "--env".to_string(),
            env,
            "cast".into(),
            "--network".into(),
            network,
            "--choices".into(),
            choices_joined,
        ];
        if let Some(tx) = delegation_tx {
            args.push("--delegation-tx".into());
            args.push(tx);
        }
        if single_share {
            args.push("--single-share".into());
        }
        if !wait {
            args.push("--no-wait".into());
        }
        let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match run_nozy_vote(&arg_refs) {
            Ok(stdout) => {
                
                Ok(VoteCastResult {
                    proposal_count,
                    stdout,
                })
            }
            Err(e) => {
                let msg = if e.message.contains("van_leaf_position")
                    || e.message.contains("van_witness")
                {
                    format!(
                        "Delegation is not confirmed in the vote DB yet (missing VAN). \
                         Re-run step 5 (Submit delegation), wait for confirmation, then cast. \
                         Detail: {}",
                        e.message.chars().take(400).collect::<String>()
                    )
                } else {
                    e.message.clone()
                };
                
                Err(map_err(msg))
            }
        }
    })
    .await
}
