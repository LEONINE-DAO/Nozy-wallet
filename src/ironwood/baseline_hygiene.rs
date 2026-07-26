//! Baseline hygiene — Layer-2 disciplines Nym transport cannot provide.
//!
//! Per Nym × Zcash wallet guidance (https://zcash-sdk.nym.com/guidance/):
//! - **V3 start-height obfuscation**: randomized overlap + checkpoint snap so
//!   compact-block / RPC resume heights stop being exact pointers to prior ends.
//! - **V2 broadcast hygiene**: randomized delay before submit; refuse tip-coupled
//!   broadcasts (do not submit immediately after catching the tip).
//! - **Transport split**: never treat sync session as the broadcast path
//!   (enforced operationally: local sync + mixnet/local broadcast; see case breakdown).
//!
//! Numbers below are engineering defaults — Nym leaves quantitative anonymity-set
//! sizing as an open question. Wider overlap / spacing ⇒ larger collision window,
//! more re-downloaded blocks.

use crate::error::{NozyError, NozyResult};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Aligns with ZIP 318 anchor-bucket spacing so snapped starts collide with
/// other ZIP 318-aware clients in the same interval.
pub const DEFAULT_CHECKPOINT_SPACING_BLOCKS: u32 = 256;

/// Max random blocks rewound before the true resume height (inclusive).
pub const DEFAULT_MAX_OVERLAP_BLOCKS: u32 = 128;

/// Minimum wall-clock pause before a real migrate-broadcast (seconds).
pub const DEFAULT_BROADCAST_DELAY_MIN_SECS: u64 = 30;

/// Maximum wall-clock pause before a real migrate-broadcast (seconds).
pub const DEFAULT_BROADCAST_DELAY_MAX_SECS: u64 = 300;

/// Refuse migrate-broadcast if the wallet caught tip this recently (seconds).
pub const DEFAULT_MIN_SECS_AFTER_TIP_SYNC: u64 = 120;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineHygieneConfig {
    /// When true, auto-resume sync rewinds start height (overlap + checkpoint).
    #[serde(default = "default_true")]
    pub obfuscate_sync_start: bool,

    /// Upper bound for random overlap rewind (blocks).
    #[serde(default = "default_max_overlap")]
    pub max_overlap_blocks: u32,

    /// Absolute checkpoint spacing for start-height snap (blocks).
    #[serde(default = "default_checkpoint_spacing")]
    pub checkpoint_spacing_blocks: u32,

    /// When true, real migrate-broadcast sleeps a random delay first.
    #[serde(default = "default_true")]
    pub broadcast_delay_enabled: bool,

    #[serde(default = "default_broadcast_delay_min")]
    pub broadcast_delay_min_secs: u64,

    #[serde(default = "default_broadcast_delay_max_secs")]
    pub broadcast_delay_max_secs: u64,

    /// Block migrate-broadcast if tip sync completed within this many seconds.
    #[serde(default = "default_true")]
    pub tip_sync_guard_enabled: bool,

    #[serde(default = "default_min_secs_after_tip_sync")]
    pub min_secs_after_tip_sync: u64,
}

fn default_true() -> bool {
    true
}

fn default_max_overlap() -> u32 {
    DEFAULT_MAX_OVERLAP_BLOCKS
}

fn default_checkpoint_spacing() -> u32 {
    DEFAULT_CHECKPOINT_SPACING_BLOCKS
}

fn default_broadcast_delay_min() -> u64 {
    DEFAULT_BROADCAST_DELAY_MIN_SECS
}

fn default_broadcast_delay_max_secs() -> u64 {
    DEFAULT_BROADCAST_DELAY_MAX_SECS
}

fn default_min_secs_after_tip_sync() -> u64 {
    DEFAULT_MIN_SECS_AFTER_TIP_SYNC
}

