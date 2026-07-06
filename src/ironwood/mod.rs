//! Ironwood (NU6.3) wallet support — new shielded pool + Orchard → Ironwood migration.
//!
//! See `docs/reference/IRONWOOD_WALLET_READINESS.md` for the full implementation plan.

pub mod migration;
pub mod status;

pub use migration::{
    assess_orchard_migration_readiness, build_schedule_from_plan, execute_orchard_migration,
    execute_orchard_migration_broadcast, execute_orchard_note_split,
    flatten_canonical_denomination_zatoshis, ironwood_migration_schedule_path,
    load_orchard_migration_schedule, next_zip318_anchor_boundary, note_requires_canonical_split,
    plan_orchard_migration, plan_orchard_migration_at, plan_orchard_note_split_outputs,
    presigned_transfer_broadcastable, reconcile_migration_broadcast_confirmations,
    refresh_orchard_migration_schedule_at, save_orchard_migration_plan_at,
    save_orchard_migration_schedule, validate_orchard_migration_schedule, MigrationBroadcastResult,
    MigrationDenomination, MigrationExecutionResult, MigrationPlanSummary,
    MigrationReadinessReport, MigrationReadinessState, MigrationSchedule,
    MigrationScheduleValidation, MigrationScheduledTransfer, MigrationTransfer,
    MigrationTransferStatus, OrchardNoteSplitResult, PreparedMigrationTransaction,
    Zip318ScheduleSummary, IRONWOOD_MIGRATION_SCHEDULE_FILE, MIGRATION_SCHEDULE_VERSION,
    ZIP318_ANCHOR_BUCKET_INTERVAL_BLOCKS, ZIP318_DEFAULT_K_MAX, ZIP318_TRANSFER_EXPIRY_BLOCKS,
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
pub const NU6_3_MAINNET_ACTIVATION_TARGET: &str = "2026-07-21";

pub fn nu6_3_activation_height(testnet: bool) -> Option<u32> {
    let activation = if testnet {
        TEST_NETWORK.activation_height(NetworkUpgrade::Nu6_3)
    } else {
        MAIN_NETWORK.activation_height(NetworkUpgrade::Nu6_3)
    };
    activation.map(u32::from)
}

pub fn is_ironwood_active(height: u32, testnet: bool) -> bool {
    nu6_3_activation_height(testnet).is_some_and(|activation| height >= activation)
}
