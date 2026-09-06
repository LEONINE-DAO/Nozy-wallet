//! Detect the **consumer NymVPN OS app** (nym-vpnd / nym-vpnc).
//!
//! Used to gate the in-wallet browser. This is not mixnet sendraw and not dVPN
//! compact sync — Chrome cannot embed the Nym SDK or start the VPN.
//!
//! Status comes from `nym-vpnc status` when that CLI is on PATH (or
//! `NOZY_NYM_VPNC`). On Windows the consumer installer often omits `nym-vpnc`,
//! so we also look for NymVPN’s Wintun adapters (`WireGuard (entry/exit)`, `Nym`).

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use serde::Serialize;
use tokio::process::Command;

const ENV_VPNC: &str = "NOZY_NYM_VPNC";
const PROBE_TIMEOUT: Duration = Duration::from_secs(3);
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Snapshot for GET `/api/privacy/nym-vpn-app`.
#[derive(Debug, Clone, Serialize)]
pub struct NymVpnAppStatus {
    pub daemon_present: bool,
    pub vpnc_found: bool,
    pub vpnc_path: Option<String>,
    /// Tunnel is up (Fast / Mixnet). Connecting / disconnected are false.
    pub connected: bool,
    /// `fast` (WireGuard 2-hop), `mixnet`, `disconnected`, or `unknown`.
    pub mode: String,
    pub tunnel_state: String,
    /// Non-onramp sites may open only when this is true.
    pub browse_allowed: bool,
    /// `nym-vpnc`, `windows-adapters`, or `none`.
    pub source: String,
    pub detail: String,
}

/// Parse `nym-vpnc status` stdout (`State: Connected wg to …`).
///
/// `Disconnected` contains the substring `connected`, so matching must be
/// prefix-based on the state token.
pub fn parse_vpnc_status(stdout: &str) -> ParsedTunnel {
    let mut rest = String::new();
    for line in stdout.lines() {
        let t = line.trim();
        if let Some(after) = t.strip_prefix("State:") {
            rest = after.trim().to_string();
            break;
        }
        if t.starts_with("Connected ")
            || t.starts_with("Disconnected")
            || t.starts_with("Connecting ")
            || t.starts_with("Disconnecting")
            || t.starts_with("Offline")
            || t.starts_with("Error")
        {
            rest = t.to_string();
            break;
        }
    }
    if rest.is_empty() {
        return ParsedTunnel {
            connected: false,
            mode: "unknown".into(),
            tunnel_state: "unknown".into(),
        };
    }
    classify_state_token(&rest)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedTunnel {
    pub connected: bool,
    pub mode: String,
    pub tunnel_state: String,
}

fn classify_state_token(rest: &str) -> ParsedTunnel {
    let lower = rest.to_ascii_lowercase();
    if lower.starts_with("disconnected") {
        return ParsedTunnel {
            connected: false,
            mode: "disconnected".into(),
            tunnel_state: "disconnected".into(),
        };
    }
    if lower.starts_with("disconnecting") {
        return ParsedTunnel {
            connected: false,
            mode: "unknown".into(),
            tunnel_state: "disconnecting".into(),
        };
    }
    if lower.starts_with("connecting") {
        return ParsedTunnel {
            connected: false,
            mode: mode_from_connected_line(rest).unwrap_or("unknown").into(),
            tunnel_state: "connecting".into(),
        };
    }
    if lower.starts_with("offline") {
        return ParsedTunnel {
            connected: false,
            mode: "disconnected".into(),
            tunnel_state: "offline".into(),
        };
    }
    if lower.starts_with("error") {
        return ParsedTunnel {
            connected: false,
            mode: "unknown".into(),
            tunnel_state: "error".into(),
        };
    }
    if lower.starts_with("connected") {
        let mode = mode_from_connected_line(rest)
            .unwrap_or("unknown")
            .to_string();
        return ParsedTunnel {
            connected: true,
            mode,
            tunnel_state: "connected".into(),
        };
    }
    ParsedTunnel {
        connected: false,
        mode: "unknown".into(),
        tunnel_state: "unknown".into(),
    }
}

fn mode_from_connected_line(rest: &str) -> Option<&'static str> {
    let lower = rest.to_ascii_lowercase();
    // `Connected wg to …` / `Connecting wg …`
    if lower.contains(" wg ")
        || lower.starts_with("connected wg")
        || lower.starts_with("connecting wg")
    {
        return Some("fast");
    }
    if lower.contains(" mix ")
        || lower.starts_with("connected mix")
        || lower.starts_with("connecting mix")
    {
        return Some("mixnet");
    }
    None
}

