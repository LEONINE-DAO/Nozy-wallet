//! Companion HTTP handlers for NU7 coinholder vote (Valar Shielded Vote).
//!
//! Export/sign use `nozy` in-process. Prepare/delegate/cast shell out to the
//! `nozy-vote` helper — `zcash_voting` cannot link beside `zeaking` (sqlite).
//! Tracking: https://github.com/LEONINE-DAO/Nozy-wallet/issues/273

use axum::{
    extract::{Json, Query},
    http::StatusCode,
    response::Json as ResponseJson,
};
use nozy::{export_ironwood_vote_notes, load_config, sign_delegation_request};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::handlers::{error_response_with_code, load_wallet_with_password};

type ApiError = (StatusCode, ResponseJson<Value>);

fn vote_err(status: StatusCode, msg: impl Into<String>) -> ApiError {
    error_response_with_code(status, msg, "VOTE_ERROR")
}

fn default_vote_data_dir() -> Result<PathBuf, ApiError> {
    directories::ProjectDirs::from("org", "LeonineDAO", "NozyVote")
        .map(|d| d.data_dir().to_path_buf())
        .ok_or_else(|| {
            vote_err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "resolve NozyVote data dir",
            )
        })
}

fn ensure_vote_data_dir() -> Result<PathBuf, ApiError> {
    let dir = default_vote_data_dir()?;
    std::fs::create_dir_all(&dir).map_err(|e| {
        vote_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("create vote data dir: {e}"),
        )
    })?;
    Ok(dir)
}

