//! Send-time readiness: witness freshness checks and proving warm-up hooks.

use crate::error::{NozyError, NozyResult};
use crate::notes::SpendableNote;

/// Block send when Orchard witness lag exceeds this (forces sync-to-tip first).
///
/// At ~75 s/block, 50 blocks ≈ 1 hour of catch-up RPC at send time — still slow but bounded.
/// Typical synced wallet: lag 0–10 → send completes in ~1–3 minutes on operator hardware.
pub const MAX_SEND_WITNESS_LAG_BLOCKS: u32 = 50;

/// Parallel `getblock` fetches per batch during witness catch-up at spend time.
pub const WITNESS_CATCHUP_PARALLEL_BLOCKS: usize = 10;

/// Blocks the spend note's witness is behind `chain_tip`.
pub fn orchard_witness_lag_blocks(note: &SpendableNote, chain_tip: u32) -> u32 {
    witness_lag_from_stored_tip(note.orchard_witness_tip_height, chain_tip)
}

/// Lag from a stored witness tip height (None → treated as 0).
pub fn witness_lag_from_stored_tip(stored_tip: Option<u32>, chain_tip: u32) -> u32 {
    let stored = stored_tip.unwrap_or(0);
    chain_tip.saturating_sub(stored)
}

/// Max witness lag across all spendable notes (for diagnostics).
pub fn max_witness_lag_blocks(notes: &[SpendableNote], chain_tip: u32) -> u32 {
    notes
        .iter()
        .map(|n| orchard_witness_lag_blocks(n, chain_tip))
        .max()
        .unwrap_or(0)
}

/// Max witness lag across unspent serialized notes (pre-spendable cache check).
pub fn max_serialized_witness_lag_blocks(
    notes: &[crate::notes::SerializableOrchardNote],
    chain_tip: u32,
) -> u32 {
    notes
        .iter()
        .filter(|n| !n.spent)
        .map(|n| witness_lag_from_stored_tip(n.orchard_witness_tip_height, chain_tip))
        .max()
        .unwrap_or(0)
}

/// Reject send when any unspent cached note is too far behind tip (before note scan / spend build).
pub fn ensure_cached_witness_fresh_for_send(
    notes: &[crate::notes::SerializableOrchardNote],
    chain_tip: u32,
) -> NozyResult<()> {
    let lag = max_serialized_witness_lag_blocks(notes, chain_tip);
    if lag <= MAX_SEND_WITNESS_LAG_BLOCKS {
        return Ok(());
    }
    let stored = notes
        .iter()
        .filter(|n| !n.spent)
        .map(|n| {
            (
                witness_lag_from_stored_tip(n.orchard_witness_tip_height, chain_tip),
                n,
            )
        })
        .max_by_key(|(lag, _)| *lag)
        .map(|(_, n)| n.orchard_witness_tip_height.unwrap_or(0))
        .unwrap_or(0);
    Err(NozyError::InvalidOperation(format!(
        "Orchard witness is {lag} blocks behind chain tip (witness tip {stored}, chain tip {chain_tip}). \
         Sync to tip before sending — otherwise witness catch-up can take many minutes. \
         Run `nozy sync --to-tip` or POST /api/sync until caught up (max lag {MAX_SEND_WITNESS_LAG_BLOCKS} blocks)."
    )))
}

pub fn is_witness_stale_for_send_error(message: &str) -> bool {
    message.contains("Sync to tip before sending")
}

/// Reject send when witness catch-up would dominate latency; user should sync first.
pub fn ensure_witness_fresh_for_send(note: &SpendableNote, chain_tip: u32) -> NozyResult<()> {
    let lag = orchard_witness_lag_blocks(note, chain_tip);
    if lag <= MAX_SEND_WITNESS_LAG_BLOCKS {
        return Ok(());
    }
    let stored = note.orchard_witness_tip_height.unwrap_or(0);
    Err(NozyError::InvalidOperation(format!(
        "Orchard witness is {lag} blocks behind chain tip (witness tip {stored}, chain tip {chain_tip}). \
         Sync to tip before sending — otherwise witness catch-up can take many minutes. \
         Run `nozy sync --to-tip` or POST /api/sync until caught up (max lag {MAX_SEND_WITNESS_LAG_BLOCKS} blocks)."
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_lag_empty_notes() {
        assert_eq!(max_witness_lag_blocks(&[], 99), 0);
    }

    #[test]
    fn threshold_is_fifty_blocks() {
        assert_eq!(MAX_SEND_WITNESS_LAG_BLOCKS, 50);
    }

    #[test]
    fn witness_lag_at_threshold_allows_send() {
        assert_eq!(witness_lag_from_stored_tip(Some(100), 150), 50);
    }

    #[test]
    fn witness_lag_above_threshold_blocks_send() {
        assert_eq!(witness_lag_from_stored_tip(Some(100), 151), 51);
    }

    #[test]
    fn witness_lag_none_treated_as_zero() {
        assert_eq!(witness_lag_from_stored_tip(None, 5000), 5000);
    }

    #[test]
    fn stale_error_message_detected() {
        let msg = "Orchard witness is 5071 blocks behind chain tip. Sync to tip before sending";
        assert!(is_witness_stale_for_send_error(msg));
        assert!(!is_witness_stale_for_send_error("insufficient funds"));
    }

    #[test]
    fn serialized_max_lag_empty() {
        assert_eq!(max_serialized_witness_lag_blocks(&[], 100), 0);
    }
}