/// Infer Fast vs Mixnet from NymVPN Wintun adapter names that are **up**.
pub fn infer_mode_from_adapter_names(names: &[impl AsRef<str>]) -> Option<&'static str> {
    let mut fast = false;
    let mut mix = false;
    for n in names {
        let t = n.as_ref().trim();
        if t.eq_ignore_ascii_case("WireGuard (exit)") || t.eq_ignore_ascii_case("WireGuard (entry)")
        {
            fast = true;
        }
        if t.eq_ignore_ascii_case("Nym") {
            mix = true;
        }
    }
    if fast {
        Some("fast")
    } else if mix {
        Some("mixnet")
    } else {
        None
    }
}

/// Lines from `netsh interface show interface` whose State is Connected.
pub fn parse_netsh_connected_names(stdout: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in stdout.lines() {
        let lower = line.to_ascii_lowercase();
        if lower.contains("disconnected") {
            continue;
        }
        if !line.contains("Connected") {
            continue;
        }
        // Columns: Admin State, State, Type, Interface Name (name may contain spaces).
        let name = line
            .split_whitespace()
            .skip(3)
            .collect::<Vec<_>>()
            .join(" ");
        if !name.is_empty() {
            out.push(name);
        }
    }
    out
}

fn apply_no_window(cmd: &mut Command) {
    #[cfg(windows)]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let _ = cmd;
}

async fn run_cmd_capture(mut cmd: Command) -> Option<(i32, String, String)> {
    cmd.stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    apply_no_window(&mut cmd);
    let child = cmd.spawn().ok()?;
    let wait = child.wait_with_output();
    match tokio::time::timeout(PROBE_TIMEOUT, wait).await {
        Ok(Ok(o)) => Some((
            o.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&o.stdout).into_owned(),
            String::from_utf8_lossy(&o.stderr).into_owned(),
        )),
        _ => None,
    }
}

fn vpnc_exe_name() -> &'static str {
    if cfg!(windows) {
        "nym-vpnc.exe"
    } else {
        "nym-vpnc"
    }
}

fn collect_vpnc_candidates() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(p) = std::env::var(ENV_VPNC) {
        let path = PathBuf::from(p.trim());
        if path.is_file() {
            out.push(path);
        }
    }
    out.push(PathBuf::from(vpnc_exe_name()));
    if let Ok(pf) = std::env::var("ProgramFiles") {
        let base = PathBuf::from(pf).join("NymVPN");
        out.push(base.join(vpnc_exe_name()));
        out.push(base.join("bin").join(vpnc_exe_name()));
    }
    if let Ok(pf86) = std::env::var("ProgramFiles(x86)") {
        out.push(PathBuf::from(pf86).join("NymVPN").join(vpnc_exe_name()));
    }
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        let base = PathBuf::from(local);
        out.push(base.join("NymVPN").join(vpnc_exe_name()));
        out.push(base.join("Programs").join("NymVPN").join(vpnc_exe_name()));
    }
    out.push(PathBuf::from("/usr/bin").join(vpnc_exe_name()));
    out.push(PathBuf::from("/usr/local/bin").join(vpnc_exe_name()));
    out.push(PathBuf::from(
        "/Applications/NymVPN.app/Contents/MacOS/nym-vpnc",
    ));
    out.push(PathBuf::from(
        "/Applications/NymVPN.app/Contents/Resources/nym-vpnc",
    ));
    out
}

