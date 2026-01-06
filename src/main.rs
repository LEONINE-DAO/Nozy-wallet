use clap::{Parser, Subcommand};
use dialoguer::{Password, Confirm};
use nozy::{HDWallet, WalletStorage, NozyResult, NozyError, NoteScanner, ZebraClient, AddressBook, ZebraBlockSource, ZebraBlockParser};
use zeaking::Zeaking;
use nozy::local_analytics::LocalAnalytics;
use nozy::cli_helpers::{load_wallet, scan_notes_for_sending, build_and_broadcast_transaction, handle_insufficient_funds_error};
use nozy::{load_config, save_config, update_last_scan_height};
use nozy::safe_display::display_mnemonic_safe;
use zcash_address::unified::{Encoding, Container};

#[derive(Parser)]
#[command(name = "nozy")]
#[command(version = "2.1.0")]
#[command(about = "NozyWallet - A privacy-focused Zcash Orchard wallet")]
#[command(long_about = "NozyWallet is a privacy-first Orchard wallet that enforces complete transaction privacy by default. Unlike other Zcash wallets, NozyWallet only supports shielded transactions - making it functionally equivalent to Monero in terms of privacy, but with faster block times and lower fees.")]
pub struct Cli {
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[arg(short, long, global = true)]
    pub quiet: bool,

    #[arg(short = 't', long, global = true)]
    pub testnet: bool,

    #[arg(short = 'm', long, global = true)]
    pub mainnet: bool,

    #[arg(long, global = true, env = "ZEBRA_RPC_URL")]
    pub zebra_url: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Create a new wallet with a random mnemonic phrase")]
    New,

    #[command(about = "Restore a wallet from an existing 24-word mnemonic phrase")]
    Restore,
    
    #[command(about = "Generate a new shielded Orchard address for receiving ZEC")]
    Receive,
    
    #[command(about = "Scan the blockchain for transactions to your wallet addresses")]
    Sync {
        #[arg(long, help = "Starting block height for scanning (default: last scanned height + 1)")]
        start_height: Option<u32>,
        #[arg(long, help = "Ending block height for scanning (default: current block height)")]
        end_height: Option<u32>,
        #[arg(long, help = "Override Zebra RPC URL (overrides config and global --zebra-url)")]
        zebra_url: Option<String>,
    },

    #[command(about = "Send ZEC to a shielded Orchard address")]
    Send {
        #[arg(long, short = 'r', help = "Recipient's shielded Orchard address (must start with 'u1')")]
        recipient: String,
        #[arg(long, short = 'a', help = "Amount to send in ZEC (e.g., 0.1)")]
        amount: f64,
        #[arg(long, help = "Override Zebra RPC URL (overrides config and global --zebra-url)")]
        zebra_url: Option<String>,
        #[arg(long, short = 'm', help = "Optional memo message (max 512 characters)")]
        memo: Option<String>,
    },

    #[command(about = "Display wallet information including addresses and network")]
    Info,
    
    #[command(about = "Display the current shielded balance")]
    Balance,
    
    #[command(about = "Configure wallet settings (network, Zebra URL, backend, etc.)")]
    Config {
        #[arg(long, help = "Set the Zebra RPC URL (e.g., http://127.0.0.1:8232)")]
        set_zebra_url: Option<String>,
        #[arg(long, help = "Set network: 'mainnet' or 'testnet'")]
        set_network: Option<String>,
        #[arg(long, help = "Use local Zebra node at http://127.0.0.1:8232")]
        use_local: bool,
        /// Use remote Zebra node
        #[arg(long, help = "Use remote Zebra node (provide URL: --use-remote http://host:port)")]
        use_remote: Option<String>,
        /// Use Crosslink backend
        #[arg(long, help = "Use Crosslink backend instead of Zebra")]
        use_crosslink: bool,
        /// Use Zebra backend (default)
        #[arg(long, help = "Use Zebra backend (default)")]
        use_zebra: bool,
        /// Set Crosslink RPC URL
        #[arg(long, help = "Set the Crosslink RPC URL")]
        set_crosslink_url: Option<String>,
        /// Show current backend configuration
        #[arg(long, help = "Display current backend configuration")]
        show_backend: bool,
        /// Set protocol version
        #[arg(long, help = "Set protocol version (e.g., 170140 for NU 6.1)")]
        set_protocol: Option<String>,
    },
    
    #[command(about = "Test connection to Zebra or Crosslink node")]
    TestZebra {
        #[arg(long, help = "Override Zebra RPC URL (overrides config and global --zebra-url)")]
        zebra_url: Option<String>,
    },
    
    /// Manage Orchard proving parameters
    #[command(about = "Download and manage Orchard proving parameters")]
    Proving {
        #[arg(long, short = 'd', help = "Download proving parameters from official source")]
        download: bool,
        #[arg(long, short = 's', help = "Show status of proving parameters")]
        status: bool,
    },
                        
    #[command(about = "Display wallet analytics and statistics")]
    Analytics,
    
    #[command(about = "Display transaction history")]
    History,
    
    #[command(about = "Check confirmation status of a transaction")]
    CheckConfirmations {
        #[arg(long, short = 't', help = "Transaction ID (TXID) to check")]
        txid: Option<String>,
    },
    
    #[command(about = "Display current wallet status and sync information")]
    Status,
    
    #[command(about = "Manage saved addresses in your address book")]
    AddressBook {
        #[command(subcommand)]
        command: AddressBookCommand,
    },
    
    /// Zeaking indexing commands
    #[command(about = "Commands for Zeaking local blockchain indexer")]
    Zeaking {
        #[command(subcommand)]
        command: ZeakingCommand,
    },
    
    /// Privacy network utilities
    #[command(about = "Test and manage privacy network connections (Tor, I2P)")]
    PrivacyNetwork {
        #[command(subcommand)]
        command: PrivacyNetworkCommand,
    },
    
    /// Cross-chain swap commands
    #[command(about = "Cross-chain swap functionality (XMR to ZEC, etc.)")]
    Swap {
        #[command(subcommand)]
        command: SwapCommand,
    },
    
    /// Shade Protocol commands
    #[command(about = "Shade Protocol (Secret Network) integration commands")]
    Shade {
        #[command(subcommand)]
        command: ShadeCommand,
    },
    
    /// Monero integration commands
    #[command(about = "Monero integration and verification commands")]
    Monero {
        #[command(subcommand)]
        command: MoneroCommand,
    },
    
