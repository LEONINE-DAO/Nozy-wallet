//! Native merchant invoices (Phase 3b) — metadata only; funds stay on-chain.
//!
//! Stored as JSON beside the wallet datadir. Diversifier index advances per invoice
//! so each sale gets a distinct Orchard UA under Business account index 1.

use crate::error::{NozyError, NozyResult};
use crate::paths::get_wallet_data_dir;
use crate::zip321::build_payment_uri;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const DEFAULT_TTL_MINUTES: i64 = 60;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InvoiceStatus {
    Open,
    Detected,
    Confirmed,
    Expired,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerchantInvoice {
    pub invoice_id: String,
    pub status: InvoiceStatus,
    pub payment_address: String,
    pub amount_zatoshis: u64,
    pub amount_zec: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount_fiat: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fiat_currency: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub product_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
    /// Diversifier index under Orchard account 1 used for this invoice.
    pub diversifier_index: u32,
    pub created_at: String,
    pub expires_at: String,
    pub zcash_uri: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detected_txid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confirmed_txid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct InvoiceStore {
    next_diversifier: u32,
    invoices: Vec<MerchantInvoice>,
}

impl InvoiceStore {
    fn path() -> PathBuf {
        get_wallet_data_dir().join("merchant_invoices.json")
    }

    fn load() -> NozyResult<Self> {
        let path = Self::path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = fs::read_to_string(&path)
            .map_err(|e| NozyError::Storage(format!("Failed to read invoices: {e}")))?;
        serde_json::from_str(&raw)
            .map_err(|e| NozyError::Storage(format!("Corrupt invoice store: {e}")))
    }

    fn save(&self) -> NozyResult<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| NozyError::Storage(format!("Failed to create invoice dir: {e}")))?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| NozyError::Storage(format!("Failed to serialize invoices: {e}")))?;
        fs::write(&path, json)
            .map_err(|e| NozyError::Storage(format!("Failed to write invoices: {e}")))?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CreateInvoiceParams {
    pub amount_zec: f64,
    pub amount_fiat: Option<f64>,
    pub fiat_currency: Option<String>,
    pub product_name: Option<String>,
    pub memo: Option<String>,
    pub ttl_minutes: Option<i64>,
    pub payment_address: String,
    pub diversifier_index: u32,
}

/// Allocate next diversifier index for Business invoices (does not persist until create).
pub fn peek_next_diversifier() -> NozyResult<u32> {
    Ok(InvoiceStore::load()?.next_diversifier.max(1))
}

pub fn create_invoice(params: CreateInvoiceParams) -> NozyResult<MerchantInvoice> {
    if !(params.amount_zec.is_finite() && params.amount_zec > 0.0) {
        return Err(NozyError::InvalidInput(
            "amount_zec must be a positive finite value".into(),
        ));
    }
    let amount_zatoshis = (params.amount_zec * 100_000_000.0).round() as u64;
    if amount_zatoshis == 0 {
        return Err(NozyError::InvalidInput(
            "amount_zec is too small (rounds to 0 zats)".into(),
        ));
    }

    let mut store = InvoiceStore::load()?;
    let diversifier_index = params.diversifier_index;
    if diversifier_index >= store.next_diversifier {
        store.next_diversifier = diversifier_index.saturating_add(1);
    }

    let now = Utc::now();
    let ttl = params.ttl_minutes.unwrap_or(DEFAULT_TTL_MINUTES).max(5);
    let expires = now + Duration::minutes(ttl);
    let invoice_id = format!("inv_{}", now.timestamp_millis());
    let memo = params.memo.or_else(|| Some(invoice_id.clone()));
    let zcash_uri = build_payment_uri(
        &params.payment_address,
        Some(params.amount_zec),
        memo.as_deref(),
    )?;

    let invoice = MerchantInvoice {
        invoice_id,
        status: InvoiceStatus::Open,
        payment_address: params.payment_address,
        amount_zatoshis,
        amount_zec: params.amount_zec,
        amount_fiat: params.amount_fiat,
        fiat_currency: params.fiat_currency,
        product_name: params.product_name,
        memo,
        diversifier_index,
        created_at: now.to_rfc3339(),
        expires_at: expires.to_rfc3339(),
        zcash_uri,
        detected_txid: None,
        confirmed_txid: None,
    };

    store.invoices.push(invoice.clone());
    store.save()?;
    Ok(invoice)
}

pub fn get_invoice(id: &str) -> NozyResult<Option<MerchantInvoice>> {
    let mut store = InvoiceStore::load()?;
    expire_open(&mut store)?;
    Ok(store.invoices.iter().find(|i| i.invoice_id == id).cloned())
}

pub fn list_invoices(limit: usize) -> NozyResult<Vec<MerchantInvoice>> {
    let mut store = InvoiceStore::load()?;
    expire_open(&mut store)?;
    let mut items = store.invoices;
    items.reverse();
    items.truncate(limit.max(1));
    Ok(items)
}

pub fn cancel_invoice(id: &str) -> NozyResult<MerchantInvoice> {
    let mut store = InvoiceStore::load()?;
    let inv = store
        .invoices
        .iter_mut()
        .find(|i| i.invoice_id == id)
        .ok_or_else(|| NozyError::InvalidOperation(format!("Invoice {id} not found")))?;
    if inv.status != InvoiceStatus::Open && inv.status != InvoiceStatus::Detected {
        return Err(NozyError::InvalidOperation(format!(
            "Cannot cancel invoice in status {:?}",
            inv.status
        )));
    }
    inv.status = InvoiceStatus::Cancelled;
    let out = inv.clone();
    store.save()?;
    Ok(out)
}

/// Mark open invoices past `expires_at` as Expired.
fn expire_open(store: &mut InvoiceStore) -> NozyResult<()> {
    let now = Utc::now();
    let mut dirty = false;
    for inv in &mut store.invoices {
        if inv.status == InvoiceStatus::Open {
            if let Ok(exp) = chrono::DateTime::parse_from_rfc3339(&inv.expires_at) {
                if exp.with_timezone(&Utc) <= now {
                    inv.status = InvoiceStatus::Expired;
                    dirty = true;
                }
            }
        }
    }
    if dirty {
        store.save()?;
    }
    Ok(())
}

/// Mark matching open invoice as detected/confirmed when a note arrives.
pub fn match_incoming_payment(
    payment_address: &str,
    amount_zatoshis: u64,
    txid: &str,
    confirmed: bool,
) -> NozyResult<Option<MerchantInvoice>> {
    let mut store = InvoiceStore::load()?;
    expire_open(&mut store)?;
    let addr = payment_address.replace([' ', '\n', '\t'], "");
    let inv = store.invoices.iter_mut().find(|i| {
        (i.status == InvoiceStatus::Open || i.status == InvoiceStatus::Detected)
            && i.payment_address.replace([' ', '\n', '\t'], "") == addr
            && i.amount_zatoshis == amount_zatoshis
    });
    let Some(inv) = inv else {
        return Ok(None);
    };
    if confirmed {
        inv.status = InvoiceStatus::Confirmed;
        inv.confirmed_txid = Some(txid.to_string());
        inv.detected_txid = inv.detected_txid.clone().or(Some(txid.to_string()));
    } else {
        inv.status = InvoiceStatus::Detected;
        inv.detected_txid = Some(txid.to_string());
    }
    let out = inv.clone();
    store.save()?;
    Ok(Some(out))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::with_wallet_data_dir;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn create_and_get_invoice() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("nozy_inv_test_{nanos}"));
        let _ = fs::create_dir_all(&dir);
        with_wallet_data_dir(&dir, || {
            let inv = create_invoice(CreateInvoiceParams {
                amount_zec: 0.05,
                amount_fiat: Some(29.99),
                fiat_currency: Some("USD".into()),
                product_name: Some("Taco".into()),
                memo: None,
                ttl_minutes: Some(30),
                payment_address: "u1business".into(),
                diversifier_index: 1,
            })
            .unwrap();
            assert_eq!(inv.status, InvoiceStatus::Open);
            assert!(inv.zcash_uri.contains("amount=0.05"));
            let got = get_invoice(&inv.invoice_id).unwrap().unwrap();
            assert_eq!(got.invoice_id, inv.invoice_id);
            cancel_invoice(&inv.invoice_id).unwrap();
            let cancelled = get_invoice(&inv.invoice_id).unwrap().unwrap();
            assert_eq!(cancelled.status, InvoiceStatus::Cancelled);
        });
        let _ = fs::remove_dir_all(&dir);
    }
}