fn first_existing_file(cands: &[PathBuf]) -> Option<PathBuf> {
    for p in cands {
        if p.as_os_str() == vpnc_exe_name() {
            // Bare name: let the OS resolve PATH when we spawn.
            continue;
        }
        if p.is_file() {
            return Some(p.clone());
        }
    }
    None
}

async fn probe_vpnc(bin: &Path) -> Option<ParsedTunnel> {
    let mut cmd = Command::new(bin);
    cmd.arg("status");
    let (_code, stdout, _stderr) = run_cmd_capture(cmd).await?;
    Some(parse_vpnc_status(&stdout))
}

async fn probe_vpnc_from_path() -> Option<(PathBuf, ParsedTunnel)> {
    let mut cmd = Command::new(vpnc_exe_name());
    cmd.arg("status");
    let (_code, stdout, _stderr) = run_cmd_capture(cmd).await?;
    let parsed = parse_vpnc_status(&stdout);
    if parsed.tunnel_state == "unknown" && stdout.trim().is_empty() {
        return None;
    }
    Some((PathBuf::from(vpnc_exe_name()), parsed))
}

async fn windows_daemon_running() -> bool {
    #[cfg(windows)]
    {
        let mut cmd = Command::new("sc.exe");
        cmd.args(["query", "nym-vpnd"]);
        if let Some((_code, stdout, _)) = run_cmd_capture(cmd).await {
            return stdout.to_ascii_uppercase().contains("RUNNING");
        }
        false
    }
    #[cfg(not(windows))]
    {
        false
    }
}

fn unix_daemon_socket_present() -> bool {
    #[cfg(unix)]
    {
        Path::new("/var/run/nym-vpn.sock").exists()
    }
    #[cfg(not(unix))]
    {
        false
    }
}

async fn windows_adapter_mode() -> Option<&'static str> {
    #[cfg(windows)]
    {
        let mut cmd = Command::new("netsh");
        cmd.args(["interface", "show", "interface"]);
        let (_code, stdout, _) = run_cmd_capture(cmd).await?;
        let names = parse_netsh_connected_names(&stdout);
        infer_mode_from_adapter_names(&names)
    }
    #[cfg(not(windows))]
    {
        None
    }
}

fn detail_for(
    connected: bool,
    mode: &str,
    daemon_present: bool,
    vpnc_found: bool,
    source: &str,
) -> String {
    if connected {
        let mode_label = match mode {
            "fast" => "Fast mode (WireGuard)",
            "mixnet" => "Mixnet / Anonymous mode",
            _ => "connected",
        };
        return format!(
            "NymVPN is {mode_label}. In-wallet browsing is unlocked. Claim/install links stay available even when the tunnel is down."
        );
    }
    if !daemon_present && !vpnc_found {
        return "NymVPN is not detected. Install it from zcash.nym.com, connect Fast mode, then Refresh. The companion API (nozywallet-api) must be running so Nozy can see the tunnel.".into();
    }
    if daemon_present && source == "none" && !vpnc_found {
        return "NymVPN’s service is running, but the tunnel is not up (or Nozy cannot see it). Open the NymVPN app, connect Fast mode, then Refresh.".into();
    }
    "NymVPN is not connected. Open the NymVPN app, turn on Fast mode, then Refresh. Sites stay locked until the tunnel is up.".into()
}

