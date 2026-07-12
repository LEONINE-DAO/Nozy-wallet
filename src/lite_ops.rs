//! Nozy Lite ops: health checks and JSON snapshots for monitoring.

use crate::cli_helpers::wallet_balance_snapshot;
use crate::config::WalletConfig;
use crate::error::NozyResult;
use crate::ironwood::{fetch_pool_balances, is_ironwood_active};
use crate::notes::load_wallet_notes;
use crate::shielded_pool::ShieldedPool;
use crate::sync_status::{gather_sync_status, SyncStatusSnapshot};
use crate::zebra_integration::ZebraClient;
use serde::Serialize;

/// Default max RPC scan gap (blocks) before `nozy health` exits 2.
pub const DEFAULT_MAX_SCAN_GAP: u32 = 1000;

#[derive(Debug, Clone, Serialize)]
pub struct LiteBalanceJson {
    pub confirmed_zatoshis: u64,
    pub pending_zatoshis: u64,
    pub available_zatoshis: u64,
    pub unspent_note_count: usize,
    pub orchard_unspent_zatoshis: u64,
    pub ironwood_unspent_zatoshis: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct LiteSyncJson {
    pub zebra_tip: Option<u32>,
    pub last_scan_height: Option<u32>,
    pub rpc_scan_gap: Option<u32>,
    pub lightwalletd_url: String,
    pub lwd_tip: Option<u64>,
    pub lwd_error: Option<String>,
    pub compact_max_height: Option<u64>,
    pub compact_db_exists: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct LiteHealthReport {
    pub ok: bool,
    pub exit_code: u8,
    pub zebra_reachable: bool,
    pub zebra_tip: Option<u32>,
    pub last_scan_height: Option<u32>,
    pub rpc_scan_gap: Option<u32>,
    pub max_scan_gap: u32,
    pub network: String,
    pub ironwood_active: bool,
    pub ironwood_rpc_detected: bool,
    pub lwd_ok: bool,
    pub balance: Option<LiteBalanceJson>,
    pub sync: LiteSyncJson,
    pub checks: Vec<String>,
    pub failures: Vec<String>,
}

pub fn rpc_scan_gap(snapshot: &SyncStatusSnapshot) -> Option<u32> {
    match (snapshot.zebra_tip, snapshot.last_scan_height) {
        (Some(tip), Some(last)) if tip >= last => Some(tip - last),
        (Some(_tip), Some(_last)) => Some(0), // last ahead of tip while node catches up
        (Some(_), None) => None,
        _ => None,
    }
}

pub fn sync_to_json(snapshot: &SyncStatusSnapshot) -> LiteSyncJson {
    LiteSyncJson {
        zebra_tip: snapshot.zebra_tip,
        last_scan_height: snapshot.last_scan_height,
        rpc_scan_gap: rpc_scan_gap(snapshot),
        lightwalletd_url: snapshot.lightwalletd_url.clone(),
        lwd_tip: snapshot.lwd_tip,
        lwd_error: snapshot.lwd_error.clone(),
        compact_max_height: snapshot.compact_max_height,
        compact_db_exists: snapshot.compact_db_exists,
    }
}

pub fn balance_to_json() -> NozyResult<LiteBalanceJson> {
    let snapshot = wallet_balance_snapshot()?;
    let notes = load_wallet_notes().unwrap_or_default();
    let orchard_unspent_zatoshis: u64 = notes
        .iter()
        .filter(|n| !n.spent && n.pool == ShieldedPool::Orchard)
        .map(|n| n.value)
        .sum();
    let ironwood_unspent_zatoshis: u64 = notes
        .iter()
        .filter(|n| !n.spent && n.pool == ShieldedPool::Ironwood)
        .map(|n| n.value)
        .sum();
    Ok(LiteBalanceJson {
        confirmed_zatoshis: snapshot.confirmed_zatoshis,
        pending_zatoshis: snapshot.pending_zatoshis,
        available_zatoshis: snapshot.available_zatoshis,
        unspent_note_count: snapshot.unspent_note_count,
        orchard_unspent_zatoshis,
        ironwood_unspent_zatoshis,
    })
}

pub async fn gather_health_report(
    zebra: &ZebraClient,
    config: &WalletConfig,
    max_scan_gap: u32,
    require_lwd: bool,
    require_ironwood_rpc: bool,
) -> LiteHealthReport {
    let sync = gather_sync_status(zebra, config).await;
    let sync_json = sync_to_json(&sync);
    let is_testnet = config.network.eq_ignore_ascii_case("testnet");
    let zebra_reachable = sync.zebra_tip.is_some();
    let ironwood_active =
        zebra_reachable && is_ironwood_active(sync.zebra_tip.unwrap_or(0), is_testnet);

    let mut ironwood_rpc_detected = false;
    if zebra_reachable {
        if let Ok(pools) = fetch_pool_balances(zebra).await {
            ironwood_rpc_detected = pools
                .iter()
                .any(|p| matches!(p.pool, ShieldedPool::Ironwood));
        }
    }

    let lwd_ok = sync.lwd_tip.is_some() && sync.lwd_error.is_none();
    let gap = rpc_scan_gap(&sync);

    let mut checks = Vec::new();
    let mut failures = Vec::new();
    let mut exit_code: u8 = 0;

    if zebra_reachable {
        checks.push(format!(
            "zebra_reachable tip={}",
            sync.zebra_tip.unwrap_or(0)
        ));
    } else {
        failures.push("zebra_unreachable".to_string());
        exit_code = 1;
    }

    if exit_code == 0 {
        match gap {
            Some(g) if g > max_scan_gap => {
                failures.push(format!(
                    "rpc_scan_gap={g} exceeds max_scan_gap={max_scan_gap}"
                ));
                exit_code = 2;
            }
            Some(g) => checks.push(format!("rpc_scan_gap={g} ok")),
            None if zebra_reachable => {
                failures.push(format!(
                    "never_scanned (treat as gap); use --max-scan-gap or run sync"
                ));
                exit_code = 2;
            }
            None => {}
        }
    }

    if require_lwd && exit_code == 0 {
        if lwd_ok {
            checks.push("lwd_ok".to_string());
        } else {
            failures.push(format!(
                "lwd_required_failed: {}",
                sync.lwd_error
                    .clone()
                    .unwrap_or_else(|| "no tip".to_string())
            ));
            exit_code = 3;
        }
    }

    if require_ironwood_rpc && exit_code == 0 {
        if ironwood_rpc_detected {
            checks.push("ironwood_rpc_detected".to_string());
        } else {
            failures.push("ironwood_rpc_required_but_not_detected".to_string());
            exit_code = 3;
        }
    } else if ironwood_active && !ironwood_rpc_detected {
        checks.push("ironwood_active_but_rpc_pool_not_seen (warn only)".to_string());
    }

    let balance = match balance_to_json() {
        Ok(b) => {
            checks.push(format!("balance_ok available_zat={}", b.available_zatoshis));
            Some(b)
        }
        Err(e) => {
            if exit_code == 0 {
                failures.push(format!("wallet_data_unreadable: {e}"));
                exit_code = 4;
            } else {
                checks.push(format!("wallet_data_skipped: {e}"));
            }
            None
        }
    };

    LiteHealthReport {
        ok: exit_code == 0,
        exit_code,
        zebra_reachable,
        zebra_tip: sync.zebra_tip,
        last_scan_height: sync.last_scan_height,
        rpc_scan_gap: gap,
        max_scan_gap,
        network: config.network.clone(),
        ironwood_active,
        ironwood_rpc_detected,
        lwd_ok,
        balance,
        sync: sync_json,
        checks,
        failures,
    }
}

pub fn print_health_human(report: &LiteHealthReport) {
    println!("Nozy Lite health");
    println!("================");
    println!("result: {}", if report.ok { "OK" } else { "UNHEALTHY" });
    println!("exit_code: {}", report.exit_code);
    println!("network: {}", report.network);
    println!(
        "zebra: {} tip={:?}",
        if report.zebra_reachable { "up" } else { "down" },
        report.zebra_tip
    );
    println!(
        "scan: last={:?} gap={:?} (max={})",
        report.last_scan_height, report.rpc_scan_gap, report.max_scan_gap
    );
    println!(
        "ironwood: active={} rpc_pool={}",
        report.ironwood_active, report.ironwood_rpc_detected
    );
    println!("lwd: {}", if report.lwd_ok { "ok" } else { "down/skip" });
    if let Some(b) = &report.balance {
        println!(
            "balance: available={:.8} ZEC (orchard={} zat, ironwood={} zat)",
            b.available_zatoshis as f64 / 100_000_000.0,
            b.orchard_unspent_zatoshis,
            b.ironwood_unspent_zatoshis
        );
    }
    for c in &report.checks {
        println!("  check: {c}");
    }
    for f in &report.failures {
        println!("  fail:  {f}");
    }
}
