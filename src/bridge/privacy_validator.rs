// Privacy Validator for Bridge Operations


use crate::error::NozyResult;
use crate::privacy_network::proxy::ProxyConfig;
use crate::config::load_config;
use crate::monero::transaction_history::MoneroTransactionStorage;
use chrono::{Utc, Duration};
use std::collections::HashMap;

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
    
    pub async fn validate_privacy_requirements(&mut self) -> NozyResult<PrivacyCheckResult> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        
        let proxy = ProxyConfig::auto_detect().await;
        if !proxy.enabled {
            errors.push("Privacy network (Tor/I2P) is required but not available".to_string());
            errors.push("Please start Tor or I2P before continuing".to_string());
        } else {
            self.privacy_network_active = true;
            println!("✅ Privacy network: {:?} active", proxy.network);
        }
        
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
        
        if self.check_address_reuse().await? {
            errors.push("Address reuse detected - blocked for privacy".to_string());
            errors.push("Each swap must use a new address".to_string());
            self.address_reused = true;
        } else {
            println!("✅ No address reuse detected");
        }
        
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
    
    async fn is_using_remote_node(&self) -> bool {
        let config = load_config();
        
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
        // Check transaction history for address reuse
        // Address reuse is a privacy violation - each swap should use a unique address
        let tx_storage = MoneroTransactionStorage::new()?;
        let transactions = tx_storage.get_all_transactions();
        
        // Count how many times each address has been used as a recipient
        let mut address_usage: HashMap<String, usize> = HashMap::new();
        for tx in transactions {
            if tx.status == crate::monero::transaction_history::MoneroTransactionStatus::Confirmed {
                *address_usage.entry(tx.recipient_address).or_insert(0) += 1;
            }
        }
        
        // Check if any address has been used more than once (reuse detected)
        let has_reuse = address_usage.values().any(|&count| count > 1);
        
        Ok(has_reuse)
    }
    
    /// Check if Monero outputs have been churned
    async fn check_monero_churned(&self) -> NozyResult<bool> {
        // Check transaction history for churn transactions
        // A churn transaction is typically a self-send (recipient == sender's address)
        // We detect this by looking for addresses that appear multiple times as recipients
        // within a recent time window, which indicates churning activity
        
        let tx_storage = MoneroTransactionStorage::new()?;
        let transactions = tx_storage.get_all_transactions();
        
        // Filter to only confirmed transactions from the last 48 hours
        let cutoff_time = Utc::now() - Duration::hours(48);
        let recent_txs: Vec<_> = transactions
            .into_iter()
            .filter(|tx| {
                tx.status == crate::monero::transaction_history::MoneroTransactionStatus::Confirmed
                    && tx.block_time.map_or(false, |time| time >= cutoff_time)
            })
            .collect();
        
        if recent_txs.is_empty() {
            return Ok(false);
        }
        
        // Check if there are multiple transactions to the same address (potential churn)
        // Churning typically involves sending to your own address multiple times
        let mut address_counts: HashMap<String, usize> = HashMap::new();
        for tx in &recent_txs {
            *address_counts.entry(tx.recipient_address.clone()).or_insert(0) += 1;
        }
        
        // If we have multiple transactions to the same address, it's likely churning
        // Also check if we have at least 1-2 recent confirmed transactions (heuristic)
        let has_multiple_to_same = address_counts.values().any(|&count| count >= 2);
        let has_recent_activity = recent_txs.len() >= 1;
        
        // Consider churned if we have recent activity and multiple transactions to same address
        // OR if we have at least 2 recent transactions (indicating churn activity)
        Ok(has_recent_activity && (has_multiple_to_same || recent_txs.len() >= 2))
    }
    
    pub fn display_privacy_checklist(&self, result: &PrivacyCheckResult) {
        println!();
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("🔒 PRIVACY CHECKLIST");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!();
        
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
        
        if !result.errors.is_empty() {
            println!("❌ ERRORS (Must fix):");
            for error in &result.errors {
                println!("   • {}", error);
            }
            println!();
        }
        
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
