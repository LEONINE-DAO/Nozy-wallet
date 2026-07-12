//! Safer Ironwood migration — network, cover, and amount/timing priorities.
//!
//! Priority 1: protect broadcasting IP (local Zebrad OR Tor/I2P/Nym).
//!   Biggest win for remote stacks: route **all outgoing tx submits** over Nym
//!   (smolmix), not only sync. See `docs/reference/NYM_IP_PRIVACY_CASE_BREAKDOWN.md`.
//! Priority 2: shared cohort / cover traffic (scaffolding; cross-user health later).
//! Priority 3: amount + timing selection algorithm (ZIP 318 now; Zooko path planned).
//!
//! See `docs/reference/SAFE_MIGRATION_NETWORK_PRIVACY_FORUM_POST.md`.

use crate::error::{NozyError, NozyResult};
use crate::privacy_network::proxy::{PrivacyNetwork, ProxyConfig};
use crate::zebra_integration::ZebraClient;
use serde::{Deserialize, Serialize};

/// How the wallet chose amounts and broadcast windows for migration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AmountTimingAlgorithm {
    /// ZIP 318 greedy power-of-ten denominations + 256-block anchor buckets.
    Zip318PowerOfTen,
    /// Zooko-style `{1,2,5}×10^k` + concurrency/cover gates (planned).
    Zooko125Planned,
}

impl AmountTimingAlgorithm {
    pub fn label(self) -> &'static str {
        match self {
            Self::Zip318PowerOfTen => "ZIP 318 power-of-ten + shared anchor buckets",
            Self::Zooko125Planned => "Zooko {1,2,5}×10^k + concurrency gates (planned)",
        }
    }
}

/// Active amount/timing algorithm for Orchard → Ironwood migration.
pub fn selected_amount_timing_algorithm() -> AmountTimingAlgorithm {
    AmountTimingAlgorithm::Zip318PowerOfTen
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationNetworkPrivacyMode {
    /// Local / LAN Zebrad — preferred desktop path.
    LocalNode,
    /// Detected Tor or I2P SOCKS proxy.
    DetectedPrivacyProxy,
    /// Remote submit routed via Nym smolmix helper (`broadcast_via_nym_mixnet` / env).
    NymMixnetBroadcast,
    /// User attested system-wide NymVPN / Tor (not detectable as local SOCKS).
    UserAttestedPrivateNetwork,
    /// Explicit clearnet override (discouraged).
    ForceClearnet,
}

impl MigrationNetworkPrivacyMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::LocalNode => "local Zebrad",
            Self::DetectedPrivacyProxy => "detected Tor/I2P proxy",
            Self::NymMixnetBroadcast => "Nym smolmix broadcast helper",
            Self::UserAttestedPrivateNetwork => "user-attested Nym/Tor",
            Self::ForceClearnet => "forced clearnet (discouraged)",
        }
    }
}

