//! ZIP-321 payment URI helpers (`zcash:`).
//!
//! Payer-side only: parse/build payment requests. Do not call ZGo, NozyPay, or any
//! merchant HTTP API from this module — a pasted or scanned URI is enough to pay.
//!
//! Grammar: <https://zips.z.cash/zip-0321>
//! Nozy overlay: Unified Addresses only (`u1` / `utest1` / `uregtest1`). Transparent,
//! Sprout, and Sapling-only addresses are rejected. Unknown `req-` parameters and
//! multi-recipient (`address.1`) requests are rejected.

use crate::error::{NozyError, NozyResult};

const MAX_ZEC: f64 = 21_000_000.0;
const MAX_MEMO_BYTES: usize = 512;

#[derive(Debug, Clone, PartialEq)]
pub struct PaymentRequest {
    pub address: String,
    pub amount_zec: Option<f64>,
    pub memo: Option<String>,
    pub message: Option<String>,
    pub label: Option<String>,
}

/// Build `zcash:<address>?amount=…&memo=…` with a ZIP-321 base64url memo.
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
    assert_shielded_ua(addr)?;
    if !is_zip321_address_token(addr) {
        return Err(NozyError::InvalidInput(
            "ZIP-321 address may only contain letters and digits".into(),
        ));
    }
    let mut uri = format!("zcash:{addr}");
    let mut params: Vec<String> = Vec::new();
    if let Some(amt) = amount_zec {
        params.push(format!("amount={}", format_zip321_amount(amt)?));
    }
    if let Some(m) = memo.map(str::trim).filter(|s| !s.is_empty()) {
        let raw = m.as_bytes();
        if raw.len() > MAX_MEMO_BYTES {
            return Err(NozyError::InvalidInput(
                "ZIP-321 memo exceeds 512 bytes".into(),
            ));
        }
        params.push(format!("memo={}", base64url_encode(raw)));
    }
    if !params.is_empty() {
        uri.push('?');
        uri.push_str(&params.join("&"));
    }
    Ok(uri)
}

/// Parse a `zcash:` URI into address + optional amount/memo/message/label.
pub fn parse_payment_uri(raw: &str) -> NozyResult<PaymentRequest> {
    let extracted = extract_zip321_uri(raw).ok_or_else(|| {
        NozyError::InvalidInput(
            "Not a zcash: payment URI. Paste the ZIP-321 URI from the QR, not a zgo.cash page URL."
                .into(),
        )
    })?;
    parse_zip321_strict(extracted)
}

/// Find a `zcash:` URI in clipboard / deep-link text. Does not fetch HTTP pages.
pub fn extract_zip321_uri(raw: &str) -> Option<&str> {
    let s = raw.trim();
    if looks_like_zgo_page(s) && !s.to_ascii_lowercase().contains("zcash:") {
        return None;
    }
    let lower = s.to_ascii_lowercase();
    let idx = lower.find("zcash:")?;
    let rest = &s[idx..];
    let end = rest
        .find(|c: char| c.is_whitespace() || c == '"' || c == '<' || c == '>')
        .unwrap_or(rest.len());
    Some(rest[..end].trim_end_matches(['.', ',', ';', ')']))
}

pub fn looks_like_zgo_page(raw: &str) -> bool {
    let s = raw.trim().to_ascii_lowercase();
    (s.starts_with("http://") || s.starts_with("https://")) && s.contains("zgo.cash")
}