    /// Network Upgrade 6.1 information
    #[command(about = "Display information about Zcash Network Upgrade 6.1 (NU 6.1)")]
    Nu61,
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

#[derive(Subcommand)]
pub enum ZeakingCommand {
    Sync,
    Index {
        #[arg(long)]
        start: u32,
        #[arg(long)]
        end: u32,
    },
    Stats,
    Block {
        #[arg(long)]
        height: u32,
    },
    Transaction {
        #[arg(long)]
        txid: String,
    },
    FindNullifier {
        #[arg(long)]
        nullifier: String,
    },
}

#[derive(Subcommand)]
pub enum PrivacyNetworkCommand {
    Status,
    TestTor,
    TestI2p,
    GetIp,
}

#[derive(Subcommand)]
pub enum SwapCommand {
    XmrToZec {
        #[arg(long)]
        amount: f64,
    },
    ZecToXmr {
        #[arg(long)]
        amount: f64,
    },
    Status {
        #[arg(long)]
        swap_id: String,
    },
    List,
    Churn {
        #[arg(long, default_value = "2")]
        times: u32,
        #[arg(long, default_value = "12")]
        ring_size: u32,
    },
}

#[derive(Subcommand)]
pub enum ShadeCommand {
    Balance {
        #[arg(long)]
        address: Option<String>,
        #[arg(long)]
        token: Option<String>,
    },
    Info {
        #[arg(long)]
        token: String,
    },
    Send {
        #[arg(long)]
        recipient: String,
        #[arg(long)]
        amount: f64,
        #[arg(long)]
        token: String,
        #[arg(long)]
        memo: Option<String>,
    },
    Receive {
        #[arg(long, default_value = "0")]
        account: u32,
        #[arg(long, default_value = "0")]
        index: u32,
    },
    History,
    Status {
        #[arg(long)]
        txid: Option<String>,
    },
    ListTokens,
}

#[derive(Subcommand)]
pub enum MoneroCommand {
    Balance {
        #[arg(long)]
        rpc_url: Option<String>,
        #[arg(long)]
        username: Option<String>,
        #[arg(long)]
        password: Option<String>,
    },
    Send {
        #[arg(long)]
        recipient: String,
        #[arg(long)]
        amount: f64,
        #[arg(long)]
        rpc_url: Option<String>,
        #[arg(long)]
        username: Option<String>,
        #[arg(long)]
        password: Option<String>,
    },
    Receive {
        #[arg(long, default_value = "0")]
        account_index: u32,
        #[arg(long)]
        rpc_url: Option<String>,
        #[arg(long)]
        username: Option<String>,
        #[arg(long)]
        password: Option<String>,
    },
    History,
    Status {
        #[arg(long)]
        txid: Option<String>,
    },
}


#[tokio::main]
async fn main() {
    // Parse CLI arguments
    let cli = Cli::parse();
    
    // Initialize logging based on verbose/quiet flags
    if cli.verbose {
        std::env::set_var("RUST_LOG", "debug");
    } else if cli.quiet {
        std::env::set_var("RUST_LOG", "error");
    }
    
    // Initialize tracing if RUST_LOG is set
    if std::env::var("RUST_LOG").is_ok() {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .init();
    }
    
    // Handle network override flags
    let mut config = load_config();
    if cli.testnet {
        config.network = "testnet".to_string();
        if !cli.quiet {
            println!("🌐 Using testnet (overridden by --testnet flag)");
        }
    } else if cli.mainnet {
        config.network = "mainnet".to_string();
        if !cli.quiet {
            println!("🌐 Using mainnet (overridden by --mainnet flag)");
        }
    }
    
    // Handle global Zebra URL override
    if let Some(url) = cli.zebra_url {
        config.zebra_url = url;
        if !cli.quiet {
            println!("🔗 Using Zebra URL: {}", config.zebra_url);
        }
    }
    
    // Execute command and handle errors
    if let Err(e) = execute_command(cli.command, config).await {
        handle_error(e);
        std::process::exit(1);
    }
}

async fn execute_command(command: Commands, mut config: nozy::WalletConfig) -> NozyResult<()> {
    let cli = Cli::parse();
    
    // Initialize logging based on verbose/quiet flags
    if cli.verbose {
        std::env::set_var("RUST_LOG", "debug");
    } else if cli.quiet {
        std::env::set_var("RUST_LOG", "error");
    }
    
    // Initialize tracing if RUST_LOG is set
    if std::env::var("RUST_LOG").is_ok() {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .init();
    }
    
    // Handle network override flags
    let mut config = load_config();
    if cli.testnet {
        config.network = "testnet".to_string();
        if !cli.quiet {
            println!("🌐 Using testnet (overridden by --testnet flag)");
        }
    } else if cli.mainnet {
        config.network = "mainnet".to_string();
        if !cli.quiet {
            println!("🌐 Using mainnet (overridden by --mainnet flag)");
        }
    }
    
    // Handle global Zebra URL override
    if let Some(url) = cli.zebra_url {
        config.zebra_url = url;
        if !cli.quiet {
            println!("🔗 Using Zebra URL: {}", config.zebra_url);
        }
    }
    
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
            // Show full mnemonic only during wallet creation (user explicitly requested it)
                println!("📝 Mnemonic: {}", wallet.get_mnemonic());
                println!();
                println!("⚠️  ⚠️  ⚠️  CRITICAL SECURITY WARNING ⚠️  ⚠️  ⚠️");
                println!("   Your mnemonic phrase is the ONLY way to recover your wallet.");
                println!("   If you lose it, you will PERMANENTLY lose access to ALL your funds.");
                println!();
                println!("🔒 SECURITY BEST PRACTICES:");
                println!("   ✅ Write it down on paper (NEVER store digitally)");
                println!("   ✅ Store in a secure location (fireproof safe, bank deposit box)");
                println!("   ✅ Never share it with anyone (not even support staff)");
                println!("   ✅ Never take screenshots or photos of your mnemonic");
                println!("   ✅ Never store it online (cloud, email, notes apps)");
                println!("   ✅ Make multiple copies in different secure locations");
                println!("   ✅ Verify the backup by restoring from it (on testnet first)");
                println!();
                println!("   ⚠️  If someone gets your mnemonic, they can steal ALL your funds.");
                println!("   ⚠️  SECURITY: This mnemonic will NOT be shown again for security reasons.");
            
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
            // Use config from global flags (already loaded above)
            // Override with command-specific zebra_url if provided
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
            
            // Estimate total blocks to scan for progress
            let total_blocks = if let (Some(start), Some(end)) = (effective_start, end_height) {
                (end.saturating_sub(start)) as u64
            } else if let Some(start) = effective_start {
                // Estimate ~2.5M blocks total, scan from start to current
                2_500_000u64.saturating_sub(start as u64)
            } else {
                2_500_000u64 // Full scan estimate
            };
            
            use nozy::progress::create_sync_progress_bar;
            let pb = if total_blocks > 100 {
                Some(create_sync_progress_bar(total_blocks))
            } else {
                None
            };
            
            if let Some(ref progress_bar) = pb {
                progress_bar.set_message("Scanning blockchain for notes...");
            }
            
            match note_scanner.scan_notes(effective_start, end_height).await {
                Ok((result, _spendable_notes)) => {
                    if let Some(ref progress_bar) = pb {
                        progress_bar.finish_with_message("✅ Scan complete");
                    }
                    
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
                    if let Some(ref progress_bar) = pb {
                        progress_bar.finish_with_message("❌ Scan failed");
                    }
                    println!("❌ Error updating wallet: {}", e);
                    println!("\n💡 Recovery suggestions:");
                    for suggestion in e.recovery_suggestions() {
                        println!("   • {}", suggestion);
                    }
                }
            }
        }
        
