// Enhanced input validation utilities

use crate::error::{NozyError, NozyResult};

/// Strip line breaks and other whitespace from pasted unified addresses.
pub fn normalize_unified_address(address: &str) -> String {
    address.chars().filter(|c| !c.is_whitespace()).collect()
}

pub fn validate_zcash_address(address: &str) -> NozyResult<()> {
    let address = normalize_unified_address(address);
    if address.is_empty() {
        return Err(NozyError::InvalidInput(
            "Address cannot be empty".to_string(),
        ));
    }

    if !address.starts_with("u1") && !address.starts_with("utest1") {
        return Err(NozyError::InvalidInput(
            "Address must be a unified address starting with 'u1' or 'utest1'".to_string(),
        ));
    }

    // Unified addresses (`u1…` / `utest1…`) vary in length; 140 was too small for many real UAs.
    // See Zcash unified address encoding (ZIP 316); keep a generous upper bound.
    const UA_MIN_LEN: usize = 78;
    const UA_MAX_LEN: usize = 256;
    if address.len() < UA_MIN_LEN || address.len() > UA_MAX_LEN {
        return Err(NozyError::InvalidInput(format!(
            "Invalid address length: {} (expected {}-{} characters for a unified address)",
            address.len(),
            UA_MIN_LEN,
            UA_MAX_LEN
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_long_unified_addresses_from_nozy() {
        // Regression: api-server used to cap u1 at 100 chars; Nozy-generated UAs can be ~106+.
        let addr = "u13nkpl0xejf50y2l2nwq44jeg6u28ayey0k80htxspz6vqfa4zru4v45ez7n3qz9c3e6h29m89w4ket6wlmpgpq4ra4f7gd42uyp7c94e";
        assert_eq!(addr.len(), 106);
        validate_zcash_address(addr).expect("106-char Nozy UA should validate");
    }

    #[test]
    fn accepts_testnet_unified_addresses_from_nozy() {
        let addr = "utest1dt8gy9uhr638jrpjzlacn3m7jengue30p3g849xwu7kj29yvrkfeyczq694qsyjh2f9tzs2krccjq0mtpzelgkr2p8735teapcy88mrx";
        validate_zcash_address(addr).expect("real testnet UA should validate");
    }

    #[test]
    fn normalizes_multiline_unified_addresses() {
        use zcash_address::unified::Encoding;
        let joined = "utest1455x45ffymw8x0zqsnjfrp63h09vfw8ps5vmuecey5shrnfn7egy9fwrgypexkef9d52njhw3y0tnegjtz2xs2xpyj5dsnvuduzjrg2y";
        let multiline = "utest1455x45ffymw8x0zqsnjfrp63h09vfw8\nps5vmuecey5shrnfn7egy9fwrgypexkef9d52\nnjhw3y0tnegjtz2xs2xpyj5dsnvuduzjrg2y";
        assert_eq!(normalize_unified_address(multiline), joined);
        validate_zcash_address(multiline)
            .expect("multiline UA should validate after normalization");
        zcash_address::unified::Address::decode(&normalize_unified_address(multiline))
            .expect("multiline UA should decode after normalization");
    }
}
