//! Desktop Crosslink Protocol Guardian (Season 1 feature-net).
//! Thin wrappers around `nozy::crosslink` — same path as CLI / api-server.

use crate::error::TauriError;
use nozy::crosslink::{
    block_finality, bond_info, build_crosslink_client, ctaz_to_zat, fetch_guardian_snapshot,
    finality_tip, normalize_finalizer_hex, roster, staking_action, staking_day_at,
    staking_positions, tx_finality, wallet_ufvk, GuardianSnapshot, StakingAction, StakingPositions,
};
use nozy::load_config;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::command;

fn client() -> Result<nozy::ZebraClient, TauriError> {
    let config = load_config();
    build_crosslink_client(&config).map_err(TauriError::from)
}

fn is_stake_busy_error(err: &impl std::fmt::Display) -> bool {
    err.to_string()
        .to_ascii_lowercase()
        .contains("another stake in progress")
}

/// Crosslink monolith often returns "Another stake in progress" while the bond is still
/// confirming — and sometimes after it already landed. Reconcile by watching for a *new* bond.
async fn reconcile_busy_stake(
    client: &nozy::ZebraClient,
    amount_zats: u64,
    _target_finalizer: &str,
    tip_before: u32,
    before_pks: &std::collections::HashSet<String>,
) -> Result<Option<Value>, TauriError> {
    // Quick check (~5s) for already-confirmed bond; otherwise treat busy as uncertain.
    for attempt in 1..=3u32 {
        tokio::time::sleep(std::time::Duration::from_millis(1_200 * u64::from(attempt))).await;
        let positions = staking_positions(client).await.map_err(TauriError::from)?;
        for (fin, bonds) in &positions.active {
            for bond in bonds {
                let key = format!("{fin}:{}", bond.pk);
                if before_pks.contains(&key) {
                    continue;
                }
                let amount_match = bond.initial_val == amount_zats;
                let height_ok = bond
                    .create_height
                    .map(|h| h + 5 >= tip_before)
                    .unwrap_or(true);
                if amount_match && height_ok {
                    return Ok(Some(serde_json::json!({
                        "reconciled": true,
                        "message": "Stake landed on chain (node busy reply was a false alarm)",
                        "bond_pk": bond.pk,
                        "create_height": bond.create_height,
                        "amount_zats": amount_zats,
                        "finalizer": fin,
                    })));
                }
            }
        }
    }
    Ok(None)
}

/// Reject a second identical stake while the first is still confirming (or just landed).
fn recent_duplicate_bond(
    positions: &StakingPositions,
    amount_zats: u64,
    target_finalizer: &str,
    tip: u32,
) -> Option<Value> {
    const LOOKBACK_BLOCKS: u32 = 40;
    let target = target_finalizer.to_ascii_lowercase();
    for (fin, bonds) in &positions.active {
        if !fin.eq_ignore_ascii_case(&target) {
            continue;
        }
        for bond in bonds {
            if bond.initial_val != amount_zats {
                continue;
            }
            let Some(h) = bond.create_height else {
                continue;
            };
            if tip.saturating_sub(h) <= LOOKBACK_BLOCKS {
                return Some(serde_json::json!({
                    "duplicate_recent": true,
                    "message": "A matching bond to this finalizer was created recently — skipped to avoid a double stake. Check Your bonds.",
                    "bond_pk": bond.pk,
                    "create_height": h,
                    "amount_zats": amount_zats,
                    "finalizer": fin,
                }));
            }
        }
    }
    None
}

async fn ensure_staking_day(force: bool) -> Result<(), TauriError> {
    let client = client()?;
    let height = client.get_block_count().await.map_err(TauriError::from)?;
    let day = staking_day_at(height);
    if day.open || force {
        return Ok(());
    }
    Err(TauriError {
        message: format!(
            "Staking Day is closed ({} blocks until next). Retarget anytime, or pass force.",
            day.blocks_until_next.unwrap_or(0)
        ),
        code: Some("CROSSLINK_STAKING_DAY_CLOSED".into()),
    })
}

#[command]
pub async fn crosslink_status() -> Result<GuardianSnapshot, TauriError> {
    let client = client()?;
    fetch_guardian_snapshot(&client)
        .await
        .map_err(TauriError::from)
}

#[command]
pub async fn crosslink_positions() -> Result<StakingPositions, TauriError> {
    let client = client()?;
    staking_positions(&client).await.map_err(TauriError::from)
}

#[derive(Debug, Deserialize)]
pub struct CrosslinkRosterRequest {
    #[serde(default)]
    pub zats: bool,
}

#[command]
pub async fn crosslink_roster(request: CrosslinkRosterRequest) -> Result<Value, TauriError> {
    let client = client()?;
    let entries = roster(&client, request.zats)
        .await
        .map_err(TauriError::from)?;
    serde_json::to_value(entries).map_err(|e| TauriError::from(e.to_string()))
}

#[derive(Debug, Deserialize, Default)]
pub struct CrosslinkFinalityRequest {
    pub block: Option<String>,
    pub tx: Option<String>,
}

#[command]
pub async fn crosslink_finality(request: CrosslinkFinalityRequest) -> Result<Value, TauriError> {
    let client = client()?;
    let tip = finality_tip(&client).await.map_err(TauriError::from)?;
    let detail = if let Some(ref h) = request.block {
        Some(block_finality(&client, h).await.map_err(TauriError::from)?)
    } else if let Some(ref t) = request.tx {
        Some(tx_finality(&client, t).await.map_err(TauriError::from)?)
    } else {
        None
    };
    Ok(serde_json::json!({ "tip": tip, "detail": detail }))
}