fn parse_zip321_strict(raw: &str) -> NozyResult<PaymentRequest> {
    let s = raw.trim();
    let rest = strip_scheme(s)
        .ok_or_else(|| NozyError::InvalidInput("Not a zcash: payment URI".into()))?;
    if rest.starts_with("//") {
        return Err(NozyError::InvalidInput(
            "ZIP-321 URIs must not use // (not a hierarchical URI)".into(),
        ));
    }

    let (hier, query) = match rest.split_once('?') {
        Some((a, q)) => (a, Some(q)),
        None => (rest, None),
    };
    if hier.contains('%') {
        return Err(NozyError::InvalidInput(
            "ZIP-321 address must not be percent-encoded".into(),
        ));
    }

    let hier_address = hier.trim().to_string();
    if !hier_address.is_empty() && !is_zip321_address_token(&hier_address) {
        return Err(NozyError::InvalidInput(
            "ZIP-321 address may only contain letters and digits".into(),
        ));
    }

    let mut query_address: Option<String> = None;
    let mut amount_zec = None;
    let mut memo = None;
    let mut message = None;
    let mut label = None;
    let mut seen: Vec<(String, String)> = Vec::new();

    if let Some(q) = query {
        if q.is_empty() {
            return Err(NozyError::InvalidInput(
                "ZIP-321 URI has an empty query".into(),
            ));
        }
        for pair in q.split('&') {
            if pair.is_empty() {
                continue;
            }
            let (raw_key, raw_val) = match pair.split_once('=') {
                Some((k, v)) => (k, Some(v)),
                None => (pair, None),
            };
            if raw_key.contains('%') {
                return Err(NozyError::InvalidInput(
                    "ZIP-321 parameter names must not be percent-encoded".into(),
                ));
            }
            let (name, index) = split_name_index(raw_key)?;
            if !index.is_empty() {
                return Err(NozyError::InvalidInput(
                    "Multi-recipient ZIP-321 requests are not supported".into(),
                ));
            }
            let key = (name.clone(), index.clone());
            if seen.iter().any(|s| s == &key) {
                return Err(NozyError::InvalidInput(format!(
                    "Duplicate ZIP-321 parameter `{raw_key}`"
                )));
            }
            seen.push(key);

            if name.starts_with("req-") {
                return Err(NozyError::InvalidInput(format!(
                    "Unsupported required ZIP-321 parameter `{name}`"
                )));
            }

            match name.as_str() {
                "address" => {
                    let val = raw_val.ok_or_else(|| {
                        NozyError::InvalidInput("ZIP-321 address= is missing a value".into())
                    })?;
                    if val.contains('%') {
                        return Err(NozyError::InvalidInput(
                            "ZIP-321 address must not be percent-encoded".into(),
                        ));
                    }
                    if !is_zip321_address_token(val) {
                        return Err(NozyError::InvalidInput(
                            "ZIP-321 address may only contain letters and digits".into(),
                        ));
                    }
                    query_address = Some(val.to_string());
                }
                "amount" => {
                    let val = raw_val.ok_or_else(|| {
                        NozyError::InvalidInput("ZIP-321 amount= is missing a value".into())
                    })?;
                    if val.contains('%') {
                        return Err(NozyError::InvalidInput(
                            "ZIP-321 amount must not be percent-encoded".into(),
                        ));
                    }
                    amount_zec = Some(parse_zip321_amount(val)?);
                }
                "memo" => {
                    let val = raw_val.unwrap_or("");
                    memo = Some(decode_zip321_memo(val)?);
                }
                "message" => {
                    message = Some(percent_decode(raw_val.unwrap_or("")));
                }
                "label" => {
                    label = Some(percent_decode(raw_val.unwrap_or("")));
                }
                _ => {
                    // Unknown non-req parameter: ignore (ZIP-321 forward compatibility).
                }
            }
        }
    }

    if !hier_address.is_empty() && query_address.is_some() {
        return Err(NozyError::InvalidInput(
            "ZIP-321 address specified twice (path and address=)".into(),
        ));
    }
    let address = if !hier_address.is_empty() {
        hier_address
    } else {
        query_address.unwrap_or_default()
    };
    if address.is_empty() {
        return Err(NozyError::InvalidInput("zcash: URI missing address".into()));
    }
    assert_shielded_ua(&address)?;

    Ok(PaymentRequest {
        address,
        amount_zec,
        memo,
        message,
        label,
    })
}

