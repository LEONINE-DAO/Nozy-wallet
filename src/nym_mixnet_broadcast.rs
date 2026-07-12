//! Opt-in Nym mixnet broadcast for remote `sendrawtransaction` (issue #147).
//!
//! Enable with `NOZY_BROADCAST_VIA_NYM_MIXNET=1` and/or
//! `privacy_network.broadcast_via_nym_mixnet` in config.
//!
//! Local / RFC1918 Zebrad stays on the direct path (Case A1). Remote submits are
//! handed to the **subprocess** helper `nym-smolmix-broadcast-spike --sendraw-stdin`
//! (avoids linking smolmix into `nozy`: sqlite `links` clash with zeaking).
//!
//! Set `NOZY_NYM_SMOLMIX_BIN` to the helper path (recommended). Optional `NOZY_NYM_IPR`.

use std::path::{Path, PathBuf};
use std::process::Stdio;

use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::error::{NozyError, NozyResult};
use crate::zebra_integration::ZebraClient;

const ENV_FLAG: &str = "NOZY_BROADCAST_VIA_NYM_MIXNET";
const ENV_BIN: &str = "NOZY_NYM_SMOLMIX_BIN";
const ENV_IPR: &str = "NOZY_NYM_IPR";

fn env_flag_truthy(name: &str) -> bool {
    match std::env::var(name) {
        Ok(v) => {
            let t = v.trim();
            t == "1" || t.eq_ignore_ascii_case("true") || t.eq_ignore_ascii_case("yes")
        }
        Err(_) => false,
    }
}

/// True when operator opted into mixnet broadcast (env and/or config).
pub fn mixnet_broadcast_requested(config_flag: bool) -> bool {
    config_flag || env_flag_truthy(ENV_FLAG)
}

/// Env-only convenience (status / connection_mode when config not handy).
pub fn env_enabled() -> bool {
    env_flag_truthy(ENV_FLAG)
}

fn helper_exe_name() -> &'static str {
    if cfg!(windows) {
        "nym-smolmix-broadcast-spike.exe"
    } else {
        "nym-smolmix-broadcast-spike"
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
                .join("tools/nym-smolmix-broadcast-spike/target")
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
        "{ENV_FLAG} is set for remote broadcast, but helper binary `{name}` was not found. \
         Build it (`cd tools/nym-smolmix-broadcast-spike && cargo build --release`) and set \
         {ENV_BIN} to the full path. See docs/reference/NYM_IP_PRIVACY_CASE_BREAKDOWN.md D2c."
    )))
}

async fn broadcast_via_helper(zebra_url: &str, raw_tx_hex: &str) -> NozyResult<String> {
    let bin = resolve_helper_bin()?;
    let ipr = std::env::var(ENV_IPR).ok().filter(|s| !s.trim().is_empty());

    tracing::info!(
        target: "nozy::nym_mixnet",
        bin = %bin.display(),
        zebra = %zebra_url,
        "sendrawtransaction via Nym smolmix helper subprocess"
    );

    let mut cmd = Command::new(&bin);
    cmd.arg("--sendraw-stdin")
        .arg("--zebra")
        .arg(zebra_url)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    if let Some(ipr) = ipr.as_deref() {
        cmd.arg("--ipr").arg(ipr);
    }

    let mut child = cmd.spawn().map_err(|e| {
        NozyError::NetworkError(format!(
            "failed to spawn Nym smolmix helper {}: {e}",
            bin.display()
        ))
    })?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(raw_tx_hex.as_bytes())
            .await
            .map_err(|e| NozyError::NetworkError(format!("helper stdin write failed: {e}")))?;
        stdin
            .shutdown()
            .await
            .map_err(|e| NozyError::NetworkError(format!("helper stdin close failed: {e}")))?;
    }

    let output = child
        .wait_with_output()
        .await
        .map_err(|e| NozyError::NetworkError(format!("Nym smolmix helper wait failed: {e}")))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.trim().is_empty() {
        tracing::debug!(target: "nozy::nym_mixnet", helper_stderr = %stderr.trim());
    }

    if !output.status.success() {
        return Err(NozyError::NetworkError(format!(
            "Nym smolmix helper exited {}: {}",
            output.status,
            stderr.trim().lines().last().unwrap_or(stdout.trim())
        )));
    }

    let txid = stdout
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .ok_or_else(|| {
            NozyError::NetworkError(format!(
                "Nym smolmix helper produced no txid on stdout (stderr: {})",
                stderr.trim()
            ))
        })?
        .to_string();

    if txid.len() < 64 || !txid.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(NozyError::NetworkError(format!(
            "Nym smolmix helper returned unexpected txid line: {txid:?}"
        )));
    }

    Ok(txid)
}

/// If mixnet broadcast is requested for a remote URL, run the helper.
/// Returns `Ok(None)` when the caller should use the normal clearnet/Tor path.
pub async fn maybe_broadcast_via_nym_mixnet(
    zebra_url: &str,
    raw_tx_hex: &str,
    enabled: bool,
) -> NozyResult<Option<String>> {
    if !enabled {
        return Ok(None);
    }
    if ZebraClient::url_is_local(zebra_url) {
        tracing::info!(
            target: "nozy::nym_mixnet",
            "{ENV_FLAG}/config set but zebra URL is local/LAN — using direct path (Case A1)"
        );
        return Ok(None);
    }

    // Fail closed if helper missing or private URL (helper also refuses RFC1918).
    let txid = broadcast_via_helper(zebra_url, raw_tx_hex).await?;
    Ok(Some(txid))
}

/// Whether a path looks like the helper (tests / diagnostics).
#[allow(dead_code)]
pub fn helper_path_looks_valid(path: &Path) -> bool {
    path.is_file()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixnet_requested_from_config_alone() {
        assert!(mixnet_broadcast_requested(true));
    }
}
