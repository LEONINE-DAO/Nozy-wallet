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
mod doctor;
mod payout;
mod rpc;
mod types;

pub use display::{
    display_bond, display_finality, display_positions, display_roster, display_status,
    display_wallet, print_feature_net_banner,
};
pub use doctor::{
    display_doctor, grades_for_bonds, DoctorCheck, DoctorInput, DoctorReport, LifecycleHint,
    ObserverGrade,
};
pub use payout::{
    display_payout_claim, looks_like_mainnet_orchard, ufvk_fingerprint, PayoutClaimInput,
    PayoutClaimPack,
};
pub use rpc::{
    block_finality, bond_info, build_crosslink_client, fetch_doctor_report,
    fetch_guardian_snapshot, fetch_payout_claim, finality_tip, is_tfl_activated, roster,
    staking_action, staking_positions, tx_finality, wallet_sync_status, wallet_ufvk,
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

/// v15 creates a bond only when `amount_zats` is a positive power of ten
/// (`0.01`, `0.1`, `1`, `10`, … cTAZ).
pub fn ensure_power_of_ten_bond_zats(amount_zats: u64) -> NozyResult<()> {
    if is_power_of_ten(amount_zats) {
        return Ok(());
    }
    Err(NozyError::InvalidInput(
        "Bond amount must be a power of 10 in zatoshis (0.01, 0.1, 1, 10, 100, … cTAZ)".into(),
    ))
}

fn is_power_of_ten(mut n: u64) -> bool {
    if n == 0 {
        return false;
    }
    while n % 10 == 0 {
        n /= 10;
    }
    n == 1
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

/// v15 `target_finalizer` is the finalizer's `zfinv1…` address. The 64-hex roster key is rejected.
pub fn normalize_finalizer_address(s: &str) -> NozyResult<String> {
    let addr = s.trim();
    let hex = addr.trim_start_matches("0x");
    if hex.len() == 64 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(NozyError::InvalidInput(
            "v15 stake and retarget need the finalizer's zfinv1 address, not the 64-hex key. Copy it from the Crosslink GUI or the node log line \"finalizer address:\".".into(),
        ));
    }
    let body_ok = addr.len() >= 16
        && addr
            .get(..6)
            .is_some_and(|p| p.eq_ignore_ascii_case("zfinv1"))
        && addr
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if body_ok {
        return Ok(addr.to_string());
    }
    Err(NozyError::InvalidInput(
        "Finalizer must be a zfinv1 address (copy from the Crosslink GUI or the node log).".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn staking_day_open_closed() {
        let open = staking_day_at(0);
        assert!(open.open);
        assert_eq!(open.blocks_remaining_in_window, Some(3_456));

        let last_open = staking_day_at(3_455);
        assert!(last_open.open);
        assert_eq!(last_open.blocks_remaining_in_window, Some(1));

        // height 3456: offset == window → closed (offset < 3456 is false)
        let closed = staking_day_at(3_456);
        assert!(!closed.open);
        assert_eq!(closed.blocks_until_next, Some(6_912));

        let mid = staking_day_at(1_728);
        assert!(mid.open);
        assert_eq!(mid.blocks_remaining_in_window, Some(1_728));
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

    #[test]
    fn v15_bond_amounts_are_powers_of_ten() {
        for ctaz in [0.01, 0.1, 1.0, 10.0, 100.0] {
            let zats = ctaz_to_zat(ctaz).unwrap();
            assert!(ensure_power_of_ten_bond_zats(zats).is_ok(), "{ctaz}");
        }
        for ctaz in [0.02, 0.5, 2.0, 15.0] {
            let zats = ctaz_to_zat(ctaz).unwrap();
            assert!(ensure_power_of_ten_bond_zats(zats).is_err(), "{ctaz}");
        }
    }

    #[test]
    fn v15_stake_target_is_zfin_address() {
        let addr = "zfinv1ExampleAddress_with-mixedCASE12";
        assert_eq!(normalize_finalizer_address(addr).unwrap(), addr);
        assert!(normalize_finalizer_address(&"ab".repeat(32)).is_err());
        assert!(normalize_finalizer_address("not-an-address").is_err());
    }
}
