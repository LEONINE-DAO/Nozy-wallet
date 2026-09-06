//! Crosslink Protocol Guardian HTTP API (Season 1 feature-net).
//! Thin wrappers around `nozy::crosslink` — same path as CLI.

use axum::{extract::Query, http::StatusCode, response::Json as ResponseJson, Json};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::handlers::error_response_with_code;
use nozy::crosslink::{
    block_finality, bond_info, build_crosslink_client, ctaz_to_zat, fetch_guardian_snapshot,
    finality_tip, normalize_finalizer_hex, roster, staking_action, staking_day_at,
    staking_positions, tx_finality, wallet_sync_status, wallet_ufvk, GuardianSnapshot,
    StakingAction, StakingPositions, WalletSyncStatus,
};
use nozy::load_config;

type ApiError = (StatusCode, ResponseJson<Value>);

fn map_err(e: impl std::fmt::Display) -> ApiError {
    error_response_with_code(StatusCode::BAD_GATEWAY, format!("{e}"), "CROSSLINK_ERROR")
}

fn map_user_err(e: impl std::fmt::Display) -> ApiError {
    error_response_with_code(StatusCode::BAD_REQUEST, format!("{e}"), "CROSSLINK_INVALID")
}

async fn client() -> Result<nozy::ZebraClient, ApiError> {
    let config = load_config();
    build_crosslink_client(&config).map_err(map_user_err)
}

async fn ensure_staking_day(force: bool) -> Result<(), ApiError> {
    let client = client().await?;
    let height = client.get_block_count().await.map_err(map_err)?;
    let day = staking_day_at(height);
    if day.open || force {
        return Ok(());
    }
    Err(map_user_err(format!(
        "Staking Day is closed ({} blocks until next). Retarget anytime, or pass force=true.",
        day.blocks_until_next.unwrap_or(0)
    )))
}

pub async fn crosslink_status() -> Result<ResponseJson<GuardianSnapshot>, ApiError> {
    let client = client().await?;
    let snap = fetch_guardian_snapshot(&client).await.map_err(map_err)?;
    Ok(ResponseJson(snap))
}

pub async fn crosslink_positions() -> Result<ResponseJson<StakingPositions>, ApiError> {
    let client = client().await?;
    let positions = staking_positions(&client).await.map_err(map_err)?;
    Ok(ResponseJson(positions))
}

#[derive(Debug, Deserialize)]
pub struct RosterQuery {
    #[serde(default)]
    pub zats: bool,
}

pub async fn crosslink_roster(
    Query(q): Query<RosterQuery>,
) -> Result<ResponseJson<Value>, ApiError> {
    let client = client().await?;
    let entries = roster(&client, q.zats).await.map_err(map_err)?;
    Ok(ResponseJson(
        serde_json::to_value(entries).map_err(map_err)?,
    ))
}

#[derive(Debug, Deserialize)]
pub struct FinalityQuery {
    pub block: Option<String>,
    pub tx: Option<String>,
}

pub async fn crosslink_finality(
    Query(q): Query<FinalityQuery>,
) -> Result<ResponseJson<Value>, ApiError> {
    let client = client().await?;
    let tip = finality_tip(&client).await.map_err(map_err)?;
    let detail = if let Some(ref h) = q.block {
        Some(block_finality(&client, h).await.map_err(map_err)?)
    } else if let Some(ref t) = q.tx {
        Some(tx_finality(&client, t).await.map_err(map_err)?)
    } else {
        None
    };
    Ok(ResponseJson(serde_json::json!({
        "tip": tip,
        "detail": detail,
    })))
}

#[derive(Debug, Deserialize)]
pub struct BondQuery {
    pub key: String,
}

pub async fn crosslink_bond(Query(q): Query<BondQuery>) -> Result<ResponseJson<Value>, ApiError> {
    let client = client().await?;
    let info = bond_info(&client, &q.key).await.map_err(map_err)?;
    Ok(ResponseJson(info))
}