fn notes_path(data_dir: &Path) -> PathBuf {
    data_dir.join("vote-notes.json")
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

fn resolve_nozy_vote_bin() -> Result<PathBuf, ApiError> {
    if let Ok(p) = std::env::var("NOZY_VOTE_BIN") {
        let path = PathBuf::from(p);
        if path.is_file() {
            return Ok(path);
        }
        return Err(vote_err(
            StatusCode::SERVICE_UNAVAILABLE,
            format!("NOZY_VOTE_BIN set but not a file: {}", path.display()),
        ));
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

    Err(vote_err(
        StatusCode::SERVICE_UNAVAILABLE,
        "nozy-vote binary not found. Build tools/nozy-vote (`cargo build --release`) \
         or set NOZY_VOTE_BIN. (api-server cannot link zcash_voting beside zeaking.)",
    ))
}

fn run_nozy_vote(args: &[&str]) -> Result<String, ApiError> {
    let bin = resolve_nozy_vote_bin()?;
    let data_dir = ensure_vote_data_dir()?;
    let output = Command::new(&bin)
        .args(args)
        .arg("--data-dir")
        .arg(&data_dir)
        .output()
        .map_err(|e| {
            vote_err(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("spawn {}: {e}", bin.display()),
            )
        })?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if !output.status.success() {
        let detail = if !stderr.trim().is_empty() {
            stderr
        } else {
            stdout
        };
        return Err(vote_err(
            StatusCode::BAD_REQUEST,
            format!("nozy-vote {} failed: {}", args.join(" "), detail.trim()),
        ));
    }
    Ok(stdout)
}

async fn run_blocking<T, F>(f: F) -> Result<T, ApiError>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, ApiError> + Send + 'static,
{
    tokio::task::spawn_blocking(f).await.map_err(|e| {
        vote_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("vote task join: {e}"),
        )
    })?
}

fn default_env() -> String {
    "prod".into()
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct VoteEnvQuery {
    #[serde(default = "default_env")]
    pub env: String,
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

pub async fn vote_status(
    Query(q): Query<VoteEnvQuery>,
) -> Result<ResponseJson<VoteStatusResponse>, ApiError> {
    let env = q.env.clone();
    let status = run_blocking(move || {
        let out = run_nozy_vote(&["--env", &env, "status", "--json"])?;
        serde_json::from_str(&out).map_err(|e| {
            vote_err(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("decode status json: {e}\n{out}"),
            )
        })
    })
    .await?;
    Ok(ResponseJson(status))
}

pub async fn vote_active(Query(q): Query<VoteEnvQuery>) -> Result<ResponseJson<Value>, ApiError> {
    let env = q.env.clone();
    let active = run_blocking(move || {
        let out = run_nozy_vote(&["--env", &env, "active", "--json"])?;
        serde_json::from_str(&out).map_err(|e| {
            vote_err(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("decode active json: {e}\n{out}"),
            )
        })
    })
    .await?;
    Ok(ResponseJson(active))
}

#[derive(Debug, Deserialize, Default)]
pub struct VotePasswordBody {
    pub password: Option<String>,
    #[serde(default = "default_env")]
    pub env: String,
}

#[derive(Debug, Serialize)]
pub struct VoteExportNotesResponse {
    pub notes_path: String,
    pub note_count: usize,
    pub total_value_zat: u64,
    pub message: String,
}

pub async fn vote_export_notes(
    Json(body): Json<VotePasswordBody>,
) -> Result<ResponseJson<VoteExportNotesResponse>, ApiError> {
    let (wallet, _storage) = load_wallet_with_password(body.password)
        .await
        .map_err(|e| vote_err(StatusCode::UNAUTHORIZED, e))?;
    let network = consensus_network();
    let data_dir = ensure_vote_data_dir()?;
    let out = notes_path(&data_dir);
    let file = export_ironwood_vote_notes(&wallet, network, &out)
        .map_err(|e| vote_err(StatusCode::BAD_REQUEST, e.user_friendly_message()))?;
    let total_value_zat = file.notes.iter().map(|n| n.value).sum();
    Ok(ResponseJson(VoteExportNotesResponse {
        notes_path: out.display().to_string(),
        note_count: file.notes.len(),
        total_value_zat,
        message: format!("Exported {} Ironwood note(s) for voting.", file.notes.len()),
    }))
}

#[derive(Debug, Deserialize)]
pub struct VoteEnvBody {
    #[serde(default = "default_env")]
    pub env: String,
}

#[derive(Debug, Serialize)]
pub struct VotePrepareResult {
    pub round_id: String,
    pub message: String,
    pub stdout: String,
}

pub async fn vote_prepare(
    Json(body): Json<VoteEnvBody>,
) -> Result<ResponseJson<VotePrepareResult>, ApiError> {
    let env = body.env.clone();
    let network = network_label();
    let result = run_blocking(move || {
        let data_dir = ensure_vote_data_dir()?;
        let notes = notes_path(&data_dir);
        if !notes.is_file() {
            return Err(vote_err(
                StatusCode::BAD_REQUEST,
                "export Ironwood notes first (POST /api/vote/export-notes)",
            ));
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
        ])?;
        let round_id = extract_round_id(&init_out).unwrap_or_default();
        Ok(VotePrepareResult {
            round_id,
            message: "Prepared voting hotkey, round, and imported notes.".into(),
            stdout: format!("{init_out}\n{import_out}"),
        })
    })
    .await?;
    Ok(ResponseJson(result))
}

fn extract_round_id(stdout: &str) -> Option<String> {
    for line in stdout.lines() {
        if line.contains("round_id:") {
            return line.split("round_id:").nth(1).map(|s| s.trim().to_string());
        }
    }
    None
}

#[derive(Debug, Serialize)]
pub struct VoteDelegatePrepareResult {
    pub message: String,
    pub stdout: String,
}

pub async fn vote_delegate(
    Json(body): Json<VoteEnvBody>,
) -> Result<ResponseJson<VoteDelegatePrepareResult>, ApiError> {
    let env = body.env.clone();
    let network = network_label();
    let result = run_blocking(move || {
        let data_dir = ensure_vote_data_dir()?;
        let notes = notes_path(&data_dir);
        if !notes.is_file() {
            return Err(vote_err(
                StatusCode::BAD_REQUEST,
                "export Ironwood notes first",
            ));
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
    .await?;
    Ok(ResponseJson(result))
}

#[derive(Debug, Serialize)]
pub struct VoteSignResponse {
    pub round_id: String,
    pub sig_path: String,
    pub message: String,
}

pub async fn vote_sign_delegation(
    Json(body): Json<VotePasswordBody>,
) -> Result<ResponseJson<VoteSignResponse>, ApiError> {
    let env = body.env.clone();
    let (wallet, _storage) = load_wallet_with_password(body.password)
        .await
        .map_err(|e| vote_err(StatusCode::UNAUTHORIZED, e))?;
    let data_dir = ensure_vote_data_dir()?;
    let active_json = run_blocking({
        let env = env.clone();
        move || run_nozy_vote(&["--env", &env, "active", "--json"])
    })
    .await?;
    let active: Value = serde_json::from_str(&active_json).map_err(|e| {
        vote_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("decode active: {e}"),
        )
    })?;
    let rid = active
        .get("vote_round_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            vote_err(
                StatusCode::BAD_REQUEST,
                "active round missing vote_round_id",
            )
        })?
        .to_string();
    let req_path =
        find_signing_request(&data_dir)?.unwrap_or_else(|| signing_request_path(&data_dir, &rid));
    if !req_path.is_file() {
        return Err(vote_err(
            StatusCode::BAD_REQUEST,
            "No signing request yet — run prepare / delegate first.",
        ));
    }
    let round_from_file = req_path
        .file_stem()
        .and_then(|s| s.to_str())
        .and_then(|s| s.strip_prefix("signing-request-"))
        .unwrap_or(&rid)
        .to_string();
    let out = sig_path(&data_dir, &round_from_file);
    let _sig = sign_delegation_request(&wallet, &req_path, &out)
        .map_err(|e| vote_err(StatusCode::BAD_REQUEST, e.user_friendly_message()))?;
    Ok(ResponseJson(VoteSignResponse {
        round_id: round_from_file,
        sig_path: out.display().to_string(),
        message: "Delegation signed with companion wallet.".into(),
    }))
}

fn find_signing_request(data_dir: &Path) -> Result<Option<PathBuf>, ApiError> {
    let mut best: Option<PathBuf> = None;
    let entries = std::fs::read_dir(data_dir).map_err(|e| {
        vote_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("read vote data dir: {e}"),
        )
    })?;
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
pub struct VoteDelegateFinishBody {
    #[serde(default = "default_env")]
    pub env: String,
    #[serde(default = "default_true")]
    pub wait: bool,
}

#[derive(Debug, Serialize)]
pub struct VoteDelegateFinishResult {
    pub tx_hash: String,
    pub confirmed: bool,
    pub stdout: String,
}

pub async fn vote_delegate_finish(
    Json(body): Json<VoteDelegateFinishBody>,
) -> Result<ResponseJson<VoteDelegateFinishResult>, ApiError> {
    let env = body.env.clone();
    let wait = body.wait;
    let network = network_label();
    let result = run_blocking(move || {
        let data_dir = ensure_vote_data_dir()?;
        let notes = notes_path(&data_dir);
        let sig = find_sig_file(&data_dir)?
            .ok_or_else(|| vote_err(StatusCode::BAD_REQUEST, "sign the delegation first"))?;
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
        Ok(VoteDelegateFinishResult {
            tx_hash,
            confirmed,
            stdout,
        })
    })
    .await?;
    Ok(ResponseJson(result))
}

fn find_sig_file(data_dir: &Path) -> Result<Option<PathBuf>, ApiError> {
    let mut best: Option<PathBuf> = None;
    let entries = std::fs::read_dir(data_dir).map_err(|e| {
        vote_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("read vote data dir: {e}"),
        )
    })?;
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
pub struct VoteCastBody {
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

pub async fn vote_cast(
    Json(body): Json<VoteCastBody>,
) -> Result<ResponseJson<VoteCastResult>, ApiError> {
    let env = body.env.clone();
    let network = network_label();
    let mut choice_strs: Vec<String> = body
        .choices
        .iter()
        .map(|(pid, opt)| format!("{pid}={opt}"))
        .collect();
    choice_strs.sort();
    let choices_joined = choice_strs.join(",");
    let proposal_count = choice_strs.len();
    let delegation_tx = body.delegation_tx.clone();
    let single_share = body.single_share;
    let wait = body.wait;
    let result = run_blocking(move || {
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
        let stdout = run_nozy_vote(&arg_refs)?;
        Ok(VoteCastResult {
            proposal_count,
            stdout,
        })
    })
    .await?;
    Ok(ResponseJson(result))
}
