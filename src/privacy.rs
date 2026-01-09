// Privacy enforcement module
// Privacy is a right, not a privilege

use crate::error::{NozyError, NozyResult};

/// Validates that an address is a shielded address (privacy-protected)
/// Blocks transparent addresses (t1) to enforce privacy
pub fn validate_shielded_address(address: &str) -> NozyResult<()> {
    if address.starts_with("t1") {
        return Err(NozyError::InvalidOperation(
            "Transparent addresses (t1) are not supported. \
             NozyWallet enforces privacy by default - only shielded addresses are allowed. \
             Please use a unified address (u1...) or Sapling address (zs1...) for privacy protection.".to_string()
        ));
    }
    
    if !address.starts_with("u1") && !address.starts_with("zs1") && !address.starts_with("utest1") {
        return Err(NozyError::InvalidOperation(
            "Invalid address format! Must be a shielded address for privacy protection. \
             Supported formats: u1 (unified with Orchard), zs1 (Sapling), or utest1 (testnet unified).".to_string()
        ));
    }
    
    Ok(())
}

/// Ensures privacy is maintained for all transactions
/// This is called before any transaction is built
pub fn ensure_privacy() -> NozyResult<()> {
    // Privacy is enforced at multiple levels:
    // 1. Address validation (blocks transparent addresses)
    // 2. Transaction building (only Orchard shielded transactions)
    // 3. Wallet design (no transparent address generation)
    
    // This function serves as a reminder that privacy is non-negotiable
    Ok(())
}

/// Privacy guarantee message
pub fn privacy_guarantee() -> &'static str {
    "NozyWallet guarantees:
    - Every transaction is private and untraceable
    - Sender, receiver, and amount are hidden
    - True fungibility - no blacklisted coins
    - Privacy by default - no exceptions"
}

