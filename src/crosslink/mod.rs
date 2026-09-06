//! Nozy × Crosslink — Protocol Guardian companion for Season 1 feature-nets.
//!
//! Talks to a local [`crosslink_monolith`](https://github.com/ShieldedLabs/crosslink_monolith)
//! / zebra-crosslink node over JSON-RPC. Staking txs are built and signed by the **node
//! wallet** (`wallet_staking_action`); Nozy provides privacy-framed status, next-action
//! guidance, and a safe CLI — not a bare RPC dump.
//!
//! Community RPC maps that informed this module:
//! - <https://github.com/dismad/crosslinkUtilities>
//! - <https://github.com/dismad/zcash-explorer/tree/crosslink>

mod display;
mod rpc;
mod types;

pub use display::{
    display_bond, display_finality, display_positions, display_roster, display_status,
    display_wallet, print_feature_net_banner,
};
pub use rpc::{
    block_finality, bond_info, build_crosslink_client, fetch_guardian_snapshot, finality_tip,
    is_tfl_activated, roster, staking_action, staking_positions, tx_finality, wallet_sync_status,
    wallet_ufvk,
};
pub use types::{
    BondPosition, FinalizedTip, GuardianSnapshot, NextAction, RosterEntry, StakingAction,
    StakingDayWindow, StakingPositions, WalletSyncStatus, SEASON1_STAKING_CYCLE_BLOCKS,
    SEASON1_STAKING_WINDOW_BLOCKS,
};

use crate::error::{NozyError, NozyResult};

/// Season 1 Staking Day window from chain tip height.
pub fn staking_day_at(height: u32) -> StakingDayWindow {
    staking_day_at_with_params(
        height,
        SEASON1_STAKING_CYCLE_BLOCKS,
        SEASON1_STAKING_WINDOW_BLOCKS,
    )
}

pub fn staking_day_at_with_params(height: u32, cycle: u32, window: u32) -> StakingDayWindow {
    let cycle = cycle.max(1);
    let window = window.min(cycle);
    let offset = height % cycle;
    if offset < window {
        StakingDayWindow {
            height,
            offset,
            cycle,
            window,
            open: true,
            blocks_remaining_in_window: Some(window - offset),
            blocks_until_next: None,
        }
    } else {
        StakingDayWindow {
            height,
            offset,
            cycle,
            window,
            open: false,
            blocks_remaining_in_window: None,
            blocks_until_next: Some(cycle - offset),
        }
    }
}

/// `wallet_staking_positions` returns `pk` as JSON byte-reversed PubKeyID.
/// `getbondinfo` expects the native (reversed-back) hex key.
pub fn reverse_bond_pk_hex(pk_hex: &str) -> NozyResult<String> {
    let hex = pk_hex.trim().trim_start_matches("0x");
    let bytes = hex::decode(hex).map_err(|e| {
        NozyError::InvalidInput(format!(
            "Invalid bond key hex ({e}): use the `pk` from positions"
        ))
    })?;
    if bytes.is_empty() {
        return Err(NozyError::InvalidInput("Bond key is empty".into()));
    }
    let mut rev = bytes;
    rev.reverse();
    Ok(hex::encode(rev))
}

/// Truncate long hex for display (full key still available via `--json` / `--full-keys`).
pub fn short_hex(hex: &str, head: usize, tail: usize) -> String {
    let h = hex.trim();
    if h.len() <= head + tail + 1 {
        return h.to_string();
    }
    format!("{}…{}", &h[..head], &h[h.len() - tail..])
}

pub fn zat_to_ctaz(zat: u64) -> f64 {
    zat as f64 / 100_000_000.0
}

pub fn ctaz_to_zat(amount: f64) -> NozyResult<u64> {
    if !amount.is_finite() || amount <= 0.0 {
        return Err(NozyError::InvalidInput(
            "Amount must be a positive finite number (cTAZ)".into(),
        ));
    }
    let zat = (amount * 100_000_000.0).round();
    if zat < 1.0 || zat > u64::MAX as f64 {
        return Err(NozyError::InvalidInput("Amount out of range".into()));
    }
    Ok(zat as u64)
}

pub fn normalize_finalizer_hex(s: &str) -> NozyResult<String> {
    let hex = s.trim().trim_start_matches("0x").to_ascii_lowercase();
    if hex.len() != 64 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(NozyError::InvalidInput(
            "Finalizer identity must be 64 hex characters (copy from roster / monolith UI)".into(),
        ));
    }
    Ok(hex)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn staking_day_open_closed() {
        let open = staking_day_at(0);
        assert!(open.open);
        assert_eq!(open.blocks_remaining_in_window, Some(70));

        let last_open = staking_day_at(69);
        assert!(last_open.open);
        assert_eq!(last_open.blocks_remaining_in_window, Some(1));

        // height 70: offset == window → closed (offset < 70 is false)
        let closed = staking_day_at(70);
        assert!(!closed.open);
        assert_eq!(closed.blocks_until_next, Some(80));

        let mid = staking_day_at(35);
        assert!(mid.open);
        assert_eq!(mid.blocks_remaining_in_window, Some(35));
    }

    #[test]
    fn reverse_pk_roundtrip() {
        let original = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
        let rev = reverse_bond_pk_hex(original).unwrap();
        let back = reverse_bond_pk_hex(&rev).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn ctaz_conversion() {
        assert_eq!(ctaz_to_zat(0.01).unwrap(), 1_000_000);
        assert!((zat_to_ctaz(1_000_000) - 0.01).abs() < 1e-12);
    }
}
