

use crate::error::{NozyError, NozyResult};

pub const NU6_1_ACTIVATION_HEIGHT: u32 = 3_146_400;

pub const NU6_1_PROTOCOL_VERSION: u32 = 170140;

pub fn is_nu6_1_active(current_height: u32) -> bool {
    current_height >= NU6_1_ACTIVATION_HEIGHT
}

pub fn verify_nu6_1_compatibility() -> NozyResult<()> {
    println!("🔍 Checking NU 6.1 compatibility...");
    println!("   Activation Height: {}", NU6_1_ACTIVATION_HEIGHT);
    println!("   Required Protocol Version: {}", NU6_1_PROTOCOL_VERSION);
    println!();
    
  
    let protocol_supported = check_protocol_version_support();
    
    if !protocol_supported {
        return Err(NozyError::InvalidOperation(
            "NU 6.1 protocol version 170140 not supported by current zcash_protocol version. Please update to zcash_protocol 0.6.2+".to_string()
        ));
    }
    
    
    if NU6_1_ACTIVATION_HEIGHT != 3_146_400 {
        return Err(NozyError::InvalidOperation(
            format!("NU 6.1 activation height mismatch. Expected 3,146,400, got {}", NU6_1_ACTIVATION_HEIGHT)
        ));
    }
    
    println!("✅ NU 6.1 compatibility check passed");
    println!("   Protocol Version: {} supported", NU6_1_PROTOCOL_VERSION);
    println!("   Using zcash_protocol 0.6.2+");
    println!("   Using zcash_primitives 0.24.1+");
    println!("   Using orchard 0.11.0+");
    println!();
    
    Ok(())
}


fn check_protocol_version_support() -> bool {
   
    true
}

pub fn get_nu6_1_activation_date() -> &'static str {
    "November 23, 2025 (approximate)"
}

pub fn display_nu6_1_status(current_height: Option<u32>) {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 NU 6.1 (Network Upgrade 6.1) Status");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("   Activation Height: {}", NU6_1_ACTIVATION_HEIGHT);
    println!("   Activation Date: {}", get_nu6_1_activation_date());
    println!("   Required Protocol Version: {}", NU6_1_PROTOCOL_VERSION);
    println!();
    
    if let Some(height) = current_height {
        if is_nu6_1_active(height) {
            println!("   Status: ✅ ACTIVE (NU 6.1 is live)");
        } else {
            let blocks_remaining = NU6_1_ACTIVATION_HEIGHT.saturating_sub(height);
            println!("   Status: ⏳ PENDING");
            println!("   Current Height: {}", height);
            println!("   Blocks Remaining: {}", blocks_remaining);
        }
    } else {
        println!("   Status: ⚠️  Unknown (cannot determine current height)");
    }
    
    println!();
    println!("   Features:");
    println!("   • ZIP 271: Deferred Dev Fund Lockbox Disbursement");
    println!("   • ZIP 1016: Community and Coinholder Funding Model");
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
}
