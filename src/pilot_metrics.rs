//! Local-only dynamic-fee pilot counters (Phase A′2).
//!
//! Counts only — no amounts, addresses, or txids. Opt-in via file presence after first event;
//! never leaves the machine unless the operator copies `pilot_metrics.json`.

use crate::error::{NozyError, NozyResult};
use crate::paths::get_wallet_data_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PilotMetrics {
    pub priority_sends: u64,
    pub speed_ups: u64,
    pub expired_unmined: u64,
    pub speed_up_confirmed: u64,
    #[serde(default)]
    pub updated_at: u64,
}

impl PilotMetrics {
    pub fn metrics_path() -> PathBuf {
        get_wallet_data_dir().join("pilot_metrics.json")
    }

    pub fn load() -> Self {
        let path = Self::metrics_path();
        if !path.exists() {
            return Self::default();
        }
        fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> NozyResult<()> {
        let path = Self::metrics_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                NozyError::Storage(format!("Failed to create pilot metrics dir: {e}"))
            })?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| NozyError::Storage(format!("Failed to serialize pilot metrics: {e}")))?;
        fs::write(&path, json)
            .map_err(|e| NozyError::Storage(format!("Failed to write pilot metrics: {e}")))?;
        Ok(())
    }

    fn touch(&mut self) {
        self.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
    }

    pub fn bump_priority_send(&mut self) {
        self.priority_sends = self.priority_sends.saturating_add(1);
        self.touch();
    }

    pub fn bump_speed_up(&mut self) {
        self.speed_ups = self.speed_ups.saturating_add(1);
        self.touch();
    }

    pub fn bump_expired_unmined(&mut self, n: u64) {
        self.expired_unmined = self.expired_unmined.saturating_add(n);
        self.touch();
    }

    pub fn bump_speed_up_confirmed(&mut self) {
        self.speed_up_confirmed = self.speed_up_confirmed.saturating_add(1);
        self.touch();
    }
}

static LOCK: Mutex<()> = Mutex::new(());

fn with_metrics<F: FnOnce(&mut PilotMetrics)>(f: F) {
    let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut m = PilotMetrics::load();
    f(&mut m);
    let _ = m.save();
}

/// Record a priority (pilot) send. Best-effort; never fails the send path.
pub fn record_priority_send() {
    with_metrics(|m| m.bump_priority_send());
}

pub fn record_speed_up() {
    with_metrics(|m| m.bump_speed_up());
}

pub fn record_expired_unmined(count: usize) {
    if count == 0 {
        return;
    }
    with_metrics(|m| m.bump_expired_unmined(count as u64));
}

pub fn record_speed_up_confirmed() {
    with_metrics(|m| m.bump_speed_up_confirmed());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bump_counters() {
        let mut m = PilotMetrics::default();
        m.bump_priority_send();
        m.bump_speed_up();
        m.bump_expired_unmined(2);
        m.bump_speed_up_confirmed();
        assert_eq!(m.priority_sends, 1);
        assert_eq!(m.speed_ups, 1);
        assert_eq!(m.expired_unmined, 2);
        assert_eq!(m.speed_up_confirmed, 1);
        assert!(m.updated_at > 0);
    }
}
