use crate::error::TauriError;
use crate::session::load_session_wallet;
use serde::Deserialize;
use tauri::command;

#[derive(Debug, Deserialize)]
pub struct CreateGiftCardRequest {
    pub amount_zec: f64,
    pub label: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CancelGiftCardRequest {
    pub gift_id: String,
}

#[derive(Debug, Deserialize)]
pub struct RedeemGiftCardRequest {
    pub claim_code: String,
    pub password: Option<String>,
}

#[command]
pub async fn create_gift_card(
    request: CreateGiftCardRequest,
) -> Result<serde_json::Value, TauriError> {
    let wallet = load_session_wallet(request.password.as_deref()).await?;
    let zebra_url = nozy::load_config().zebra_url;
    let created =
        nozy::gift_cards::buy_gift_card(&wallet, request.amount_zec, request.label, &zebra_url)
            .await?;
    serde_json::to_value(created).map_err(|e| TauriError::from(e.to_string()))
}

#[command]
pub async fn list_gift_cards(limit: Option<usize>) -> Result<serde_json::Value, TauriError> {
    let cards = nozy::gift_cards::list_gift_cards(limit.unwrap_or(20))?;
    Ok(serde_json::json!({ "gift_cards": cards }))
}

#[command]
pub async fn cancel_gift_card(
    request: CancelGiftCardRequest,
) -> Result<serde_json::Value, TauriError> {
    let card = nozy::gift_cards::cancel_gift_card(&request.gift_id)?;
    serde_json::to_value(card).map_err(|e| TauriError::from(e.to_string()))
}

#[command]
pub async fn redeem_gift_card(
    request: RedeemGiftCardRequest,
) -> Result<serde_json::Value, TauriError> {
    let wallet = load_session_wallet(request.password.as_deref()).await?;
    let zebra_url = nozy::load_config().zebra_url;
    let card = nozy::gift_cards::redeem_gift_card(&wallet, &request.claim_code, &zebra_url).await?;
    serde_json::to_value(card).map_err(|e| TauriError::from(e.to_string()))
}