/// Probe the consumer NymVPN app. Always returns a snapshot (never an error).
pub async fn probe_nym_vpn_app() -> NymVpnAppStatus {
    let daemon_present = windows_daemon_running().await || unix_daemon_socket_present();

    let cands = collect_vpnc_candidates();
    let vpnc_file = first_existing_file(&cands);

    if let Some(ref bin) = vpnc_file {
        if let Some(parsed) = probe_vpnc(bin).await {
            let browse_allowed = parsed.connected;
            let source = "nym-vpnc".to_string();
            let detail = detail_for(parsed.connected, &parsed.mode, true, true, &source);
            return NymVpnAppStatus {
                daemon_present: daemon_present || parsed.connected,
                vpnc_found: true,
                vpnc_path: Some(bin.display().to_string()),
                connected: parsed.connected,
                mode: parsed.mode,
                tunnel_state: parsed.tunnel_state,
                browse_allowed,
                source,
                detail,
            };
        }
    }

    if let Some((path, parsed)) = probe_vpnc_from_path().await {
        if parsed.tunnel_state != "unknown" || parsed.connected {
            let source = "nym-vpnc".to_string();
            let detail = detail_for(parsed.connected, &parsed.mode, true, true, &source);
            return NymVpnAppStatus {
                daemon_present: daemon_present || parsed.connected,
                vpnc_found: true,
                vpnc_path: Some(path.display().to_string()),
                connected: parsed.connected,
                mode: parsed.mode,
                tunnel_state: parsed.tunnel_state,
                browse_allowed: parsed.connected,
                source,
                detail,
            };
        }
    }

    if let Some(mode) = windows_adapter_mode().await {
        let source = "windows-adapters".to_string();
        let detail = detail_for(true, mode, true, vpnc_file.is_some(), &source);
        return NymVpnAppStatus {
            daemon_present: true,
            vpnc_found: vpnc_file.is_some(),
            vpnc_path: vpnc_file.map(|p| p.display().to_string()),
            connected: true,
            mode: mode.to_string(),
            tunnel_state: "connected".into(),
            browse_allowed: true,
            source,
            detail,
        };
    }

    let source = "none".to_string();
    let detail = detail_for(
        false,
        "disconnected",
        daemon_present,
        vpnc_file.is_some(),
        &source,
    );
    NymVpnAppStatus {
        daemon_present,
        vpnc_found: vpnc_file.is_some(),
        vpnc_path: vpnc_file.map(|p| p.display().to_string()),
        connected: false,
        mode: if daemon_present {
            "disconnected".into()
        } else {
            "unknown".into()
        },
        tunnel_state: if daemon_present {
            "disconnected".into()
        } else {
            "not_found".into()
        },
        browse_allowed: false,
        source,
        detail,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disconnected_is_not_connected() {
        let p = parse_vpnc_status("State: Disconnected\n");
        assert!(!p.connected);
        assert_eq!(p.tunnel_state, "disconnected");
    }

    #[test]
    fn connected_wg_is_fast() {
        let p = parse_vpnc_status("State: Connected wg to 1.2.3.4 [abc] → 5.6.7.8 [def]\n");
        assert!(p.connected);
        assert_eq!(p.mode, "fast");
        assert_eq!(p.tunnel_state, "connected");
    }

    #[test]
    fn connected_mix_is_mixnet() {
        let p = parse_vpnc_status("State: Connected mix to 1.2.3.4 [abc] → 5.6.7.8 [def]\n");
        assert!(p.connected);
        assert_eq!(p.mode, "mixnet");
    }

    #[test]
    fn connecting_is_not_browse_allowed() {
        let p = parse_vpnc_status("State: Connecting wg to 1.2.3.4, handshake, try #1\n");
        assert!(!p.connected);
        assert_eq!(p.tunnel_state, "connecting");
    }

    #[test]
    fn adapter_names_fast() {
        assert_eq!(
            infer_mode_from_adapter_names(&["Wi-Fi", "WireGuard (exit)"]),
            Some("fast")
        );
        assert_eq!(infer_mode_from_adapter_names(&["Nym"]), Some("mixnet"));
        assert_eq!(infer_mode_from_adapter_names(&["Wi-Fi"]), None);
    }

    #[test]
    fn netsh_skips_disconnected() {
        let sample = "\
Admin State    State          Type             Interface Name
-------------------------------------------------------------------------
Enabled        Connected      Dedicated        Wi-Fi
Enabled        Disconnected   Dedicated        Ethernet
Enabled        Connected      Dedicated        WireGuard (exit)
";
        let names = parse_netsh_connected_names(sample);
        assert!(names.iter().any(|n| n == "WireGuard (exit)"));
        assert!(!names.iter().any(|n| n == "Ethernet"));
        assert_eq!(infer_mode_from_adapter_names(&names), Some("fast"));
    }
}
