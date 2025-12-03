use clap::{Parser, Subcommand};
use dialoguer::{Password, Confirm};
use nozy::{HDWallet, WalletStorage, NozyResult, NozyError, NoteScanner, ZebraClient, AddressBook};
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
        #[arg(long)]
        use_local: bool,
        #[arg(long)]
        use_remote: Option<String>,
        #[arg(long)]
        use_crosslink: bool,
        #[arg(long)]
        use_zebra: bool,
        #[arg(long)]
        set_crosslink_url: Option<String>,
        #[arg(long)]
        show_backend: bool,
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
    
    History,
    
    CheckConfirmations {
        #[arg(long)]
        txid: Option<String>,
    },
    
    Status,
    
    AddressBook {
        #[command(subcommand)]
        command: AddressBookCommand,
    },
}

#[derive(Subcommand)]
pub enum AddressBookCommand {
    List,
    Add {
        #[arg(long)]
        name: String,
        #[arg(long)]
        address: String,
        #[arg(long)]
        notes: Option<String>,
    },
    Remove {
        #[arg(long)]
        name: String,
    },
    Get {
        #[arg(long)]
        name: String,
    },
    Search {
        #[arg(long)]
        query: String,
    },
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
            
            let password = if use_password {
                let pwd = Password::new()
                    .with_prompt("Enter password for wallet encryption")
                    .with_confirmation("Confirm password", "Passwords don't match")
                    .interact()
                    .map_err(|e| NozyError::InvalidOperation(format!("Password input error: {}", e)))?;
                
                wallet.set_password(&pwd)?;
                println!("✅ Password protection enabled");
                pwd
            } else {
                println!("⚠️  Wallet will be stored without password protection");
                String::new()
            };
            
            let storage = WalletStorage::with_xdg_dir();
            storage.save_wallet(&wallet, &password).await?;
            
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
            let storage = WalletStorage::with_xdg_dir();
            
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
            let mut config = load_config();

            if let Some(url) = zebra_url {
                config.zebra_url = url;
            }

            let (wallet, _storage) = load_wallet().await?;
            let zebra_client = ZebraClient::from_config(&config);
            
            let effective_start = if let Some(start) = start_height {
                Some(start)
            } else if let Some(last_height) = config.last_scan_height {
                Some(last_height + 1)
            } else {
                None
            };
            
            if let Some(start) = effective_start {
                if let Some(last) = config.last_scan_height {
                    println!("🔄 Incremental sync: scanning from block {} (last sync: {})", start, last);
                } else {
                    println!("🔄 Full sync: scanning from block {}", start);
                }
            } else {
                println!("🔄 Full sync: scanning from default starting point");
            }
            
            let mut note_scanner = NoteScanner::new(wallet, zebra_client.clone());
            