fn strip_scheme(s: &str) -> Option<&str> {
    let lower = s.to_ascii_lowercase();
    if lower.starts_with("zcash:") {
        Some(&s[6..])
    } else {
        None
    }
}

fn split_name_index(raw_key: &str) -> NozyResult<(String, String)> {
    if let Some((name, index)) = raw_key.split_once('.') {
        if name.is_empty() || !is_param_name(name) {
            return Err(NozyError::InvalidInput(format!(
                "Invalid ZIP-321 parameter `{raw_key}`"
            )));
        }
        if !is_valid_paramindex(index) {
            return Err(NozyError::InvalidInput(format!(
                "Invalid ZIP-321 parameter index in `{raw_key}`"
            )));
        }
        Ok((name.to_ascii_lowercase(), index.to_string()))
    } else {
        if !is_param_name(raw_key) {
            return Err(NozyError::InvalidInput(format!(
                "Invalid ZIP-321 parameter `{raw_key}`"
            )));
        }
        Ok((raw_key.to_ascii_lowercase(), String::new()))
    }
}

fn is_param_name(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-')
}

fn is_valid_paramindex(s: &str) -> bool {
    if s.is_empty() || s.len() > 4 {
        return false;
    }
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_digit() && c != '0' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_digit())
}

fn is_zip321_address_token(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric())
}

fn assert_shielded_ua(addr: &str) -> NozyResult<()> {
    let a = addr.to_ascii_lowercase();
    if a.starts_with("t1") || a.starts_with("t3") || a.starts_with("tm") || a.starts_with("t2") {
        return Err(NozyError::InvalidInput(
            "Transparent ZEC payment URIs are not supported".into(),
        ));
    }
    if a.starts_with("zc") {
        return Err(NozyError::InvalidInput(
            "Sprout addresses are not supported in payment requests".into(),
        ));
    }
    if a.starts_with("zs1") || a.starts_with("ztestsapling") || a.starts_with("zregtestsapling") {
        return Err(NozyError::InvalidInput(
            "Nozy pays Unified Addresses only (u1…). Sapling-only payment requests are not supported."
                .into(),
        ));
    }
    if !(a.starts_with("u1") || a.starts_with("utest1") || a.starts_with("uregtest1")) {
        return Err(NozyError::InvalidInput(
            "Payment URI must use a shielded Unified Address (u1 / utest1)".into(),
        ));
    }
    Ok(())
}

fn format_zip321_amount(amt: f64) -> NozyResult<String> {
    if !(amt.is_finite() && amt > 0.0 && amt <= MAX_ZEC) {
        return Err(NozyError::InvalidInput(
            "ZIP-321 amount must be a positive ZEC value of at most 21000000".into(),
        ));
    }
    let s = format!("{amt:.8}");
    Ok(s.trim_end_matches('0').trim_end_matches('.').to_string())
}

fn parse_zip321_amount(val: &str) -> NozyResult<f64> {
    if val.is_empty()
        || val.starts_with('.')
        || val.ends_with('.')
        || val.contains(',')
        || val.chars().filter(|c| *c == '.').count() > 1
    {
        return Err(NozyError::InvalidInput(format!(
            "Invalid ZIP-321 amount: {val}"
        )));
    }
    let (whole, frac) = match val.split_once('.') {
        Some((w, f)) => (w, Some(f)),
        None => (val, None),
    };
    if whole.is_empty() || !whole.chars().all(|c| c.is_ascii_digit()) {
        return Err(NozyError::InvalidInput(format!(
            "Invalid ZIP-321 amount: {val}"
        )));
    }
    if let Some(f) = frac {
        if f.is_empty() || f.len() > 8 || !f.chars().all(|c| c.is_ascii_digit()) {
            return Err(NozyError::InvalidInput(format!(
                "Invalid ZIP-321 amount: {val}"
            )));
        }
    }
    let amt: f64 = val
        .parse()
        .map_err(|_| NozyError::InvalidInput(format!("Invalid ZIP-321 amount: {val}")))?;
    if !(amt.is_finite() && amt >= 0.0 && amt <= MAX_ZEC) {
        return Err(NozyError::InvalidInput(format!(
            "ZIP-321 amount out of range: {val}"
        )));
    }
    Ok(amt)
}

