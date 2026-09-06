//! Human-facing Nozy × Crosslink output (guardian framing, not raw jq).

use super::types::{
    FinalizedTip, GuardianSnapshot, NextAction, RosterEntry, StakingPositions, WalletSyncStatus,
};
use super::{short_hex, zat_to_ctaz};
use serde_json::Value;

pub fn print_feature_net_banner() {
    println!("Nozy × Crosslink");
    println!();
}

pub fn display_status(snap: &GuardianSnapshot, full_keys: bool) {
    print_feature_net_banner();
    println!("RPC: {}", snap.rpc_url);
    println!("PoW height: {}", snap.height);

    match snap.tfl_activated {
        Some(true) => println!("TFL: activated"),
        Some(false) => println!("TFL: not activated yet"),
        None => println!("TFL: unknown (is_tfl_activated unavailable)"),
    }

    if let Some(ref tip) = snap.finalized_tip {
        display_tip_line(tip, snap.height);
    }

    if let Some(n) = snap.finalizer_count {
        println!("Finalizers (recency list): {n}");
    }
    if let Some(h) = snap.pos_height {
        println!("Your finalizer PoS height: {h}");
    }

    let day = &snap.staking_day;
    if day.open {
        println!(
            "Staking Day: OPEN — {} block(s) left in window",
            day.blocks_remaining_in_window.unwrap_or(0)
        );
    } else {
        println!(
            "Staking Day: CLOSED — {} block(s) until next window",
            day.blocks_until_next.unwrap_or(0)
        );
    }
    println!(
        "  (Season 1: every {} blocks, open {} blocks)",
        day.cycle, day.window
    );

    let p = &snap.positions;
    println!();
    println!("Bonds");
    println!(
        "  Active: {}  |  bonded ~{:.8} cTAZ",
        p.active_count(),
        zat_to_ctaz(p.bonded_zat())
    );
    println!(
        "  Withdrawable: {}  |  ~{:.8} cTAZ",
        p.withdrawable.len(),
        zat_to_ctaz(p.withdrawable_zat())
    );
    println!(
        "  Rewards earned (latest − initial): ~{:.8} cTAZ",
        zat_to_ctaz(p.total_rewards_zat())
    );
    if let Some(ref w) = snap.wallet {
        println!();
        println!("Node wallet (get_wallet_sync_status)");
        println!(
            "  Available to stake: ~{:.8} cTAZ",
            zat_to_ctaz(w.available_to_stake_zats())
        );
        if w.user_shielded_pending_zats > 0 {
            println!(
                "  Pending (confirming): ~{:.8} cTAZ",
                zat_to_ctaz(w.user_shielded_pending_zats)
            );
        }
        println!(
            "  Wallet scan: {} / {} {}",
            w.sync_height,
            w.tip_height,
            if w.wallet_synced() {
                "(synced)"
            } else {
                "(still scanning — balance may be low)"
            }
        );
    } else {
        println!();
        println!(
            "Node wallet: spendable balance unavailable (node lacks get_wallet_sync_status RPC)"
        );
        println!("  Upgrade: docs/CROSSLINK_WALLET_RPC_UPGRADE.md");
    }

    println!();
    println!("Next: {}", format_next_action(&snap.next_action));

    if full_keys && (p.active_count() > 0 || !p.withdrawable.is_empty()) {
        println!();
        display_positions(p, true);
    }

    println!();
    println!("Commands: nozy crosslink positions | wallet | roster | stake | unbond | withdraw");
}

/// Human-readable node wallet balances (`get_wallet_sync_status`).
pub fn display_wallet(w: &WalletSyncStatus) {
    print_feature_net_banner();
    println!("Node wallet (get_wallet_sync_status)");
    println!(
        "  Available to stake: ~{:.8} cTAZ",
        zat_to_ctaz(w.available_to_stake_zats())
    );
    if w.user_shielded_spendable_zats > 0 || w.user_unshielded_zats > 0 {
        println!(
            "  Shielded spendable: ~{:.8} cTAZ",
            zat_to_ctaz(w.user_shielded_spendable_zats)
        );
        if w.user_unshielded_zats > 0 {
            println!(
                "  Unshielded: ~{:.8} cTAZ",
                zat_to_ctaz(w.user_unshielded_zats)
            );
        }
    }
    if w.user_shielded_pending_zats > 0 {
        println!(
            "  Pending (confirming): ~{:.8} cTAZ",
            zat_to_ctaz(w.user_shielded_pending_zats)
        );
    }
    if w.staked_zats > 0 {
        println!(
            "  Staked (node view): ~{:.8} cTAZ",
            zat_to_ctaz(w.staked_zats)
        );
    }
    if w.withdrawable_zats > 0 {
        println!(
            "  Withdrawable (node view): ~{:.8} cTAZ",
            zat_to_ctaz(w.withdrawable_zats)
        );
    }
    println!(
        "  Wallet scan: {} / {} {}",
        w.sync_height,
        w.tip_height,
        if w.wallet_synced() {
            "(synced)"
        } else {
            "(still scanning — balance may be low)"
        }
    );
    println!();
}

