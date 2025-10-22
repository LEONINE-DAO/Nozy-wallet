use std::path::PathBuf;
use clap::{Parser, Subcommand};
use dialoguer::{Password, Confirm};
use nozy::{HDWallet, WalletStorage, NozyResult, NozyError, NoteScanner, ZebraClient, ZcashTransactionBuilder};

#[derive(Parser)]
#[command(name = "nozy")]
#[command(about = "A privacy-focused Zcash wallet")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new wallet
    /// 
    /// Example:
    ///   nozy new
    New,
    /// Restore wallet from mnemonic
    /// 
    /// Example:
    ///   nozy restore
    Restore,
    /// Generate new addresses
    /// 
    /// Examples:
    ///   nozy addresses                    # Generate 1 address
    ///   nozy addresses --count 5         # Generate 5 addresses
    Addresses {
        #[arg(short, long, default_value_t = 1)]
        count: u32,
    },
    /// Scan for notes
    /// 
    /// Examples:
    ///   nozy scan                                    # Scan recent blocks
    ///   nozy scan --start-height 2800000            # Scan from specific height
    ///   nozy scan --start-height 2800000 --end-height 2900000  # Scan specific range
    Scan {
        #[arg(long)]
        start_height: Option<u32>,
        #[arg(long)]
        end_height: Option<u32>,
    },
    /// Send ZEC
    /// 
    /// Examples:
    ///   nozy send --recipient u1abc123... --amount 0.001                    # Send 0.001 ZEC
    ///   nozy send --recipient u1abc123... --amount 0.001 --zebra-url http://127.0.0.1:18232  # Use testnet
    Send {
        /// Recipient address (u1, zs1, or t1 format)
        #[arg(long)]
        recipient: String,
        /// Amount in ZEC
        #[arg(long)]
        amount: f64,
        /// Custom Zebra node URL (optional)
        #[arg(long, default_value = "http://127.0.0.1:8232")]
        zebra_url: String,
    },
    /// Show wallet info
    /// 
    /// Example:
    ///   nozy info
    Info,
    /// Test Zebra node connection
    /// 
    /// Examples:
    ///   nozy test-zebra                           # Test default node
    ///   nozy test-zebra --zebra-url http://127.0.0.1:18232  # Test testnet node
    TestZebra {
        /// Zebra node URL to test
        #[arg(long, default_value = "http://127.0.0.1:8232")]
        zebra_url: String,
    },
    /// List stored notes
    /// 
    /// Example:
    ///   nozy list-notes
    ListNotes,
    /// Manage Orchard proving parameters
    /// 
    /// Examples:
    ///   nozy proving --status                    # Check proving status
    ///   nozy proving --download                  # Download proving parameters
    Proving {
        /// Download proving parameters
        #[arg(long)]
        download: bool,
        /// Show proving status
        #[arg(long)]
        status: bool,
    },
}

async fn load_wallet() -> NozyResult<(HDWallet, WalletStorage)> {
    let storage = WalletStorage::new(PathBuf::from("wallet_data"));
    
    // Check if wallet file exists
    let wallet_path = std::path::Path::new("wallet_data/wallet.dat");
    if wallet_path.exists() {
        // Load existing wallet from storage
        let password = Password::new()
            .with_prompt("Enter wallet password")
            .interact()
            .map_err(|e| NozyError::InvalidOperation(format!("Password input error: {}", e)))?;
        
        let wallet = storage.load_wallet(&password).await?;
        Ok((wallet, storage))
    } else {
        // No wallet exists - this should only happen for 'new' command
        return Err(NozyError::Storage("No wallet found. Use 'nozy new' or 'nozy restore' to create a wallet first.".to_string()));
    }
}

