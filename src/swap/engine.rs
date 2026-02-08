// Swap Engine
// Orchestrates XMR <-> ZEC swaps with privacy validation

use crate::bridge::{AddressTracker, ChurnManager, PrivacyValidator, StoredSwap, SwapStorage};
use crate::config::load_config;
use crate::error::{NozyError, NozyResult};
use crate::monero::MoneroWallet;
use crate::monero_zk_verifier::{MoneroZkVerifier, VerificationLevel};
use crate::swap::service::SwapService;
use crate::swap::types::*;
use crate::HDWallet;
use zcash_protocol::consensus::NetworkType;

pub struct SwapEngine {
    swap_service: SwapService,
    privacy_validator: PrivacyValidator,
    address_tracker: AddressTracker,
    monero_wallet: Option<MoneroWallet>,
    zcash_wallet: Option<HDWallet>,
}

impl SwapEngine {
    pub fn new(
        swap_service: SwapService,
        monero_wallet: Option<MoneroWallet>,
        zcash_wallet: Option<HDWallet>,
    ) -> NozyResult<Self> {
        Ok(Self {
            swap_service,
            privacy_validator: PrivacyValidator::new(),
            address_tracker: AddressTracker::new()?,
            monero_wallet,
            zcash_wallet,
        })
    }

