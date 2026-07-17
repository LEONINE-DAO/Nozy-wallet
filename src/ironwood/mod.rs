//! Ironwood (NU6.3) wallet support — new shielded pool + Orchard → Ironwood migration.
//!
//! See `docs/reference/IRONWOOD_WALLET_READINESS.md` for the full implementation plan.

pub mod migration;
pub mod network_privacy;
pub mod status;

pub use migration::{
    assess_orchard_migration_readiness, build_schedule_from_plan, execute_orchard_migration,
    execute_orchard_migration_broadcast, execute_orchard_note_split,
    flatten_canonical_denomination_zatoshis, ironwood_migration_schedule_path,
    load_orchard_migration_schedule, next_zip318_anchor_boundary, note_requires_canonical_split,
    plan_orchard_migration, plan_orchard_migration_at, plan_orchard_note_split_outputs,
    presigned_transfer_broadcastable, previous_zip318_anchor_boundary,
    reconcile_migration_broadcast_confirmations, refresh_orchard_migration_schedule_at,
    save_orchard_migration_plan_at, save_orchard_migration_schedule, select_zooko_round_amount,
    validate_orchard_migration_schedule, MigrationBroadcastResult, MigrationDenomination,
    MigrationExecutionResult, MigrationPlanSummary, MigrationReadinessReport,
    MigrationReadinessState, MigrationSchedule, MigrationScheduleValidation,
    MigrationScheduledTransfer, MigrationTransfer, MigrationTransferStatus, OrchardNoteSplitResult,
    PreparedMigrationTransaction, Zip318ScheduleSummary, IRONWOOD_MIGRATION_SCHEDULE_FILE,
    MIGRATION_SCHEDULE_VERSION, ZIP318_ANCHOR_BUCKET_INTERVAL_BLOCKS, ZIP318_DEFAULT_K_MAX,
    ZIP318_TRANSFER_EXPIRY_BLOCKS, ZOOKO_RESIDUAL_ABANDON_ZAT,
};
pub use network_privacy::{
    amount_timing_status, assess_migration_cover_traffic, assess_migration_network_privacy,
    require_migration_network_privacy, safer_migration_status_snapshot,
    selected_amount_timing_algorithm, AmountTimingAlgorithm, AmountTimingStatus,
    MigrationCoverAssessment, MigrationNetworkPrivacyAssessment, MigrationNetworkPrivacyMode,
    MigrationNetworkPrivacyOpts, SaferMigrationStatusSnapshot,
};
pub use status::{
    display_ironwood_status, fetch_pool_balances, ironwood_software_send_available,
    legacy_hardware_send_blocker, orchard_only_send_blocker, IronwoodWalletStatus,
    PoolBalanceSummary, ORCHARD_ONLY_SENDS_DISABLED_AFTER_IRONWOOD,
};
use zcash_protocol::consensus::{NetworkUpgrade, Parameters, MAIN_NETWORK, TEST_NETWORK};

/// Target from the ZF/ZODL miner timeline circulated July 2026.
pub const NU6_3_TESTNET_DEPLOYMENT_TARGET: &str = "2026-07-01";
pub const NU6_3_TESTNET_ACTIVATION_TARGET: &str = "2026-07-03";
pub const NU6_3_MAINNET_DEPLOYMENT_TARGET: &str = "2026-07-09";
pub const ZCASHD_END_OF_SUPPORT_TARGET: &str = "2026-07-15";
/// Mainnet Ironwood / NU6.3 activation calendar date (ecosystem PSA: 2026-07-28).
pub const NU6_3_MAINNET_ACTIVATION_TARGET: &str = "2026-07-28";
/// Mainnet Ironwood / NU6.3 activation height (ecosystem PSA).
/// Used when `zcash_protocol` has not yet pinned `NetworkUpgrade::Nu6_3`.
pub const NU6_3_MAINNET_ACTIVATION_HEIGHT: u32 = 3_428_143;

/// Pre-activation / freeze notice for wallet UIs (PSA ask #1).
pub const IRONWOOD_ACTIVATION_FREEZE_NOTICE: &str = "Ironwood activates at mainnet block 3,428,143 \
(target 2026-07-28). At that height, Orchard spends are frozen except Orchard→Ironwood migration. \
Unmigrated Orchard funds may be temporarily unavailable for normal sends until migrated.";

/// Factual migration risk warnings (PSA ask #2 + Shielded Labs threat model).
pub const IRONWOOD_MIGRATION_PRIVACY_WARNINGS: &[&str] = &[
    "Turnstile migration reveals the migrated amount on the public blockchain.",
    "Broadcasting without network-level privacy (local node, Nym, or Tor) can link that amount to your IP or lightwalletd session.",
    "Naive one-shot migration of your full Orchard balance is a privacy risk; Nozy uses bucketed {1,2,5}×10^k turnstiles and requires a privacy path before Broadcast.",
];

pub fn nu6_3_activation_height(testnet: bool) -> Option<u32> {
    let from_protocol = if testnet {
        TEST_NETWORK.activation_height(NetworkUpgrade::Nu6_3)
    } else {
        MAIN_NETWORK.activation_height(NetworkUpgrade::Nu6_3)
    };
    match from_protocol.map(u32::from) {
        Some(height) => Some(height),
        // Protocol crate may lag ecosystem ratification; keep mainnet wallets on the PSA height.
        None if !testnet => Some(NU6_3_MAINNET_ACTIVATION_HEIGHT),
        None => None,
    }
}

pub fn is_ironwood_active(height: u32, testnet: bool) -> bool {
    nu6_3_activation_height(testnet).is_some_and(|activation| height >= activation)
}

/// User-facing activation + privacy copy for status panels.
#[derive(Debug, Clone, serde::Serialize)]
pub struct IronwoodUserNotices {
    pub activation_notice: String,
    pub migration_privacy_warnings: Vec<String>,
    pub orchard_funds_at_risk: bool,
}

pub fn ironwood_user_notices(
    ironwood_active: bool,
    orchard_wallet_zat: u64,
) -> IronwoodUserNotices {
    let orchard_funds_at_risk = orchard_wallet_zat > 0;
    let activation_notice = if ironwood_active && orchard_funds_at_risk {
        "Ironwood is active. Normal Orchard sends are frozen. Migrate remaining Orchard funds \
         to Ironwood to restore spendability — turnstile amounts are public and clearnet broadcast \
         can link them to your IP."
            .to_string()
    } else if !ironwood_active && orchard_funds_at_risk {
        IRONWOOD_ACTIVATION_FREEZE_NOTICE.to_string()
    } else if ironwood_active {
        "Ironwood is active. New shielded sends use the Ironwood pool."
            .to_string()
    } else {
        IRONWOOD_ACTIVATION_FREEZE_NOTICE.to_string()
    };

    IronwoodUserNotices {
        activation_notice,
        migration_privacy_warnings: IRONWOOD_MIGRATION_PRIVACY_WARNINGS
            .iter()
            .map(|s| (*s).to_string())
            .collect(),
        orchard_funds_at_risk,
    }
}