#[derive(Debug, Deserialize)]
pub struct CrosslinkBondRequest {
    pub key: String,
}

#[command]
pub async fn crosslink_bond(request: CrosslinkBondRequest) -> Result<Value, TauriError> {
    let client = client()?;
    bond_info(&client, &request.key)
        .await
        .map_err(TauriError::from)
}

#[derive(Debug, Serialize)]
pub struct CrosslinkActionResponse {
    pub action: String,
    pub result: Value,
}

#[derive(Debug, Deserialize)]
pub struct CrosslinkStakeRequest {
    pub amount_ctaz: f64,
    pub finalizer: String,
    #[serde(default)]
    pub force: bool,
}

#[command]
pub async fn crosslink_stake(
    request: CrosslinkStakeRequest,
) -> Result<CrosslinkActionResponse, TauriError> {
    ensure_staking_day(request.force).await?;
    let amount_zats = ctaz_to_zat(request.amount_ctaz).map_err(TauriError::from)?;
    let target = normalize_finalizer_hex(&request.finalizer).map_err(TauriError::from)?;
    let action = StakingAction::CreateNewDelegationBond {
        amount_zats,
        target_finalizer: target.clone(),
    };
    let client = client()?;
    let tip_before = client.get_block_count().await.map_err(TauriError::from)?;
    let positions_before = staking_positions(&client).await.map_err(TauriError::from)?;
    if let Some(dup) = recent_duplicate_bond(&positions_before, amount_zats, &target, tip_before) {
        return Ok(CrosslinkActionResponse {
            action: action.label().to_string(),
            result: dup,
        });
    }
    let mut before_pks = std::collections::HashSet::new();
    for (fin, bonds) in &positions_before.active {
        for b in bonds {
            before_pks.insert(format!("{fin}:{}", b.pk));
        }
    }
    let result = match staking_action(&client, &action).await {
        Ok(r) => r,
        Err(e) => {
            if is_stake_busy_error(&e) {
                if let Some(reconciled) =
                    reconcile_busy_stake(&client, amount_zats, &target, tip_before, &before_pks)
                        .await?
                {
                    return Ok(CrosslinkActionResponse {
                        action: action.label().to_string(),
                        result: reconciled,
                    });
                }
                // Busy with no new bond: do NOT claim the stake was submitted — another
                // in-flight stake (often GUI) may have blocked ours entirely.
                return Ok(CrosslinkActionResponse {
                    action: action.label().to_string(),
                    result: serde_json::json!({
                        "busy": true,
                        "submitted_uncertain": true,
                        "message": "Crosslink node is still finishing a stake (its one-at-a-time lock). That can happen after one click — not only when you stake twice. Check the GUI pending list / Your bonds; wait until it clears, then try again if no new bond appeared.",
                        "amount_zats": amount_zats,
                        "finalizer": target,
                    }),
                });
            }
            return Err(TauriError::from(e));
        }
    };
    Ok(CrosslinkActionResponse {
        action: action.label().to_string(),
        result,
    })
}

#[derive(Debug, Deserialize)]
pub struct CrosslinkRetargetRequest {
    pub bond: String,
    pub finalizer: String,
}

#[command]
pub async fn crosslink_retarget(
    request: CrosslinkRetargetRequest,
) -> Result<CrosslinkActionResponse, TauriError> {
    let target = normalize_finalizer_hex(&request.finalizer).map_err(TauriError::from)?;
    let action = StakingAction::RetargetDelegationBond {
        bond_key: request.bond,
        target_finalizer: target,
    };
    let client = client()?;
    let result = staking_action(&client, &action)
        .await
        .map_err(TauriError::from)?;
    Ok(CrosslinkActionResponse {
        action: action.label().to_string(),
        result,
    })
}

#[derive(Debug, Deserialize)]
pub struct CrosslinkBondActionRequest {
    pub bond: String,
    #[serde(default)]
    pub force: bool,
}

#[command]
pub async fn crosslink_unbond(
    request: CrosslinkBondActionRequest,
) -> Result<CrosslinkActionResponse, TauriError> {
    ensure_staking_day(request.force).await?;
    let action = StakingAction::BeginDelegationUnbonding {
        bond_key: request.bond,
    };
    let client = client()?;
    let result = staking_action(&client, &action)
        .await
        .map_err(TauriError::from)?;
    Ok(CrosslinkActionResponse {
        action: action.label().to_string(),
        result,
    })
}

#[command]
pub async fn crosslink_withdraw(
    request: CrosslinkBondActionRequest,
) -> Result<CrosslinkActionResponse, TauriError> {
    ensure_staking_day(request.force).await?;
    let action = StakingAction::WithdrawDelegationBond {
        bond_key: request.bond,
    };
    let client = client()?;
    let result = staking_action(&client, &action)
        .await
        .map_err(TauriError::from)?;
    Ok(CrosslinkActionResponse {
        action: action.label().to_string(),
        result,
    })
}

#[derive(Debug, Serialize)]
pub struct CrosslinkWalletUfvkResponse {
    pub ufvk: String,
}

#[command]
pub async fn crosslink_wallet_ufvk() -> Result<CrosslinkWalletUfvkResponse, TauriError> {
    let client = client()?;
    let ufvk = wallet_ufvk(&client).await.map_err(TauriError::from)?;
    Ok(CrosslinkWalletUfvkResponse { ufvk })
}
