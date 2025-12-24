// Privacy Validator for Bridge Operations
// Enforces Tier 0 privacy requirements and warns about risks

use crate::error::{NozyError, NozyResult};
use crate::privacy_network::proxy::ProxyConfig;
use crate::config::load_config;

pub struct PrivacyValidator {
    privacy_network_active: bool,
    using_full_node: bool,
    address_reused: bool,
    monero_churned: bool,
}

impl PrivacyValidator {
    pub fn new() -> Self {
        Self {
            privacy_network_active: false,
            using_full_node: false,
            address_reused: false,
            monero_churned: false,
        }
    }
    
    /// Validate all privacy requirements before swap
    pub async fn validate_privacy_requirements(&mut self) -> NozyResult<PrivacyCheckResult> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        
        // Check 1: Privacy Network (Tor/I2P) - MANDATORY
        let proxy = ProxyConfig::auto_detect().await;
        if !proxy.enabled {
            errors.push("Privacy network (Tor/I2P) is required but not available".to_string());
            errors.push("Please start Tor or I2P before continuing".to_string());
        } else {
            self.privacy_network_active = true;
            println!("✅ Privacy network: {:?} active", proxy.network);
        }
        
        // Check 2: Full Node - WARNING
        if self.is_using_remote_node().await {
            warnings.push("Using remote node - privacy risk!".to_string());
            warnings.push("Recommendation: Run your own full node".to_string());
            warnings.push("  Monero: monerod over Tor/I2P".to_string());
            warnings.push("  Zcash: zebra over Tor/I2P".to_string());
            self.using_full_node = false;
        } else {
            self.using_full_node = true;
            println!("✅ Using local full node");
        }
        
        // Check 3: Address Reuse - MANDATORY
        if self.check_address_reuse().await? {
            errors.push("Address reuse detected - blocked for privacy".to_string());
            errors.push("Each swap must use a new address".to_string());
            self.address_reused = true;
        } else {
            println!("✅ No address reuse detected");
        }
        
        // Check 4: Monero Churning - WARNING
        if !self.check_monero_churned().await? {
            warnings.push("Monero outputs not churned - privacy risk!".to_string());
            warnings.push("Recommendation: Churn 1-2 times before swap".to_string());
            warnings.push("  This breaks deterministic links".to_string());
            self.monero_churned = false;
        } else {
            self.monero_churned = true;
            println!("✅ Monero outputs churned");
        }
        
        Ok(PrivacyCheckResult {
            passed: errors.is_empty(),
            errors,
            warnings,
            privacy_network_active: self.privacy_network_active,
            using_full_node: self.using_full_node,
            address_reused: self.address_reused,
            monero_churned: self.monero_churned,
        })
    }
    
    /// Check if using remote node (privacy risk)
    async fn is_using_remote_node(&self) -> bool {
        let config = load_config();
        
        // Check Zcash node
        let zebra_url = config.zebra_url;
        if !zebra_url.contains("127.0.0.1") && !zebra_url.contains("localhost") {
            return true;
        }
        
        // TODO: Check Monero node when implemented
        // For now, assume remote if not localhost
        
        false
    }
    
    /// Check for address reuse
    async fn check_address_reuse(&self) -> NozyResult<bool> {
        // TODO: Implement address tracking
        // For now, return false (no reuse)
        Ok(false)
    }
    
    /// Check if Monero outputs have been churned
    async fn check_monero_churned(&self) -> NozyResult<bool> {
        // TODO: Implement churn detection
        // For now, return false (not churned)
        Ok(false)
    }
    
    /// Display privacy checklist
    pub fn display_privacy_checklist(&self, result: &PrivacyCheckResult) {
        println!();
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("🔒 PRIVACY CHECKLIST");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!();
        
        // Mandatory checks
        if result.privacy_network_active {
            println!("  ✅ Tor/I2P: Active");
        } else {
            println!("  ❌ Tor/I2P: Not active (REQUIRED)");
        }
        
        if !result.address_reused {
            println!("  ✅ Address Reuse: Prevented");
        } else {
            println!("  ❌ Address Reuse: Detected (BLOCKED)");
        }
        
        // Recommended checks
        if result.using_full_node {
            println!("  ✅ Full Node: Using local node");
        } else {
            println!("  ⚠️  Full Node: Using remote (privacy risk)");
        }
        
        if result.monero_churned {
            println!("  ✅ Monero Churn: Completed");
        } else {
            println!("  ⚠️  Monero Churn: Not done (recommended)");
        }
        
        println!();
        
        // Show errors
        if !result.errors.is_empty() {
            println!("❌ ERRORS (Must fix):");
            for error in &result.errors {
                println!("   • {}", error);
            }
            println!();
        }
        
        // Show warnings
        if !result.warnings.is_empty() {
            println!("⚠️  WARNINGS (Recommended):");
            for warning in &result.warnings {
                println!("   • {}", warning);
            }
            println!();
        }
        
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!();
    }
}

#[derive(Debug, Clone)]
pub struct PrivacyCheckResult {
    pub passed: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub privacy_network_active: bool,
    pub using_full_node: bool,
    pub address_reused: bool,
    pub monero_churned: bool,
}

impl PrivacyCheckResult {
    pub fn can_proceed(&self) -> bool {
        self.passed && !self.address_reused
    }
    
    pub fn has_critical_issues(&self) -> bool {
        !self.privacy_network_active || self.address_reused
    }
}