#[derive(Debug, Clone)]
pub struct MigrationNetworkPrivacyAssessment {
    pub allowed: bool,
    pub mode: Option<MigrationNetworkPrivacyMode>,
    pub zebra_url_local: bool,
    pub privacy_proxy_detected: bool,
    pub privacy_proxy_label: Option<String>,
    pub user_attested: bool,
    pub force_clearnet: bool,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct MigrationNetworkPrivacyOpts {
    /// User confirms NymVPN / Tor (or equivalent) is protecting egress.
    pub attest_private_network: bool,
    /// Emergency override — broadcast on clearnet despite policy.
    pub force_clearnet: bool,
    /// Config `privacy_network.broadcast_via_nym_mixnet` (env also checked).
    pub broadcast_via_nym_mixnet: bool,
}

/// Assess whether migration broadcast may proceed under Priority 1 policy.
pub async fn assess_migration_network_privacy(
    zebra_url: &str,
    opts: &MigrationNetworkPrivacyOpts,
) -> MigrationNetworkPrivacyAssessment {
    let zebra_url_local = ZebraClient::url_is_local(zebra_url);
    let proxy = ProxyConfig::auto_detect().await;
    let privacy_proxy_detected = proxy.enabled;
    let privacy_proxy_label = if proxy.enabled {
        Some(match proxy.network {
            PrivacyNetwork::Tor => format!("Tor ({})", proxy.proxy_url),
            PrivacyNetwork::I2P => format!("I2P ({})", proxy.proxy_url),
            PrivacyNetwork::None => proxy.proxy_url.clone(),
        })
    } else {
        None
    };

    let mut warnings = Vec::new();
    let mut blockers = Vec::new();

    if opts.force_clearnet {
        warnings.push(
            "Forced clearnet: migration broadcast may link your IP to the pool-crossing event."
                .to_string(),
        );
        return MigrationNetworkPrivacyAssessment {
            allowed: true,
            mode: Some(MigrationNetworkPrivacyMode::ForceClearnet),
            zebra_url_local,
            privacy_proxy_detected,
            privacy_proxy_label,
            user_attested: opts.attest_private_network,
            force_clearnet: true,
            blockers,
            warnings,
        };
    }

    if zebra_url_local {
        return MigrationNetworkPrivacyAssessment {
            allowed: true,
            mode: Some(MigrationNetworkPrivacyMode::LocalNode),
            zebra_url_local,
            privacy_proxy_detected,
            privacy_proxy_label,
            user_attested: opts.attest_private_network,
            force_clearnet: false,
            blockers,
            warnings,
        };
    }

    if privacy_proxy_detected {
        warnings.push(
            "Remote Zebrad with Tor/I2P proxy detected. Prefer a local node when possible."
                .to_string(),
        );
        return MigrationNetworkPrivacyAssessment {
            allowed: true,
            mode: Some(MigrationNetworkPrivacyMode::DetectedPrivacyProxy),
            zebra_url_local,
            privacy_proxy_detected,
            privacy_proxy_label,
            user_attested: opts.attest_private_network,
            force_clearnet: false,
            blockers,
            warnings,
        };
    }

    if crate::nym_mixnet_broadcast::mixnet_broadcast_requested(opts.broadcast_via_nym_mixnet) {
        match crate::nym_mixnet_broadcast::resolve_helper_bin() {
            Ok(bin) => {
                warnings.push(format!(
                    "Remote submit will use Nym smolmix helper at {}. \
                     Zebra URL must be exit-reachable (not RFC1918/loopback).",
                    bin.display()
                ));
                return MigrationNetworkPrivacyAssessment {
                    allowed: true,
                    mode: Some(MigrationNetworkPrivacyMode::NymMixnetBroadcast),
                    zebra_url_local,
                    privacy_proxy_detected,
                    privacy_proxy_label,
                    user_attested: opts.attest_private_network,
                    force_clearnet: false,
                    blockers,
                    warnings,
                };
            }
            Err(e) => {
                blockers.push(format!(
                    "Nym mixnet broadcast is enabled, but the helper is missing ({e}). \
                     Build tools/nym-smolmix-broadcast-spike and set NOZY_NYM_SMOLMIX_BIN."
                ));
                return MigrationNetworkPrivacyAssessment {
                    allowed: false,
                    mode: None,
                    zebra_url_local,
                    privacy_proxy_detected,
                    privacy_proxy_label,
                    user_attested: opts.attest_private_network,
                    force_clearnet: false,
                    blockers,
                    warnings,
                };
            }
        }
    }

    if opts.attest_private_network {
        warnings.push(
            "Accepted user attestation that Nym/Tor (or equivalent) protects egress. \
             Nozy cannot verify system-wide VPN paths automatically."
                .to_string(),
        );
        return MigrationNetworkPrivacyAssessment {
            allowed: true,
            mode: Some(MigrationNetworkPrivacyMode::UserAttestedPrivateNetwork),
            zebra_url_local,
            privacy_proxy_detected,
            privacy_proxy_label,
            user_attested: true,
            force_clearnet: false,
            blockers,
            warnings,
        };
    }

    blockers.push(
        "Safer migration (Priority 1): refuse clearnet broadcast to a remote node. \
         Use a local Zebrad, start Tor/I2P (SOCKS), pass --attest-private-network if NymVPN/Tor \
         is already protecting this machine, or --force-clearnet to override (discouraged)."
            .to_string(),
    );

    MigrationNetworkPrivacyAssessment {
        allowed: false,
        mode: None,
        zebra_url_local,
        privacy_proxy_detected,
        privacy_proxy_label,
        user_attested: false,
        force_clearnet: false,
        blockers,
        warnings,
    }
}

/// Hard gate used by `ironwood broadcast` before submitting a turnstile tx.
pub async fn require_migration_network_privacy(
    zebra_url: &str,
    opts: &MigrationNetworkPrivacyOpts,
) -> NozyResult<MigrationNetworkPrivacyAssessment> {
    let assessment = assess_migration_network_privacy(zebra_url, opts).await;
    if assessment.allowed {
        Ok(assessment)
    } else {
        Err(NozyError::InvalidOperation(
            assessment
                .blockers
                .first()
                .cloned()
                .unwrap_or_else(|| "Migration network privacy check failed.".to_string()),
        ))
    }
}

/// Priority 2 scaffolding: local multiplicity in the current ZIP 318 bucket.
/// Cross-user cohort health is not available yet (no tracking channel).
#[derive(Debug, Clone)]
pub struct MigrationCoverAssessment {
    pub current_bucket_height: u32,
    pub local_transfers_in_bucket: usize,
    pub k_max: u8,
    pub thin_local_cohort: bool,
    pub warnings: Vec<String>,
    pub notes: Vec<String>,
}

pub fn assess_migration_cover_traffic(
    chain_tip: u32,
    transfers_in_current_bucket: usize,
    k_max: u8,
) -> MigrationCoverAssessment {
    let current_bucket_height =
        crate::ironwood::migration::previous_zip318_anchor_boundary(chain_tip);
    let mut warnings = Vec::new();
    let mut notes = Vec::new();

    notes.push(
        "Cross-user cohort health is not queried yet; cover relies on shared ZIP 318 buckets \
         across wallets (Priority 2)."
            .to_string(),
    );

    let thin_local_cohort = transfers_in_current_bucket == 0;
    if thin_local_cohort {
        warnings.push(format!(
            "No local transfers assigned to current bucket {current_bucket_height}. \
             Prefer waiting for a shared ZIP 318 window rather than ad hoc broadcast timing."
        ));
    } else if transfers_in_current_bucket == 1 {
        warnings.push(format!(
            "Only 1 local transfer in bucket {current_bucket_height} (k_max={k_max}). \
             Privacy still depends on other wallets sharing this cohort."
        ));
    }

    MigrationCoverAssessment {
        current_bucket_height,
        local_transfers_in_bucket: transfers_in_current_bucket,
        k_max,
        thin_local_cohort,
        warnings,
        notes,
    }
}

/// Priority 3 status line for preflight / docs.
#[derive(Debug, Clone)]
pub struct AmountTimingStatus {
    pub active: AmountTimingAlgorithm,
    pub planned: AmountTimingAlgorithm,
    pub notes: Vec<String>,
}

pub fn amount_timing_status() -> AmountTimingStatus {
    AmountTimingStatus {
        active: selected_amount_timing_algorithm(),
        planned: AmountTimingAlgorithm::Zooko125Planned,
        notes: vec![
            "Active: ZIP 318 canonical power-of-ten denominations and 256-block buckets."
                .to_string(),
            "Planned: align with Zooko {1,2,5}×10^k + concurrency/cover gates from the \
             coordination-migration writeup without fragmenting the anonymity set."
                .to_string(),
        ],
    }
}

/// Snapshot of Priorities 1–3 for desktop / API status panels.
#[derive(Debug, Clone, Serialize)]
pub struct SaferMigrationStatusSnapshot {
    pub network_privacy_allowed: bool,
    pub network_privacy_mode: Option<String>,
    pub zebra_url_local: bool,
    pub privacy_proxy_detected: bool,
    pub privacy_proxy_label: Option<String>,
    pub user_attested: bool,
    pub force_clearnet: bool,
    pub network_privacy_blockers: Vec<String>,
    pub network_privacy_warnings: Vec<String>,
    pub cover_bucket_height: u32,
    pub cover_local_transfers: usize,
    pub cover_k_max: u8,
    pub cover_thin: bool,
    pub cover_warnings: Vec<String>,
    pub cover_notes: Vec<String>,
    pub amount_timing_active: String,
    pub amount_timing_planned: String,
    pub amount_timing_notes: Vec<String>,
}

/// Build Priorities 1–3 status for UI surfaces.
pub async fn safer_migration_status_snapshot(
    zebra_url: &str,
    chain_tip: u32,
    local_transfers_in_bucket: usize,
    opts: &MigrationNetworkPrivacyOpts,
) -> SaferMigrationStatusSnapshot {
    let privacy = assess_migration_network_privacy(zebra_url, opts).await;
    let cover = assess_migration_cover_traffic(
        chain_tip,
        local_transfers_in_bucket,
        crate::ironwood::migration::ZIP318_DEFAULT_K_MAX,
    );
    let amount = amount_timing_status();

    SaferMigrationStatusSnapshot {
        network_privacy_allowed: privacy.allowed,
        network_privacy_mode: privacy.mode.map(|m| m.label().to_string()),
        zebra_url_local: privacy.zebra_url_local,
        privacy_proxy_detected: privacy.privacy_proxy_detected,
        privacy_proxy_label: privacy.privacy_proxy_label,
        user_attested: privacy.user_attested,
        force_clearnet: privacy.force_clearnet,
        network_privacy_blockers: privacy.blockers,
        network_privacy_warnings: privacy.warnings,
        cover_bucket_height: cover.current_bucket_height,
        cover_local_transfers: cover.local_transfers_in_bucket,
        cover_k_max: cover.k_max,
        cover_thin: cover.thin_local_cohort,
        cover_warnings: cover.warnings,
        cover_notes: cover.notes,
        amount_timing_active: amount.active.label().to_string(),
        amount_timing_planned: amount.planned.label().to_string(),
        amount_timing_notes: amount.notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn local_node_allows_without_attest() {
        let assessment = assess_migration_network_privacy(
            "http://127.0.0.1:18232",
            &MigrationNetworkPrivacyOpts::default(),
        )
        .await;
        assert!(assessment.allowed);
        assert_eq!(
            assessment.mode,
            Some(MigrationNetworkPrivacyMode::LocalNode)
        );
    }

    #[tokio::test]
    async fn remote_without_attest_blocks() {
        let assessment = assess_migration_network_privacy(
            "https://remote.example:8232",
            &MigrationNetworkPrivacyOpts::default(),
        )
        .await;
        // May still allow if Tor/I2P SOCKS is running on the machine; otherwise block.
        if !assessment.privacy_proxy_detected {
            assert!(!assessment.allowed);
            assert!(assessment.mode.is_none());
        }
    }

    #[tokio::test]
    async fn remote_with_attest_allows() {
        let assessment = assess_migration_network_privacy(
            "https://remote.example:8232",
            &MigrationNetworkPrivacyOpts {
                attest_private_network: true,
                force_clearnet: false,
                broadcast_via_nym_mixnet: false,
            },
        )
        .await;
        if !assessment.privacy_proxy_detected {
            assert!(assessment.allowed);
            assert_eq!(
                assessment.mode,
                Some(MigrationNetworkPrivacyMode::UserAttestedPrivateNetwork)
            );
        }
    }

    #[tokio::test]
    async fn force_clearnet_allows_with_warning() {
        let assessment = assess_migration_network_privacy(
            "https://remote.example:8232",
            &MigrationNetworkPrivacyOpts {
                attest_private_network: false,
                force_clearnet: true,
                broadcast_via_nym_mixnet: false,
            },
        )
        .await;
        assert!(assessment.allowed);
        assert_eq!(
            assessment.mode,
            Some(MigrationNetworkPrivacyMode::ForceClearnet)
        );
        assert!(!assessment.warnings.is_empty());
    }

    #[test]
    fn cover_warns_when_bucket_empty() {
        let cover = assess_migration_cover_traffic(1000, 0, 4);
        assert!(cover.thin_local_cohort);
        assert!(!cover.warnings.is_empty());
    }

    #[test]
    fn amount_timing_defaults_to_zip318() {
        assert_eq!(
            selected_amount_timing_algorithm(),
            AmountTimingAlgorithm::Zip318PowerOfTen
        );
    }
}
