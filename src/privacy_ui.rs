// Privacy UI indicators and messages
// Shows privacy status and guarantees to users

use crate::error::NozyResult;

pub fn show_privacy_indicator() {
    println!();
    println!("🛡️  PRIVACY PROTECTED");
    println!("   ✅ Shielded Transaction");
    println!("   ✅ Sender Hidden");
    println!("   ✅ Receiver Hidden");
    println!("   ✅ Amount Hidden");
    println!("   ✅ Untraceable");
    println!();
}

/// Display privacy guarantee message
pub fn show_privacy_guarantee() {
    println!();
    println!("🔒 Privacy Guarantee:");
    println!("   This transaction is completely private and untraceable.");
    println!("   NozyWallet enforces privacy by default - every transaction is shielded.");
    println!();
}

/// Display privacy comparison with Monero
pub fn show_privacy_comparison() {
    println!();
    println!("📊 Privacy Level: Monero-Equivalent");
    println!("   • Same privacy guarantees as Monero");
    println!("   • Zero-knowledge proofs (zkSNARKs)");
    println!("   • True fungibility (no blacklisted coins)");
    println!("   • Faster than Monero (75s vs 2min blocks)");
    println!();
}

/// Display privacy status for address
pub fn show_address_privacy_status(address: &str) {
    if address.starts_with("u1") || address.starts_with("zs1") || address.starts_with("utest1") {
        println!("🛡️  Shielded Address - Privacy Protected");
    } else if address.starts_with("t1") {
        println!("⚠️  WARNING: Transparent address detected!");
        println!("   NozyWallet blocks transparent addresses to enforce privacy.");
        println!("   Please use a shielded address (u1... or zs1...).");
    }
}

/// Display privacy enforcement message
pub fn show_privacy_enforcement() {
    println!();
    println!("🔒 Privacy Enforcement:");
    println!("   NozyWallet only allows shielded transactions.");
    println!("   Transparent addresses are blocked to protect your privacy.");
    println!("   Privacy is mandatory, not optional.");
    println!();
}

/// Display transaction privacy summary
pub fn show_transaction_privacy_summary() {
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🛡️  TRANSACTION PRIVACY SUMMARY");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("   Privacy Level:     MAXIMUM (Monero-Equivalent)");
    println!("   Transaction Type: Shielded (Orchard)");
    println!("   Sender:           Hidden ✅");
    println!("   Receiver:          Hidden ✅");
    println!("   Amount:            Hidden ✅");
    println!("   Traceability:     Untraceable ✅");
    println!("   Fungibility:      True ✅");
    println!();
    println!("   🔒 This transaction is completely private.");
    println!("   🛡️  Privacy is enforced by NozyWallet.");
    println!("   ✅ No transparent transactions possible.");
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
}

/// Display privacy badge for successful transaction
pub fn show_privacy_badge() {
    println!();
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║                                                            ║");
    println!("║          🛡️  PRIVACY PROTECTED TRANSACTION 🛡️              ║");
    println!("║                                                            ║");
    println!("║  ✅ Shielded Transaction                                    ║");
    println!("║  ✅ Untraceable                                              ║");
    println!("║  ✅ Fungible                                                 ║");
    println!("║  ✅ Monero-Level Privacy                                     ║");
    println!("║                                                            ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
}

/// Validate and show privacy status for recipient address
pub fn validate_and_show_privacy(address: &str) -> NozyResult<()> {
    use crate::privacy::validate_shielded_address;
    
    match validate_shielded_address(address) {
        Ok(_) => {
            show_address_privacy_status(address);
            Ok(())
        }
        Err(e) => {
            show_address_privacy_status(address);
            Err(e)
        }
    }
}