            match note_scanner.scan_notes(effective_start, end_height).await {
                Ok((result, _spendable_notes)) => {
                    use std::fs;
                    use nozy::paths::get_wallet_data_dir;
                    let notes_dir = get_wallet_data_dir();
                    let notes_path = notes_dir.join("notes.json");
                    
                    use nozy::SerializableOrchardNote;
                    let mut existing_notes: Vec<SerializableOrchardNote> = if notes_path.exists() {
                        if let Ok(content) = fs::read_to_string(&notes_path) {
                            serde_json::from_str(&content).unwrap_or_default()
                        } else {
                            Vec::new()
                        }
                    } else {
                        Vec::new()
                    };
                    
                    let existing_nullifiers: std::collections::HashSet<Vec<u8>> = existing_notes.iter()
                        .map(|n| n.nullifier_bytes.clone())
                        .collect();
                    
                    for new_note in &result.notes {
                        if !existing_nullifiers.contains(&new_note.nullifier_bytes) {
                            existing_notes.push(new_note.clone());
                        }
                    }
                    
                    if let Ok(serialized) = serde_json::to_string_pretty(&existing_notes) {
                        let _ = fs::write(&notes_path, serialized);
                    }
                    
                    let final_height = if let Some(end) = end_height {
                        end
                    } else {
                        zebra_client.get_block_count().await.unwrap_or(0)
                    };
                    let _ = update_last_scan_height(final_height);
                    
                    let total_balance: u64 = existing_notes.iter()
                        .map(|n| n.value)
                        .sum();
                    let balance_zec = total_balance as f64 / 100_000_000.0;
                    
                    if total_balance > 0 {
                        println!("✅ Sync complete! Balance: {:.8} ZEC", balance_zec);
                        println!("   Found {} new notes, {} total notes", result.notes.len(), existing_notes.len());
                    } else {
                        println!("✅ Sync complete! Balance: 0.00000000 ZEC");
                        println!("   Found {} new notes", result.notes.len());
                    }
                    println!("   Last scanned height: {}", final_height);
                },
                Err(e) => {
                    println!("❌ Error updating wallet: {}", e);
                }
            }
        }
        
        Commands::Send { recipient, amount, zebra_url, memo } => {
            let mut config = load_config();

            if let Some(url) = zebra_url {
                config.zebra_url = url;
            }

            let (wallet, _storage) = load_wallet().await?;
            
            let address_book = AddressBook::new()?;
            let actual_recipient = if let Some(address) = address_book.get_address_by_name(&recipient) {
                println!("📇 Found '{}' in address book: {}", recipient, address);
                let _ = address_book.update_address_usage(&recipient);
                address
            } else {
                recipient.clone()
            };
            
            if actual_recipient.starts_with("t1") {
                return Err(NozyError::AddressParsing(
                    "Transparent addresses (t1) are not supported. NozyWallet only supports shielded addresses (u1 unified addresses with Orchard receivers) for privacy protection. Please use a shielded address.".to_string()
                ));
            }
            
            match zcash_address::unified::Address::decode(&actual_recipient) {
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

            let zebra_client = ZebraClient::from_config(&config);
            let network = config.network.clone();
            let is_mainnet = network == "mainnet";
            
            let amount_zatoshis = (amount * 100_000_000.0) as u64;
            
            println!("\n📋 Transaction Summary");
            println!("{}", "=".repeat(60));
            println!("  Recipient: {}", actual_recipient);
            println!("  Amount:    {} ZEC", amount);
            if let Some(m) = memo.as_ref() {
                println!("  Memo:      {}", m);
            }
            println!("  Network:   {}", if is_mainnet { "MAINNET ⚠️" } else { "TESTNET" });
            
            println!("\n💸 Estimating transaction fee...");
            let fee_zatoshis = nozy::cli_helpers::estimate_transaction_fee(&zebra_client).await;
            let total_amount = amount_zatoshis + fee_zatoshis;
            println!("  Fee:       {:.8} ZEC", fee_zatoshis as f64 / 100_000_000.0);
            println!("  Total:     {:.8} ZEC (amount + fee)", total_amount as f64 / 100_000_000.0);
            
            use nozy::orchard_tx::OrchardTransactionBuilder;
            let mut tx_builder_check = OrchardTransactionBuilder::new_async(false).await?;
            let proving_status = tx_builder_check.get_proving_status();
            if !proving_status.can_prove {
                return Err(NozyError::InvalidOperation(
                    "Cannot create proofs: proving system not ready. Run 'nozy proving --status' to check.".to_string()
                ));
            }
            println!("  Proving:   ✅ Ready (Halo 2)");
            
            println!("\n🔍 Scanning for spendable notes...");
            let spendable_notes = scan_notes_for_sending(wallet, &config.zebra_url).await?;
            
            if spendable_notes.is_empty() {
                return Err(NozyError::InvalidOperation(
                    "No spendable notes found. Run 'sync' to scan the blockchain first.".to_string()
                ));
            }
            
            let total_available: u64 = spendable_notes.iter()
                .map(|note| note.orchard_note.note.value().inner())
                .sum();
            
            println!("  Available: {:.8} ZEC (from {} notes)", total_available as f64 / 100_000_000.0, spendable_notes.len());
            
            if total_available < total_amount {
                return Err(NozyError::InvalidOperation(format!(
                    "Insufficient funds. Available: {:.8} ZEC, Required: {:.8} ZEC",
                    total_available as f64 / 100_000_000.0,
                    total_amount as f64 / 100_000_000.0
                )));
            }
            
            println!("\n{}", "=".repeat(60));
            
            if is_mainnet {
                println!("⚠️  WARNING: This will send REAL ZEC on MAINNET!");
                println!("⚠️  This transaction cannot be undone!");
                println!();
                println!("Please confirm:");
                println!("  1. The recipient address is correct");
                println!("  2. The amount is correct");
                println!("  3. You understand this will spend real ZEC");
                println!();
                println!("Type 'SEND' (all caps) to confirm, or anything else to cancel:");
            } else {
                println!("ℹ️  This will send ZEC on TESTNET (not real money)");
                println!("Type 'yes' to continue, or anything else to cancel:");
            }
            
            use std::io::{self, Write};
            print!("> ");
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let trimmed = input.trim();
            
            let enable_broadcast = if is_mainnet {
                trimmed == "SEND"
            } else {
                trimmed.to_lowercase() == "yes" || trimmed.to_lowercase() == "y"
            };
            
            if !enable_broadcast {
                println!("❌ Transaction cancelled.");
                return Ok(());
            }
            
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
            
            println!("\n🔨 Building transaction...");
            
            use std::fs;
            use nozy::paths::get_wallet_data_dir;
            let notes_path = get_wallet_data_dir().join("notes.json");
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
                &actual_recipient,
                amount_zatoshis,
                Some(fee_zatoshis),
                memo_bytes_opt.as_deref(),
                enable_broadcast,
                &config.zebra_url,
            ).await {
                Ok(_) => {
                    let amount_with_fee = amount_zatoshis + fee_zatoshis;
                    let balance_after = balance_before.saturating_sub(amount_with_fee);
                    
                    println!("\n✅ Transaction sent successfully!");
                    println!("{}", "=".repeat(60));
                    println!("  Amount sent: {:.8} ZEC", amount);
                    println!("  Fee paid:    {:.8} ZEC", fee_zatoshis as f64 / 100_000_000.0);
                    println!("  Total spent: {:.8} ZEC", amount_with_fee as f64 / 100_000_000.0);
                    println!("  Remaining:   {:.8} ZEC", balance_after as f64 / 100_000_000.0);
                    println!("{}", "=".repeat(60));
                    println!("\n💡 Run 'nozy history' to view transaction details");
                },
                Err(e) => {
                    println!("❌ Failed to send: {}", e);
                    handle_insufficient_funds_error(&e);
                }
            }
        }
        
        Commands::Balance => {
            use std::fs;
            use nozy::paths::get_wallet_data_dir;
            use nozy::transaction_history::SentTransactionStorage;
            
            let notes_path = get_wallet_data_dir().join("notes.json");
            
            let confirmed_balance = if notes_path.exists() {
                match fs::read_to_string(&notes_path) {
                    Ok(content) => {
                        let parsed: serde_json::Value = match serde_json::from_str(&content) {
                            Ok(v) => v,
                            Err(_) => serde_json::json!([]),
                        };
                        parsed.as_array()
                            .unwrap_or(&vec![])
                            .iter()
                            .filter_map(|n| n.get("value").and_then(|v| v.as_u64()))
                            .sum::<u64>()
                    },
                    Err(_) => 0,
                }
            } else {
                0
            };
            
            let pending_amount = if let Ok(tx_storage) = SentTransactionStorage::new() {
                let pending = tx_storage.get_pending_transactions();
                pending.iter()
                    .map(|tx| tx.amount_zatoshis + tx.fee_zatoshis)
                    .sum::<u64>()
            } else {
                0
            };
            
            let available_balance = confirmed_balance.saturating_sub(pending_amount);
            
            println!("💰 Balance Information");
            println!("{}", "=".repeat(50));
            println!("   Confirmed: {:.8} ZEC", confirmed_balance as f64 / 100_000_000.0);
            
            if pending_amount > 0 {
                println!("   Pending:   -{:.8} ZEC", pending_amount as f64 / 100_000_000.0);
                println!("   Available: {:.8} ZEC", available_balance as f64 / 100_000_000.0);
                println!("\n   💡 Pending transactions reduce available balance until confirmed");
            } else {
                println!("   Available: {:.8} ZEC", available_balance as f64 / 100_000_000.0);
            }
            
            if !notes_path.exists() {
                println!("\n   ⚠️  Run 'sync' to update your balance.");
            }
        }
        
        Commands::Info => {
            let (wallet, _storage) = load_wallet().await?;
            println!("Wallet loaded successfully!");
            println!("Mnemonic: {}", wallet.get_mnemonic());
        }
        
        Commands::Config { 
            set_zebra_url, 
            set_network, 
            use_local, 
            use_remote,
            use_crosslink,
            use_zebra,
            set_crosslink_url,
            show_backend,
        } => {
            use nozy::BackendKind;
            let mut config = load_config();
            let mut changed = false;
            
            if use_crosslink {
                config.backend = BackendKind::Crosslink;
                save_config(&config)?;
                println!("✅ Backend switched to: Crosslink");
                println!("🔗 NozyWallet will now use Zebra Crosslink node");
                changed = true;
            }
            
            if use_zebra {
                config.backend = BackendKind::Zebra;
                save_config(&config)?;
                println!("✅ Backend switched to: Zebra (standard)");
                println!("🔗 NozyWallet will now use standard Zebra node");
                changed = true;
            }
            
            if let Some(ref url) = set_crosslink_url {
                config.crosslink_url = url.clone();
                save_config(&config)?;
                println!("✅ Crosslink URL set to: {}", url);
                changed = true;
            }
            
            if show_backend {
                println!("Current Backend Configuration:");
                println!("{}", "=".repeat(50));
                match config.backend {
                    BackendKind::Zebra => {
                        println!("  Backend: Zebra (standard)");
                        println!("  Zebra URL: {}", config.zebra_url);
                    },
                    BackendKind::Crosslink => {
                        println!("  Backend: Crosslink (experimental)");
                        if config.crosslink_url.is_empty() {
                            println!("  Crosslink URL: {} (using zebra_url as fallback)", config.zebra_url);
                        } else {
                            println!("  Crosslink URL: {}", config.crosslink_url);
                        }
                    },
                }
                println!("  Network: {}", config.network);
                return Ok(());
            }
            
            if use_local {
                config.zebra_url = "http://127.0.0.1:8232".to_string();
                save_config(&config)?;
                println!("✅ Zebra URL set to local node: http://127.0.0.1:8232");
                println!("🔗 NozyWallet will now connect to your local Zebra node");
                changed = true;
            }
            
            if let Some(ref remote_url) = use_remote {
                let normalized = if remote_url.starts_with("http://") || remote_url.starts_with("https://") {
                    remote_url.clone()
                } else if remote_url.contains(":443") {
                    format!("https://{}", remote_url)
                } else {
                    format!("https://{}", remote_url)
                };
                config.zebra_url = normalized.clone();
                save_config(&config)?;
                println!("✅ Zebra URL set to remote node: {}", normalized);
                println!("🔗 NozyWallet will now connect to the remote node");
                changed = true;
            }
            
            if let Some(ref url) = set_zebra_url {
                config.zebra_url = url.clone();
                save_config(&config)?;
                println!("✅ Zebra URL set to: {}", url);
                changed = true;
            }
            
            if let Some(ref network) = set_network {
                config.network = network.clone();
                save_config(&config)?;
                println!("✅ Network set to: {}", network);
                changed = true;
            }
            
            if !changed {
                println!("Current configuration:");
                println!("{}", "=".repeat(50));
                match config.backend {
                    BackendKind::Zebra => {
                        println!("  Backend: Zebra (standard)");
                    },
                    BackendKind::Crosslink => {
                        println!("  Backend: Crosslink (experimental) ⚠️");
                    },
                }
                println!("  Zebra URL: {}", config.zebra_url);
                if !config.crosslink_url.is_empty() {
                    println!("  Crosslink URL: {}", config.crosslink_url);
                }
                let is_local = config.zebra_url.contains("127.0.0.1") || config.zebra_url.contains("localhost");
                if is_local {
                    println!("    ✅ Connected to local node");
                } else {
                    println!("    🌐 Connected to remote node");
                }
                println!("  Network: {}", config.network);
                if let Some(last_height) = config.last_scan_height {
                    println!("  Last scanned height: {}", last_height);
                } else {
                    println!("  Last scanned height: (none)");
                }
                println!("\nTo change settings:");
                println!("  Backend switching:");
                println!("    nozy config --use-zebra                    # Use standard Zebra");
                println!("    nozy config --use-crosslink                 # Use Crosslink (experimental)");
                println!("    nozy config --set-crosslink-url <url>        # Set Crosslink node URL");
                println!("    nozy config --show-backend                  # Show backend info");
                println!("  Node URL:");
                println!("    nozy config --use-local                     # Connect to local Zebra node");
                println!("    nozy config --use-remote <host:port>         # Connect to remote node");
                println!("    nozy config --set-zebra-url <url>           # Set custom URL");
                println!("  Network:");
                println!("    nozy config --set-network mainnet|testnet   # Set network");
            }
        }
        
        Commands::TestZebra { zebra_url } => {
            let mut config = load_config();
            
            if let Some(url) = zebra_url {
                config.zebra_url = url.clone();
            }
            
            let client = ZebraClient::from_config(&config);
            let test_url = match config.backend {
                nozy::BackendKind::Zebra => config.zebra_url.clone(),
                nozy::BackendKind::Crosslink => {
                    if config.crosslink_url.is_empty() {
                        config.zebra_url.clone()
                    } else {
                        config.crosslink_url.clone()
                    }
                },
            };
            
            println!("🔗 Testing {} node connection...", match config.backend {
                nozy::BackendKind::Zebra => "Zebra",
                nozy::BackendKind::Crosslink => "Crosslink",
            });
            println!("📡 Connecting to: {}", test_url);
            println!();
            
            match client.test_connection().await {
                Ok(_) => {
                    println!();
                    println!("🎉 Connection successful!");
                    let backend_name = match config.backend {
                        nozy::BackendKind::Zebra => "Zebra",
                        nozy::BackendKind::Crosslink => "Crosslink",
                    };
                    let is_local = test_url.contains("127.0.0.1") || test_url.contains("localhost");
                    if is_local {
                        println!("✅ NozyWallet is connected to your local {} node", backend_name);
                    } else {
                        println!("✅ NozyWallet is connected to the remote {} node", backend_name);
                    }
                    println!("✅ Ready to sync and send transactions!");
                    
                    match client.get_network_info().await {
                        Ok(info) => {
                            if let Some(chain) = info.get("chain") {
                                println!("   Network: {:?}", chain);
                            }
                            if let Some(blocks) = info.get("blocks") {
                                println!("   Blocks: {:?}", blocks);
                            }
                        },
                        Err(_) => {
                           
                        }
                    }
                },
                Err(e) => {
                    println!("❌ Connection failed!");
                    println!("   Error: {}", e);
                    println!();
                    let is_local = test_url.contains("127.0.0.1") || test_url.contains("localhost");
                    if is_local {
                        println!("💡 Troubleshooting steps for local node:");
                        println!("   1. Make sure the node is running on this PC");
                        println!("   2. Check if RPC is enabled in the node config");
                        println!("   3. Verify it is listening on {}", test_url);
                    } else {
                        println!("💡 Troubleshooting steps for remote node:");
                        println!("   1. Check if the remote node is accessible");
                        println!("   2. Verify the URL is correct: {}", test_url);
                        println!("   3. Check your internet connection");
                        println!("   4. The node might be temporarily unavailable");
                        println!();
                        println!("   To switch to local node: nozy config --use-local");
                    }
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
            use nozy::paths::get_wallet_data_dir;
            let analytics_path = get_wallet_data_dir().join("analytics.json");
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
        
        Commands::History => {
            use nozy::transaction_history::SentTransactionStorage;
            use nozy::load_config;
            
            let config = load_config();
            let zebra_client = ZebraClient::from_config(&config);
            let tx_storage = SentTransactionStorage::new()?;
            
            let _ = tx_storage.update_confirmations(&zebra_client).await;
            
            let all_txs = tx_storage.get_all_transactions();
            
            if all_txs.is_empty() {
                println!("📝 No transaction history found.");
                println!("   Transactions will appear here after you send ZEC.");
            } else {
                println!("📜 Transaction History ({} transactions)", all_txs.len());
                println!("{}", "=".repeat(80));
                
                for (i, tx) in all_txs.iter().enumerate() {
                    let amount_zec = tx.amount_zatoshis as f64 / 100_000_000.0;
                    let fee_zec = tx.fee_zatoshis as f64 / 100_000_000.0;
                    
                    println!("\n{}. {}", i + 1, tx.txid);
                    println!("   Status: {:?}", tx.status);
                    println!("   Amount: {:.8} ZEC", amount_zec);
                    println!("   Fee: {:.8} ZEC", fee_zec);
                    println!("   To: {}", tx.recipient_address);
                    
                    if let Some(block_height) = tx.block_height {
                        println!("   Block: {} ({} confirmations)", block_height, tx.confirmations);
                    } else {
                        println!("   Status: Pending in mempool");
                    }
                    
                    if let Some(broadcast_at) = tx.broadcast_at {
                        println!("   Broadcast: {}", broadcast_at.format("%Y-%m-%d %H:%M:%S UTC"));
                    }
                    
                    if let Some(memo) = &tx.memo {
                        if let Ok(memo_str) = String::from_utf8(memo.clone()) {
                            if !memo_str.trim().is_empty() {
                                println!("   Memo: {}", memo_str);
                            }
                        }
                    }
                }
            }
        }
        
        Commands::CheckConfirmations { txid } => {
            use nozy::transaction_history::SentTransactionStorage;
            use nozy::load_config;
            
            let config = load_config();
            let zebra_client = ZebraClient::from_config(&config);
            let tx_storage = SentTransactionStorage::new()?;
            
            if let Some(txid_str) = txid {
                println!("🔍 Checking confirmation status for transaction: {}", txid_str);
                
                match tx_storage.check_transaction_confirmation(&zebra_client, &txid_str).await {
                    Ok(updated) => {
                        if updated {
                            println!("✅ Transaction confirmed! Status updated.");
                            
                            if let Some(tx) = tx_storage.get_transaction(&txid_str) {
                                if let Some(block_height) = tx.block_height {
                                    println!("   Block: {} ({} confirmations)", block_height, tx.confirmations);
                                }
                            }
                        } else {
                            println!("⏳ Transaction still pending in mempool.");
                        }
                    },
                    Err(e) => {
                        println!("❌ Error checking transaction: {}", e);
                    }
                }
            } else {
                println!("🔍 Checking all pending transactions...");
                
                match tx_storage.check_all_pending_transactions(&zebra_client).await {
                    Ok(updated_count) => {
                        if updated_count > 0 {
                            println!("✅ Updated {} transaction(s) to confirmed status.", updated_count);
                        } else {
                            println!("⏳ No transactions confirmed yet. All still pending.");
                        }
                        
                        let conf_updated = tx_storage.update_confirmations(&zebra_client).await?;
                        if conf_updated > 0 {
                            println!("📊 Updated confirmation counts for {} transaction(s).", conf_updated);
                        }
                    },
                    Err(e) => {
                        println!("❌ Error checking transactions: {}", e);
                    }
                }
            }
        }
        
        Commands::Status => {
            use nozy::transaction_history::SentTransactionStorage;
            use nozy::load_config;
            use std::fs;
            use nozy::paths::get_wallet_data_dir;
            
            let (wallet, _storage) = load_wallet().await?;
            let config = load_config();
            let zebra_client = ZebraClient::from_config(&config);
            
            println!("📊 NozyWallet Status Dashboard");
            println!("{}", "=".repeat(60));
            
            println!("\n🔐 Wallet:");
            println!("   Mnemonic: {}...", &wallet.get_mnemonic()[..20]);
            
            println!("\n🔗 Connection:");
            match config.backend {
                nozy::BackendKind::Zebra => {
                    println!("   Backend: Zebra (standard)");
                    println!("   URL: {}", config.zebra_url);
                },
                nozy::BackendKind::Crosslink => {
                    println!("   Backend: Crosslink (experimental) ⚠️");
                    if config.crosslink_url.is_empty() {
                        println!("   URL: {} (using zebra_url)", config.zebra_url);
                    } else {
                        println!("   URL: {}", config.crosslink_url);
                    }
                },
            }
            match zebra_client.test_connection().await {
                Ok(_) => {
                    if let Ok(block_count) = zebra_client.get_block_count().await {
                        println!("   ✅ Connected - Block height: {}", block_count);
                    } else {
                        println!("   ✅ Connected");
                    }
                },
                Err(e) => {
                    println!("   ❌ Not connected: {}", e);
                }
            }
            
            println!("\n💰 Balance:");
            let notes_path = get_wallet_data_dir().join("notes.json");
            if notes_path.exists() {
                if let Ok(content) = fs::read_to_string(&notes_path) {
                    if let Ok(notes) = serde_json::from_str::<Vec<nozy::SerializableOrchardNote>>(&content) {
                        let total_balance: u64 = notes.iter().filter(|n| !n.spent).map(|n| n.value).sum();
                        let balance_zec = total_balance as f64 / 100_000_000.0;
                        let unspent_count = notes.iter().filter(|n| !n.spent).count();
                        println!("   Total: {:.8} ZEC", balance_zec);
                        println!("   Notes: {} unspent notes", unspent_count);
                    }
                }
            } else {
                println!("   Total: 0.00000000 ZEC");
                println!("   Run 'sync' to update balance");
            }
            
            println!("\n📜 Transactions:");
            if let Ok(tx_storage) = SentTransactionStorage::new() {
                let pending = tx_storage.get_pending_transactions();
                let stats = tx_storage.get_statistics();
                
                println!("   Total: {}", stats.total_count);
                println!("   Pending: {}", pending.len());
                println!("   Confirmed: {}", stats.confirmed_count);
                println!("   Failed: {}", stats.failed_count);
                
                if !pending.is_empty() {
                    println!("\n   ⏳ Pending transactions:");
                    for tx in pending.iter().take(5) {
                        let amount_zec = tx.amount_zatoshis as f64 / 100_000_000.0;
                        println!("      {} - {:.8} ZEC", &tx.txid[..16], amount_zec);
                    }
                    if pending.len() > 5 {
                        println!("      ... and {} more", pending.len() - 5);
                    }
                }
            }
            
            println!("\n🔄 Sync Status:");
            if let Some(last_height) = config.last_scan_height {
                println!("   Last scanned: Block {}", last_height);
                if let Ok(current_height) = zebra_client.get_block_count().await {
                    let blocks_behind = current_height.saturating_sub(last_height);
                    if blocks_behind > 0 {
                        println!("   Behind: {} blocks", blocks_behind);
                        println!("   💡 Run 'sync' to catch up");
                    } else {
                        println!("   ✅ Up to date");
                    }
                }
            } else {
                println!("   ⚠️  Never synced");
                println!("   💡 Run 'sync' to start");
            }
            
            println!("\n{}", "=".repeat(60));
        }
        
        Commands::AddressBook { command } => {
            let address_book = AddressBook::new()?;
            
            match command {
                AddressBookCommand::List => {
                    let addresses = address_book.list_addresses();
                    
                    if addresses.is_empty() {
                        println!("📇 Address book is empty.");
                        println!("   Use 'nozy address-book add --name <name> --address <address>' to add entries.");
                    } else {
                        println!("📇 Address Book ({} entries)", addresses.len());
                        println!("{}", "=".repeat(80));
                        
                        for entry in addresses {
                            println!("\n📌 {}", entry.name);
                            println!("   Address: {}", entry.address);
                            if let Some(notes) = &entry.notes {
                                println!("   Notes: {}", notes);
                            }
                            println!("   Created: {}", entry.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
                            if let Some(last_used) = entry.last_used {
                                println!("   Last used: {} ({} times)", 
                                    last_used.format("%Y-%m-%d %H:%M:%S UTC"),
                                    entry.usage_count);
                            } else {
                                println!("   Never used");
                            }
                        }
                    }
                }
                
                AddressBookCommand::Add { name, address, notes } => {
                    match address_book.add_address(name.clone(), address.clone(), notes) {
                        Ok(()) => {
                            println!("✅ Added '{}' to address book", name);
                            println!("   Address: {}", address);
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to add address: {}", e);
                        }
                    }
                }
                
                AddressBookCommand::Remove { name } => {
                    match address_book.remove_address(&name) {
                        Ok(true) => {
                            println!("✅ Removed '{}' from address book", name);
                        }
                        Ok(false) => {
                            eprintln!("❌ Address '{}' not found in address book", name);
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to remove address: {}", e);
                        }
                    }
                }
                
                AddressBookCommand::Get { name } => {
                    match address_book.get_address(&name) {
                        Some(entry) => {
                            println!("📌 {}", entry.name);
                            println!("   Address: {}", entry.address);
                            if let Some(notes) = &entry.notes {
                                println!("   Notes: {}", notes);
                            }
                            println!("   Created: {}", entry.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
                            if let Some(last_used) = entry.last_used {
                                println!("   Last used: {} ({} times)", 
                                    last_used.format("%Y-%m-%d %H:%M:%S UTC"),
                                    entry.usage_count);
                            } else {
                                println!("   Never used");
                            }
                        }
                        None => {
                            eprintln!("❌ Address '{}' not found in address book", name);
                        }
                    }
                }
                
                AddressBookCommand::Search { query } => {
                    let results = address_book.search_addresses(&query);
                    
                    if results.is_empty() {
                        println!("🔍 No addresses found matching '{}'", query);
                    } else {
                        println!("🔍 Search results for '{}' ({} found)", query, results.len());
                        println!("{}", "=".repeat(80));
                        
                        for entry in results {
                            println!("\n📌 {}", entry.name);
                            println!("   Address: {}", entry.address);
                            if let Some(notes) = &entry.notes {
                                println!("   Notes: {}", notes);
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}
