//! Ironwood readiness and pool balance reporting.

use crate::error::NozyResult;
use crate::shielded_pool::ShieldedPool;
use crate::zebra_integration::{OrchardPoolStats, ShieldedPoolStats, ZebraClient};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

pub const ORCHARD_ONLY_SENDS_DISABLED_AFTER_IRONWOOD: &str = "Ironwood (NU6.3) is active on this network. Orchard-only sends are disabled; normal sends route through the Ironwood pool when Ironwood notes are indexed.";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolBalanceSummary {
    pub pool: ShieldedPool,
    pub chain_value_zec: f64,
    pub chain_value_zat: u64,
    pub monitored: bool,
    pub block_height: u32,
}

impl From<OrchardPoolStats> for PoolBalanceSummary {
    fn from(stats: OrchardPoolStats) -> Self {
        Self {
            pool: ShieldedPool::Orchard,
            chain_value_zec: stats.chain_value_zec,
            chain_value_zat: stats.chain_value_zat,
            monitored: stats.monitored,
            block_height: stats.block_height,
        }
    }
}

impl From<ShieldedPoolStats> for PoolBalanceSummary {
    fn from(stats: ShieldedPoolStats) -> Self {
        Self {
            pool: ShieldedPool::Ironwood,
            chain_value_zec: stats.chain_value_zec,
            chain_value_zat: stats.chain_value_zat,
            monitored: stats.monitored,
            block_height: stats.block_height,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IronwoodWalletStatus {
    pub chain_tip: u32,
    pub ironwood_active: bool,
    pub nu6_3_activation_height: Option<u32>,
    pub pools: Vec<PoolBalanceSummary>,
    pub orchard_notes_unspent: u64,
    pub ironwood_notes_unspent: u64,
    pub migration_recommended: bool,
    pub wallet_ready: bool,
    pub blockers: Vec<String>,
}

pub async fn fetch_pool_balances(zebra: &ZebraClient) -> NozyResult<Vec<PoolBalanceSummary>> {
    let mut pools = Vec::new();
    pools.push(zebra.get_orchard_pool_stats().await?.into());
    if let Ok(stats) = zebra.get_ironwood_pool_stats().await {
        pools.push(stats.into());
    }
    Ok(pools)
}

pub fn get_blockchain_info_reports_ironwood_active(
    info: &HashMap<String, Value>,
    chain_tip: u32,
) -> bool {
    let upgrades_active = info
        .get("upgrades")
        .and_then(|v| v.as_object())
        .map(|upgrades| {
            upgrades.values().any(|upgrade| {
                let name_matches = upgrade
                    .get("name")
                    .and_then(|v| v.as_str())
                    .is_some_and(|name| name.eq_ignore_ascii_case("NU6.3"));
                if !name_matches {
                    return false;
                }

                let status_active = upgrade
                    .get("status")
                    .and_then(|v| v.as_str())
                    .is_some_and(|status| status.eq_ignore_ascii_case("active"));
                let activation_reached = upgrade
                    .get("activationheight")
                    .and_then(|v| v.as_u64())
                    .is_some_and(|height| chain_tip >= height as u32);
                status_active || activation_reached
            })
        })
        .unwrap_or(false);

    if upgrades_active {
        return true;
    }

    let is_testnet = info
        .get("chain")
        .and_then(|v| v.as_str())
        .is_some_and(|chain| matches!(chain, "test" | "testnet" | "regtest"));
    super::nu6_3_activation_height(is_testnet).is_some_and(|height| chain_tip >= height)
}

pub async fn orchard_only_send_blocker(
    zebra: &ZebraClient,
    chain_tip: u32,
) -> NozyResult<Option<String>> {
    let info = zebra.get_blockchain_info().await?;
    if get_blockchain_info_reports_ironwood_active(&info, chain_tip) {
        Ok(Some(ORCHARD_ONLY_SENDS_DISABLED_AFTER_IRONWOOD.to_string()))
    } else {
        Ok(None)
    }
}

/// Block legacy Orchard-only send builders (Keystone / co-sign) when Ironwood is active.
pub async fn legacy_hardware_send_blocker(
    zebra: &ZebraClient,
    chain_tip: u32,
    orchard_notes_unspent_zat: u64,
    ironwood_notes_unspent_zat: u64,
) -> NozyResult<Option<String>> {
    let info = zebra.get_blockchain_info().await?;
    if !get_blockchain_info_reports_ironwood_active(&info, chain_tip) {
        return Ok(None);
    }
    if ironwood_notes_unspent_zat > 0 && orchard_notes_unspent_zat == 0 {
        return Ok(Some(
            "Ironwood (NU6.3) is active and this wallet holds Ironwood notes only. \
             Hardware signing (Keystone) for Ironwood sends is not supported yet; \
             use software Send (`nozy send` or the desktop app)."
                .to_string(),
        ));
    }
    if orchard_notes_unspent_zat > 0 {
        return Ok(Some(
            "Orchard notes remain after Ironwood activation. Run `nozy ironwood migrate` \
             before sending from a hardware wallet."
                .to_string(),
        ));
    }
    Ok(None)
}

/// True when software send should route through Ironwood (post-activation with Ironwood balance).
pub fn ironwood_software_send_available(ironwood_active: bool, ironwood_wallet_zat: u64) -> bool {
    ironwood_active && ironwood_wallet_zat > 0
}

pub fn display_ironwood_status(status: &IronwoodWalletStatus) {
    println!();
    println!("🌲 Ironwood (NU6.3) Wallet Status");
    println!("   Chain tip: {}", status.chain_tip);
    println!(
        "   NU6.3 active: {} (activation height {})",
        if status.ironwood_active { "yes" } else { "no" },
        status
            .nu6_3_activation_height
            .map(|h| h.to_string())
            .unwrap_or_else(|| "not configured for this network".to_string())
    );
    if !status.ironwood_active {
        match status.nu6_3_activation_height {
            Some(height) => {
                println!(
                    "   Blocks until activation: {}",
                    height.saturating_sub(status.chain_tip)
                );
                println!("   Migration mode: planning only until activation");
            }
            None => {
                println!("   Migration mode: planning blocked until activation height is known");
            }
        }
    }
    println!();
    println!("   Pool balances (chain):");
    for pool in &status.pools {
        println!(
            "     {} — {:.2} ZEC (monitored: {})",
            pool.pool, pool.chain_value_zec, pool.monitored
        );
    }
    println!();
    println!(
        "   Wallet notes — Orchard: {} zat | Ironwood: {} zat",
        status.orchard_notes_unspent, status.ironwood_notes_unspent
    );
    println!(
        "   Migration recommended: {}",
        if status.migration_recommended {
            "yes"
        } else {
            "no"
        }
    );
    println!(
        "   Ironwood-ready: {}",
        if status.wallet_ready {
            "yes"
        } else {
            "no (see blockers)"
        }
    );
    for blocker in &status.blockers {
        println!("     • {blocker}");
    }
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_active_ironwood_from_zebra_upgrade_status() {
        let mut info = HashMap::new();
        info.insert("chain".to_string(), serde_json::json!("test"));
        info.insert(
            "upgrades".to_string(),
            serde_json::json!({
                "37a5165b": {
                    "name": "NU6.3",
                    "activationheight": 4_134_000,
                    "status": "active"
                }
            }),
        );

        assert!(get_blockchain_info_reports_ironwood_active(
            &info, 4_134_000
        ));
    }

    #[test]
    fn leaves_orchard_sends_enabled_before_ironwood_activation() {
        let mut info = HashMap::new();
        info.insert("chain".to_string(), serde_json::json!("test"));
        info.insert(
            "upgrades".to_string(),
            serde_json::json!({
                "37a5165b": {
                    "name": "NU6.3",
                    "activationheight": 4_134_000,
                    "status": "pending"
                }
            }),
        );

        assert!(!get_blockchain_info_reports_ironwood_active(
            &info, 2_400_000
        ));
    }
}