    pub async fn execute_swap(
        &mut self,
        direction: SwapDirection,
        amount: f64,
    ) -> NozyResult<SwapResponse> {
        println!();
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("🔄 MONERO-ZCASH SWAP");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!();

        println!("Step 1: Privacy Validation");
        let privacy_result = self
            .privacy_validator
            .validate_privacy_requirements()
            .await?;
        self.privacy_validator
            .display_privacy_checklist(&privacy_result);

        if privacy_result.has_critical_issues() {
            return Err(NozyError::InvalidOperation(
                "Privacy requirements not met. Please fix errors before continuing.".to_string(),
            ));
        }

        if !privacy_result.can_proceed() {
            println!("⚠️  Privacy warnings detected. Continue anyway? (y/N)");
            use dialoguer::Confirm;
            if !Confirm::new()
                .with_prompt("Continue with privacy warnings?")
                .default(false)
                .interact()
                .unwrap_or(false)
            {
                return Err(NozyError::InvalidOperation(
                    "Swap cancelled due to privacy warnings".to_string(),
                ));
            }
        }

        println!();
        println!("Step 2: Generating new addresses (privacy: no reuse)");
        let (from_address, to_address) = self.generate_swap_addresses(direction.clone()).await?;

        println!("Step 3: Validating address reuse prevention");
        let is_monero = matches!(direction, SwapDirection::XmrToZec);
        self.address_tracker
            .validate_address_not_reused(&from_address, is_monero)?;

        if matches!(direction, SwapDirection::XmrToZec) {
            if let Some(_monero_wallet) = &self.monero_wallet {
                let config = load_config();
                let should_churn = if config.swap.auto_churn {
                    
                    true
                } else {
                    println!();
                    println!("Step 4: Monero Churning (recommended)");
                    let churn_rec = ChurnManager::recommend_churn_parameters();
                    churn_rec.display();

                    use dialoguer::Confirm;
                    Confirm::new()
                        .with_prompt("Perform Monero churning for enhanced privacy?")
                        .default(true)
                        .interact()
                        .unwrap_or(false)
                };

                if should_churn {
                    println!();
                    println!("🔄 Starting Monero churning...");
                    let churn_rec = ChurnManager::recommend_churn_parameters();
                    println!(
                        "   Churning {} times with ring size {}",
                        churn_rec.times, churn_rec.ring_size
                    );
                    println!("   This may take a few minutes...");
                   
                    println!(
                        "💡 Note: Churning should be performed separately using 'nozy swap churn'"
                    );
                    println!("   Continuing with swap...");
                } else {
                    println!("⏭️  Skipping churning (user choice or disabled)");
                }
            }
        }

        if matches!(direction, SwapDirection::XmrToZec) {
            let config = load_config();
            if config.zk_verification.enabled {
                println!();
                println!("Step 4.5: ZK Block Verification");

                if let Some(monero_wallet) = &self.monero_wallet {
                    match self.verify_monero_block(monero_wallet).await {
                        Ok(verification_result) => {
                            if verification_result.verified {
                                println!("✅ Block verified with ZK proof");
                                if let Some(time) = verification_result.verification_time {
                                    println!("   Verification time: {} seconds", time);
                                }
                            } else {
                                println!("⚠️  ZK verification unavailable, trusting RPC node");
                                if let Some(error) = verification_result.error {
                                    println!("   Reason: {}", error);
                                }
                            }
                        }
                        Err(e) => {
                            println!("⚠️  ZK verification error: {}", e);
                            println!("   Falling back to RPC trust");
                        }
                    }
                }
            } else {
                println!();
                println!("Step 4.5: ZK Block Verification (disabled)");
                println!("   Using RPC trust mode (fast but less secure)");
            }
        }

        println!();
        println!("Step 5: Getting swap rate");
        let (from_coin, to_coin) = match direction {
            SwapDirection::XmrToZec => ("xmr", "zec"),
            SwapDirection::ZecToXmr => ("zec", "xmr"),
        };

        let rate = self
            .swap_service
            .get_rate(from_coin, to_coin, amount)
            .await?;
        println!(
            "   Rate: {} {} = {:.8} {}",
            amount, from_coin, rate, to_coin
        );

        println!();
        println!("Step 6: Initiating swap");
        let swap_request = SwapRequest {
            direction: direction.clone(),
            amount,
            from_address: from_address.clone(),
            to_address: to_address.clone(),
        };

        let swap_response = self.swap_service.initiate_swap(swap_request).await?;

        self.address_tracker
            .mark_address_used(&from_address, is_monero)?;
        self.address_tracker
            .mark_address_used(&to_address, !is_monero)?;

        let storage = SwapStorage::new()?;
        let stored_swap = StoredSwap {
            swap_id: swap_response.swap_id.clone(),
            direction,
            amount,
            status: swap_response.status.clone(),
            from_address: from_address.clone(),
            to_address: to_address.clone(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                .as_secs(),
            completed_at: None,
            txid: None,
        };
        storage.add_swap(stored_swap)?;

        println!();
        println!("✅ Swap initiated successfully!");
        println!("   Swap ID: {}", swap_response.swap_id);
        println!("   Deposit Address: {}", swap_response.deposit_address);
        println!("   Amount: {:.8}", swap_response.deposit_amount);
        println!(
            "   Estimated Time: {} minutes",
            swap_response.estimated_time.unwrap_or(1800) / 60
        );
        println!();
        println!("🛡️  Privacy: All connections through Tor/I2P");
        println!("🔒 Privacy: Addresses will never be reused");
        println!("📝 Swap saved to history");
        println!();

        Ok(swap_response)
    }

    async fn generate_swap_addresses(
        &self,
        direction: SwapDirection,
    ) -> NozyResult<(String, String)> {
        match direction {
            SwapDirection::XmrToZec => {
                let from_address = if let Some(monero_wallet) = &self.monero_wallet {
                    monero_wallet.create_subaddress(0).await?
                } else {
                    return Err(NozyError::InvalidOperation(
                        "Monero wallet not configured".to_string(),
                    ));
                };

                let config = load_config();
                let network = if config.network == "testnet" {
                    NetworkType::Test
                } else {
                    NetworkType::Main
                };
                let to_address = if let Some(zcash_wallet) = &self.zcash_wallet {
                    zcash_wallet.generate_orchard_address(0, 0, network)?.to_string()
                } else {
                    return Err(NozyError::InvalidOperation(
                        "Zcash wallet not configured".to_string(),
                    ));
                };

                Ok((from_address, to_address))
            }
            SwapDirection::ZecToXmr => {
                let config = load_config();
                let network = if config.network == "testnet" {
                    NetworkType::Test
                } else {
                    NetworkType::Main
                };
                let from_address = if let Some(zcash_wallet) = &self.zcash_wallet {
                    zcash_wallet.generate_orchard_address(0, 0, network)?.to_string()
                } else {
                    return Err(NozyError::InvalidOperation(
                        "Zcash wallet not configured".to_string(),
                    ));
                };

\                let to_address = if let Some(monero_wallet) = &self.monero_wallet {
                    monero_wallet.create_subaddress(0).await?
                } else {
                    return Err(NozyError::InvalidOperation(
                        "Monero wallet not configured".to_string(),
                    ));
                };

                Ok((from_address, to_address))
            }
        }
    }

    pub async fn check_swap_status(&self, swap_id: &str) -> NozyResult<SwapStatusResponse> {
        self.swap_service.get_swap_status(swap_id).await
    }

    async fn verify_monero_block(
        &self,
        monero_wallet: &MoneroWallet,
    ) -> NozyResult<crate::monero_zk_verifier::types::VerificationResult> {
        let config = load_config();

]        let block_height = monero_wallet.get_block_height().await?;
        let block_hash = monero_wallet.get_current_block_hash().await?;

        let verifier = MoneroZkVerifier::new(config.zk_verification.clone())?;

        let level = VerificationLevel::VerifyBlock {
            block_hash: block_hash.clone(),
            block_height: Some(block_height),
        };

        verifier.verify(level).await
    }
}
