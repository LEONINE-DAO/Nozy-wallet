//! Gift card HTTP handlers. Keys stay in the local wallet; redeem pays out from it.

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::Json as ResponseJson,
    Json,
};
use serde::Deserialize;

use crate::handlers::{error_response, load_wallet_with_password};

#[derive(Debug, Deserialize)]
pub struct CreateGiftCardBody {
    pub amount_zec: f64,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RedeemGiftCardBody {
    pub claim_code: String,
    #[serde(default)]
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    50
}

/// POST `/api/gift-cards`
pub async fn create_gift_card(
    Json(body): Json<CreateGiftCardBody>,
) -> Result<ResponseJson<serde_json::Value>, (StatusCode, ResponseJson<serde_json::Value>)> {
    let (wallet, _storage) = load_wallet_with_password(body.password)
        .await
        .map_err(|e| error_response(StatusCode::UNAUTHORIZED, e))?;

    let config = nozy::load_config();
    let created =
        nozy::gift_cards::buy_gift_card(&wallet, body.amount_zec, body.label, &config.zebra_url)
            .await
            .map_err(|e| error_response(StatusCode::BAD_REQUEST, e.to_string()))?;

    Ok(ResponseJson(
        serde_json::to_value(created).unwrap_or_default(),
    ))
}

/// GET `/api/gift-cards`
pub async fn list_gift_cards(
    Query(q): Query<ListQuery>,
) -> Result<ResponseJson<serde_json::Value>, (StatusCode, ResponseJson<serde_json::Value>)> {
    let cards = nozy::gift_cards::list_gift_cards(q.limit).map_err(|e| {
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Gift card store error: {e}"),
        )
    })?;
    Ok(ResponseJson(serde_json::json!({ "gift_cards": cards })))
}

/// GET `/api/gift-cards/{id}`
pub async fn get_gift_card(
    Path(id): Path<String>,
) -> Result<ResponseJson<serde_json::Value>, (StatusCode, ResponseJson<serde_json::Value>)> {
    let card = nozy::gift_cards::get_gift_card(&id)
        .map_err(|e| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Gift card store error: {e}"),
            )
        })?
        .ok_or_else(|| {
            error_response(StatusCode::NOT_FOUND, format!("Gift card {id} not found"))
        })?;
    Ok(ResponseJson(serde_json::to_value(card).unwrap_or_default()))
}

/// POST `/api/gift-cards/{id}/cancel`
pub async fn cancel_gift_card(
    Path(id): Path<String>,
) -> Result<ResponseJson<serde_json::Value>, (StatusCode, ResponseJson<serde_json::Value>)> {
    let card = nozy::gift_cards::cancel_gift_card(&id)
        .map_err(|e| error_response(StatusCode::BAD_REQUEST, e.to_string()))?;
    Ok(ResponseJson(serde_json::to_value(card).unwrap_or_default()))
}

/// POST `/api/gift-cards/redeem`
pub async fn redeem_gift_card(
    Json(body): Json<RedeemGiftCardBody>,
) -> Result<ResponseJson<serde_json::Value>, (StatusCode, ResponseJson<serde_json::Value>)> {
    let (wallet, _storage) = load_wallet_with_password(body.password)
        .await
        .map_err(|e| error_response(StatusCode::UNAUTHORIZED, e))?;
    let config = nozy::load_config();
    let card = nozy::gift_cards::redeem_gift_card(&wallet, &body.claim_code, &config.zebra_url)
        .await
        .map_err(|e| error_response(StatusCode::BAD_REQUEST, e.to_string()))?;
    Ok(ResponseJson(serde_json::to_value(card).unwrap_or_default()))
}