fn decode_zip321_memo(val: &str) -> NozyResult<String> {
    if val
        .chars()
        .any(|c| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
    {
        return Err(NozyError::InvalidInput(
            "ZIP-321 memo must be unpadded base64url".into(),
        ));
    }
    let bytes = base64url_decode(val)?;
    if bytes.len() > MAX_MEMO_BYTES {
        return Err(NozyError::InvalidInput(
            "ZIP-321 memo exceeds 512 bytes".into(),
        ));
    }
    let trimmed = bytes
        .iter()
        .rev()
        .skip_while(|b| **b == 0)
        .copied()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>();
    Ok(String::from_utf8_lossy(&trimmed).into_owned())
}

fn base64url_encode(input: &[u8]) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::new();
    let mut i = 0;
    while i < input.len() {
        let b0 = input[i];
        let b1 = if i + 1 < input.len() { input[i + 1] } else { 0 };
        let b2 = if i + 2 < input.len() { input[i + 2] } else { 0 };
        let n = ((b0 as u32) << 16) | ((b1 as u32) << 8) | (b2 as u32);
        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
        if i + 1 < input.len() {
            out.push(TABLE[((n >> 6) & 63) as usize] as char);
        }
        if i + 2 < input.len() {
            out.push(TABLE[(n & 63) as usize] as char);
        }
        i += 3;
    }
    out
}

