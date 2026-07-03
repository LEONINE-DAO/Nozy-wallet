//! Operator-facing sync status: Zebra tip, RPC scan progress, lightwalletd compact cache.

use crate::config::WalletConfig;
use crate::paths::get_wallet_data_dir;
use crate::zebra_integration::ZebraClient;

/// Snapshot of node and wallet sync positions (node-tip-relative; not network-tip).
#[derive(Debug, Clone)]
pub struct SyncStatusSnapshot {
    pub zebra_tip: Option<u32>,
    pub last_scan_height: Option<u32>,
    pub lightwalletd_url: String,
    pub lwd_tip: Option<u64>,
    pub lwd_error: Option<String>,
    pub compact_max_height: Option<u64>,
    pub compact_db_exists: bool,
}

pub fn resolve_lightwalletd_url() -> String {
    std::env::var("LIGHTWALLETD_GRPC").unwrap_or_else(|_| "http://127.0.0.1:9067".to_string())
}

/// lightwalletd is optional for RPC-only sync; do not block status on a dead gRPC port.
const LWD_CONNECT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(3);

pub async fn gather_sync_status(
    zebra_client: &ZebraClient,
    config: &WalletConfig,
) -> SyncStatusSnapshot {
    let zebra_tip = zebra_client.get_block_count().await.ok();
    let lightwalletd_url = resolve_lightwalletd_url();

    let (lwd_tip, lwd_error) = match tokio::time::timeout(
        LWD_CONNECT_TIMEOUT,
        zeaking::lwd::connect_lightwalletd(&lightwalletd_url),
    )
    .await
    {
        Ok(Ok(mut client)) => match zeaking::lwd::chain_tip_height(&mut client).await {
            Ok(tip) => (Some(tip), None),
            Err(e) => (None, Some(e.to_string())),
        },
        Ok(Err(e)) => (None, Some(e.to_string())),
        Err(_) => (
            None,
            Some(format!(
                "lightwalletd connection timed out after {}s (is it running on {}?)",
                LWD_CONNECT_TIMEOUT.as_secs(),
                lightwalletd_url
            )),
        ),
    };

    let compact_db = get_wallet_data_dir().join("lwd_compact.sqlite");
    let compact_db_exists = compact_db.exists();
    let compact_max_height = if compact_db_exists {
        zeaking::lwd::LwdCompactStore::open(&compact_db)
            .ok()
            .and_then(|store| store.max_compact_height().ok().flatten())
    } else {
        None
    };

    SyncStatusSnapshot {
        zebra_tip,
        last_scan_height: config.last_scan_height,
        lightwalletd_url,
        lwd_tip,
        lwd_error,
        compact_max_height,
        compact_db_exists,
    }
}

pub fn print_sync_status(snapshot: &SyncStatusSnapshot) {
    println!("\n🔄 Sync Status:");

    match snapshot.zebra_tip {
        Some(tip) => println!("   Zebra node tip:     {}", tip),
        None => println!("   Zebra node tip:     (unreachable)"),
    }

    match snapshot.last_scan_height {
        Some(last) => {
            println!("   RPC last scanned:   {}", last);
            if let Some(tip) = snapshot.zebra_tip {
                let behind = tip.saturating_sub(last);
                if behind > 0 {
                    println!(
                        "   RPC scan gap:       {} blocks behind node tip — run `nozy sync`",
                        behind
                    );
                } else {
                    println!("   RPC scan gap:       ✅ caught up to node tip");
                }
            }
        }
        None => {
            println!("   RPC last scanned:   never");
            println!("   💡 Run `nozy sync` to scan for Orchard notes via Zebra RPC");
        }
    }

    println!("   Lightwalletd URL:   {}", snapshot.lightwalletd_url);
    match (&snapshot.lwd_tip, &snapshot.lwd_error) {
        (Some(tip), _) => println!("   Lightwalletd tip:   {}", tip),
        (None, Some(err)) => println!("   Lightwalletd tip:   ❌ {}", err),
        (None, None) => println!("   Lightwalletd tip:   (unknown)"),
    }

    match snapshot.compact_max_height {
        Some(h) => {
            println!("   Compact cache high: {}", h);
            if let Some(lwd_tip) = snapshot.lwd_tip {
                if h > lwd_tip {
                    println!(
                        "   Compact cache gap:  ⚠️  {} blocks ABOVE LWD tip (stale cache) — run `nozy lwd prune`",
                        h - lwd_tip
                    );
                } else {
                    let behind = lwd_tip - h;
                    if behind > 0 {
                        println!(
                            "   Compact cache gap:  {} blocks behind LWD tip — run `nozy lwd sync-to-tip`",
                            behind
                        );
                    } else {
                        println!("   Compact cache gap:  ✅ caught up to LWD tip");
                    }
                }
            }
        }
        None if snapshot.compact_db_exists => {
            println!("   Compact cache high: (empty — run `nozy lwd sync-to-tip`)");
            if let Some(lwd_tip) = snapshot.lwd_tip {
                println!(
                    "   Compact cache gap:  {} blocks behind LWD tip — run `nozy lwd sync-to-tip`",
                    lwd_tip
                );
            }
        }
        None => println!("   Compact cache high: (no lwd_compact.sqlite yet)"),
    }

    if let (Some(zebra), Some(lwd)) = (snapshot.zebra_tip, snapshot.lwd_tip) {
        let zebra_u64 = u64::from(zebra);
        let diff = zebra_u64.abs_diff(lwd);
        if diff > 2 {
            println!(
                "   ⚠️  Zebra tip ({}) and LWD tip ({}) differ by {} blocks — check both backends",
                zebra, lwd, diff
            );
        }
    }

    println!(
        "   ℹ️  Sync is node-tip-relative; a catching-up node is OK for scan/compact download."
    );
}
