use std::path::PathBuf;
use clap::{Parser, Subcommand};
use dialoguer::{Password, Confirm};
use nozy::{HDWallet, WalletStorage, NozyResult, NozyError, NoteScanner, ZebraClient};
use nozy::local_analytics::LocalAnalytics;
use nozy::cli_helpers::{load_wallet, scan_notes_for_sending, build_and_broadcast_transaction, handle_insufficient_funds_error};
use nozy::{load_config, save_config, update_last_scan_height};
use zcash_address::unified::{Encoding, Container};

#[derive(Parser)]
#[command(name = "nozy")]
#[command(about = "A privacy-focused Zcash wallet")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    
    New,

    Restore,
    
    Receive,
    
    Sync {
        #[arg(long)]
        start_height: Option<u32>,
        #[arg(long)]
        end_height: Option<u32>,
        #[arg(long)]
        zebra_url: Option<String>,
    },
   
    Send {
        #[arg(long)]
        recipient: String,
        #[arg(long)]
        amount: f64,
        #[arg(long)]
        zebra_url: Option<String>,
        #[arg(long)]
        memo: Option<String>,
    },
   
    Info,
    
    Balance,
    
    Config {
        #[arg(long)]
        set_zebra_url: Option<String>,
        #[arg(long)]
        set_network: Option<String>,
    },
    
    TestZebra {
        #[arg(long)]
        zebra_url: Option<String>,
    },
    
    
    Proving {
        
        #[arg(long)]
        download: bool,
        #[arg(long)]
        status: bool,
    },
                        
    Analytics,
}


