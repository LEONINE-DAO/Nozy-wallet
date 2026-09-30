//! zk-CosmWasm × Nym claim draft (step D prep).
//!
//! Nozy proves shielded notes; their VM verifies on CosmWasm; mixnet wraps public LCD POSTs.
//! See `docs/reference/ZK_COSMWASM_NYM_DEMO.md`.

use serde::Serialize;
use std::path::Path;

use crate::config::WalletConfig;
use crate::error::{NozyError, NozyResult};
use crate::send_egress::{assess_send_egress, SendEgressSnapshot};
use crate::vote_export::{build_ironwood_vote_notes, VoteNoteExportFile};

pub const CLAIM_DRAFT_FORMAT: &str = "nozy-zk-cosmwasm-claim-draft-v1";
pub const UPSTREAM_SPEC_URL: &str =
    "https://github.com/permissionlessweb/cosmwasm/blob/feat/zk-v2/ZK_CIRCUIT_QUICK_REFERENCE.md";
pub const SERIALIZATION_SPEC_URL: &str =
    "https://github.com/permissionlessweb/cosmwasm/blob/feat/zk-v2/ZK_CIRCUIT_SERIALIZATION_FORMAT.md";

/// Environment variable for the zk-CosmWasm LCD base URL (verify POST target).
pub const LCD_URL_ENV: &str = "NOZY_ZK_COSMWASM_LCD_URL";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TrackStepStatus {
    NotStarted,
    BlockedExternal,
    Partial,
    Ready,
}