#[derive(Debug, Deserialize)]
pub struct StakeBody {
    pub amount_ctaz: f64,
    pub finalizer: String,
    #[serde(default)]
    pub force: bool,
}

#[derive(Debug, Serialize)]
pub struct ActionResponse {
    pub action: String,
    pub result: Value,
}

pub async fn crosslink_stake(
    Json(body): Json<StakeBody>,
) -> Result<ResponseJson<ActionResponse>, ApiError> {
    ensure_staking_day(body.force).await?;
    let amount_zats = ctaz_to_zat(body.amount_ctaz).map_err(map_user_err)?;
    let target = normalize_finalizer_hex(&body.finalizer).map_err(map_user_err)?;
    let action = StakingAction::CreateNewDelegationBond {
        amount_zats,
        target_finalizer: target,
    };
    let client = client().await?;
    let result = staking_action(&client, &action).await.map_err(map_err)?;
    Ok(ResponseJson(ActionResponse {
        action: action.label().to_string(),
        result,
    }))
}

#[derive(Debug, Deserialize)]
pub struct RetargetBody {
    pub bond: String,
    pub finalizer: String,
}

pub async fn crosslink_retarget(
    Json(body): Json<RetargetBody>,
) -> Result<ResponseJson<ActionResponse>, ApiError> {
    let target = normalize_finalizer_hex(&body.finalizer).map_err(map_user_err)?;
    let action = StakingAction::RetargetDelegationBond {
        bond_key: body.bond,
        target_finalizer: target,
    };
    let client = client().await?;
    let result = staking_action(&client, &action).await.map_err(map_err)?;
    Ok(ResponseJson(ActionResponse {
        action: action.label().to_string(),
        result,
    }))
}

#[derive(Debug, Deserialize)]
pub struct BondActionBody {
    pub bond: String,
    #[serde(default)]
    pub force: bool,
}

pub async fn crosslink_unbond(
    Json(body): Json<BondActionBody>,
) -> Result<ResponseJson<ActionResponse>, ApiError> {
    ensure_staking_day(body.force).await?;
    let action = StakingAction::BeginDelegationUnbonding {
        bond_key: body.bond,
    };
    let client = client().await?;
    let result = staking_action(&client, &action).await.map_err(map_err)?;
    Ok(ResponseJson(ActionResponse {
        action: action.label().to_string(),
        result,
    }))
}

pub async fn crosslink_withdraw(
    Json(body): Json<BondActionBody>,
) -> Result<ResponseJson<ActionResponse>, ApiError> {
    ensure_staking_day(body.force).await?;
    let action = StakingAction::WithdrawDelegationBond {
        bond_key: body.bond,
    };
    let client = client().await?;
    let result = staking_action(&client, &action).await.map_err(map_err)?;
    Ok(ResponseJson(ActionResponse {
        action: action.label().to_string(),
        result,
    }))
}

#[derive(Debug, Serialize)]
pub struct WalletUfvkResponse {
    pub ufvk: String,
}

pub async fn crosslink_wallet_ufvk() -> Result<ResponseJson<WalletUfvkResponse>, ApiError> {
    let client = client().await?;
    let ufvk = wallet_ufvk(&client).await.map_err(map_err)?;
    Ok(ResponseJson(WalletUfvkResponse { ufvk }))
}

pub async fn crosslink_wallet_status() -> Result<ResponseJson<WalletSyncStatus>, ApiError> {
    let client = client().await?;
    match wallet_sync_status(&client).await {
        Some(w) => Ok(ResponseJson(w)),
        None => Err(error_response_with_code(
            StatusCode::NOT_IMPLEMENTED,
            "Node does not expose get_wallet_sync_status — see docs/CROSSLINK_WALLET_RPC_UPGRADE.md",
            "CROSSLINK_WALLET_RPC_UNAVAILABLE",
        )),
    }
}
