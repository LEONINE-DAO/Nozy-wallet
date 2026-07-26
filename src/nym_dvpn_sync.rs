//! Opt-in Nym dVPN compact sync via **subprocess** (issue #146 / track C6).
//!
//! Enable with `NOZY_SYNC_VIA_NYM_DVPN=1` and/or
//! `privacy_network.sync_via_nym_dvpn` in config.
//!
//! Spawns `nym-dvpn-lwd-spike` (same sqlite-link reason as smolmix: do **not**
//! link `smoldvpn` into `nozy` / Tauri). Local/loopback LWD is refused for dVPN
//! (exit cannot reach it — Case C4).
//!
//! Set `NOZY_NYM_DVPN_BIN` to the helper path. Probe/sync needs funded
//! `MNEMONIC` / `NYX_ACCOUNT_MNEMONIC` in the process environment (never store
//! the seed in config.json).

use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use serde::Serialize;
use tokio::process::Command;

use crate::error::{NozyError, NozyResult};
use crate::zebra_integration::ZebraClient;

const ENV_FLAG: &str = "NOZY_SYNC_VIA_NYM_DVPN";
const ENV_BIN: &str = "NOZY_NYM_DVPN_BIN";
const DEFAULT_PROBE_BLOCKS: u64 = 100;
const PROBE_TIMEOUT: Duration = Duration::from_secs(900);

fn env_flag_truthy(name: &str) -> bool {
    match std::env::var(name) {
        Ok(v) => {
            let t = v.trim();
            t == "1" || t.eq_ignore_ascii_case("true") || t.eq_ignore_ascii_case("yes")
        }
        Err(_) => false,
    }
}

/// True when operator opted into dVPN compact sync (env and/or config).
pub fn dvpn_sync_requested(config_flag: bool) -> bool {
    config_flag || env_flag_truthy(ENV_FLAG)
}

pub fn env_enabled() -> bool {
    env_flag_truthy(ENV_FLAG)
}

fn helper_exe_name() -> &'static str {
    if cfg!(windows) {
        "nym-dvpn-lwd-spike.exe"
    } else {
        "nym-dvpn-lwd-spike"
    }
}

pub fn resolve_helper_bin() -> NozyResult<PathBuf> {
    if let Ok(p) = std::env::var(ENV_BIN) {
        let path = PathBuf::from(p.trim());
        if path.is_file() {
            return Ok(path);
        }
        return Err(NozyError::InvalidOperation(format!(
            "{ENV_BIN}={path:?} is not a file. Build the spike and point this env at the binary."
        )));
    }

    let name = helper_exe_name();
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join(name));
        }
    }

    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for profile in ["release", "debug"] {
        candidates.push(
            manifest
                .join("tools/nym-dvpn-lwd-spike/target")
                .join(profile)
                .join(name),
        );
        candidates.push(manifest.join("target").join(profile).join(name));
    }

    for c in &candidates {
        if c.is_file() {
            return Ok(c.clone());
        }
    }

    Err(NozyError::InvalidOperation(format!(
        "Nym dVPN sync helper `{name}` was not found. Build it \
         (`cd tools/nym-dvpn-lwd-spike && cargo build --release`) and set {ENV_BIN} \
         to the full path. See docs/reference/NYM_DVPN_SYNC_CASE_BREAKDOWN.md C6."
    )))
}

fn resolve_lwd_url(override_url: Option<&str>) -> String {
    override_url
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .or_else(|| std::env::var("LIGHTWALLETD_GRPC").ok())
        .unwrap_or_else(|| "http://127.0.0.1:9067".to_string())
}

fn mnemonic_env_present() -> bool {
    std::env::var("MNEMONIC")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .is_some()
        || std::env::var("NYX_ACCOUNT_MNEMONIC")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .is_some()
}

/// Operator-facing readiness for C6 (no tunnel; instant).
#[derive(Debug, Clone, Serialize)]
pub struct NymDvpnSyncReadiness {
    pub requested: bool,
    pub lwd_url: String,
    pub lwd_url_local: bool,
    pub would_use_dvpn: bool,
    pub helper_ok: bool,
    pub helper_path: Option<String>,
    pub helper_error: Option<String>,
    pub mnemonic_env_ok: bool,
    pub notes: Vec<String>,
}