#[tokio::main]
async fn main() -> NozyResult<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::New => {
            println!("🔐 Creating new wallet...");
            
            let mut wallet = HDWallet::new()?;
            
            let use_password = Confirm::new()
                .with_prompt("Do you want to set a password for this wallet?")
                .default(true)
                .interact()
                .map_err(|e| NozyError::InvalidOperation(format!("Input error: {}", e)))?;
            
            if use_password {
                let password = Password::new()
                    .with_prompt("Enter password for wallet encryption")
                    .with_confirmation("Confirm password", "Passwords don't match")
                    .interact()
                    .map_err(|e| NozyError::InvalidOperation(format!("Password input error: {}", e)))?;
                
                wallet.set_password(&password)?;
                println!("✅ Password protection enabled");
            } else {
                println!("⚠️  Wallet will be stored without password protection");
            }
            
            let storage = WalletStorage::new(PathBuf::from("wallet_data"));
            storage.save_wallet(&wallet, "").await?;
            
            println!("🎉 Wallet created successfully!");
            println!("📝 Mnemonic: {}", wallet.get_mnemonic());
            println!("⚠️  IMPORTANT: Save this mnemonic in a safe place!");
            println!("   It's the only way to recover your wallet if you lose access.");
            
            match wallet.generate_orchard_address(0, 0) {
                Ok(address) => {
                    println!("📍 Your wallet address:");
                    println!("{}", address);
                    println!("\n💡 Share this address to receive ZEC.");
                },
                Err(e) => println!("❌ Failed to generate address: {}", e),
            }
        }
        
        Commands::Restore => {
            println!("Restore wallet from mnemonic...");
            use dialoguer::Input;
            let mnemonic: String = Input::new()
                .with_prompt("Enter your 24-word mnemonic")
                .with_initial_text("")
                .interact_text()
                .map_err(|e| nozy::NozyError::InvalidOperation(format!("Mnemonic input error: {}", e)))?;
            
            let wallet = HDWallet::from_mnemonic(&mnemonic)?;
            let storage = WalletStorage::new(PathBuf::from("wallet_data"));
            
            let password = Password::new()
                .with_prompt("Enter password to encrypt wallet")
                .interact()
                .map_err(|e| nozy::NozyError::InvalidOperation(format!("Password input error: {}", e)))?;
            
            storage.save_wallet(&wallet, &password).await?;
            println!("✅ Wallet restored and saved.");
        }
        
        Commands::Receive => {
            let (wallet, _storage) = load_wallet().await?;
            
            match wallet.generate_orchard_address(0, 0) {
                Ok(address) => {
                    println!("Your wallet address:");
                    println!("{}", address);
                    println!("\n💡 Share this address to receive ZEC.");
                    println!("   All funds go to your wallet automatically.");
                },
                Err(e) => {
                    println!("❌ Failed to generate address: {}", e);
                }
            }
        }
        
        Commands::Sync { start_height, end_height, zebra_url } => {
            let config = load_config();
            let zebra_url = zebra_url.unwrap_or_else(|| config.zebra_url.clone());
            
            println!("Updating your wallet balance...");
            
            let (wallet, _storage) = load_wallet().await?;
            let zebra_client = ZebraClient::new(zebra_url);
            
            let effective_start = start_height.or(config.last_scan_height);
            
            let mut note_scanner = NoteScanner::new(wallet, zebra_client.clone());
            
            match note_scanner.scan_notes(effective_start, end_height).await {
                Ok((result, _spendable_notes)) => {
                    use std::fs;
                    use std::path::Path;
                    let notes_dir = Path::new("wallet_data");
                    if !notes_dir.exists() { let _ = fs::create_dir_all(notes_dir); }
                    let notes_path = notes_dir.join("notes.json");
                    if let Ok(serialized) = serde_json::to_string_pretty(&result.notes) {
                        let _ = fs::write(&notes_path, serialized);
                    }
                    
                    if let Some(end) = end_height {
                        let _ = update_last_scan_height(end);
                    } else {
                        if let Ok(block_count) = zebra_client.get_block_count().await {
                            let _ = update_last_scan_height(block_count);
                        }
                    }
                    
                    let balance_zec = result.total_balance as f64 / 100_000_000.0;
                    
                    if result.total_balance > 0 {
                        println!("✅ Your balance: {:.8} ZEC", balance_zec);
                    } else {
                        println!("💰 Your balance: 0.00000000 ZEC");
                        println!("   Sync completed. Balance will update when you receive ZEC.");
                    }
                },
                Err(e) => {
                    println!("❌ Error updating wallet: {}", e);
                }
            }
        }
        
        Commands::Send { recipient, amount, zebra_url, memo } => {
            let config = load_config();
            let zebra_url = zebra_url.unwrap_or_else(|| config.zebra_url.clone());
            
            let (wallet, _storage) = load_wallet().await?;
            
            match zcash_address::unified::Address::decode(&recipient) {
                Ok((_, ua)) => {
                    let mut has_orchard = false;
                    for item in ua.items() {
                        if let zcash_address::unified::Receiver::Orchard(_) = item { has_orchard = true; break; }
                    }
                    if !has_orchard {
                        return Err(NozyError::AddressParsing("Recipient must include an Orchard receiver".to_string()));
                    }
                },
                Err(e) => {
                    return Err(NozyError::AddressParsing(format!("Invalid recipient address: {}", e)));
                }
            }
            
            println!("Sending {} ZEC to {}", amount, recipient);
            println!("⚠️  This will send REAL ZEC. Continue? (y/N)");
            
            use std::io::{self, Write};
            print!("> ");
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let enable_broadcast = input.trim().to_lowercase() == "y" || input.trim().to_lowercase() == "yes";
            
            if !enable_broadcast {
                println!("❌ Cancelled.");
                return Ok(());
            }
            
            let spendable_notes = scan_notes_for_sending(wallet, &zebra_url).await?;
            
            let memo_bytes_opt = if let Some(m) = memo.as_ref() {
                let trimmed = m.trim();
                if trimmed.is_empty() { None } else { Some(trimmed.as_bytes().to_vec()) }
            } else {
                print!("Enter memo (optional, press Enter to skip): ");
                io::stdout().flush().unwrap();
                let mut memo_input = String::new();
                let _ = io::stdin().read_line(&mut memo_input);
                let trimmed = memo_input.trim();
                if trimmed.is_empty() { None } else { Some(trimmed.as_bytes().to_vec()) }
            };
            
            let amount_zatoshis = (amount * 100_000_000.0) as u64;
            let fee_zatoshis = 10_000;
            let zebra_client = ZebraClient::new(zebra_url.clone());
            
            use std::fs;
            use std::path::Path;
            let notes_path = Path::new("wallet_data/notes.json");
            let balance_before = if notes_path.exists() {
                match fs::read_to_string(notes_path) {
                    Ok(content) => {
                        match serde_json::from_str::<serde_json::Value>(&content) {
                            Ok(parsed) => {
                                parsed.as_array()
                                    .unwrap_or(&vec![])
                                    .iter()
                                    .filter_map(|n| n.get("value").and_then(|v| v.as_u64()))
                                    .sum::<u64>()
                            },
                            Err(_) => 0,
                        }
                    },
                    Err(_) => 0,
                }
            } else {
                0
            };
            
            println!("Processing...");
            
            match build_and_broadcast_transaction(
                &zebra_client,
                &spendable_notes,
                &recipient,
                amount_zatoshis,
                fee_zatoshis,
                memo_bytes_opt.as_deref(),
                enable_broadcast,
                &zebra_url,
            ).await {
                Ok(_) => {
                    let amount_with_fee = amount_zatoshis + fee_zatoshis;
                    let balance_after = balance_before.saturating_sub(amount_with_fee);
                    
                    println!("✅ Sent {} ZEC", amount);
                    println!("💰 Your balance: {:.8} ZEC", balance_after as f64 / 100_000_000.0);
                },
                Err(e) => {
                    println!("❌ Failed to send: {}", e);
                    handle_insufficient_funds_error(&e);
                }
            }
        }
        
        Commands::Balance => {
            use std::fs;
            use std::path::Path;
            let notes_path = Path::new("wallet_data/notes.json");
            if !notes_path.exists() {
                println!("💰 Your balance: 0.00000000 ZEC");
                println!("   Run 'sync' to update your balance.");
            } else {
                match fs::read_to_string(notes_path) {
                    Ok(content) => {
                        let parsed: serde_json::Value = match serde_json::from_str(&content) {
                            Ok(v) => v,
                            Err(_) => { 
                                println!("💰 Your balance: 0.00000000 ZEC");
                                return Ok(());
                            }
                        };
                        let total_zat: u64 = parsed.as_array()
                            .unwrap_or(&vec![])
                            .iter()
                            .filter_map(|n| n.get("value").and_then(|v| v.as_u64()))
                            .sum();
                        println!("💰 Your balance: {:.8} ZEC", total_zat as f64 / 100_000_000.0);
                    },
                    Err(_) => {
                        println!("💰 Your balance: 0.00000000 ZEC");
                    },
                }
            }
        }
        
        Commands::Info => {
            let (wallet, _storage) = load_wallet().await?;
            println!("Wallet loaded successfully!");
            println!("Mnemonic: {}", wallet.get_mnemonic());
        }
        
        Commands::Config { set_zebra_url, set_network } => {
            let mut config = load_config();
            
            if let Some(ref url) = set_zebra_url {
                config.zebra_url = url.clone();
                save_config(&config)?;
                println!("✅ Zebra URL set to: {}", url);
            }
            
            if let Some(ref network) = set_network {
                config.network = network.clone();
                save_config(&config)?;
                println!("✅ Network set to: {}", network);
            }
            
            if set_zebra_url.is_none() && set_network.is_none() {
                println!("Current configuration:");
                println!("  Zebra URL: {}", config.zebra_url);
                println!("  Network: {}", config.network);
                if let Some(last_height) = config.last_scan_height {
                    println!("  Last scanned height: {}", last_height);
                } else {
                    println!("  Last scanned height: (none)");
                }
                println!("\nTo change settings:");
                println!("  nozy config --set-zebra-url <url>");
                println!("  nozy config --set-network mainnet|testnet");
            }
        }
        
        Commands::TestZebra { zebra_url } => {
            let config = load_config();
            let zebra_url = zebra_url.unwrap_or_else(|| config.zebra_url.clone());
            
            println!("🔗 Testing Zebra node connection...");
            println!("📡 Connecting to: {}", zebra_url);
            
            match std::process::Command::new("curl")
                .arg("-s")
                .arg("-X")
                .arg("POST")
                .arg("-H")
                .arg("Content-Type: application/json")
                .arg("-d")
                .arg(r#"{"jsonrpc":"2.0","id":"test","method":"getinfo"}"#)
                .arg(&zebra_url)
                .output()
            {
                Ok(output) => {
                    if output.status.success() {
                        let response = String::from_utf8_lossy(&output.stdout);
                        println!("✅ Zebra node is ONLINE!");
                        println!("📨 Response: {}", response);
                        
                        if response.contains("result") {
                            println!("🎉 Zebra RPC is working correctly!");
                            println!("✅ Ready for mainnet transactions!");
                        }
                    } else {
                        let error = String::from_utf8_lossy(&output.stderr);
                        println!("❌ Zebra connection failed: {}", error);
                    }
                },
                Err(e) => {
                    println!("❌ Cannot test connection: {}", e);
                    println!("💡 Make sure curl is installed or test manually:");
                    println!("   curl -X POST -H \"Content-Type: application/json\" \\");
                    println!("        -d '{{\"jsonrpc\":\"1.0\",\"id\":\"test\",\"method\":\"getinfo\"}}' \\");
                    println!("        {}", zebra_url);
                }
            }
        }
        Commands::Proving { download, status } => {
            use nozy::orchard_tx::OrchardTransactionBuilder;
            
            println!("🔧 Orchard Proving Parameters Management");
            println!("=====================================");
            
            let mut builder = OrchardTransactionBuilder::new_async(download).await?;
            
            if status {
                let proving_status = builder.get_proving_status();
                println!("\n📊 Proving Status:");
                println!("   Spend Parameters: {}", if proving_status.spend_params { "✅" } else { "❌" });
                println!("   Output Parameters: {}", if proving_status.output_params { "✅" } else { "❌" });
                println!("   Spend Verifying Key: {}", if proving_status.spend_vk { "✅" } else { "❌" });
                println!("   Output Verifying Key: {}", if proving_status.output_vk { "✅" } else { "❌" });
                println!("   Can Prove: {}", if proving_status.can_prove { "✅" } else { "❌" });
                
                if let Some(key_info) = builder.get_proving_key_info() {
                    println!("\n🔑 Proving Key Info:");
                    println!("   {}", key_info);
                }
            }
            
            if download {
                println!("\n📥 Downloading proving parameters...");
                match builder.download_parameters().await {
                    Ok(_) => {
                        println!("✅ Parameters downloaded successfully!");
                        println!("💡 Note: These are placeholder parameters for testing");
                        println!("   Replace with real parameters for production use");
                    },
                    Err(e) => {
                        println!("❌ Failed to download parameters: {}", e);
                    }
                }
            }
            
            if !status && !download {
                println!("\n💡 Use --status to check proving parameters");
                println!("💡 Use --download to download placeholder parameters");
            }
        }
        
        Commands::Analytics => {
            let analytics_path = PathBuf::from("wallet_data/analytics.json");
            let mut analytics = LocalAnalytics::load_from_file(&analytics_path)?;
            
            analytics.track_command("analytics");
            analytics.save_to_file(&analytics_path)?;
            
            println!("{}", analytics.generate_summary());
            
            if Confirm::new()
                .with_prompt("Export anonymized data for development insights? (completely anonymous)")
                .interact()
                .unwrap_or(false) {
                
                match analytics.export_anonymized() {
                    Ok(anonymized_json) => {
                        std::fs::write("anonymized_analytics.json", &anonymized_json)
                            .map_err(|e| NozyError::Storage(format!("Failed to write anonymized analytics: {}", e)))?;
                        println!("✅ Anonymized data exported to anonymized_analytics.json");
                        println!("💡 This data contains NO personal information");
                        println!("💡 You can share this with developers to help improve NozyWallet");
                    },
                    Err(e) => {
                        println!("❌ Failed to export anonymized data: {}", e);
                    }
                }
            }
        }
    }
    
    Ok(())
}