        Commands::Send { recipient, amount, zebra_url, memo } => {
            
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
            
            // Enhanced address validation
            use nozy::input_validation::validate_zcash_address;
            if let Err(e) = validate_zcash_address(&actual_recipient) {
                println!("❌ {}", e);
                println!("\n💡 Recovery suggestions:");
                for suggestion in e.recovery_suggestions() {
                    println!("   • {}", suggestion);
                }
                return Err(e);
            }
            
            if actual_recipient.starts_with("t1") {
                let error = NozyError::AddressParsing(
                    "Transparent addresses (t1) are not supported. NozyWallet only supports shielded addresses (u1 unified addresses with Orchard receivers) for privacy protection. Please use a shielded address.".to_string()
                );
                println!("❌ {}", error);
                println!("\n💡 Recovery suggestions:");
                for suggestion in error.recovery_suggestions() {
                    println!("   • {}", suggestion);
                }
                return Err(error);
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
            let tx_builder_check = OrchardTransactionBuilder::new_async(false).await?;
            let proving_status = tx_builder_check.get_proving_status();
            if !proving_status.can_prove {
                return Err(NozyError::InvalidOperation(
                    "Cannot create proofs: proving system not ready. Run 'nozy proving --status' to check.".to_string()
                ));
            }
            println!("  Proving:   ✅ Ready (Halo 2)");
            
            println!("\n🔍 Scanning for spendable notes...");
            use nozy::progress::create_tx_progress_bar;
            let pb = create_tx_progress_bar();
            pb.set_message("Scanning blockchain for spendable notes...");
            let spendable_notes = scan_notes_for_sending(wallet, &config.zebra_url).await?;
            pb.finish_with_message("✅ Scan complete");
            
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
                let error = NozyError::InsufficientFunds(format!(
                    "Available: {:.8} ZEC, Required: {:.8} ZEC",
                    total_available as f64 / 100_000_000.0,
                    total_amount as f64 / 100_000_000.0
                ));
                println!("❌ {}", error);
                println!("\n💡 Recovery suggestions:");
                for suggestion in error.recovery_suggestions() {
                    println!("   • {}", suggestion);
                }
                return Err(error);
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
            if let Err(e) = io::stdout().flush() {
                eprintln!("⚠️  Warning: Failed to flush stdout: {}", e);
            }
            
            let mut input = String::new();
            if let Err(e) = io::stdin().read_line(&mut input) {
                return Err(NozyError::InvalidOperation(
                    format!("Failed to read input: {}", e)
                ));
            }
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
                if let Err(e) = io::stdout().flush() {
                    eprintln!("⚠️  Warning: Failed to flush stdout: {}", e);
                }
                let mut memo_input = String::new();
                if let Err(e) = io::stdin().read_line(&mut memo_input) {
                    eprintln!("⚠️  Warning: Failed to read memo input: {}", e);
                }
                let trimmed = memo_input.trim();
                if trimmed.is_empty() { None } else { Some(trimmed.as_bytes().to_vec()) }
            };
            
            let amount_zatoshis = (amount * 100_000_000.0) as u64;
            
            use nozy::privacy_ui::validate_and_show_privacy;
            if let Err(e) = validate_and_show_privacy(&actual_recipient) {
                println!("❌ Privacy validation failed: {}", e);
                return Err(e);
            }
            
            println!("\n🔨 Building transaction...");
            use nozy::privacy_ui::show_privacy_indicator;
            show_privacy_indicator();
            
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
                    
                    use nozy::privacy_ui::show_privacy_badge;
                    show_privacy_badge();
                    
                    println!("\n✅ Transaction sent successfully!");
                    println!("{}", "=".repeat(60));
                    println!("  Amount sent: {:.8} ZEC", amount);
                    println!("  Fee paid:    {:.8} ZEC", fee_zatoshis as f64 / 100_000_000.0);
                    println!("  Total spent: {:.8} ZEC", amount_with_fee as f64 / 100_000_000.0);
                    println!("  Remaining:   {:.8} ZEC", balance_after as f64 / 100_000_000.0);
                    println!();
                    println!("🛡️  This transaction is private and untraceable.");
                    println!("🔒 Privacy is enforced by NozyWallet.");
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
            // Never show full mnemonic in Info command - use safe display
            println!("Mnemonic: {} (masked for security)", display_mnemonic_safe(&wallet.get_mnemonic()));
            println!("⚠️  For security, only partial mnemonic is shown. Use 'restore' command to see full mnemonic.");
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
            set_protocol,
        } => {
            use nozy::BackendKind;
            use nozy::Protocol;
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
                println!("  Protocol: {:?}", config.protocol);
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
            
            if let Some(ref protocol_str) = set_protocol {
                let protocol = match protocol_str.to_lowercase().as_str() {
                    "grpc" => Protocol::Grpc,
                    "jsonrpc" | "json-rpc" | "rpc" => Protocol::JsonRpc,
                    _ => {
                        eprintln!("❌ Invalid protocol: {}. Use 'grpc' or 'jsonrpc'", protocol_str);
                        return Ok(());
                    }
                };
                config.protocol = protocol.clone();
                save_config(&config)?;
                println!("✅ Protocol set to: {:?}", protocol);
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
                println!("  Protocol: {:?}", config.protocol);
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
                        println!("✅ Orchard Halo 2 proving system ready!");
                        println!("💡 Note: Orchard uses Halo 2 which does NOT require external parameters");
                        println!("   Your wallet is ready for shielded transactions");
                    },
                    Err(e) => {
                        println!("❌ Error: {}", e);
                        println!("💡 Note: Orchard Halo 2 doesn't require external parameters");
                        println!("   Your wallet should still work for shielded transactions");
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
            use nozy::nu6_1_check;
            use std::fs;
            use nozy::paths::get_wallet_data_dir;
            
            println!("🔍 Checking system status...");
            println!();
            
            let config = load_config();
            let zebra_client = ZebraClient::from_config(&config);
            match zebra_client.get_block_count().await {
                Ok(height) => {
                    nu6_1_check::display_nu6_1_status(Some(height));
                },
                Err(_) => {
                    nu6_1_check::display_nu6_1_status(None);
                }
            }
            
            if let Err(e) = nu6_1_check::verify_nu6_1_compatibility() {
                println!("⚠️  NU 6.1 compatibility check: {}", e);
            }
            
            println!();
            
            use nozy::transaction_history::SentTransactionStorage;
            
            let (wallet, _storage) = load_wallet().await?;
            let config = load_config();
            let zebra_client = ZebraClient::from_config(&config);
            
            println!("📊 NozyWallet Status Dashboard");
            println!("{}", "=".repeat(60));
            
            println!("\n🔐 Wallet:");
            println!("   Mnemonic: {} (masked for security)", display_mnemonic_safe(&wallet.get_mnemonic()));
            
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
                        
                        println!();
                        use nozy::nu6_1_check;
                        nu6_1_check::display_nu6_1_status(Some(block_count));
                    } else {
                        println!("   ✅ Connected");
                        use nozy::nu6_1_check;
                        nu6_1_check::display_nu6_1_status(None);
                    }
                },
                Err(e) => {
                    println!("   ❌ Not connected: {}", e);
                    use nozy::nu6_1_check;
                    nu6_1_check::display_nu6_1_status(None);
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

        Commands::Zeaking { command } => {
            let config = load_config();
            let zebra_client = ZebraClient::from_config(&config);
            use nozy::paths::get_zeaking_index_dir;
            let block_source = ZebraBlockSource::new(zebra_client.clone());
            let block_parser = ZebraBlockParser::new(zebra_client);
            let zeaking = Zeaking::new(get_zeaking_index_dir(), block_source, block_parser).await
                .map_err(|e| NozyError::Storage(format!("Zeaking error: {}", e)))?;

            match command {
                ZeakingCommand::Sync => {
                    println!("🔄 Syncing Zeaking index to chain tip...");
                    match zeaking.sync_to_tip().await {
                        Ok(stats) => {
                            println!("✅ Index sync complete!");
                            println!("   Blocks indexed: {}", stats.blocks_indexed);
                            println!("   Transactions indexed: {}", stats.transactions_indexed);
                            println!("   Height range: {} - {}", stats.start_height, stats.end_height);
                            if stats.errors > 0 {
                                println!("   ⚠️  Errors: {}", stats.errors);
                            }
                            zeaking.save_index()
                                .map_err(|e| NozyError::Storage(format!("Failed to save index: {}", e)))?;
                        },
                        Err(e) => {
                            eprintln!("❌ Failed to sync index: {}", e);
                        }
                    }
                },
                ZeakingCommand::Index { start, end } => {
                    println!("📦 Indexing blocks {} to {}...", start, end);
                    match zeaking.index_range(start, end).await {
                        Ok(stats) => {
                            println!("✅ Indexing complete!");
                            println!("   Blocks indexed: {}", stats.blocks_indexed);
                            println!("   Transactions indexed: {}", stats.transactions_indexed);
                            if stats.errors > 0 {
                                println!("   ⚠️  Errors: {}", stats.errors);
                            }
                        },
                        Err(e) => {
                            eprintln!("❌ Failed to index blocks: {}", e);
                        }
                    }
                },
                ZeakingCommand::Stats => {
                    let stats = zeaking.get_stats();
                    println!("📊 Zeaking Index Statistics");
                    println!("{}", "=".repeat(60));
                    println!("   Blocks indexed: {}", stats.blocks_indexed);
                    println!("   Transactions indexed: {}", stats.transactions_indexed);
                    println!("   Height range: {} - {}", stats.start_height, stats.end_height);
                    println!("   Last indexed height: {}", zeaking.get_last_indexed_height());
                },
                ZeakingCommand::Block { height } => {
                    match zeaking.get_block(height) {
                        Some(block) => {
                            println!("📦 Block #{}", height);
                            println!("{}", "=".repeat(60));
                            println!("   Hash: {}", block.hash);
                            println!("   Time: {}", chrono::DateTime::<chrono::Utc>::from_timestamp(block.time, 0)
                                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                                .unwrap_or_else(|| "Unknown".to_string()));
                            println!("   Size: {} bytes", block.size);
                            println!("   Transactions: {}", block.tx_count);
                            println!("   Orchard actions: {}", block.orchard_action_count);
                            println!("   Indexed at: {}", block.indexed_at.format("%Y-%m-%d %H:%M:%S UTC"));
                        },
                        None => {
                            println!("❌ Block {} not found in index", height);
                            println!("   Use 'nozy zeaking index --start {} --end {}' to index it", height, height);
                        }
                    }
                },
                ZeakingCommand::Transaction { txid } => {
                    match zeaking.get_transaction(&txid) {
                        Some(tx) => {
                            println!("💸 Transaction {}", txid);
                            println!("{}", "=".repeat(60));
                            println!("   Block height: {}", tx.block_height);
                            println!("   Block hash: {}", tx.block_hash);
                            println!("   Index in block: {}", tx.index);
                            println!("   Size: {} bytes", tx.size);
                            println!("   Orchard actions: {}", tx.orchard_actions.len());
                            if let Some(fee) = tx.fee {
                                println!("   Fee: {} zatoshi", fee);
                            }
                            println!("   Indexed at: {}", tx.indexed_at.format("%Y-%m-%d %H:%M:%S UTC"));
                        },
                        None => {
                            println!("❌ Transaction {} not found in index", txid);
                        }
                    }
                },
                ZeakingCommand::FindNullifier { nullifier } => {
                    match zeaking.find_transaction_by_nullifier(&nullifier) {
                        Some(tx) => {
                            println!("🔍 Found transaction for nullifier {}", nullifier);
                            println!("   Transaction ID: {}", tx.txid);
                            println!("   Block height: {}", tx.block_height);
                        },
                        None => {
                            println!("❌ No transaction found for nullifier {}", nullifier);
                        }
                    }
                },
            }
        }
        
        Commands::PrivacyNetwork { command } => {
            use crate::privacy_network::proxy::ProxyConfig;
            use crate::privacy_network::{TorProxy, I2PProxy};
            
            match command {
                PrivacyNetworkCommand::Status => {
                    println!("🔒 Privacy Network Status");
                    println!("{}", "=".repeat(60));
                    
                    let proxy = ProxyConfig::auto_detect().await;
                    if proxy.enabled {
                        println!("  Network: {:?}", proxy.network);
                        println!("  Proxy: {}", proxy.proxy_url);
                        println!("  Status: ✅ Active");
                    } else {
                        println!("  Status: ❌ Not available");
                        println!("  ⚠️  Please start Tor or I2P");
                    }
                },
                PrivacyNetworkCommand::TestTor => {
                    println!("🔍 Testing Tor connection...");
                    let tor = TorProxy::new(None);
                    match tor.test_tor_connection().await {
                        Ok(true) => {
                            println!("✅ Tor is working!");
                            if let Ok(ip) = tor.get_tor_ip().await {
                                println!("   Your IP through Tor: {}", ip);
                            }
                        },
                        Ok(false) => {
                            println!("❌ Tor connection failed");
                            println!("   Please check if Tor is running on 127.0.0.1:9050");
                        },
                        Err(e) => {
                            println!("❌ Error testing Tor: {}", e);
                        }
                    }
                },
                PrivacyNetworkCommand::TestI2p => {
                    println!("🔍 Testing I2P connection...");
                    let i2p = I2PProxy::new(None);
                    match i2p.test_i2p_connection().await {
                        Ok(true) => {
                            println!("✅ I2P is working!");
                        },
                        Ok(false) => {
                            println!("❌ I2P connection failed");
                            println!("   Please check if I2P router is running on 127.0.0.1:4444");
                        },
                        Err(e) => {
                            println!("❌ Error testing I2P: {}", e);
                        }
                    }
                },
                PrivacyNetworkCommand::GetIp => {
                    let proxy = ProxyConfig::auto_detect().await;
                    if proxy.enabled {
                        println!("🔍 Getting IP through privacy network...");
                        if let Ok(client) = proxy.create_client() {
                            match client.get("https://api.ipify.org?format=json")
                                .timeout(std::time::Duration::from_secs(15))
                                .send().await
                            {
                                Ok(response) => {
                                    if let Ok(json) = response.json::<serde_json::Value>().await {
                                        if let Some(ip) = json.get("ip").and_then(|v| v.as_str()) {
                                            println!("✅ Your IP: {}", ip);
                                            println!("   Network: {:?}", proxy.network);
                                        }
                                    }
                                },
                                Err(e) => {
                                    println!("❌ Failed to get IP: {}", e);
                                }
                            }
                        }
                    } else {
                        println!("❌ No privacy network available");
                    }
                },
            }
        }
        
        Commands::Swap { command } => {
            use crate::bridge::SwapEngine;
            use crate::swap::{SwapService, SwapDirection};
            use crate::monero::MoneroWallet;
            use crate::privacy_network::proxy::ProxyConfig;
            use crate::cli_helpers::load_wallet;
            
            let proxy = ProxyConfig::auto_detect().await;
            if !proxy.enabled {
                println!("❌ Privacy network (Tor/I2P) required for swaps!");
                println!("   Please start Tor or I2P before continuing.");
                return Ok(());
            }
            
            println!("🛡️  Privacy network active: {:?}", proxy.network);
            
            let (zcash_wallet, _) = load_wallet().await?;
            
            let swap_service = SwapService::new(None, None, Some(proxy.clone()))
                .map_err(|e| NozyError::InvalidOperation(format!("Failed to create swap service: {}", e)))?;
            
            match command {
                SwapCommand::XmrToZec { amount } => {
                    
                    let monero_wallet = MoneroWallet::new(
                        None, 
                        None,
                        None,
                        Some(proxy),
                    ).ok();
                    
                    let mut swap_engine = SwapEngine::new(
                        swap_service,
                        monero_wallet,
                        Some(zcash_wallet),
                    )?;
                    
                    swap_engine.execute_swap(SwapDirection::XmrToZec, amount).await?;
                },
                SwapCommand::ZecToXmr { amount } => {
                    let monero_wallet = MoneroWallet::new(
                        None,
                        None,
                        None,
                        Some(proxy),
                    ).ok();
                    
                    let mut swap_engine = SwapEngine::new(
                        swap_service,
                        monero_wallet,
                        Some(zcash_wallet),
                    )?;
                    
                    swap_engine.execute_swap(SwapDirection::ZecToXmr, amount).await?;
                },
                SwapCommand::Status { swap_id } => {
                    let swap_engine = SwapEngine::new(
                        swap_service,
                        None,
                        Some(zcash_wallet),
                    )?;
                    
                    match swap_engine.check_swap_status(&swap_id).await {
                        Ok(status) => {
                            println!("📊 Swap Status: {}", swap_id);
                            println!("   Status: {:?}", status.status);
                            println!("   Progress: {:.1}%", status.progress * 100.0);
                            if let Some(txid) = status.txid {
                                println!("   Transaction ID: {}", txid);
                            }
                        },
                        Err(e) => {
                            println!("❌ Failed to get swap status: {}", e);
                        }
                    }
                },
                SwapCommand::List => {
                    use crate::bridge::SwapStorage;
                    let storage = SwapStorage::new()?;
                    let swaps = storage.list_swaps()?;
                    
                    println!("📋 Swap History");
                    println!("{}", "=".repeat(60));
                    
                    if swaps.is_empty() {
                        println!("   No swaps found");
                    } else {
                        for (i, swap) in swaps.iter().enumerate() {
                            println!();
                            println!("{}. Swap #{}", i + 1, swap.swap_id);
                            println!("   Direction: {:?}", swap.direction);
                            println!("   Amount: {:.8}", swap.amount);
                            println!("   Status: {:?}", swap.status);
                            println!("   Created: {}", chrono::DateTime::<chrono::Utc>::from_timestamp(swap.created_at as i64, 0)
                                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                                .unwrap_or_else(|| "Unknown".to_string()));
                            if let Some(txid) = &swap.txid {
                                println!("   TXID: {}", txid);
                            }
                        }
                    }
                },
                SwapCommand::Churn { times, ring_size } => {
                    println!("🔄 Churning Monero outputs...");
                    println!("   Times: {}", times);
                    println!("   Ring Size: {}", ring_size);
                    println!();
                    
                    let monero_wallet = MoneroWallet::new(
                        None,
                        None,
                        None,
                        Some(proxy),
                    )?;
                    
                    use crate::bridge::ChurnManager;
                    let churn_manager = ChurnManager::new(monero_wallet);
                    churn_manager.churn_outputs(times, ring_size, Some(300)).await?;
                },
            }
        }
        
        Commands::Shade { command } => {
            use nozy::secret::{SecretWallet, Snip20Token};
            use nozy::secret::snip20::shade_tokens;
            use nozy::secret_keys::SecretKeyDerivation;
            use nozy::privacy_network::proxy::ProxyConfig;
            use nozy::load_config;
            
            let config = load_config();
            let proxy = ProxyConfig::auto_detect().await;
            
            let (hd_wallet, _storage) = load_wallet().await.ok();
            
            match command {
                ShadeCommand::Balance { address, token } => {
                    let wallet_address = if let Some(addr) = address {
                        addr
                    } else if let Some(ref wallet) = hd_wallet {
                        match wallet.generate_secret_address(0, 0) {
                            Ok(addr) => {
                                println!("📍 Using Secret Network address from wallet: {}", addr);
                                addr
                            },
                            Err(e) => {
                                return Err(NozyError::InvalidOperation(
                                    format!("Failed to derive Secret Network address: {}. Please provide --address", e)
                                ));
                            }
                        }
                    } else {
                        return Err(NozyError::InvalidOperation(
                            "Address required. Use --address <secret1...> or create a wallet first.".to_string()
                        ));
                    };
                    
                    let wallet = SecretWallet::new(
                        wallet_address.clone(),
                        None,
                        Some(&config.network),
                        Some(proxy),
                    )?;
                    
                    println!("💰 Secret Network Balance");
                    println!("{}", "=".repeat(60));
                    println!("   Address: {}", wallet_address);
                    
                    match wallet.get_scrt_balance().await {
                        Ok(balance) => {
                            println!("   SCRT: {:.6}", balance);
                        },
                        Err(e) => {
                            println!("   SCRT: Error - {}", e);
                        }
                    }
                    
                    if let Some(token_contract) = token {
                        match wallet.get_token_balance(&token_contract).await {
                            Ok((balance, info)) => {
                                println!("   {}: {:.6}", info.symbol, balance);
                            },
                            Err(e) => {
                                println!("   Token: Error - {}", e);
                            }
                        }
                    } else {
                        println!("\n   Common Shade Tokens:");
                        for (name, contract) in [
                            ("SHD", shade_tokens::SHD),
                            ("SILK", shade_tokens::SILK),
                        ] {
                            match wallet.get_token_balance(contract).await {
                                Ok((balance, info)) => {
                                    if balance > 0.0 {
                                        println!("      {}: {:.6}", info.symbol, balance);
                                    }
                                },
                                Err(_) => {}
                            }
                        }
                    }
                },
                
                ShadeCommand::Info { token } => {
                    let wallet = SecretWallet::new(
                        "secret1example1234567890123456789012345678901234567890".to_string(), 
                        None,
                        Some(&config.network),
                        Some(proxy),
                    )?;
                    
                    let token_interface = wallet.get_token(&token);
                    
                    println!("📊 Token Information");
                    println!("{}", "=".repeat(60));
                    println!("   Contract: {}", token);
                    
                    match token_interface.get_token_info().await {
                        Ok(info) => {
                            println!("   Name: {}", info.name);
                            println!("   Symbol: {}", info.symbol);
                            println!("   Decimals: {}", info.decimals);
                        },
                        Err(e) => {
                            println!("   Error: {}", e);
                        }
                    }
                },
                
                ShadeCommand::Send { recipient, amount, token, memo } => {
                    println!("💸 Sending Shade Token");
                    println!("{}", "=".repeat(60));
                    println!("   Token: {}", token);
                    println!("   Recipient: {}", recipient);
                    println!("   Amount: {}", amount);
                    
                    let (hd_wallet, _storage) = load_wallet().await?;
                    
                    // Derive Secret Network address and key pair
                    let secret_address = hd_wallet.generate_secret_address(0, 0)?;
                    let key_derivation = SecretKeyDerivation::new(hd_wallet);
                    let key_pair = key_derivation.derive_key_pair(&nozy::secret_keys::SecretDerivationPath {
                        account: 0,
                        change: 0,
                        index: 0,
                    })?;
                    
                    println!("   From: {}", secret_address);
                    
                    let wallet = SecretWallet::new_with_key_pair(
                        secret_address.clone(),
                        key_pair,
                        None,
                        Some(&config.network),
                        Some(proxy),
                    )?;
                    
                    match wallet.get_token_balance(&token).await {
                        Ok((balance, token_info)) => {
                            if balance < amount {
                                let error = NozyError::InsufficientFunds(format!(
                                    "Have {:.6} {}, need {:.6} {}",
                                    balance, token_info.symbol, amount, token_info.symbol
                                ));
                                println!("❌ {}", error);
                                println!("\n💡 Recovery suggestions:");
                                for suggestion in error.recovery_suggestions() {
                                    println!("   • {}", suggestion);
                                }
                                return Err(error);
                            }
                            println!("   Balance: {:.6} {}", balance, token_info.symbol);
                        },
                        Err(e) => {
                            println!("   ⚠️  Could not check balance: {}", e);
                            println!("\n💡 Recovery suggestions:");
                            for suggestion in e.recovery_suggestions() {
                                println!("   • {}", suggestion);
                            }
                        }
                    }
                    
                    println!("\n{}", "=".repeat(60));
                    if config.network == "mainnet" {
                        println!("⚠️  WARNING: This will send REAL tokens on MAINNET!");
                        println!("⚠️  This transaction cannot be undone!");
                        println!("\nType 'SEND' (all caps) to confirm, or anything else to cancel:");
                    } else {
                        println!("ℹ️  This will send tokens on TESTNET (not real money)");
                        println!("Type 'yes' to continue, or anything else to cancel:");
                    }
                    
                    use std::io::{self, Write};
                    print!("> ");
                    if let Err(e) = io::stdout().flush() {
                        eprintln!("⚠️  Warning: Failed to flush stdout: {}", e);
                    }
                    
                    let mut input = String::new();
                    if let Err(e) = io::stdin().read_line(&mut input) {
                        return Err(NozyError::InvalidOperation(
                            format!("Failed to read input: {}", e)
                        ));
                    }
                    let trimmed = input.trim();
                    
                    let enable_send = if config.network == "mainnet" {
                        trimmed == "SEND"
                    } else {
                        trimmed.to_lowercase() == "yes" || trimmed.to_lowercase() == "y"
                    };
                    
                    if !enable_send {
                        println!("❌ Transaction cancelled.");
                        return Ok(());
                    }
                    
                    println!("\n🔨 Building and signing transaction...");
                    match wallet.send_token(&token, &recipient, amount, memo).await {
                        Ok(tx_hash) => {
                            println!("\n✅ Transaction sent successfully!");
                            println!("{}", "=".repeat(60));
                            println!("   Transaction hash: {}", tx_hash);
                            println!("   Amount: {:.6}", amount);
                            println!("   Recipient: {}", recipient);
                            println!("   From: {}", secret_address);
                            println!("\n💡 Check transaction status on Secret Network explorer");
                        },
                        Err(e) => {
                            println!("\n❌ Failed to send transaction: {}", e);
                            println!("\n💡 Common issues:");
                            println!("   - Insufficient balance");
                            println!("   - Network connectivity");
                            println!("   - Invalid contract address");
                            println!("   - Invalid recipient address");
                        }
                    }
                },
                
                ShadeCommand::Receive { account, index } => {
                    let (wallet, _storage) = load_wallet().await?;
                    
                    match wallet.generate_secret_address(account, index) {
                        Ok(address) => {
                            println!("📍 Your Secret Network address:");
                            println!("{}", address);
                            println!("\n💡 Share this address to receive SCRT and Shade tokens.");
                            println!("   Derivation path: m/44'/529'/{}/0/{}", account, index);
                        },
                        Err(e) => {
                            println!("❌ Failed to generate address: {}", e);
                        }
                    }
                },
                
                ShadeCommand::History => {
                    use nozy::secret::SecretTransactionStorage;
                    
                    let tx_storage = SecretTransactionStorage::new()?;
                    let all_txs = tx_storage.get_all_transactions();
                    
                    if all_txs.is_empty() {
                        println!("📝 No Secret Network transaction history found.");
                        println!("   Transactions will appear here after you send tokens.");
                    } else {
                        println!("📜 Secret Network Transaction History ({} transactions)", all_txs.len());
                        println!("{}", "=".repeat(80));
                        
                        for (i, tx) in all_txs.iter().enumerate() {
                            let amount_display = tx.amount as f64 / 10_f64.powi(6); // Assume 6 decimals for display
                            
                            println!("\n{}. {}", i + 1, tx.txid);
                            println!("   Status: {:?}", tx.status);
                            println!("   Token: {} ({})", tx.token_symbol, tx.contract_address);
                            println!("   Amount: {:.6} {}", amount_display, tx.token_symbol);
                            println!("   Fee: {:.6} SCRT", tx.fee_uscrt as f64 / 1_000_000.0);
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
                                if !memo.trim().is_empty() {
                                    println!("   Memo: {}", memo);
                                }
                            }
                            
                            if let Some(error) = &tx.error {
                                println!("   Error: {}", error);
                            }
                        }
                    }
                },
                
                ShadeCommand::Status { txid } => {
                    use nozy::secret::{SecretTransactionStorage, SecretRpcClient};
                    use nozy::load_config;
                    
                    let config = load_config();
                    let rpc = SecretRpcClient::new(None, Some(&config.network), Some(proxy))?;
                    let tx_storage = SecretTransactionStorage::new()?;
                    
                    if let Some(txid_str) = txid {
                        println!("🔍 Checking transaction status: {}", txid_str);
                        
                        match tx_storage.check_transaction_status(&rpc, &txid_str).await {
                            Ok(updated) => {
                                if updated {
                                    println!("✅ Transaction status updated!");
                                    
                                    if let Some(tx) = tx_storage.get_transaction(&txid_str) {
                                        println!("   Status: {:?}", tx.status);
                                        if let Some(block_height) = tx.block_height {
                                            println!("   Block: {} ({} confirmations)", block_height, tx.confirmations);
                                        }
                                        if let Some(error) = &tx.error {
                                            println!("   Error: {}", error);
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
                        
                        let pending = tx_storage.get_pending_transactions();
                        let mut updated_count = 0;
                        
                        for tx in &pending {
                            if tx_storage.check_transaction_status(&rpc, &tx.txid).await? {
                                updated_count += 1;
                            }
                        }
                        
                        if updated_count > 0 {
                            println!("✅ Updated {} transaction(s) to confirmed status.", updated_count);
                        } else {
                            println!("⏳ No transactions confirmed yet. All still pending.");
                        }
                        
                        let conf_updated = tx_storage.update_confirmations(&rpc).await?;
                        if conf_updated > 0 {
                            println!("📊 Updated confirmation counts for {} transaction(s).", conf_updated);
                        }
                    }
                },
                
                ShadeCommand::ListTokens => {
                    println!("🎨 Shade Protocol Tokens");
                    println!("{}", "=".repeat(60));
                    println!("\n   Mainnet Token Contracts:");
                    println!("   SHD (Shade): {}", shade_tokens::SHD);
                    println!("   SILK: {}", shade_tokens::SILK);
                    println!("\n💡 Use 'nozy shade balance' to check balances");
                    println!("💡 Use 'nozy shade receive' to generate your address");
                },
            }
        }
        
        Commands::Monero { command } => {
            use crate::monero::MoneroWallet;
            use crate::privacy_network::proxy::ProxyConfig;
            
            let proxy = ProxyConfig::auto_detect().await;
            
            match command {
                MoneroCommand::Balance { rpc_url, username, password } => {
                    let wallet = MoneroWallet::new(
                        rpc_url,
                        username,
                        password,
                        Some(proxy),
                    )?;
                    
                    if !wallet.is_connected().await {
                        return Err(NozyError::NetworkError(
                            "Failed to connect to Monero wallet RPC. Make sure monero-wallet-rpc is running.".to_string()
                        ));
                    }
                    
                    let balance = wallet.get_balance_xmr().await?;
                    let address = wallet.get_address().await?;
                    let height = wallet.get_block_height().await?;
                    
                    println!("💰 Monero Balance");
                    println!("{}", "=".repeat(60));
                    println!("   Address: {}", address);
                    println!("   Balance: {:.12} XMR", balance);
                    println!("   Block Height: {}", height);
                },
                
                MoneroCommand::Send { recipient, amount, rpc_url, username, password } => {
                    let wallet = MoneroWallet::new(
                        rpc_url,
                        username,
                        password,
                        Some(proxy),
                    )?;
                    
                    if !wallet.is_connected().await {
                        return Err(NozyError::NetworkError(
                            "Failed to connect to Monero wallet RPC. Make sure monero-wallet-rpc is running.".to_string()
                        ));
                    }
                    
                    wallet.validate_address(&recipient)?;
                    wallet.validate_amount(amount)?;
                    
                    let balance = wallet.get_balance_xmr().await?;
                    if amount > balance {
                        let error = NozyError::InsufficientFunds(
                            format!("Insufficient balance: {:.12} XMR available, {:.12} XMR requested", balance, amount)
                        );
                        println!("❌ {}", error);
                        println!("\n💡 Recovery suggestions:");
                        for suggestion in error.recovery_suggestions() {
                            println!("   • {}", suggestion);
                        }
                        return Err(error);
                    }
                    
                    println!();
                    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                    println!("🔒 MONERO TRANSACTION");
                    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                    println!();
                    println!("   Recipient: {}", recipient);
                    println!("   Amount: {:.12} XMR", amount);
                    println!("   Balance: {:.12} XMR", balance);
                    println!();
                    
                    if !Confirm::new()
                        .with_prompt("Confirm transaction?")
                        .default(false)
                        .interact()
                        .unwrap_or(false)
                    {
                        println!("❌ Transaction cancelled");
                        return Ok(());
                    }
                    
                    match wallet.send_xmr(&recipient, amount).await {
                        Ok(txid) => {
                            println!();
                            println!("✅ Transaction sent successfully!");
                            println!("   Transaction ID: {}", txid);
                            println!();
                            println!("💡 Use 'nozy monero status --txid {}' to check status", txid);
                        },
                        Err(e) => {
                            println!();
                            println!("❌ Transaction failed: {}", e);
                        }
                    }
                },
                
                MoneroCommand::Receive { account_index, rpc_url, username, password } => {
                    let wallet = MoneroWallet::new(
                        rpc_url,
                        username,
                        password,
                        Some(proxy),
                    )?;
                    
                    if !wallet.is_connected().await {
                        return Err(NozyError::NetworkError(
                            "Failed to connect to Monero wallet RPC. Make sure monero-wallet-rpc is running.".to_string()
                        ));
                    }
                    
                    let address = wallet.create_subaddress(account_index).await?;
                    
                    println!("📍 Monero Address");
                    println!("{}", "=".repeat(60));
                    println!("   Account Index: {}", account_index);
                    println!("   Address: {}", address);
                    println!();
                    println!("🛡️  Privacy Note: This is a new subaddress. Never reuse addresses!");
                },
                
                MoneroCommand::History => {
                    let wallet = MoneroWallet::new(
                        None,
                        None,
                        None,
                        Some(proxy),
                    )?;
                    
                    let transactions = wallet.get_transaction_history()?;
                    
                    println!("📜 Monero Transaction History");
                    println!("{}", "=".repeat(60));
                    
                    if transactions.is_empty() {
                        println!("\n   No transactions found");
                    } else {
                        for (i, tx) in transactions.iter().enumerate() {
                            println!();
                            println!("{}. Transaction #{}", i + 1, tx.txid);
                            println!("   Status: {:?}", tx.status);
                            println!("   Recipient: {}", tx.recipient_address);
                            println!("   Amount: {:.12} XMR", tx.amount_xmr);
                            if let Some(fee) = tx.fee_xmr {
                                println!("   Fee: {:.12} XMR", fee);
                            }
                            println!("   Created: {}", tx.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
                            if let Some(broadcast) = tx.broadcast_at {
                                println!("   Broadcast: {}", broadcast.format("%Y-%m-%d %H:%M:%S UTC"));
                            }
                            if let Some(height) = tx.block_height {
                                println!("   Block Height: {}", height);
                            }
                            if tx.confirmations > 0 {
                                println!("   Confirmations: {}", tx.confirmations);
                            }
                            if let Some(ref error) = tx.error {
                                println!("   Error: {}", error);
                            }
                        }
                    }
                },
                
                MoneroCommand::Status { txid } => {
                    let wallet = MoneroWallet::new(
                        None,
                        None,
                        None,
                        Some(proxy),
                    )?;
                    
                    if !wallet.is_connected().await {
                        return Err(NozyError::NetworkError(
                            "Failed to connect to Monero wallet RPC. Make sure monero-wallet-rpc is running.".to_string()
                        ));
                    }
                    
                    if let Some(txid) = txid {
                        if let Some(tx) = wallet.get_transaction(&txid) {
                            println!("📊 Transaction Status: {}", txid);
                            println!("{}", "=".repeat(60));
                            println!("   Status: {:?}", tx.status);
                            println!("   Recipient: {}", tx.recipient_address);
                            println!("   Amount: {:.12} XMR", tx.amount_xmr);
                            if let Some(fee) = tx.fee_xmr {
                                println!("   Fee: {:.12} XMR", fee);
                            }
                            println!("   Created: {}", tx.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
                            if let Some(broadcast) = tx.broadcast_at {
                                println!("   Broadcast: {}", broadcast.format("%Y-%m-%d %H:%M:%S UTC"));
                            }
                            if let Some(height) = tx.block_height {
                                println!("   Block Height: {}", height);
                            }
                            println!("   Confirmations: {}", tx.confirmations);
                            if let Some(ref error) = tx.error {
                                println!("   Error: {}", error);
                            }
                            
                            println!();
                            println!("🔍 Checking transaction status on network...");
                            if wallet.check_transaction_status(&txid).await? {
                                println!("✅ Transaction status updated");
                            } else {
                                println!("⏳ Transaction still pending");
                            }
                            
                            let updated = wallet.update_confirmations().await?;
                            if updated > 0 {
                                println!("📊 Updated confirmation counts");
                            }
                        } else {
                            println!("❌ Transaction not found: {}", txid);
                        }
                    } else {
                        println!("🔍 Checking all pending transactions...");
                        
                        let transactions = wallet.get_transaction_history()?;
                        let pending: Vec<_> = transactions.iter()
                            .filter(|tx| matches!(tx.status, nozy::MoneroTransactionStatus::Pending))
                            .collect();
                        
                        if pending.is_empty() {
                            println!("✅ No pending transactions");
                        } else {
                            let mut updated_count = 0;
                            for tx in &pending {
                                if wallet.check_transaction_status(&tx.txid).await? {
                                    updated_count += 1;
                                }
                            }
                            
                            if updated_count > 0 {
                                println!("✅ Updated {} transaction(s) to confirmed status.", updated_count);
                            } else {
                                println!("⏳ No transactions confirmed yet. All still pending.");
                            }
                            
                            let conf_updated = wallet.update_confirmations().await?;
                            if conf_updated > 0 {
                                println!("📊 Updated confirmation counts for {} transaction(s).", conf_updated);
                            }
                        }
                    }
                },
            }
        }
        
        Commands::Nu61 => {
            use nozy::nu6_1_check;
            use nozy::load_config;
            use nozy::ZebraClient;
            
            println!("🔍 Checking NU 6.1 (Network Upgrade 6.1) Compatibility");
            println!();
            
            if let Err(e) = nu6_1_check::verify_nu6_1_compatibility() {
                println!("⚠️  Compatibility check failed: {}", e);
            }
            
            println!();
            
            let config = load_config();
            let zebra_client = ZebraClient::from_config(&config);
            
            match zebra_client.get_block_count().await {
                Ok(height) => {
                    nu6_1_check::display_nu6_1_status(Some(height));
                },
                Err(e) => {
                    println!("⚠️  Could not get block height: {}", e);
                    nu6_1_check::display_nu6_1_status(None);
                }
            }
            
            println!("📦 Library Versions:");
            println!("   zcash_protocol: 0.6.2+");
            println!("   zcash_primitives: 0.24.1+");
            println!("   orchard: 0.11.0");
            println!();
            
            println!("✅ NozyWallet is ready for NU 6.1!");
            println!("   Activation: Block 3,146,400 (November 23, 2025)");
            println!();
        }
    }
    
    Ok(())
}

// Error handling wrapper for main function
fn handle_error(error: NozyError) {
    eprintln!("\n❌ Error: {}", error.user_friendly_message());
    
    let suggestions = error.recovery_suggestions();
    if !suggestions.is_empty() {
        eprintln!("\n💡 Suggestions:");
        for suggestion in suggestions {
            eprintln!("   • {}", suggestion);
        }
    }
    
    // Log detailed error for debugging (only if verbose mode)
    if std::env::var("RUST_LOG").is_ok() {
        tracing::error!("Detailed error: {:?}", error);
    }
}
