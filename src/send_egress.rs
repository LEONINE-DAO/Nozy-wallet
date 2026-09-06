//! What the next `sendraw` / Ironwood broadcast will use for IP↔tx.
//!
//! Product badge: Local / Mixnet / Tor / I2P / Clearnet remote / Blocked,
//! plus the NymVPN stopgap when the in-app path is remote and not mixnet.
//! See `docs/reference/NYM_SEND_EGRESS_CASE_BREAKDOWN.md`.

use serde::Serialize;

use crate::config::WalletConfig;
use crate::ironwood::{nymvpn_ironwood_stopgap_hint, ZCASH_NYM_FREE_URL};
use crate::nym_mixnet_broadcast::{assess_mixnet_broadcast_readiness, MixnetBroadcastReadiness};
use crate::zebra_integration::{ZebraClient, ZebraConnectionMode};

/// Wallet-facing submit path (not compact-sync; sync is dVPN / local LWD).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SendEgressKind {
    Local,
    Trusted,
    Mixnet,
    Tor,
    I2p,
    DirectRemote,
    Blocked,
}

impl SendEgressKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Local => "Local",
            Self::Trusted => "Trusted node",
            Self::Mixnet => "Mixnet",
            Self::Tor => "Tor",
            Self::I2p => "I2P",
            Self::DirectRemote => "Clearnet remote",
            Self::Blocked => "Blocked",
        }
    }
}

/// Snapshot for CLI / desktop badge. No tunnel is opened.
#[derive(Debug, Clone, Serialize)]
pub struct SendEgressSnapshot {
    pub kind: SendEgressKind,
    pub label: String,
    pub connection_mode: String,
    pub zebra_url: String,
    pub zebra_url_local: bool,
    pub mixnet_requested: bool,
    pub mixnet_helper_ok: bool,
    pub would_use_mixnet: bool,
    pub show_stopgap: bool,
    pub stopgap_url: String,
    pub stopgap_hint: String,
    pub summary: String,
    pub detail: String,
}

/// Map Zebra connection mode + mixnet helper readiness to the badge.
pub fn classify_send_egress(
    mode: ZebraConnectionMode,
    mix: &MixnetBroadcastReadiness,
) -> SendEgressKind {
    match mode {
        ZebraConnectionMode::DirectLocal => SendEgressKind::Local,
        ZebraConnectionMode::DirectTrusted => SendEgressKind::Trusted,
        ZebraConnectionMode::NymMixnet => {
            if mix.would_use_mixnet {
                SendEgressKind::Mixnet
            } else {
                SendEgressKind::Blocked
            }
        }
        ZebraConnectionMode::TorProxy => SendEgressKind::Tor,
        ZebraConnectionMode::I2pProxy => SendEgressKind::I2p,
        ZebraConnectionMode::DirectRemote => SendEgressKind::DirectRemote,
        ZebraConnectionMode::Blocked => SendEgressKind::Blocked,
    }
}

fn summary_and_detail(kind: SendEgressKind, mix: &MixnetBroadcastReadiness) -> (String, String) {
    match kind {
        SendEgressKind::Local => (
            "Submit goes direct to local/LAN Zebrad.".to_string(),
            "Case A1: mixnet is skipped on purpose. Compact sync may still use local LWD or dVPN on a separate URL."
                .to_string(),
        ),
        SendEgressKind::Trusted => (
            "Submit goes to a trusted Zebrad URL (privacy gate treats it like operator infra)."
                .to_string(),
            "Not mixnet. Prefer loopback/LAN when you can. Remote trusted RPC still sees this host IP unless you add mixnet or Tor."
                .to_string(),
        ),
        SendEgressKind::Mixnet => (
            "Remote sendraw will use the Nym smolmix helper.".to_string(),
            "Issue #147 / D2c. Compact sync must stay on a different URL (dVPN or local LWD). Do not sync and submit through the same hosted lightwalletd a minute later."
                .to_string(),
        ),
        SendEgressKind::Tor => (
            "Remote submit uses the configured Tor SOCKS proxy.".to_string(),
            "Nym mixnet helper is not on this hop. Consumer NymVPN remains a separate OS stopgap."
                .to_string(),
        ),
        SendEgressKind::I2p => (
            "Remote submit uses the configured I2P proxy.".to_string(),
            "Nym mixnet helper is not on this hop.".to_string(),
        ),
        SendEgressKind::DirectRemote => (
            "Remote Zebrad over clearnet — this host IP can be linked to the submit."
                .to_string(),
            "Use a local node, enable the Nym smolmix helper, Tor/I2P, or the NymVPN stopgap. Do not call this “Nym integrated.”"
                .to_string(),
        ),
        SendEgressKind::Blocked => {
            let extra = if mix.requested && !mix.zebra_url_local && !mix.helper_ok {
                mix.helper_error.clone().unwrap_or_else(|| {
                    "Mixnet broadcast is requested but the smolmix helper binary is missing."
                        .to_string()
                })
            } else {
                "Remote submit is blocked until local Zebrad, Tor/I2P, mixnet helper, or attestation."
                    .to_string()
            };
            (
                "This send will not broadcast until a private egress is available.".to_string(),
                extra,
            )
        }
    }
}