impl Default for BaselineHygieneConfig {
    fn default() -> Self {
        Self {
            obfuscate_sync_start: true,
            max_overlap_blocks: DEFAULT_MAX_OVERLAP_BLOCKS,
            checkpoint_spacing_blocks: DEFAULT_CHECKPOINT_SPACING_BLOCKS,
            broadcast_delay_enabled: true,
            broadcast_delay_min_secs: DEFAULT_BROADCAST_DELAY_MIN_SECS,
            broadcast_delay_max_secs: DEFAULT_BROADCAST_DELAY_MAX_SECS,
            tip_sync_guard_enabled: true,
            min_secs_after_tip_sync: DEFAULT_MIN_SECS_AFTER_TIP_SYNC,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObfuscatedStart {
    pub true_start: u32,
    pub obfuscated_start: u32,
    pub overlap_blocks: u32,
    pub checkpoint_snapped: bool,
}

/// Rewind `true_start` by a random overlap, then snap down to a checkpoint.
///
/// `floor` is the lowest allowed height (e.g. first-scan default / Sapling birth).
/// Never raises the start above `true_start`.
pub fn obfuscate_scan_start<R: Rng + ?Sized>(
    true_start: u32,
    floor: u32,
    max_overlap_blocks: u32,
    checkpoint_spacing_blocks: u32,
    rng: &mut R,
) -> ObfuscatedStart {
    let overlap = if max_overlap_blocks == 0 {
        0
    } else {
        rng.gen_range(0..=max_overlap_blocks)
    };
    let after_overlap = true_start.saturating_sub(overlap).max(floor);

    let spacing = checkpoint_spacing_blocks.max(1);
    let snapped = after_overlap - (after_overlap % spacing);
    let obfuscated_start = snapped.max(floor).min(true_start);
    let checkpoint_snapped = obfuscated_start != after_overlap || overlap > 0;

    ObfuscatedStart {
        true_start,
        obfuscated_start,
        overlap_blocks: true_start.saturating_sub(obfuscated_start),
        checkpoint_snapped,
    }
}

/// Apply config policy; returns `None` when obfuscation is disabled.
pub fn maybe_obfuscate_scan_start<R: Rng + ?Sized>(
    true_start: u32,
    floor: u32,
    cfg: &BaselineHygieneConfig,
    rng: &mut R,
) -> Option<ObfuscatedStart> {
    if !cfg.obfuscate_sync_start {
        return None;
    }
    Some(obfuscate_scan_start(
        true_start,
        floor,
        cfg.max_overlap_blocks,
        cfg.checkpoint_spacing_blocks,
        rng,
    ))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BroadcastDelayPlan {
    pub delay: Duration,
    pub reason: String,
}

/// Pick a randomized broadcast delay from config (zero when disabled / skipped).
pub fn plan_broadcast_delay<R: Rng + ?Sized>(
    cfg: &BaselineHygieneConfig,
    skip: bool,
    rng: &mut R,
) -> BroadcastDelayPlan {
    if skip || !cfg.broadcast_delay_enabled {
        return BroadcastDelayPlan {
            delay: Duration::ZERO,
            reason: if skip {
                "broadcast delay skipped by operator".to_string()
            } else {
                "broadcast delay disabled in config".to_string()
            },
        };
    }
    let min = cfg.broadcast_delay_min_secs;
    let max = cfg.broadcast_delay_max_secs.max(min);
    let secs = if min == max {
        min
    } else {
        rng.gen_range(min..=max)
    };
    BroadcastDelayPlan {
        delay: Duration::from_secs(secs),
        reason: format!(
            "randomized migrate-broadcast delay {secs}s (range {min}–{max}s; Nym V2 vs L2)"
        ),
    }
}

pub async fn apply_broadcast_delay(plan: &BroadcastDelayPlan) {
    if plan.delay.is_zero() {
        return;
    }
    tracing::info!(
        delay_secs = plan.delay.as_secs(),
        reason = %plan.reason,
        "baseline hygiene: sleeping before migrate-broadcast"
    );
    tokio::time::sleep(plan.delay).await;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TipSyncGuardResult {
    pub allowed: bool,
    pub secs_since_tip_sync: Option<u64>,
    pub required_secs: u64,
    pub message: String,
}

fn unix_now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Refuse broadcast when the wallet just finished catching tip (V2 tip coupling).
pub fn assess_tip_sync_guard(
    cfg: &BaselineHygieneConfig,
    last_tip_sync_unix: Option<u64>,
    skip: bool,
) -> TipSyncGuardResult {
    let required = cfg.min_secs_after_tip_sync;
    if skip || !cfg.tip_sync_guard_enabled {
        return TipSyncGuardResult {
            allowed: true,
            secs_since_tip_sync: None,
            required_secs: required,
            message: if skip {
                "tip-sync guard skipped by operator".to_string()
            } else {
                "tip-sync guard disabled in config".to_string()
            },
        };
    }
    let Some(last) = last_tip_sync_unix else {
        return TipSyncGuardResult {
            allowed: true,
            secs_since_tip_sync: None,
            required_secs: required,
            message: "no recorded tip-sync timestamp; guard not triggered".to_string(),
        };
    };
    let now = unix_now_secs();
    let elapsed = now.saturating_sub(last);
    if elapsed < required {
        TipSyncGuardResult {
            allowed: false,
            secs_since_tip_sync: Some(elapsed),
            required_secs: required,
            message: format!(
                "Safer migration (baseline hygiene): refuse broadcast {elapsed}s after tip sync \
                 (need ≥{required}s). Wait, or pass --skip-broadcast-hygiene for tests only."
            ),
        }
    } else {
        TipSyncGuardResult {
            allowed: true,
            secs_since_tip_sync: Some(elapsed),
            required_secs: required,
            message: format!("tip sync was {elapsed}s ago (≥{required}s required)"),
        }
    }
}

/// Hard gate used by migrate-broadcast.
pub fn require_tip_sync_guard(
    cfg: &BaselineHygieneConfig,
    last_tip_sync_unix: Option<u64>,
    skip: bool,
) -> NozyResult<TipSyncGuardResult> {
    let result = assess_tip_sync_guard(cfg, last_tip_sync_unix, skip);
    if result.allowed {
        Ok(result)
    } else {
        Err(NozyError::InvalidOperation(result.message.clone()))
    }
}

/// Human-readable hygiene summary for preflight / status panels.
pub fn baseline_hygiene_status_notes(cfg: &BaselineHygieneConfig) -> Vec<String> {
    vec![
        format!(
            "Start-height obfuscation: {} (max overlap {} blocks, checkpoint every {}).",
            if cfg.obfuscate_sync_start {
                "on"
            } else {
                "off"
            },
            cfg.max_overlap_blocks,
            cfg.checkpoint_spacing_blocks
        ),
        format!(
            "Migrate-broadcast delay: {} ({}–{}s randomized).",
            if cfg.broadcast_delay_enabled {
                "on"
            } else {
                "off"
            },
            cfg.broadcast_delay_min_secs,
            cfg.broadcast_delay_max_secs
        ),
        format!(
            "Tip-sync decorrelation: {} (wait ≥{}s after catching tip).",
            if cfg.tip_sync_guard_enabled {
                "on"
            } else {
                "off"
            },
            cfg.min_secs_after_tip_sync
        ),
        "Destination split: prefer local/Zebrad sync; route remote submits via Nym smolmix \
         (never reuse the sync transport for broadcast)."
            .to_string(),
        "Source: Nym × Zcash implementation guidance — baseline hygiene \
         (https://zcash-sdk.nym.com/guidance/)."
            .to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn obfuscation_never_raises_start() {
        let mut rng = StdRng::seed_from_u64(42);
        for _ in 0..50 {
            let out = obfuscate_scan_start(3_050_100, 1, 128, 256, &mut rng);
            assert!(out.obfuscated_start <= out.true_start);
            assert!(out.obfuscated_start >= 1);
        }
    }

    #[test]
    fn checkpoint_snap_lands_on_spacing() {
        let mut rng = StdRng::seed_from_u64(7);
        // true_start 300, overlap 0 → 300, snap → 256.
        let out = obfuscate_scan_start(300, 0, 0, 256, &mut rng);
        assert_eq!(out.obfuscated_start, 256);
        assert!(out.checkpoint_snapped);
    }

    #[test]
    fn maybe_obfuscate_respects_disable() {
        let mut rng = StdRng::seed_from_u64(1);
        let mut cfg = BaselineHygieneConfig::default();
        cfg.obfuscate_sync_start = false;
        assert!(maybe_obfuscate_scan_start(1000, 0, &cfg, &mut rng).is_none());
    }

    #[test]
    fn broadcast_delay_skip_is_zero() {
        let mut rng = StdRng::seed_from_u64(3);
        let plan = plan_broadcast_delay(&BaselineHygieneConfig::default(), true, &mut rng);
        assert!(plan.delay.is_zero());
    }

    #[test]
    fn broadcast_delay_in_range() {
        let mut rng = StdRng::seed_from_u64(9);
        let cfg = BaselineHygieneConfig {
            broadcast_delay_min_secs: 10,
            broadcast_delay_max_secs: 20,
            ..BaselineHygieneConfig::default()
        };
        for _ in 0..30 {
            let plan = plan_broadcast_delay(&cfg, false, &mut rng);
            let secs = plan.delay.as_secs();
            assert!((10..=20).contains(&secs));
        }
    }

    #[test]
    fn tip_guard_blocks_recent_sync() {
        let cfg = BaselineHygieneConfig {
            min_secs_after_tip_sync: 120,
            ..BaselineHygieneConfig::default()
        };
        let now = unix_now_secs();
        let blocked = assess_tip_sync_guard(&cfg, Some(now.saturating_sub(30)), false);
        assert!(!blocked.allowed);
        let ok = assess_tip_sync_guard(&cfg, Some(now.saturating_sub(200)), false);
        assert!(ok.allowed);
    }

    #[test]
    fn tip_guard_skip_allows() {
        let cfg = BaselineHygieneConfig::default();
        let now = unix_now_secs();
        let r = assess_tip_sync_guard(&cfg, Some(now), true);
        assert!(r.allowed);
    }
}
