//! Structured Crosslink diagnostic dump for forum / Shielded Labs paste.
//!
//! Does not submit stake, retarget, unbond, or withdraw. Lifecycle fields only
//! say whether those actions are *available* on the current window.

use super::payout::ufvk_fingerprint;
use super::types::{GuardianSnapshot, NextAction, StakingPositions};
use super::{short_hex, zat_to_ctaz};
use serde::{Deserialize, Serialize};

/// PoW minus finalized tip above this is treated as a stall signal.
pub const TFL_LAG_WARN_BLOCKS: u32 = 32;

/// Same B− bar as the desktop / extension Hybrid PoS helpers.
pub const RELIABLE_MIN_SCORE: f64 = 80.0;

pub const HYBRID_POS_SCOREBOARD_URL: &str = "https://zcash-hybrid-pos.vercel.app/api/scoreboard";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CheckLevel {
    Ok,
    Warn,
    Fail,
    Skip,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorCheck {
    pub id: String,
    pub level: CheckLevel,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObserverGrade {
    pub finalizer: String,
    pub grade: Option<String>,
    pub score: Option<f64>,
    pub live: Option<bool>,
    pub standing: String,
    pub needs_retarget: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleHint {
    pub staking_day_open: bool,
    pub can_unbond: bool,
    pub can_withdraw: bool,
    pub sample_unbond_pk: Option<String>,
    pub sample_withdraw_pk: Option<String>,
    pub note: String,
}

#[derive(Debug, Clone)]
pub struct DoctorInput {
    pub snapshot: GuardianSnapshot,
    pub ufvk_fingerprint: Option<String>,
    pub observer: Option<Vec<ObserverGrade>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorReport {
    pub ok: bool,
    pub rpc_url: String,
    pub height: u32,
    pub tfl_activated: Option<bool>,
    pub tfl_lag: Option<u32>,
    pub recency_finalizers: Option<usize>,
    pub pos_height: Option<u32>,
    pub staking_day_open: bool,
    pub active_bonds: usize,
    pub withdrawable_bonds: usize,
    pub earned_zat: u64,
    pub bonded_zat: u64,
    pub ufvk_fingerprint: Option<String>,
    pub wallet_synced: Option<bool>,
    pub next_action: NextAction,
    pub checks: Vec<DoctorCheck>,
    pub observer: Option<Vec<ObserverGrade>>,
    pub lifecycle: LifecycleHint,
    pub notes: Vec<String>,
    pub paste_body: String,
}

impl DoctorReport {
    pub fn build(input: DoctorInput) -> Self {
        let snap = &input.snapshot;
        let tfl_lag = snap
            .finalized_tip
            .as_ref()
            .and_then(|t| t.height)
            .map(|h| snap.height.saturating_sub(h));

        let mut checks = Vec::new();
        checks.push(check_tfl_activated(snap.tfl_activated));
        checks.push(check_tfl_lag(tfl_lag));
        checks.push(check_recency(snap.finalizer_count));
        checks.push(check_ufvk(input.ufvk_fingerprint.as_deref()));
        checks.push(check_wallet(snap));
        checks.push(check_observer(input.observer.as_deref()));

        let lifecycle = lifecycle_hint(snap);
        let notes = collect_notes(snap, &lifecycle, input.observer.as_deref());
        let ok = checks.iter().all(|c| c.level != CheckLevel::Fail);

        let mut report = Self {
            ok,
            rpc_url: snap.rpc_url.clone(),
            height: snap.height,
            tfl_activated: snap.tfl_activated,
            tfl_lag,
            recency_finalizers: snap.finalizer_count,
            pos_height: snap.pos_height,
            staking_day_open: snap.staking_day.open,
            active_bonds: snap.positions.active_count(),
            withdrawable_bonds: snap.positions.withdrawable.len(),
            earned_zat: snap.positions.total_rewards_zat(),
            bonded_zat: snap.positions.bonded_zat(),
            ufvk_fingerprint: input.ufvk_fingerprint,
            wallet_synced: snap.wallet.as_ref().map(|w| w.wallet_synced()),
            next_action: snap.next_action.clone(),
            checks,
            observer: input.observer,
            lifecycle,
            notes,
            paste_body: String::new(),
        };
        report.paste_body = report.render_paste();
        report
    }

    fn render_paste(&self) -> String {
        let mut lines = vec![
            "Nozy × Crosslink doctor".into(),
            format!("ok: {}", self.ok),
            format!("RPC: {}", self.rpc_url),
            format!("PoW height: {}", self.height),
        ];
        lines.push(match self.tfl_activated {
            Some(true) => "TFL: activated".into(),
            Some(false) => "TFL: not activated".into(),
            None => "TFL: unknown".into(),
        });
        match self.tfl_lag {
            Some(lag) => lines.push(format!("TFL lag: {lag} block(s)")),
            None => lines.push("TFL lag: unknown".into()),
        }
        match self.recency_finalizers {
            Some(n) => lines.push(format!("Recency finalizers: {n}")),
            None => lines.push("Recency finalizers: unavailable".into()),
        }
        if let Some(h) = self.pos_height {
            lines.push(format!("PoS height: {h}"));
        }
        lines.push(format!(
            "Staking Day: {}",
            if self.staking_day_open {
                "OPEN"
            } else {
                "CLOSED"
            }
        ));
        lines.push(format!(
            "Bonds: {} active / {} withdrawable",
            self.active_bonds, self.withdrawable_bonds
        ));
        lines.push(format!(
            "Earned (latest − initial): {:.8} cTAZ",
            zat_to_ctaz(self.earned_zat)
        ));
        lines.push(format!(
            "Bonded principal: {:.8} cTAZ",
            zat_to_ctaz(self.bonded_zat)
        ));
        match &self.ufvk_fingerprint {
            Some(fp) => lines.push(format!("UFVK fingerprint: {fp}")),
            None => lines.push("UFVK fingerprint: unavailable".into()),
        }
        match self.wallet_synced {
            Some(true) => lines.push("Node wallet: synced".into()),
            Some(false) => lines.push("Node wallet: still scanning".into()),
            None => lines.push("Node wallet: get_wallet_sync_status unavailable".into()),
        }
        lines.push(String::new());
        lines.push("Checks:".into());
        for c in &self.checks {
            lines.push(format!(
                "- [{}] {}: {}",
                level_tag(&c.level),
                c.id,
                c.detail
            ));
        }
        if let Some(ref grades) = self.observer {
            lines.push(String::new());
            lines.push("Observer grades (bonded finalizers):".into());
            if grades.is_empty() {
                lines.push("- (no active bonds)".into());
            } else {
                for g in grades {
                    let id = short_hex(&g.finalizer, 10, 8);
                    let grade = g.grade.clone().unwrap_or_else(|| "—".into());
                    let live = match g.live {
                        Some(true) => "live",
                        Some(false) => "offline",
                        None => "?",
                    };
                    let flag = if g.needs_retarget { "  RETARGET" } else { "" };
                    lines.push(format!("- {id}  {grade}  {live}  {}{flag}", g.standing));
                }
            }
        }
        lines.push(String::new());
        lines.push("Lifecycle (not submitted):".into());
        lines.push(format!("- {}", self.lifecycle.note));
        if let Some(ref pk) = self.lifecycle.sample_unbond_pk {
            lines.push(format!("- sample unbond pk: {}", short_hex(pk, 10, 8)));
        }
        if let Some(ref pk) = self.lifecycle.sample_withdraw_pk {
            lines.push(format!("- sample withdraw pk: {}", short_hex(pk, 10, 8)));
        }
        lines.push(String::new());
        lines.push("Notes:".into());
        for note in &self.notes {
            lines.push(format!("- {note}"));
        }
        lines.join("\n")
    }
}

fn level_tag(level: &CheckLevel) -> &'static str {
    match level {
        CheckLevel::Ok => "ok",
        CheckLevel::Warn => "warn",
        CheckLevel::Fail => "fail",
        CheckLevel::Skip => "skip",
    }
}

fn check_tfl_activated(activated: Option<bool>) -> DoctorCheck {
    match activated {
        Some(true) => DoctorCheck {
            id: "tfl_activated".into(),
            level: CheckLevel::Ok,
            detail: "is_tfl_activated = true".into(),
        },
        Some(false) => DoctorCheck {
            id: "tfl_activated".into(),
            level: CheckLevel::Fail,
            detail: "TFL is not activated on this node".into(),
        },
        None => DoctorCheck {
            id: "tfl_activated".into(),
            level: CheckLevel::Warn,
            detail: "is_tfl_activated RPC unavailable".into(),
        },
    }
}

fn check_tfl_lag(lag: Option<u32>) -> DoctorCheck {
    match lag {
        Some(n) if n > TFL_LAG_WARN_BLOCKS => DoctorCheck {
            id: "tfl_lag".into(),
            level: CheckLevel::Fail,
            detail: format!("{n} blocks behind PoW tip (warn at {TFL_LAG_WARN_BLOCKS})"),
        },
        Some(n) => DoctorCheck {
            id: "tfl_lag".into(),
            level: CheckLevel::Ok,
            detail: format!("{n} block(s) behind PoW tip"),
        },
        None => DoctorCheck {
            id: "tfl_lag".into(),
            level: CheckLevel::Warn,
            detail: "Finalized tip unavailable".into(),
        },
    }
}

fn check_recency(count: Option<usize>) -> DoctorCheck {
    match count {
        Some(n) => DoctorCheck {
            id: "recency".into(),
            level: CheckLevel::Ok,
            detail: format!("{n} finalizer(s) in get_tfl_recency_status"),
        },
        None => DoctorCheck {
            id: "recency".into(),
            level: CheckLevel::Warn,
            detail: "get_tfl_recency_status unavailable".into(),
        },
    }
}

fn check_ufvk(fp: Option<&str>) -> DoctorCheck {
    match fp {
        Some(fp) => DoctorCheck {
            id: "ufvk".into(),
            level: CheckLevel::Ok,
            detail: format!("fingerprint {fp}"),
        },
        None => DoctorCheck {
            id: "ufvk".into(),
            level: CheckLevel::Warn,
            detail: "get_wallet_ufvk unavailable".into(),
        },
    }
}

fn check_wallet(snap: &GuardianSnapshot) -> DoctorCheck {
    match &snap.wallet {
        Some(w) if w.wallet_synced() => DoctorCheck {
            id: "wallet_sync".into(),
            level: CheckLevel::Ok,
            detail: format!("scan {} / {}", w.sync_height, w.tip_height),
        },
        Some(w) => DoctorCheck {
            id: "wallet_sync".into(),
            level: CheckLevel::Warn,
            detail: format!(
                "still scanning {} / {} — spendable may be low",
                w.sync_height, w.tip_height
            ),
        },
        None => DoctorCheck {
            id: "wallet_sync".into(),
            level: CheckLevel::Skip,
            detail: "get_wallet_sync_status not on this node build".into(),
        },
    }
}

fn check_observer(grades: Option<&[ObserverGrade]>) -> DoctorCheck {
    match grades {
        None => DoctorCheck {
            id: "observer".into(),
            level: CheckLevel::Skip,
            detail: "Hybrid PoS scoreboard not fetched".into(),
        },
        Some(rows) => {
            let n = rows.iter().filter(|g| g.needs_retarget).count();
            if n > 0 {
                DoctorCheck {
                    id: "observer".into(),
                    level: CheckLevel::Warn,
                    detail: format!("{n} bonded finalizer(s) offline or below B−"),
                }
            } else {
                DoctorCheck {
                    id: "observer".into(),
                    level: CheckLevel::Ok,
                    detail: format!("{} bonded finalizer(s) graded", rows.len()),
                }
            }
        }
    }
}

fn lifecycle_hint(snap: &GuardianSnapshot) -> LifecycleHint {
    let open = snap.staking_day.open;
    let sample_unbond_pk = snap
        .positions
        .active
        .values()
        .flatten()
        .next()
        .map(|b| b.pk.clone());
    let sample_withdraw_pk = snap.positions.withdrawable.first().map(|b| b.pk.clone());
    let can_unbond = open && sample_unbond_pk.is_some();
    let can_withdraw = open && sample_withdraw_pk.is_some();
    let note = if can_withdraw {
        "Staking Day open with withdrawable bonds — `nozy crosslink withdraw --bond <pk>` (doctor does not submit)."
            .into()
    } else if can_unbond {
        "Staking Day open with active bonds — unbond one, wait until it is withdrawable, then withdraw. Community indexer saw almost no unbond/withdraw; this is the missing lifecycle."
            .into()
    } else if sample_withdraw_pk.is_some() {
        format!(
            "Withdrawable bonds waiting — window opens in {} block(s).",
            snap.staking_day.blocks_until_next.unwrap_or(0)
        )
    } else if sample_unbond_pk.is_some() {
        format!(
            "Active bonds; unbond when Staking Day opens ({} block(s)). Retarget anytime.",
            snap.staking_day.blocks_until_next.unwrap_or(0)
        )
    } else {
        "No bonds — stake during Staking Day before an unbond→withdraw pass is possible.".into()
    };
    LifecycleHint {
        staking_day_open: open,
        can_unbond,
        can_withdraw,
        sample_unbond_pk,
        sample_withdraw_pk,
        note,
    }
}

fn collect_notes(
    snap: &GuardianSnapshot,
    lifecycle: &LifecycleHint,
    observer: Option<&[ObserverGrade]>,
) -> Vec<String> {
    let mut notes = vec![
        "Paste this dump for forum / Shielded Labs — no secrets (UFVK is fingerprint only).".into(),
        "Doctor never submits unbond or withdraw.".into(),
    ];
    if observer.is_some_and(|rows| rows.iter().any(|g| g.needs_retarget)) {
        notes.push(
            "Observer flagged a bonded finalizer — Prepare retarget in the Guardian UI (no auto-submit)."
                .into(),
        );
    }
    if lifecycle.can_unbond || lifecycle.can_withdraw {
        notes.push(lifecycle.note.clone());
    }
    if snap.wallet.is_none() {
        notes.push(
            "Upgrade node for get_wallet_sync_status — docs/CROSSLINK_WALLET_RPC_UPGRADE.md".into(),
        );
    }
    notes
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScoreboardRow {
    pub pubkey: String,
    #[serde(default)]
    pub grade: Option<String>,
    #[serde(default)]
    pub score: Option<f64>,
    #[serde(default)]
    pub live: Option<bool>,
    #[serde(default)]
    pub provisional: bool,
    #[serde(default)]
    pub unobserved: bool,
}

pub fn grades_for_bonds(
    positions: &StakingPositions,
    rows: &[ScoreboardRow],
) -> Vec<ObserverGrade> {
    let index: std::collections::HashMap<String, &ScoreboardRow> = rows
        .iter()
        .map(|r| (normalize_pubkey(&r.pubkey), r))
        .collect();
    positions
        .active
        .keys()
        .map(|finalizer| {
            let key = normalize_pubkey(finalizer);
            match index.get(&key) {
                Some(row) => grade_from_row(finalizer, row),
                None => ObserverGrade {
                    finalizer: finalizer.clone(),
                    grade: None,
                    score: None,
                    live: None,
                    standing: "unknown".into(),
                    needs_retarget: false,
                },
            }
        })
        .collect()
}

fn grade_from_row(finalizer: &str, row: &ScoreboardRow) -> ObserverGrade {
    let standing = observer_standing(row);
    let needs_retarget = row.live == Some(false) || standing == "uneven";
    ObserverGrade {
        finalizer: finalizer.to_string(),
        grade: row.grade.clone(),
        score: row.score,
        live: row.live,
        standing: standing.into(),
        needs_retarget,
    }
}

fn observer_standing(row: &ScoreboardRow) -> &'static str {
    if row.provisional {
        return "provisional";
    }
    if row.unobserved || row.grade.is_none() {
        return "unknown";
    }
    if row.score.unwrap_or(0.0) >= RELIABLE_MIN_SCORE {
        "reliable"
    } else {
        "uneven"
    }
}

fn normalize_pubkey(hex: &str) -> String {
    hex.trim().trim_start_matches("0x").to_ascii_lowercase()
}

pub fn fingerprint_ufvk(ufvk: &str) -> String {
    ufvk_fingerprint(ufvk)
}

pub fn display_doctor(report: &DoctorReport) {
    super::display::print_feature_net_banner();
    println!("{}", report.paste_body);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crosslink::types::{
        BondPosition, FinalizedTip, GuardianSnapshot, NextAction, StakingDayWindow,
        StakingPositions, WalletSyncStatus,
    };
    use std::collections::BTreeMap;

    fn day(open: bool) -> StakingDayWindow {
        StakingDayWindow {
            height: 196_617,
            offset: if open { 10 } else { 80 },
            cycle: 150,
            window: 70,
            open,
            blocks_remaining_in_window: if open { Some(60) } else { None },
            blocks_until_next: if open { None } else { Some(70) },
        }
    }

    fn snap(open: bool, lag: Option<u32>) -> GuardianSnapshot {
        let mut active = BTreeMap::new();
        active.insert(
            "aa".repeat(32),
            vec![BondPosition {
                pk: "bb".repeat(32),
                create_height: Some(100),
                initial_val: 1_000_000_000,
                latest_val: 1_100_000_000,
                finalizer: Some("aa".repeat(32)),
            }],
        );
        GuardianSnapshot {
            rpc_url: "http://127.0.0.1:8232".into(),
            height: 196_617,
            staking_day: day(open),
            tfl_activated: Some(true),
            finalized_tip: lag.map(|n| FinalizedTip {
                height: Some(196_617 - n),
                hash: Some("cc".repeat(32)),
            }),
            positions: StakingPositions {
                active,
                withdrawable: vec![],
            },
            finalizer_count: Some(12),
            pos_height: Some(50),
            next_action: NextAction::RetargetIfNeeded,
            privacy_notes: vec![],
            wallet: Some(WalletSyncStatus {
                sync_height: 196_617,
                tip_height: 196_617,
                user_shielded_spendable_zats: 0,
                user_shielded_pending_zats: 0,
                user_unshielded_zats: 0,
                staked_zats: 1_100_000_000,
                withdrawable_zats: 0,
            }),
        }
    }

    #[test]
    fn healthy_report_is_ok() {
        let report = DoctorReport::build(DoctorInput {
            snapshot: snap(true, Some(2)),
            ufvk_fingerprint: Some("uview1abc…xyz".into()),
            observer: Some(vec![]),
        });
        assert!(report.ok);
        assert_eq!(report.tfl_lag, Some(2));
        assert!(report.paste_body.contains("TFL lag: 2"));
        assert!(report.paste_body.contains("UFVK fingerprint"));
        assert!(!report.paste_body.contains("uview1abcdefghijklmnopqrst"));
        assert!(report.lifecycle.can_unbond);
        assert!(!report.lifecycle.can_withdraw);
    }

    #[test]
    fn large_tfl_lag_fails() {
        let report = DoctorReport::build(DoctorInput {
            snapshot: snap(true, Some(40)),
            ufvk_fingerprint: None,
            observer: None,
        });
        assert!(!report.ok);
        assert!(report
            .checks
            .iter()
            .any(|c| c.id == "tfl_lag" && c.level == CheckLevel::Fail));
    }

    #[test]
    fn tfl_off_fails() {
        let mut s = snap(true, Some(1));
        s.tfl_activated = Some(false);
        let report = DoctorReport::build(DoctorInput {
            snapshot: s,
            ufvk_fingerprint: None,
            observer: None,
        });
        assert!(!report.ok);
    }

    #[test]
    fn missing_observer_does_not_fail() {
        let report = DoctorReport::build(DoctorInput {
            snapshot: snap(true, Some(1)),
            ufvk_fingerprint: Some("fp".into()),
            observer: None,
        });
        assert!(report.ok);
        assert!(report
            .checks
            .iter()
            .any(|c| c.id == "observer" && c.level == CheckLevel::Skip));
    }

    #[test]
    fn withdrawable_lifecycle_when_window_open() {
        let mut s = snap(true, Some(1));
        s.positions.withdrawable = vec![BondPosition {
            pk: "dd".repeat(32),
            create_height: None,
            initial_val: 1,
            latest_val: 1,
            finalizer: None,
        }];
        let report = DoctorReport::build(DoctorInput {
            snapshot: s,
            ufvk_fingerprint: None,
            observer: None,
        });
        assert!(report.lifecycle.can_withdraw);
        assert!(report.paste_body.contains("withdraw"));
    }

    #[test]
    fn observer_uneven_needs_retarget() {
        let grades = grades_for_bonds(
            &snap(true, Some(1)).positions,
            &[ScoreboardRow {
                pubkey: "aa".repeat(32),
                grade: Some("C".into()),
                score: Some(40.0),
                live: Some(true),
                provisional: false,
                unobserved: false,
            }],
        );
        assert_eq!(grades.len(), 1);
        assert!(grades[0].needs_retarget);
        assert_eq!(grades[0].standing, "uneven");
    }

    #[test]
    fn observer_offline_needs_retarget_even_if_scored() {
        let grades = grades_for_bonds(
            &snap(true, Some(1)).positions,
            &[ScoreboardRow {
                pubkey: format!("0x{}", "aa".repeat(32)),
                grade: Some("A".into()),
                score: Some(95.0),
                live: Some(false),
                provisional: false,
                unobserved: false,
            }],
        );
        assert!(grades[0].needs_retarget);
        assert_eq!(grades[0].standing, "reliable");
    }

    #[test]
    fn missing_scoreboard_row_is_unknown_not_nagged() {
        let grades = grades_for_bonds(&snap(true, Some(1)).positions, &[]);
        assert_eq!(grades[0].standing, "unknown");
        assert!(!grades[0].needs_retarget);
    }
}
