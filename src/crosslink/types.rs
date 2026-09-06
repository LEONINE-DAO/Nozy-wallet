//! Typed Crosslink / TFL RPC shapes and Nozy guardian UX models.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// Season 1 feature-net: new Staking Day every 150 PoW blocks, open for 70.
pub const SEASON1_STAKING_CYCLE_BLOCKS: u32 = 150;
pub const SEASON1_STAKING_WINDOW_BLOCKS: u32 = 70;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingDayWindow {
    pub height: u32,
    pub offset: u32,
    pub cycle: u32,
    pub window: u32,
    pub open: bool,
    pub blocks_remaining_in_window: Option<u32>,
    pub blocks_until_next: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BondPosition {
    pub pk: String,
    pub create_height: Option<u32>,
    pub initial_val: u64,
    pub latest_val: u64,
    /// Finalizer this active bond backs (None for withdrawable list).
    pub finalizer: Option<String>,
}

impl BondPosition {
    pub fn rewards_zat(&self) -> u64 {
        self.latest_val.saturating_sub(self.initial_val)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StakingPositions {
    /// Active stakes grouped by finalizer pubkey hex.
    pub active: BTreeMap<String, Vec<BondPosition>>,
    pub withdrawable: Vec<BondPosition>,
}

impl StakingPositions {
    pub fn active_count(&self) -> usize {
        self.active.values().map(|v| v.len()).sum()
    }

    pub fn bonded_zat(&self) -> u64 {
        self.active.values().flatten().map(|p| p.latest_val).sum()
    }

    pub fn withdrawable_zat(&self) -> u64 {
        self.withdrawable.iter().map(|p| p.latest_val).sum()
    }

    pub fn total_rewards_zat(&self) -> u64 {
        self.active
            .values()
            .flatten()
            .map(|p| p.rewards_zat())
            .sum::<u64>()
            + self
                .withdrawable
                .iter()
                .map(|p| p.rewards_zat())
                .sum::<u64>()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalizedTip {
    pub height: Option<u32>,
    pub hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RosterEntry {
    pub finalizer: String,
    pub stake_zat: u64,
    pub share: f64,
}

/// Suggested next step for the operator (Nozy twist vs raw RPC dumps).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NextAction {
    WaitForStakingDay { blocks: u32 },
    WithdrawReady { count: usize },
    UnbondToExit,
    StakeOrGuardian,
    RetargetIfNeeded,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WalletSyncStatus {
    pub sync_height: u32,
    pub tip_height: u32,
    pub user_shielded_spendable_zats: u64,
    pub user_shielded_pending_zats: u64,
    pub user_unshielded_zats: u64,
    pub staked_zats: u64,
    pub withdrawable_zats: u64,
}

impl WalletSyncStatus {
    /// cTAZ the node wallet can use for new stakes (matches monolith GUI spendable).
    pub fn available_to_stake_zats(&self) -> u64 {
        self.user_shielded_spendable_zats
            .saturating_add(self.user_unshielded_zats)
    }

    pub fn wallet_synced(&self) -> bool {
        self.tip_height == 0 || self.sync_height + 2 >= self.tip_height
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardianSnapshot {
    pub rpc_url: String,
    pub height: u32,
    pub staking_day: StakingDayWindow,
    pub tfl_activated: Option<bool>,
    pub finalized_tip: Option<FinalizedTip>,
    pub positions: StakingPositions,
    pub finalizer_count: Option<usize>,
    pub pos_height: Option<u32>,
    pub next_action: NextAction,
    pub privacy_notes: Vec<String>,
    /// Node wallet spendable/staked totals (`get_wallet_sync_status` when supported).
    pub wallet: Option<WalletSyncStatus>,
}

impl GuardianSnapshot {
    pub fn compute_next_action(positions: &StakingPositions, day: &StakingDayWindow) -> NextAction {
        if !positions.withdrawable.is_empty() {
            if day.open {
                return NextAction::WithdrawReady {
                    count: positions.withdrawable.len(),
                };
            }
            return NextAction::WaitForStakingDay {
                blocks: day.blocks_until_next.unwrap_or(0),
            };
        }
        if !day.open {
            if positions.active_count() > 0 {
                return NextAction::RetargetIfNeeded;
            }
            return NextAction::WaitForStakingDay {
                blocks: day.blocks_until_next.unwrap_or(0),
            };
        }
        if positions.active_count() > 0 {
            NextAction::UnbondToExit
        } else {
            NextAction::StakeOrGuardian
        }
    }

    pub fn default_privacy_notes() -> Vec<String> {
        vec![]
    }
}

/// Parameter for `wallet_staking_action` (exactly one variant).
#[derive(Debug, Clone, Serialize)]
pub enum StakingAction {
    CreateNewDelegationBond {
        amount_zats: u64,
        target_finalizer: String,
    },
    RetargetDelegationBond {
        bond_key: String,
        target_finalizer: String,
    },
    BeginDelegationUnbonding {
        bond_key: String,
    },
    WithdrawDelegationBond {
        bond_key: String,
    },
}

impl StakingAction {
    pub fn to_rpc_param(&self) -> Value {
        match self {
            Self::CreateNewDelegationBond {
                amount_zats,
                target_finalizer,
            } => serde_json::json!({
                "CreateNewDelegationBond": {
                    "amount_zats": amount_zats,
                    "target_finalizer": target_finalizer,
                }
            }),
            Self::RetargetDelegationBond {
                bond_key,
                target_finalizer,
            } => serde_json::json!({
                "RetargetDelegationBond": {
                    "bond_key": bond_key,
                    "target_finalizer": target_finalizer,
                }
            }),
            Self::BeginDelegationUnbonding { bond_key } => serde_json::json!({
                "BeginDelegationUnbonding": {
                    "bond_key": bond_key,
                }
            }),
            Self::WithdrawDelegationBond { bond_key } => serde_json::json!({
                "WithdrawDelegationBond": {
                    "bond_key": bond_key,
                }
            }),
        }
    }

    pub fn requires_staking_day(&self) -> bool {
        !matches!(self, Self::RetargetDelegationBond { .. })
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::CreateNewDelegationBond { .. } => "CreateNewDelegationBond",
            Self::RetargetDelegationBond { .. } => "RetargetDelegationBond",
            Self::BeginDelegationUnbonding { .. } => "BeginDelegationUnbonding",
            Self::WithdrawDelegationBond { .. } => "WithdrawDelegationBond",
        }
    }
}

/// Loose serde for `wallet_staking_positions` (shapes vary slightly across builds).
#[derive(Debug, Deserialize)]
pub(crate) struct RawStakingPositions {
    #[serde(default)]
    pub active: BTreeMap<String, Vec<RawBond>>,
    #[serde(default)]
    pub withdrawable: Vec<RawBond>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawBond {
    pub pk: Option<String>,
    pub bond_key: Option<String>,
    pub create_height: Option<u32>,
    pub initial_val: Option<u64>,
    pub latest_val: Option<u64>,
    pub value: Option<u64>,
}

impl RawBond {
    pub fn into_position(self, finalizer: Option<String>) -> Option<BondPosition> {
        let pk = self.pk.or(self.bond_key)?;
        let latest = self.latest_val.or(self.value).unwrap_or(0);
        let initial = self.initial_val.unwrap_or(latest);
        Some(BondPosition {
            pk,
            create_height: self.create_height,
            initial_val: initial,
            latest_val: latest,
            finalizer,
        })
    }
}

impl From<RawStakingPositions> for StakingPositions {
    fn from(raw: RawStakingPositions) -> Self {
        let mut active = BTreeMap::new();
        for (finalizer, bonds) in raw.active {
            let list: Vec<_> = bonds
                .into_iter()
                .filter_map(|b| b.into_position(Some(finalizer.clone())))
                .collect();
            if !list.is_empty() {
                active.insert(finalizer, list);
            }
        }
        let withdrawable = raw
            .withdrawable
            .into_iter()
            .filter_map(|b| b.into_position(None))
            .collect();
        Self {
            active,
            withdrawable,
        }
    }
}