fn base64url_decode(s: &str) -> NozyResult<Vec<u8>> {
    if s.contains('+') || s.contains('/') || s.contains('=') {
        return Err(NozyError::InvalidInput(
            "ZIP-321 memo must be unpadded base64url (no + / =)".into(),
        ));
    }
    let mut vals = Vec::with_capacity(s.len());
    for c in s.bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'-' => 62,
            b'_' => 63,
            _ => {
                return Err(NozyError::InvalidInput(
                    "ZIP-321 memo must be unpadded base64url".into(),
                ));
            }
        };
        vals.push(v);
    }
    let mut out = Vec::new();
    let mut i = 0;
    while i < vals.len() {
        let v0 = vals[i] as u32;
        let v1 = if i + 1 < vals.len() {
            vals[i + 1] as u32
        } else {
            0
        };
        let v2 = if i + 2 < vals.len() {
            vals[i + 2] as u32
        } else {
            0
        };
        let v3 = if i + 3 < vals.len() {
            vals[i + 3] as u32
        } else {
            0
        };
        let n = (v0 << 18) | (v1 << 12) | (v2 << 6) | v3;
        if i + 1 < vals.len() {
            out.push(((n >> 16) & 0xff) as u8);
        }
        if i + 2 < vals.len() {
            out.push(((n >> 8) & 0xff) as u8);
        }
        if i + 3 < vals.len() {
            out.push((n & 0xff) as u8);
        }
        i += 4;
    }
    Ok(out)
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

    const UA: &str =
        "u1testaddressplaceholder00000000000000000000000000000000000000000000000000000000000";

    #[test]
    fn round_trip_amount_memo_base64url() {
        let uri = build_payment_uri(UA, Some(0.05), Some("invoice-1")).unwrap();
        assert!(uri.starts_with("zcash:u1"));
        assert!(uri.contains("amount=0.05"));
        assert!(uri.contains(&format!("memo={}", base64url_encode(b"invoice-1"))));
        assert!(!uri.contains("memo=invoice-1"));
        let parsed = parse_payment_uri(&uri).unwrap();
        assert_eq!(parsed.amount_zec, Some(0.05));
        assert_eq!(parsed.memo.as_deref(), Some("invoice-1"));
    }

    #[test]
    fn decode_spec_memo_example() {
        let decoded = decode_zip321_memo("VGhpcyBpcyBhIHNpbXBsZSBtZW1vLg").unwrap();
        assert_eq!(decoded, "This is a simple memo.");
    }

    #[test]
    fn parse_message_and_label() {
        let uri = format!(
            "zcash:{UA}?amount=1&memo=VGhpcyBpcyBhIHNpbXBsZSBtZW1vLg&message=Thank%20you%20for%20your%20purchase&label=Shop"
        );
        let p = parse_payment_uri(&uri).unwrap();
        assert_eq!(p.amount_zec, Some(1.0));
        assert_eq!(p.memo.as_deref(), Some("This is a simple memo."));
        assert_eq!(p.message.as_deref(), Some("Thank you for your purchase"));
        assert_eq!(p.label.as_deref(), Some("Shop"));
    }

    #[test]
    fn address_query_param() {
        let uri = format!("zcash:?address={UA}&amount=0.25");
        let p = parse_payment_uri(&uri).unwrap();
        assert_eq!(p.address, UA);
        assert_eq!(p.amount_zec, Some(0.25));
    }

    #[test]
    fn reject_transparent() {
        let err = parse_payment_uri("zcash:tmEZhbWHTpdKMw5it8YDspUXSMGQyFwovpU?amount=1")
            .unwrap_err()
            .to_string();
        assert!(err.to_lowercase().contains("transparent"));
    }

    #[test]
    fn reject_sapling_only() {
        let uri = "zcash:ztestsapling10yy2ex5dcqkclhc7z7yrnjq2z6feyjad56ptwlfgmy77dmaqqrl9gyhprdx59qgmsnyfska2kez?amount=1&memo=VGhpcyBpcyBhIHNpbXBsZSBtZW1vLg";
        let err = parse_payment_uri(uri).unwrap_err().to_string();
        assert!(err.to_lowercase().contains("unified"));
    }

    #[test]
    fn reject_double_slash() {
        let uri = format!("zcash://{UA}?amount=1");
        assert!(parse_payment_uri(&uri).is_err());
    }

    #[test]
    fn reject_percent_encoded_amount() {
        let uri = format!("zcash:{UA}?amount=1%30");
        assert!(parse_payment_uri(&uri).is_err());
    }

    #[test]
    fn reject_req_unknown() {
        let uri = format!("zcash:{UA}?amount=1&req-asset=abc");
        let err = parse_payment_uri(&uri).unwrap_err().to_string();
        assert!(err.contains("req-asset"));
    }

    #[test]
    fn reject_multi_recipient() {
        let uri = format!("zcash:?address={UA}&amount=1&address.1={UA}&amount.1=2");
        let err = parse_payment_uri(&uri).unwrap_err().to_string();
        assert!(err.to_lowercase().contains("multi-recipient"));
    }

    #[test]
    fn reject_non_zcash_scheme() {
        assert!(parse_payment_uri("bitcoin:abc").is_err());
    }

    #[test]
    fn reject_zgo_https_page_without_uri() {
        assert!(extract_zip321_uri("https://app.zgo.cash/pmtservice?owner=abc").is_none());
        assert!(parse_payment_uri("https://app.zgo.cash/pmtservice?owner=abc").is_err());
    }

    #[test]
    fn extract_from_wrapped_text() {
        let inner = build_payment_uri(UA, Some(1.0), Some("hi")).unwrap();
        let wrapped = format!("Pay here: {inner} thanks");
        let extracted = extract_zip321_uri(&wrapped).unwrap();
        let p = parse_payment_uri(extracted).unwrap();
        assert_eq!(p.amount_zec, Some(1.0));
        assert_eq!(p.memo.as_deref(), Some("hi"));
    }

    #[test]
    fn encode_spaces_in_memo_as_base64url() {
        let uri = build_payment_uri(UA, Some(1.0), Some("taco plate")).unwrap();
        assert!(!uri.contains("memo=taco%20plate"));
        let p = parse_payment_uri(&uri).unwrap();
        assert_eq!(p.memo.as_deref(), Some("taco plate"));
    }
}
