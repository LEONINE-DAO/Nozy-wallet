//! Native merchant invoice HTTP handlers (Phase 3b).

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::Json as ResponseJson,
    Json,
};
use serde::Deserialize;

use crate::handlers::{error_response, load_wallet_with_password};

#[derive(Debug, Deserialize)]
pub struct CreateInvoiceBody {
    pub amount_zec: f64,
    #[serde(default)]
    pub amount_fiat: Option<f64>,
    #[serde(default)]
    pub fiat_currency: Option<String>,
    #[serde(default)]
    pub product_name: Option<String>,
    #[serde(default)]
    pub memo: Option<String>,
    #[serde(default)]
    pub ttl_minutes: Option<i64>,
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

/// POST `/api/business/invoices`
pub async fn create_invoice(
    Json(body): Json<CreateInvoiceBody>,
) -> Result<ResponseJson<serde_json::Value>, (StatusCode, ResponseJson<serde_json::Value>)> {
    let config = nozy::load_config();
    if config.active_role != nozy::WalletRole::Business {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            "Switch to Business profile before creating invoices.",
        ));
    }

    let (wallet, _storage) = load_wallet_with_password(body.password)
        .await
        .map_err(|e| error_response(StatusCode::UNAUTHORIZED, e))?;

    let diversifier = nozy::merchant_invoices::peek_next_diversifier().map_err(|e| {
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Invoice store error: {e}"),
        )
    })?;

    let net = if config.network == "testnet" {
        zcash_protocol::consensus::NetworkType::Test
    } else {
        zcash_protocol::consensus::NetworkType::Main
    };
    let payment_address = wallet
        .generate_orchard_address(1, diversifier, net)
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let invoice =
        nozy::merchant_invoices::create_invoice(nozy::merchant_invoices::CreateInvoiceParams {
            amount_zec: body.amount_zec,
            amount_fiat: body.amount_fiat,
            fiat_currency: body.fiat_currency,
            product_name: body.product_name,
            memo: body.memo,
            ttl_minutes: body.ttl_minutes,
            payment_address,
            diversifier_index: diversifier,
        })
        .map_err(|e| error_response(StatusCode::BAD_REQUEST, e.to_string()))?;

    Ok(ResponseJson(
        serde_json::to_value(invoice).unwrap_or_default(),
    ))
}

/// GET `/api/business/invoices`
pub async fn list_invoices(
    Query(q): Query<ListQuery>,
) -> Result<ResponseJson<serde_json::Value>, (StatusCode, ResponseJson<serde_json::Value>)> {
    let items = nozy::merchant_invoices::list_invoices(q.limit).map_err(|e| {
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Invoice store error: {e}"),
        )
    })?;
    Ok(ResponseJson(serde_json::json!({ "invoices": items })))
}

/// GET `/api/business/invoices/{id}`
pub async fn get_invoice(
    Path(id): Path<String>,
) -> Result<ResponseJson<serde_json::Value>, (StatusCode, ResponseJson<serde_json::Value>)> {
    let inv = nozy::merchant_invoices::get_invoice(&id)
        .map_err(|e| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Invoice store error: {e}"),
            )
        })?
        .ok_or_else(|| error_response(StatusCode::NOT_FOUND, format!("Invoice {id} not found")))?;
    Ok(ResponseJson(serde_json::to_value(inv).unwrap_or_default()))
}

/// GET `/api/business/invoices/{id}/qr` — returns ZIP-321 URI payload for QR encoders.
pub async fn get_invoice_qr(
    Path(id): Path<String>,
) -> Result<ResponseJson<serde_json::Value>, (StatusCode, ResponseJson<serde_json::Value>)> {
    let inv = nozy::merchant_invoices::get_invoice(&id)
        .map_err(|e| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Invoice store error: {e}"),
            )
        })?
        .ok_or_else(|| error_response(StatusCode::NOT_FOUND, format!("Invoice {id} not found")))?;
    Ok(ResponseJson(serde_json::json!({
        "invoice_id": inv.invoice_id,
        "zcash_uri": inv.zcash_uri,
        "payment_address": inv.payment_address,
        "amount_zec": inv.amount_zec,
        "status": inv.status,
    })))
}

/// POST `/api/business/invoices/{id}/cancel`
pub async fn cancel_invoice(
    Path(id): Path<String>,
) -> Result<ResponseJson<serde_json::Value>, (StatusCode, ResponseJson<serde_json::Value>)> {
    let inv = nozy::merchant_invoices::cancel_invoice(&id)
        .map_err(|e| error_response(StatusCode::BAD_REQUEST, e.to_string()))?;
    Ok(ResponseJson(serde_json::to_value(inv).unwrap_or_default()))
}
