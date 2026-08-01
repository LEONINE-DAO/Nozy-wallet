//! ZIP-321 payment URI helpers (`zcash:`).
//!
//! Generate and parse amount + address (+ optional memo) for Sell / Receive / scan-to-pay.
//! Names are not encoded as the `address` param; resolve ZNS first, then build the URI.

use crate::error::{NozyError, NozyResult};

#[derive(Debug, Clone, PartialEq)]
pub struct PaymentRequest {
    pub address: String,
    pub amount_zec: Option<f64>,
    pub memo: Option<String>,
    pub message: Option<String>,
    pub label: Option<String>,
}

/// Build `zcash:<address>?amount=…&memo=…` (amount in ZEC decimal).
pub fn build_payment_uri(
    address: &str,
    amount_zec: Option<f64>,
    memo: Option<&str>,
) -> NozyResult<String> {
    let addr = address.trim();
    if addr.is_empty() {
        return Err(NozyError::InvalidInput(
            "ZIP-321 URI requires a payment address".into(),
        ));
    }
    let mut uri = format!("zcash:{addr}");
    let mut params: Vec<String> = Vec::new();
    if let Some(amt) = amount_zec {
        if !(amt.is_finite() && amt > 0.0) {
            return Err(NozyError::InvalidInput(
                "ZIP-321 amount must be a positive finite ZEC value".into(),
            ));
        }
        let s = format!("{amt:.8}");
        let s = s.trim_end_matches('0').trim_end_matches('.');
        params.push(format!("amount={s}"));
    }
    if let Some(m) = memo.map(str::trim).filter(|s| !s.is_empty()) {
        params.push(format!("memo={}", percent_encode(m)));
    }
    if !params.is_empty() {
        uri.push('?');
        uri.push_str(&params.join("&"));
    }
    Ok(uri)
}

/// Parse a `zcash:` URI into address + optional amount/memo.
pub fn parse_payment_uri(raw: &str) -> NozyResult<PaymentRequest> {
    let s = raw.trim();
    let rest = s
        .strip_prefix("zcash:")
        .or_else(|| s.strip_prefix("ZCASH:"))
        .ok_or_else(|| NozyError::InvalidInput("Not a zcash: payment URI".into()))?;

    let (address_part, query) = match rest.split_once('?') {
        Some((a, q)) => (a, Some(q)),
        None => (rest, None),
    };
    let address = address_part.trim().to_string();
    if address.is_empty() {
        return Err(NozyError::InvalidInput("zcash: URI missing address".into()));
    }

    let mut amount_zec = None;
    let mut memo = None;
    let mut message = None;
    let mut label = None;

    if let Some(q) = query {
        for pair in q.split('&') {
            let (k, v) = match pair.split_once('=') {
                Some((k, v)) => (k, v),
                None => continue,
            };
            let key = k.to_ascii_lowercase();
            let val = percent_decode(v);
            match key.as_str() {
                "amount" => {
                    let amt: f64 = val.parse().map_err(|_| {
                        NozyError::InvalidInput(format!("Invalid ZIP-321 amount: {val}"))
                    })?;
                    amount_zec = Some(amt);
                }
                "memo" => memo = Some(val),
                "message" => message = Some(val),
                "label" => label = Some(val),
                _ => {}
            }
        }
    }

    Ok(PaymentRequest {
        address,
        amount_zec,
        memo,
        message,
        label,
    })
}

fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(h), Some(l)) = (from_hex(bytes[i + 1]), from_hex(bytes[i + 2])) {
                out.push((h << 4) | l);
                i += 3;
                continue;
            }
        }
        if bytes[i] == b'+' {
            out.push(b' ');
        } else {
            out.push(bytes[i]);
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn from_hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_amount_memo() {
        let uri = build_payment_uri(
            "u1testaddressplaceholder00000000000000000000000000000000000000000000000000000000000",
            Some(0.05),
            Some("invoice-1"),
        )
        .unwrap();
        assert!(uri.starts_with("zcash:u1"));
        assert!(uri.contains("amount=0.05"));
        assert!(uri.contains("memo=invoice-1"));
        let parsed = parse_payment_uri(&uri).unwrap();
        assert_eq!(parsed.amount_zec, Some(0.05));
        assert_eq!(parsed.memo.as_deref(), Some("invoice-1"));
    }

    #[test]
    fn reject_non_zcash_scheme() {
        assert!(parse_payment_uri("bitcoin:abc").is_err());
    }

    #[test]
    fn encode_spaces_in_memo() {
        let uri = build_payment_uri("u1abc", Some(1.0), Some("taco plate")).unwrap();
        assert!(uri.contains("memo=taco%20plate"));
        let p = parse_payment_uri(&uri).unwrap();
        assert_eq!(p.memo.as_deref(), Some("taco plate"));
    }
}