pub fn assess_dvpn_sync_readiness(
    config_flag: bool,
    lwd_url_override: Option<&str>,
) -> NymDvpnSyncReadiness {
    let requested = dvpn_sync_requested(config_flag);
    let lwd_url = resolve_lwd_url(lwd_url_override);
    let lwd_url_local = ZebraClient::url_is_local(&lwd_url);
    let mnemonic_env_ok = mnemonic_env_present();
    let mut notes = Vec::new();

    let (helper_ok, helper_path, helper_error) = match resolve_helper_bin() {
        Ok(p) => (true, Some(p.display().to_string()), None),
        Err(e) => (false, None, Some(e.to_string())),
    };

    if !requested {
        notes.push(
            "dVPN compact sync not requested (set privacy_network.sync_via_nym_dvpn or \
             NOZY_SYNC_VIA_NYM_DVPN=1)."
                .to_string(),
        );
    }
    if lwd_url_local {
        notes.push(
            "LWD URL is local/LAN — dVPN exit cannot reach it (Case C4). Use a public LWD \
             such as https://zec.rocks:443 for the probe, or sync local LWD directly."
                .to_string(),
        );
    }
    if requested && !mnemonic_env_ok {
        notes.push(
            "Funded MNEMONIC / NYX_ACCOUNT_MNEMONIC is not set in this process env \
             (required for ticketbooks; never put the seed in config.json)."
                .to_string(),
        );
    }
    if requested && helper_ok && !lwd_url_local && mnemonic_env_ok {
        notes.push(
            "Ready to spawn nym-dvpn-lwd-spike for public LWD. Disconnect consumer NymVPN \
             (nym-vpnd) while measuring. Sync path ≠ mixnet broadcast."
                .to_string(),
        );
    } else if requested && !helper_ok {
        notes.push("dVPN sync requested, but helper binary is missing.".to_string());
    }

    notes.push(
        "Tracking: issue #146 · docs/reference/NYM_DVPN_SYNC_CASE_BREAKDOWN.md. \
         zeaking connect_with_connector is C5; this helper is the C6 subprocess bridge."
            .to_string(),
    );

    NymDvpnSyncReadiness {
        requested,
        lwd_url,
        lwd_url_local,
        would_use_dvpn: requested && !lwd_url_local && helper_ok && mnemonic_env_ok,
        helper_ok,
        helper_path,
        helper_error,
        mnemonic_env_ok,
        notes,
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct NymDvpnSyncProbeResult {
    pub ok: bool,
    pub exit_code: Option<i32>,
    pub helper_path: String,
    pub lwd_url: String,
    pub blocks: u64,
    pub stdout_tail: String,
    pub stderr_tail: String,
    pub timed_out: bool,
}

fn tail_chars(s: &str, max: usize) -> String {
    let t = s.trim();
    if t.chars().count() <= max {
        return t.to_string();
    }
    let skip = t.chars().count().saturating_sub(max);
    t.chars().skip(skip).collect()
}

/// Run a bounded compact-sync probe through the dVPN spike subprocess.
pub async fn run_dvpn_sync_probe(
    config_flag: bool,
    lwd_url_override: Option<&str>,
    blocks: Option<u64>,
) -> NozyResult<NymDvpnSyncProbeResult> {
    let readiness = assess_dvpn_sync_readiness(config_flag, lwd_url_override);
    if !readiness.requested {
        return Err(NozyError::InvalidOperation(
            "dVPN sync not enabled (privacy_network.sync_via_nym_dvpn or NOZY_SYNC_VIA_NYM_DVPN=1)"
                .into(),
        ));
    }
    if readiness.lwd_url_local {
        return Err(NozyError::InvalidOperation(format!(
            "refusing dVPN sync to local/LAN LWD {} (Case C4)",
            readiness.lwd_url
        )));
    }
    if !readiness.mnemonic_env_ok {
        return Err(NozyError::InvalidOperation(
            "set MNEMONIC or NYX_ACCOUNT_MNEMONIC in the environment before running the probe"
                .into(),
        ));
    }

    let bin = resolve_helper_bin()?;
    let blocks = blocks.unwrap_or(DEFAULT_PROBE_BLOCKS).max(1);
    let lwd = readiness.lwd_url.clone();

    tracing::info!(
        target: "nozy::nym_dvpn",
        bin = %bin.display(),
        lwd = %lwd,
        blocks,
        "compact sync probe via Nym dVPN helper subprocess"
    );

    let mut cmd = Command::new(&bin);
    cmd.arg("--blocks")
        .arg(blocks.to_string())
        .arg("--lwd")
        .arg(&lwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    let child = cmd.spawn().map_err(|e| {
        NozyError::NetworkError(format!(
            "failed to spawn Nym dVPN helper {}: {e}",
            bin.display()
        ))
    })?;

    let wait = child.wait_with_output();
    let output = match tokio::time::timeout(PROBE_TIMEOUT, wait).await {
        Ok(Ok(o)) => o,
        Ok(Err(e)) => {
            return Err(NozyError::NetworkError(format!(
                "Nym dVPN helper wait failed: {e}"
            )));
        }
        Err(_) => {
            return Ok(NymDvpnSyncProbeResult {
                ok: false,
                exit_code: None,
                helper_path: bin.display().to_string(),
                lwd_url: lwd,
                blocks,
                stdout_tail: String::new(),
                stderr_tail: format!(
                    "timed out after {}s (tunnel provision can be slow; check helper logs)",
                    PROBE_TIMEOUT.as_secs()
                ),
                timed_out: true,
            });
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let code = output.status.code();
    let ok = output.status.success() && stdout.contains("PASS:");

    Ok(NymDvpnSyncProbeResult {
        ok,
        exit_code: code,
        helper_path: bin.display().to_string(),
        lwd_url: lwd,
        blocks,
        stdout_tail: tail_chars(&stdout, 4000),
        stderr_tail: tail_chars(&stderr, 2000),
        timed_out: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_lwd_not_would_use_dvpn() {
        let r = assess_dvpn_sync_readiness(true, Some("http://127.0.0.1:9067"));
        assert!(r.requested);
        assert!(r.lwd_url_local);
        assert!(!r.would_use_dvpn);
    }

    #[test]
    fn public_lwd_classified_remote() {
        let r = assess_dvpn_sync_readiness(false, Some("https://zec.rocks:443"));
        assert!(!r.requested);
        assert!(!r.lwd_url_local);
    }
}