impl TrackStepStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::NotStarted => "not_started",
            Self::BlockedExternal => "blocked_external",
            Self::Partial => "partial",
            Self::Ready => "ready",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ZkCosmwasmTrackStatus {
    pub step_a_vm: TrackStepStatus,
    pub step_a_note: String,
    pub step_b_witness: TrackStepStatus,
    pub step_b_note: String,
    pub step_c_egress: TrackStepStatus,
    pub step_c_note: String,
    pub step_d_glue: TrackStepStatus,
    pub step_d_note: String,
    pub lcd_url: Option<String>,
    pub lcd_submit_egress: Option<SendEgressSnapshot>,
    pub zebra_submit_egress: SendEgressSnapshot,
    pub honest_claim: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ZkCosmwasmClaimDraft {
    pub format: String,
    pub network: String,
    pub note_count: usize,
    pub total_value_zatoshis: u64,
    pub vote_export: VoteNoteExportFile,
    pub proof_instance: Option<serde_json::Value>,
    pub proof_instance_status: String,
    pub lcd_url: Option<String>,
    pub lcd_submit_egress: Option<SendEgressSnapshot>,
    pub upstream_spec: String,
    pub serialization_spec: String,
    pub next_steps: Vec<String>,
}

/// Same submit policy as Zebrad `sendraw`, applied to an LCD verify URL.
pub fn assess_lcd_submit_egress(lcd_url: &str, config: &WalletConfig) -> SendEgressSnapshot {
    let mut probe = config.clone();
    probe.zebra_url = lcd_url.trim().to_string();
    let mut snap = assess_send_egress(&probe);
    snap.summary = format!("LCD verify POST: {}", snap.summary);
    if !snap.detail.starts_with("LCD") {
        snap.detail = format!(
            "Treat public LCD like remote submit (Harry hybrid). {}",
            snap.detail
        );
    }
    snap
}

pub fn lcd_url_from_env_or_config(_config: &WalletConfig) -> Option<String> {
    std::env::var(LCD_URL_ENV)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Living track status for CLI / companion (does not call CosmWasm).
pub fn assess_track_status(
    config: &WalletConfig,
    step_a_evidence_exists: bool,
    wallet_has_ironwood_notes: bool,
) -> ZkCosmwasmTrackStatus {
    let lcd_url = lcd_url_from_env_or_config(config);
    let lcd_egress = lcd_url
        .as_ref()
        .map(|u| assess_lcd_submit_egress(u, config));
    let zebra_egress = assess_send_egress(config);

    let (step_a, step_a_note) = if step_a_evidence_exists {
        (
            TrackStepStatus::Partial,
            "Step A evidence on disk — re-run scripts/zk-cosmwasm-step-a.ps1 after upstream updates."
                .to_string(),
        )
    } else {
        (
            TrackStepStatus::NotStarted,
            "Run scripts/zk-cosmwasm-step-a.ps1 against permissionlessweb/cosmwasm feat/zk-v2."
                .to_string(),
        )
    };

    let (step_b, step_b_note) = if wallet_has_ironwood_notes {
        (
            TrackStepStatus::Ready,
            "Ironwood notes available — nozy vote-export-notes or zk-cosmwasm claim-draft."
                .to_string(),
        )
    } else {
        (
            TrackStepStatus::Partial,
            "Sync wallet + migrate Orchard → Ironwood, then vote-export-notes.".to_string(),
        )
    };

    let (step_c, step_c_note) = (
        TrackStepStatus::Ready,
        "Mixnet submit policy wired — assess_lcd_submit_egress mirrors send-egress.".to_string(),
    );

    let (step_d, step_d_note) = (
        TrackStepStatus::BlockedExternal,
        "Await frozen circuit id + proof_instance bytes from upstream serialization spec."
            .to_string(),
    );

    let honest_claim = if step_a_evidence_exists && wallet_has_ironwood_notes {
        "VM verify is outsider-runnable; Nozy holds shielded notes and mixnet LCD policy. Not yet: proof_instance POST."
    } else {
        "Do not claim Nozy verified on zk-wasm until step D lands."
    }
    .to_string();

    ZkCosmwasmTrackStatus {
        step_a_vm: step_a,
        step_a_note,
        step_b_witness: step_b,
        step_b_note,
        step_c_egress: step_c,
        step_c_note,
        step_d_glue: step_d,
        step_d_note,
        lcd_url,
        lcd_submit_egress: lcd_egress,
        zebra_submit_egress: zebra_egress,
        honest_claim,
    }
}

pub fn load_vote_export(path: &Path) -> NozyResult<VoteNoteExportFile> {
    let raw = std::fs::read_to_string(path).map_err(|e| {
        NozyError::InvalidOperation(format!("read vote export {}: {e}", path.display()))
    })?;
    serde_json::from_str(&raw).map_err(|e| {
        NozyError::InvalidOperation(format!(
            "parse vote export {} (expect nozy-vote-notes-v1): {e}",
            path.display()
        ))
    })
}

pub fn build_claim_draft(
    vote: VoteNoteExportFile,
    lcd_url: Option<String>,
    config: &WalletConfig,
) -> ZkCosmwasmClaimDraft {
    let note_count = vote.notes.len();
    let total_value_zatoshis = vote.notes.iter().map(|n| n.value).sum();
    let lcd_submit_egress = lcd_url
        .as_ref()
        .map(|u| assess_lcd_submit_egress(u, config));

    ZkCosmwasmClaimDraft {
        format: CLAIM_DRAFT_FORMAT.to_string(),
        network: vote.network.clone(),
        note_count,
        total_value_zatoshis,
        vote_export: vote,
        proof_instance: None,
        proof_instance_status: "blocked_awaiting_upstream_circuit_spec".to_string(),
        lcd_url,
        lcd_submit_egress,
        upstream_spec: UPSTREAM_SPEC_URL.to_string(),
        serialization_spec: SERIALIZATION_SPEC_URL.to_string(),
        next_steps: vec![
            "Run scripts/zk-cosmwasm-step-a.ps1 and attach evidence under docs/reference/evidence/.".to_string(),
            "Obtain fixture proof bytes + public-input layout from permissionlessweb/cosmwasm.".to_string(),
            "Map vote_export notes to proof_instance per ZK_CIRCUIT_SERIALIZATION_FORMAT.md.".to_string(),
            "POST to LCD proof_instance_verify; if LCD is public, wrap HTTP via smolmix (same gate as sendraw).".to_string(),
        ],
    }
}

pub fn build_claim_draft_from_wallet(
    wallet: &crate::hd_wallet::HDWallet,
    network: zcash_protocol::consensus::NetworkType,
    lcd_url: Option<String>,
    config: &WalletConfig,
) -> NozyResult<ZkCosmwasmClaimDraft> {
    let vote = build_ironwood_vote_notes(wallet, network)?;
    Ok(build_claim_draft(vote, lcd_url, config))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::WalletConfig;

    #[test]
    fn lcd_local_url_classifies_as_local() {
        let config = WalletConfig::default();
        let snap = assess_lcd_submit_egress("http://127.0.0.1:1317", &config);
        assert!(snap.zebra_url_local);
        assert_eq!(snap.kind, crate::send_egress::SendEgressKind::Local);
    }

    #[test]
    fn lcd_public_respects_mixnet_flag() {
        let mut config = WalletConfig::default();
        config.zebra_url = "http://172.20.199.206:8232".to_string();
        config.privacy_network.broadcast_via_nym_mixnet = true;
        let snap = assess_lcd_submit_egress("https://lcd.example.test", &config);
        assert!(!snap.zebra_url_local);
        assert!(snap.summary.contains("LCD verify POST"));
    }
}