fn display_tip_line(tip: &FinalizedTip, pow_height: u32) {
    match (tip.height, tip.hash.as_ref()) {
        (Some(h), Some(hash)) => {
            let lag = pow_height.saturating_sub(h);
            println!(
                "Finalized tip: {} ({})  lag {} block(s)",
                h,
                short_hex(hash, 8, 8),
                lag
            );
        }
        (Some(h), None) => println!("Finalized tip height: {h}"),
        (None, Some(hash)) => println!("Finalized tip hash: {}", short_hex(hash, 8, 8)),
        _ => println!("Finalized tip: unavailable"),
    }
}

fn format_next_action(action: &NextAction) -> String {
    match action {
        NextAction::WaitForStakingDay { blocks } => {
            format!("Wait ~{blocks} blocks for Staking Day, then stake / unbond / withdraw.")
        }
        NextAction::WithdrawReady { count } => {
            format!(
                "{count} bond(s) ready — run `nozy crosslink withdraw --bond <pk>` while the window is open."
            )
        }
        NextAction::UnbondToExit => {
            "Window open with active stake — unbond to start exit, or retarget anytime."
                .into()
        }
        NextAction::StakeOrGuardian => {
            "Window open — stake to a finalizer (`nozy crosslink stake`) or back your own identity from the monolith UI."
                .into()
        }
        NextAction::RetargetIfNeeded => {
            "Window closed — you can still `nozy crosslink retarget` if a finalizer misbehaves."
                .into()
        }
    }
}

pub fn display_positions(positions: &StakingPositions, full_keys: bool) {
    print_feature_net_banner();
    println!("=== Active positions ===");
    if positions.active.is_empty() {
        println!("  (none)");
    } else {
        for (finalizer, bonds) in &positions.active {
            println!(
                "Finalizer: {}",
                if full_keys {
                    finalizer.clone()
                } else {
                    short_hex(finalizer, 10, 8)
                }
            );
            for b in bonds {
                let key = if full_keys {
                    b.pk.clone()
                } else {
                    short_hex(&b.pk, 10, 8)
                };
                println!("  bond_key (pk): {key}");
                if let Some(h) = b.create_height {
                    println!("  create_height: {h}");
                }
                println!(
                    "  initial: {:.8} cTAZ   latest: {:.8} cTAZ   rewards: {:.8} cTAZ",
                    zat_to_ctaz(b.initial_val),
                    zat_to_ctaz(b.latest_val),
                    zat_to_ctaz(b.rewards_zat())
                );
                println!();
            }
        }
    }

    println!("=== Withdrawable ===");
    if positions.withdrawable.is_empty() {
        println!("  (none)");
    } else {
        for b in &positions.withdrawable {
            let key = if full_keys {
                b.pk.clone()
            } else {
                short_hex(&b.pk, 10, 8)
            };
            println!("  bond_key (pk): {key}");
            println!(
                "  value: {:.8} cTAZ   rewards: {:.8} cTAZ",
                zat_to_ctaz(b.latest_val),
                zat_to_ctaz(b.rewards_zat())
            );
            println!();
        }
    }
    if !full_keys {
        println!("Tip: pass --full-keys to print complete pk / finalizer hex for RPC actions.");
    }
}

pub fn display_roster(entries: &[RosterEntry], full_keys: bool) {
    print_feature_net_banner();
    println!("Finalizer roster (highest stake first)");
    println!();
    if entries.is_empty() {
        println!("  (empty or unparsable — try --json for raw RPC)");
        return;
    }
    for (i, e) in entries.iter().enumerate() {
        let id = if full_keys {
            e.finalizer.clone()
        } else {
            short_hex(&e.finalizer, 10, 8)
        };
        println!(
            "{:>3}. {}  {:.8} cTAZ  ({:.2}%)",
            i + 1,
            id,
            zat_to_ctaz(e.stake_zat),
            e.share * 100.0
        );
    }
}

pub fn display_finality(tip: &Option<FinalizedTip>, detail: Option<&Value>) {
    print_feature_net_banner();
    match tip {
        Some(t) => {
            println!(
                "Finalized tip height: {}",
                t.height
                    .map(|h| h.to_string())
                    .unwrap_or_else(|| "—".into())
            );
            println!(
                "Finalized tip hash:   {}",
                t.hash.clone().unwrap_or_else(|| "—".into())
            );
        }
        None => println!("Finalized tip: unavailable"),
    }
    if let Some(v) = detail {
        println!();
        println!("{}", serde_json::to_string_pretty(v).unwrap_or_default());
    }
}

pub fn display_bond(pk: &str, info: &Value) {
    print_feature_net_banner();
    println!("Bond lookup (getbondinfo)");
    println!("  positions pk: {}", pk);
    println!(
        "  (Nozy byte-reversed pk for getbondinfo — same rule as crosslinkUtilities/bondinfo.sh)"
    );
    println!();
    println!("{}", serde_json::to_string_pretty(info).unwrap_or_default());
}