#[tokio::main]
async fn main() -> NozyResult<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::New => {
            println!("🔐 Creating new wallet...");
            
            // Create new wallet
            let mut wallet = HDWallet::new()?;
            
            // Ask for password protection
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
                Ok(address) => println!("📍 Sample address: {}", address),
                Err(e) => println!("❌ Failed to generate sample address: {}", e),
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
        
        Commands::Addresses { count } => {
            println!("Generating {} addresses...", count);
            let (wallet, _storage) = load_wallet().await?;
            
            // Use bulk generation for better performance
            match wallet.generate_multiple_addresses(0, 0, count) {
                Ok(addresses) => {
                    for (i, address) in addresses.iter().enumerate() {
                        println!("Address {}: {}", i, address);
                    }
                },
                Err(e) => {
                    println!("Failed to generate addresses: {}", e);
                    // Fallback to individual generation
                    for i in 0..count {
                        match wallet.generate_orchard_address(0, i) {
                            Ok(address) => println!("Address {}: {}", i, address),
                            Err(e) => println!("Failed to generate address {}: {}", i, e),
                        }
                    }
                }
            }
        }
        
        Commands::Scan { start_height, end_height } => {
            println!("Scanning blockchain for notes...");
            
            let (wallet, _storage) = load_wallet().await?;
            let zebra_client = ZebraClient::new("http://127.0.0.1:8232".to_string());
            
            // Create note scanner with real implementation
            let mut note_scanner = NoteScanner::new(wallet, zebra_client);
            
            match note_scanner.scan_notes(start_height, end_height).await {
                Ok((result, spendable_notes)) => {
                    // Persist notes for list-notes command
                    use std::fs;
                    use std::path::Path;
                    let notes_dir = Path::new("wallet_data");
                    if !notes_dir.exists() { let _ = fs::create_dir_all(notes_dir); }
                    let notes_path = notes_dir.join("notes.json");
                    if let Ok(serialized) = serde_json::to_string_pretty(&result.notes) {
                        let _ = fs::write(&notes_path, serialized);
                    }
                    
                    println!("Scan complete!");
                    println!("Total notes found: {}", result.notes.len());
                    println!("Total balance: {} zatoshis", result.total_balance);
                    println!("Unspent notes: {}", result.unspent_count);
                    println!("Spendable notes: {}", spendable_notes.len());
                    
                    if result.total_balance > 0 {
                        println!("🎉 Found ZEC in your wallet!");
                        println!("💰 Balance: {} ZEC", result.total_balance as f64 / 100_000_000.0);
                        
                        for (i, note) in result.notes.iter().enumerate() {
                            if !note.spent {
                                println!("  Note {}: {} ZAT (Block: {})", i + 1, note.value, note.block_height);
                            }
                        }
                    } else {
                        println!("💡 No ZEC found in scanned blocks");
                        println!("   Try scanning a wider range or different heights");
                    }
                },
                Err(e) => {
                    println!("Error scanning notes: {}", e);
                }
            }
        }
        
        Commands::Send { recipient, amount, zebra_url } => {
            println!("Sending {} ZEC to {}...", amount, recipient);
            
            let (wallet, _storage) = load_wallet().await?;
            
            let amount_zatoshis = (amount * 100_000_000.0) as u64;
            let fee_zatoshis = 10_000; 
            let mut tx_builder = ZcashTransactionBuilder::new();
            tx_builder.set_zebra_url(&zebra_url);
            
            println!("🚨 MAINNET TRANSACTION DETECTED! 🚨");
            println!("   This will send REAL ZEC on the mainnet blockchain!");
            println!("   Zebra node: {}", zebra_url);
            println!("   Do you want to enable mainnet broadcasting? (y/N)");
            
            use std::io::{self, Write};
            print!("Enter 'yes' to enable: ");
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let input = input.trim().to_lowercase();
            
            if input == "yes" || input == "y" {
                tx_builder.enable_mainnet_broadcast();
                println!("✅ Mainnet broadcasting enabled!");
            } else {
                println!("❌ Mainnet broadcasting disabled for safety.");
                println!("   Transaction will be built but not broadcast.");
            }
            
            // Scan recent blocks for spendable notes before sending
            println!("🔎 Scanning recent blocks for spendable notes...");
            let zebra_client = ZebraClient::new(zebra_url.clone());
            let tip_height = match zebra_client.get_block_count().await {
                Ok(h) => h,
                Err(e) => {
                    println!("⚠️  Could not fetch tip height from Zebra ({}). Falling back to 10k-block scan ending at default.", e);
                    3_066_071
                }
            };
            let start_height = tip_height.saturating_sub(10_000);
            let mut note_scanner = NoteScanner::new(wallet, ZebraClient::new("http://127.0.0.1:8232".to_string()));
            let spendable_notes: Vec<nozy::SpendableNote> = match note_scanner.scan_notes(Some(start_height), Some(tip_height)).await {
                Ok((_result, spendable)) => spendable,
                Err(e) => {
                    println!("⚠️  Note scan failed: {}. Proceeding with empty note set.", e);
                    Vec::new()
                }
            };
            
            // Optional memo input
            print!("Enter memo (optional, press Enter to skip): ");
            io::stdout().flush().unwrap();
            let mut memo_input = String::new();
            let _ = io::stdin().read_line(&mut memo_input);
            let memo_bytes_opt = {
                let trimmed = memo_input.trim().as_bytes();
                if trimmed.is_empty() { None } else { Some(trimmed) }
            };
            
            match tx_builder.build_send_transaction(&zebra_client, &spendable_notes, &recipient, amount_zatoshis, fee_zatoshis, memo_bytes_opt).await {
                Ok(signed_tx) => {
                    println!("Transaction built successfully!");
                    println!("Transaction ID: {}", signed_tx.txid);
                    println!("Transaction size: {} bytes", signed_tx.raw_transaction.len());
                    
                    if tx_builder.allow_mainnet_broadcast {
                        match tx_builder.broadcast_transaction(&signed_tx).await {
                            Ok(txid) => {
                                println!("✅ Transaction broadcast successful!");
                                println!("Final Transaction ID: {}", txid);
                            },
                            Err(e) => {
                                println!("❌ Broadcast failed: {}", e);
                            }
                        }
                    } else {
                        println!("💡 Transaction ready for broadcast (broadcasting disabled)");
                    }
                },
                Err(e) => {
                    println!("Failed to build transaction: {}", e);
                    if e.to_string().contains("Insufficient funds") {
                        println!("💡 Tip: You need to receive some ZEC first before you can send it.");
                        println!("   Use the 'scan' command to check for received notes.");
                    }
                }
            }
        }
        
        Commands::Info => {
            let (wallet, _storage) = load_wallet().await?;
            println!("Wallet loaded successfully!");
            println!("Mnemonic: {}", wallet.get_mnemonic());
        }
        
        Commands::TestZebra { zebra_url } => {
            println!("🔗 Testing Zebra node connection...");
            
            println!("📡 Connecting to: {}", zebra_url);
            
            // Simple test - try to connect to the URL
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
        Commands::ListNotes => {
            use std::fs;
            use std::path::Path;
            use serde_json::Value;
            let notes_path = Path::new("wallet_data/notes.json");
            if notes_path.exists() {
                let content = fs::read_to_string(notes_path)
                    .map_err(|e| nozy::NozyError::Storage(format!("Failed to read notes: {}", e)))?;
                let v: Value = serde_json::from_str(&content)
                    .map_err(|e| nozy::NozyError::Storage(format!("Failed to parse notes: {}", e)))?;
                println!("Stored notes:");
                println!("{}", serde_json::to_string_pretty(&v).unwrap_or_else(|_| "[]".to_string()));
            } else {
                println!("No stored notes yet. Run a scan first.");
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
    }
    
    Ok(())
}
