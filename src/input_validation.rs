// Enhanced input validation utilities

use crate::error::{NozyError, NozyResult};

pub fn validate_zcash_address(address: &str) -> NozyResult<()> {
    if address.is_empty() {
        return Err(NozyError::InvalidInput(
            "Address cannot be empty".to_string(),
        ));
    }

    if !address.starts_with("u1") {
        return Err(NozyError::InvalidInput(
            "Address must be a unified address starting with 'u1'".to_string(),
        ));
    }

    if address.len() < 78 || address.len() > 140 {
        return Err(NozyError::InvalidInput(format!(
            "Invalid address length: {} (expected 78-140 characters)",
            address.len()
        )));
    }

    if !address.chars().all(|c| c.is_alphanumeric() || c == '1') {
        return Err(NozyError::InvalidInput(
            "Address contains invalid characters".to_string(),
        ));
    }

    Ok(())
}

pub fn validate_amount(amount: f64, min: f64, max: f64, unit: &str) -> NozyResult<()> {
    if amount <= 0.0 {
        return Err(NozyError::InvalidInput(format!(
            "Amount must be greater than zero"
        )));
    }

    if amount < min {
        return Err(NozyError::InvalidInput(format!(
            "Amount too small: minimum is {} {}",
            min, unit
        )));
    }

    if amount > max {
        return Err(NozyError::InvalidInput(format!(
            "Amount too large: maximum is {} {}",
            max, unit
        )));
    }

    let decimal_places = (amount.fract() * 1e8) as u64;
    if decimal_places > 0 && (amount.fract() * 1e8 - decimal_places as f64).abs() > 1e-6 {
        return Err(NozyError::InvalidInput(
            "Amount has too many decimal places (maximum 8)".to_string(),
        ));
    }

    Ok(())
}

pub fn validate_memo(memo: &str) -> NozyResult<()> {
    let memo_bytes = memo.as_bytes();
    if memo_bytes.len() > 512 {
        return Err(NozyError::InvalidInput(format!(
            "Memo too long: {} bytes (maximum 512)",
            memo_bytes.len()
        )));
    }

    Ok(())
}

pub fn sanitize_input(input: &str) -> String {
    input
        .chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
        .collect()
}