/// Classify the next wallet submit from config. Does not open a mixnet tunnel.
pub fn assess_send_egress(config: &WalletConfig) -> SendEgressSnapshot {
    let client = ZebraClient::from_config(config);
    let mix = assess_mixnet_broadcast_readiness(
        &config.zebra_url,
        config.privacy_network.broadcast_via_nym_mixnet,
    );
    let kind = classify_send_egress(client.connection_mode(), &mix);
    let (summary, detail) = summary_and_detail(kind, &mix);
    let show_stopgap = matches!(
        kind,
        SendEgressKind::DirectRemote | SendEgressKind::Blocked | SendEgressKind::Trusted
    ) && !mix.zebra_url_local;

    SendEgressSnapshot {
        kind,
        label: kind.label().to_string(),
        connection_mode: client.connection_mode().as_str().to_string(),
        zebra_url: config.zebra_url.clone(),
        zebra_url_local: mix.zebra_url_local,
        mixnet_requested: mix.requested,
        mixnet_helper_ok: mix.helper_ok,
        would_use_mixnet: mix.would_use_mixnet,
        show_stopgap,
        stopgap_url: ZCASH_NYM_FREE_URL.to_string(),
        stopgap_hint: nymvpn_ironwood_stopgap_hint(),
        summary,
        detail,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::WalletConfig;

    fn mix(
        requested: bool,
        local: bool,
        helper_ok: bool,
        would_use: bool,
    ) -> MixnetBroadcastReadiness {
        MixnetBroadcastReadiness {
            requested,
            zebra_url_local: local,
            would_use_mixnet: would_use,
            helper_ok,
            helper_path: None,
            helper_error: None,
            notes: vec![],
        }
    }

    #[test]
    fn local_mode_is_local_badge() {
        let k = classify_send_egress(
            ZebraConnectionMode::DirectLocal,
            &mix(true, true, true, false),
        );
        assert_eq!(k, SendEgressKind::Local);
        assert_eq!(k.label(), "Local");
    }

    #[test]
    fn mixnet_mode_with_helper_is_mixnet_badge() {
        let k = classify_send_egress(
            ZebraConnectionMode::NymMixnet,
            &mix(true, false, true, true),
        );
        assert_eq!(k, SendEgressKind::Mixnet);
    }

    #[test]
    fn mixnet_mode_without_helper_is_blocked() {
        let k = classify_send_egress(
            ZebraConnectionMode::NymMixnet,
            &mix(true, false, false, false),
        );
        assert_eq!(k, SendEgressKind::Blocked);
    }

    #[test]
    fn blocked_and_clearnet_show_stopgap_when_remote() {
        let mut config = WalletConfig::default();
        config.zebra_url = "http://127.0.0.1:8232".to_string();
        let snap = assess_send_egress(&config);
        assert_eq!(snap.kind, SendEgressKind::Local);
        assert!(snap.zebra_url_local);
        assert!(!snap.show_stopgap);
    }

    #[test]
    fn default_config_is_local_case_a1() {
        let snap = assess_send_egress(&WalletConfig::default());
        assert_eq!(snap.kind, SendEgressKind::Local);
        assert!(!snap.would_use_mixnet);
    }
}
