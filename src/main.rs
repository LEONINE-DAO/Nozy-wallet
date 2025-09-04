use std::path::PathBuf;
use clap::{Parser, Subcommand};
use dialoguer::Password;
use nozy::{HDWallet, WalletStorage, NozyResult, NozyError, NoteScanner, ZebraClient, BlockParser, ZcashTransactionBuilder};

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
    New,
    /// Restore wallet from mnemonic
    Restore,
    /// Generate new addresses
    Addresses {
        #[arg(short, long, default_value_t = 1)]
        count: u32,
    },
    /// Scan for notes
    Scan {
        #[arg(long)]
        start_height: Option<u32>,
        #[arg(long)]
        end_height: Option<u32>,
    },
    /// Send ZEC
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
    Info,
    /// Test Zebra node connection
    TestZebra {
        /// Zebra node URL to test
        #[arg(long, default_value = "http://127.0.0.1:8232")]
        zebra_url: String,
    },
}

async fn load_wallet() -> NozyResult<(HDWallet, WalletStorage)> {
    let wallet = HDWallet::new()?;
    let storage = WalletStorage::new(PathBuf::from("wallet_data"));
    
    Ok((wallet, storage))
}

#[tokio::main]
async fn main() -> NozyResult<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::New => {
            println!("Creating new wallet...");
            let (wallet, storage) = load_wallet().await?;
            
            let password = Password::new()
                .with_prompt("Enter password for wallet encryption")
                .interact()
                .map_err(|e| nozy::NozyError::InvalidOperation(format!("Password input error: {}", e)))?;
            
            storage.save_wallet(&wallet, &password).await?;
            
            println!("Wallet created successfully!");
            println!("Mnemonic: {}", wallet.get_mnemonic());
            
            match wallet.generate_orchard_address(0, 0) {
                Ok(address) => println!("Sample address: {}", address),
                Err(e) => println!("Failed to generate sample address: {}", e),
            }
        }
        
        Commands::Restore => {
            println!("Restore wallet from mnemonic...");
            println!("Restore functionality not yet implemented");
        }
        
        Commands::Addresses { count } => {
            println!("Generating {} addresses...", count);
            let (wallet, _storage) = load_wallet().await?;
            
            for i in 0..count {
                match wallet.generate_orchard_address(0, i) {
                    Ok(address) => println!("Address {}: {}", i, address),
                    Err(e) => println!("Failed to generate address {}: {}", i, e),
                }
            }
        }
        
        Commands::Scan { start_height, end_height } => {
            println!("Scanning blockchain for notes...");
            
            let (wallet, _storage) = load_wallet().await?;
            let zebra_client = ZebraClient::new("https://zcash.electriccoin.co:8232".to_string());
            
            // Create note scanner with real implementation
            let mut note_scanner = NoteScanner::new(wallet, zebra_client);
            
            match note_scanner.scan_notes(start_height, end_height).await {
                Ok((result, spendable_notes)) => {
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
            
            let (_wallet, _storage) = load_wallet().await?;
            
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
            
            let spendable_notes = Vec::new();
            
            match tx_builder.build_send_transaction(&spendable_notes, &recipient, amount_zatoshis, fee_zatoshis) {
                Ok(signed_tx) => {
                    println!("Transaction built successfully!");
                    println!("Transaction ID: {}", signed_tx.txid);
                    println!("Transaction size: {} bytes", signed_tx.raw_transaction.len());
                    
                    if tx_builder.allow_mainnet_broadcast {
                        match tx_builder.broadcast_transaction(&signed_tx) {
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
    }
    
    Ok(())
}
