use clap::{Parser, Subcommand};
use dialoguer::{Confirm, Password};
use nozy::cli_helpers::{
    build_and_broadcast_transaction, handle_insufficient_funds_error, load_wallet,
    scan_notes_for_sending,
};
use nozy::local_analytics::LocalAnalytics;
use nozy::safe_display::display_mnemonic_safe;
use nozy::{load_config, save_config};
use nozy::{
    AddressBook, HDWallet, NozyError, NozyResult, WalletStorage, ZebraBlockParser,
    ZebraBlockSource, ZebraClient,
};
use zcash_address::unified::Encoding;
use zcash_protocol::consensus::NetworkType;
use zeaking::Zeaking;

fn use_plain_terminal_output() -> bool {
    use std::io::IsTerminal;

    if std::env::var("NOZY_PLAIN_OUTPUT").is_ok() {
        return true;
    }

    if !std::io::stdout().is_terminal() {
        return true;
    }

    std::env::var("TERM_PROGRAM")
        .map(|v| v.to_ascii_lowercase().contains("cursor"))
        .unwrap_or(false)
}

fn network_type_from_config(network: &str) -> NetworkType {
    if network == "testnet" {
        NetworkType::Test
    } else {
        NetworkType::Main
    }
}

fn configure_ironwood_testnet(_config: &mut nozy::WalletConfig, rpc_url: &str) -> NozyResult<()> {
    let profile_id = nozy::active_profile_id().ok_or_else(|| {
        NozyError::InvalidOperation(
            "No active wallet profile. Create or select a profile first.".to_string(),
        )
    })?;
    nozy::configure_profile_network(&profile_id, "testnet", rpc_url, true)
}

fn print_profile_list() -> NozyResult<()> {
    let profiles = nozy::list_wallet_profiles()?;
    let active_id = nozy::active_profile_id();

    if profiles.is_empty() {
        println!("No wallet profiles found.");
        return Ok(());
    }

    println!("Wallet profiles:");
    for profile in profiles {
        let active = if active_id.as_deref() == Some(profile.id.as_str()) {
            "*"
        } else {
            " "
        };
        let wallet = if nozy::profile_has_wallet(&profile.id) {
            "wallet"
        } else {
            "empty"
        };
        println!(" {active} {}  {}  ({wallet})", profile.id, profile.name);
    }
    Ok(())
}

async fn save_wallet_to_active_profile(wallet: &mut HDWallet) -> NozyResult<()> {
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
    storage.save_wallet(wallet, &password).await?;
    Ok(())
}

#[derive(Parser)]
#[command(name = "nozy")]
#[command(version = nozy::version_info::VERSION_DISPLAY)]
#[command(about = "NozyWallet / Nozy Lite — privacy-first Orchard CLI (ops health, sync, send)")]
#[command(
    long_about = "NozyWallet is a privacy-first Orchard wallet. The CLI is productized as Nozy Lite for operator uptime and data checks next to Zebrad (health/--json/TUI), plus sync, send, and Ironwood. Fully shielded by default."
)]
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

    /// Machine-readable JSON on supported ops commands (health, status, balance)
    #[arg(long, global = true)]
    pub json: bool,

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
        #[arg(
            long,
            help = "Starting block height for scanning (default: last scanned height + 1)"
        )]
        start_height: Option<u32>,
        #[arg(
            long,
            help = "Ending block height for scanning (default: if --start-height is set, chain tip; otherwise min(tip, start + 1000) for incremental sync)"
        )]
        end_height: Option<u32>,
        #[arg(
            long,
            help = "Scan from last scanned height (or --start-height) through chain tip — use after receiving funds"
        )]
        to_tip: bool,
        #[arg(
            long,
            help = "Override Zebra RPC URL (overrides config and global --zebra-url)"
        )]
        zebra_url: Option<String>,
    },

    #[command(about = "Send ZEC to a shielded Orchard address")]
    Send {
        #[arg(
            long,
            short = 'r',
            help = "Recipient's shielded Orchard address (must start with 'u1')"
        )]
        recipient: String,
        #[arg(long, short = 'a', help = "Amount to send in ZEC (e.g., 0.1)")]
        amount: f64,
        #[arg(
            long,
            help = "Override Zebra RPC URL (overrides config and global --zebra-url)"
        )]
        zebra_url: Option<String>,
        // No short flag: global `--mainnet` already uses `-m`.
        #[arg(long, help = "Optional memo message (max 512 characters)")]
        memo: Option<String>,
    },

    #[command(about = "Display wallet information including addresses and network")]
    Info,

    #[command(about = "Display the current shielded balance")]
    Balance {
        #[arg(long, help = "Emit JSON (also accepts global --json)")]
        json: bool,
    },

    #[command(about = "Manage local wallet profiles")]
    Profile {
        #[command(subcommand)]
        command: ProfileCommand,
    },

    #[command(about = "Create, restore, or select a dedicated Ironwood testnet wallet")]
    TestnetWallet {
        #[command(subcommand)]
        command: TestnetWalletCommand,
    },

    #[command(about = "Configure wallet settings (network, Zebra URL, backend, etc.)")]
    Config {
        #[arg(long, help = "Set the Zebra RPC URL (e.g., http://127.0.0.1:8232)")]
        set_zebra_url: Option<String>,
        #[arg(long, help = "Set network: 'mainnet' or 'testnet'")]
        set_network: Option<String>,
        #[arg(long, help = "Use local Zebra node at http://127.0.0.1:8232")]
        use_local: bool,
        #[arg(
            long,
            help = "Use remote Zebra node (provide URL: --use-remote http://host:port)"
        )]
        use_remote: Option<String>,
        #[arg(long, help = "Use Crosslink backend instead of Zebra")]
        use_crosslink: bool,
        #[arg(long, help = "Use Zebra backend (default)")]
        use_zebra: bool,
        #[arg(long, help = "Set the Crosslink RPC URL")]
        set_crosslink_url: Option<String>,
        #[arg(long, help = "Display current backend configuration")]
        show_backend: bool,
        #[arg(long, help = "Set protocol version (e.g., 170140 for NU 6.1)")]
        set_protocol: Option<String>,
    },

    #[command(about = "Test connection to Zebra or Crosslink node")]
    TestZebra {
        #[arg(
            long,
            help = "Override Zebra RPC URL (overrides config and global --zebra-url)"
        )]
        zebra_url: Option<String>,
    },

    #[command(about = "Download and manage Orchard proving parameters")]
    Proving {
        #[arg(
            long,
            short = 'd',
            help = "Download proving parameters from official source"
        )]
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
    Status {
        #[arg(long, help = "Live TUI dashboard (same as `nozy tui`)")]
        watch: bool,
        #[arg(
            long,
            default_value_t = 5,
            help = "Refresh interval seconds for --watch / TUI"
        )]
        interval: u64,
        #[arg(
            long,
            help = "Emit JSON sync+balance summary (also accepts global --json)"
        )]
        json: bool,
    },

    #[command(about = "Nozy Lite health check for monitoring (exit codes for cron/systemd)")]
    Health {
        #[arg(
            long,
            default_value_t = nozy::DEFAULT_MAX_SCAN_GAP,
            help = "Max allowed RPC scan gap (blocks) before exit 2"
        )]
        max_scan_gap: u32,
        #[arg(long, help = "Fail (exit 3) if lightwalletd tip is unavailable")]
        require_lwd: bool,
        #[arg(
            long,
            help = "Fail (exit 3) if Ironwood pool is not reported by Zebra RPC"
        )]
        require_ironwood_rpc: bool,
        #[arg(long, help = "Emit JSON report (also accepts global --json)")]
        json: bool,
    },

    #[command(about = "Nozy Lite live status TUI (chain tip, scan gap, balances)")]
    Tui {
        #[arg(long, default_value_t = 5, help = "Refresh interval seconds")]
        interval: u64,
    },

    #[command(about = "lightwalletd compact-block cache (Zeaking LWD)")]
    Lwd {
        #[command(subcommand)]
        command: LwdCommand,
    },

    #[command(about = "Manage saved addresses in your address book")]
    AddressBook {
        #[command(subcommand)]
        command: AddressBookCommand,
    },

    #[command(about = "Commands for Zeaking local blockchain indexer")]
    Zeaking {
        #[command(subcommand)]
        command: ZeakingCommand,
    },

    #[command(about = "Test and manage privacy network connections (Tor, I2P)")]
    PrivacyNetwork {
        #[command(subcommand)]
        command: PrivacyNetworkCommand,
    },

    #[command(about = "Cross-chain swap functionality (XMR to ZEC, etc.)")]
    Swap {
        #[command(subcommand)]
        command: SwapCommand,
    },

    #[cfg(feature = "secret-network")]
    #[command(about = "Shade Protocol (Secret Network) integration commands")]
    Shade {
        #[command(subcommand)]
        command: ShadeCommand,
    },

    #[command(about = "Monero integration and verification commands")]
    Monero {
        #[command(subcommand)]
        command: MoneroCommand,
    },

    #[command(about = "Display information about Zcash Network Upgrade 6.1 (NU 6.1)")]
    Nu61,

    #[command(about = "Ironwood (NU6.3) pool status and Orchard → Ironwood migration")]
    Ironwood {
        #[command(subcommand)]
        command: IronwoodCommand,
    },
}

#[derive(Subcommand)]
pub enum IronwoodCommand {
    #[command(about = "Show Ironwood readiness, pool balances, and migration status")]
    Status,
    #[command(about = "List Orchard notes that would migrate through the turnstile")]
    Plan {
        #[arg(long, help = "Persist the draft ZIP 318 migration schedule")]
        save: bool,
    },
    #[command(about = "Check Ironwood migration readiness without building transactions")]
    Preflight,
    #[command(about = "Migrate Orchard notes to Ironwood (requires NU6.3 active)")]
    Migrate,
    #[command(
        about = "Broadcast a presigned ZIP 318 turnstile transaction in its anchor bucket window"
    )]
    Broadcast {
        #[arg(
            long,
            help = "Validate the window and show the txid without broadcasting"
        )]
        dry_run: bool,
        #[arg(
            long,
            help = "After broadcast, poll Zebrad until the transaction confirms on chain"
        )]
        wait_confirm: bool,
        #[arg(
            long,
            help = "Attest that NymVPN/Tor (or equivalent) is protecting this machine's egress"
        )]
        attest_private_network: bool,
        #[arg(
            long,
            help = "Allow clearnet broadcast to a remote node (discouraged; IP may link to migration)"
        )]
        force_clearnet: bool,
    },
    #[command(
        about = "Split one Orchard note into canonical ZIP 318 denominations (send-to-self)"
    )]
    Split {
        #[arg(
            long,
            help = "Plan the split and build the transaction without broadcasting"
        )]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
pub enum ProfileCommand {
    #[command(about = "List local wallet profiles")]
    List,
    #[command(about = "Switch the active local wallet profile")]
    Switch {
        #[arg(long, help = "Wallet profile ID from `nozy profile list`")]
        profile_id: String,
    },
}

#[derive(Subcommand)]
pub enum TestnetWalletCommand {
    #[command(about = "Create a new dedicated Ironwood testnet wallet profile")]
    New {
        #[arg(long, default_value = "Ironwood Testnet")]
        name: String,
        #[arg(long, default_value = "http://127.0.0.1:18232")]
        rpc_url: String,
    },
    #[command(about = "Restore a dedicated Ironwood testnet wallet profile from mnemonic")]
    Restore {
        #[arg(long, default_value = "Ironwood Testnet")]
        name: String,
        #[arg(long, default_value = "http://127.0.0.1:18232")]
        rpc_url: String,
    },
    #[command(about = "Use an existing profile as the Ironwood testnet wallet")]
    Use {
        #[arg(long, help = "Wallet profile ID from `nozy profile list`")]
        profile_id: String,
        #[arg(long, default_value = "http://127.0.0.1:18232")]
        rpc_url: String,
    },
    #[command(about = "Show active testnet-wallet readiness")]
    Status,
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
pub enum LwdCommand {
    #[command(about = "Remove compact blocks above current lightwalletd tip (fixes stale cache)")]
    Prune,
    #[command(about = "Download compact blocks from next missing height through lightwalletd tip")]
    SyncToTip {
        #[arg(
            long,
            help = "Optional lower bound for first height to fetch (e.g. wallet birthday)"
        )]
        start_floor: Option<u64>,
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

#[cfg(feature = "secret-network")]
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
    let cli = Cli::parse();

    if cli.verbose {
        std::env::set_var("RUST_LOG", "debug");
    } else if cli.quiet {
        std::env::set_var("RUST_LOG", "error");
    }

    if std::env::var("RUST_LOG").is_ok() {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .init();
    }

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

    if let Some(url) = cli.zebra_url {
        config.zebra_url = url;
        if !cli.quiet {
            println!("🔗 Using Zebra URL: {}", config.zebra_url);
        }
    }

    if let Err(e) = execute_command(cli.command, config).await {
        handle_error(e);
        std::process::exit(1);
    }
}

async fn execute_command(_command: Commands, mut config: nozy::WalletConfig) -> NozyResult<()> {
    let cli = Cli::parse();

    if cli.verbose {
        std::env::set_var("RUST_LOG", "debug");
    } else if cli.quiet {
        std::env::set_var("RUST_LOG", "error");
    }

    if std::env::var("RUST_LOG").is_ok() {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .init();
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
                    .map_err(|e| {
                        NozyError::InvalidOperation(format!("Password input error: {}", e))
                    })?;

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
            println!(
                "   ⚠️  SECURITY: This mnemonic will NOT be shown again for security reasons."
            );

            let network = network_type_from_config(&config.network);
            match wallet.generate_orchard_address(0, 0, network) {
                Ok(address) => {
                    println!("📍 Your wallet address:");
                    println!("{}", address);
                    println!("\n💡 Share this address to receive ZEC.");
                }
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
                .map_err(|e| {
                    nozy::NozyError::InvalidOperation(format!("Mnemonic input error: {}", e))
                })?;

            let wallet = HDWallet::from_mnemonic(&mnemonic)?;
            let storage = WalletStorage::with_xdg_dir();

            let password = Password::new()
                .with_prompt("Enter password to encrypt wallet")
                .interact()
                .map_err(|e| {
                    nozy::NozyError::InvalidOperation(format!("Password input error: {}", e))
                })?;

            storage.save_wallet(&wallet, &password).await?;
            println!("✅ Wallet restored and saved.");
        }

        Commands::Receive => {
            let (wallet, _storage) = load_wallet().await?;
            let network = network_type_from_config(&config.network);

            match wallet.generate_orchard_address(0, 0, network) {
                Ok(address) => {
                    println!("Your wallet address:");
                    println!("{}", address);
                    println!("\n💡 Share this address to receive ZEC.");
                    println!("   After a deposit confirms on-chain, run:");
                    println!("     nozy -m sync --to-tip");
                    println!("   Plain `sync` only advances ~1000 blocks; --to-tip scans through chain tip.");
                }
                Err(e) => {
                    println!("❌ Failed to generate address: {}", e);
                }
            }
        }

        Commands::Sync {
            start_height,
            end_height,
            to_tip,
            zebra_url,
        } => {
            if let Some(url) = zebra_url {
                config.zebra_url = url;
            }

            let (wallet, _storage) = load_wallet().await?;

            let options = nozy::WalletSyncOptions {
                start_height,
                end_height,
                scan_to_tip: to_tip,
                zebra_url: Some(config.zebra_url.clone()),
                ..nozy::WalletSyncOptions::default()
            };

            if to_tip {
                let chain_tip = ZebraClient::from_config(&config).get_block_count().await?;
                let scan_start = start_height
                    .or_else(|| config.last_scan_height.map(|h| h.saturating_add(1)))
                    .unwrap_or(if config.network == "testnet" {
                        1
                    } else {
                        nozy::MAINNET_DEFAULT_SCAN_START
                    });
                let scan_end = end_height.unwrap_or(chain_tip).min(chain_tip);
                if scan_start <= scan_end {
                    println!(
                        "🔄 Sync to chain tip: scanning blocks {} to {} ({} blocks)",
                        scan_start,
                        scan_end,
                        scan_end.saturating_sub(scan_start).saturating_add(1)
                    );
                } else {
                    println!("🔄 Sync to chain tip: advancing Orchard witnesses toward height {chain_tip}");
                }
            }

            match nozy::sync_wallet_notes(wallet, options).await {
                Ok(result) => {
                    let balance_zec = result.balance_zatoshis as f64 / 100_000_000.0;
                    if result.already_synced {
                        println!("✅ Already synced to chain tip (witnesses fresh)");
                    } else if result.balance_zatoshis > 0 {
                        println!("✅ Sync complete! Balance: {:.8} ZEC", balance_zec);
                        println!(
                            "   Found {} new notes, {} total notes",
                            result.new_notes_in_scan, result.total_notes
                        );
                    } else {
                        println!("✅ Sync complete! Balance: 0.00000000 ZEC");
                        if result.new_notes_in_scan > 0 {
                            println!("   Found {} new notes", result.new_notes_in_scan);
                        }
                    }
                    println!("   Last scanned height: {}", result.last_scan_height);
                    if result.chain_tip > result.last_scan_height {
                        let behind = result.chain_tip - result.last_scan_height;
                        println!(
                            "   ⚠️  Chain tip is {} ({} blocks ahead of this scan)",
                            result.chain_tip, behind
                        );
                        println!(
                            "   💡 Recent deposits are often in newer blocks — run: nozy sync --to-tip"
                        );
                    }
                }
                Err(e) => {
                    println!("❌ Error updating wallet: {}", e.message);
                    if let (Some(start), Some(end), Some(tip)) =
                        (e.scan_start, e.scan_end, e.chain_tip)
                    {
                        println!("   Scan range: {start}-{end} (tip {tip})");
                    }
                    return Err(NozyError::NoteScanning(e.message));
                }
            }
        }

        Commands::Send {
            recipient,
            amount,
            zebra_url,
            memo,
        } => {
            if let Some(url) = zebra_url {
                config.zebra_url = url;
            }

            let (wallet, _storage) = load_wallet().await?;

            let address_book = AddressBook::new()?;
            let actual_recipient =
                if let Some(address) = address_book.get_address_by_name(&recipient) {
                    println!("📇 Found '{}' in address book: {}", recipient, address);
                    let _ = address_book.update_address_usage(&recipient);
                    address
                } else {
                    recipient.clone()
                };

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
                    use zcash_address::unified::Container;
                    let mut has_orchard = false;
                    for item in ua.items() {
                        if matches!(item, zcash_address::unified::Receiver::Orchard(_)) {
                            has_orchard = true;
                        }
                    }
                    if !has_orchard {
                        return Err(NozyError::AddressParsing(
                            "Recipient must include an Orchard receiver (Orchard-only wallet)."
                                .to_string(),
                        ));
                    }
                }
                Err(e) => {
                    return Err(NozyError::AddressParsing(format!(
                        "Invalid recipient address: {}",
                        e
                    )));
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
            println!(
                "  Network:   {}",
                if is_mainnet {
                    "MAINNET ⚠️"
                } else {
                    "TESTNET"
                }
            );

            println!("\n💸 Estimating transaction fee...");
            let memo_preview = memo
                .as_deref()
                .map(|m| m.trim().as_bytes())
                .filter(|b| !b.is_empty());
            let fee_zatoshis = nozy::cli_helpers::estimate_transaction_fee_for_send(
                &zebra_client,
                memo_preview,
                true,
            )
            .await;
            let total_amount = amount_zatoshis + fee_zatoshis;
            println!(
                "  Fee:       {:.8} ZEC",
                fee_zatoshis as f64 / 100_000_000.0
            );
            println!(
                "  Total:     {:.8} ZEC (amount + fee)",
                total_amount as f64 / 100_000_000.0
            );

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
            let spendable_notes = if use_plain_terminal_output() {
                println!("   Plain terminal mode enabled (no spinner/progress UI).");
                scan_notes_for_sending(wallet, &config.zebra_url).await?
            } else {
                use nozy::progress::create_tx_progress_bar;
                let pb = create_tx_progress_bar();
                pb.set_message("Scanning blockchain for spendable notes...");
                let res = scan_notes_for_sending(wallet, &config.zebra_url).await?;
                pb.finish_with_message("✅ Scan complete");
                res
            };

            if spendable_notes.is_empty() {
                return Err(NozyError::InvalidOperation(
                    "No spendable notes found. Run 'sync' to scan the blockchain first."
                        .to_string(),
                ));
            }

            let orchard_total: u64 = spendable_notes
                .iter()
                .map(|note| note.orchard_note.note.value().inner())
                .sum();
            let total_available = orchard_total;

            println!(
                "  Shielded ZEC: {:.8} ZEC ({} notes)",
                total_available as f64 / 100_000_000.0,
                spendable_notes.len(),
            );
            let ironwood_count = spendable_notes
                .iter()
                .filter(|n| n.pool == nozy::shielded_pool::ShieldedPool::Ironwood)
                .count();
            if ironwood_count > 0 {
                println!("  Pool:      Ironwood (NU6.3 post-activation send path)");
            }

            if total_available < total_amount {
                let error = NozyError::InsufficientFunds(format!(
                    "Shielded ZEC: {:.8} ZEC, Required: {:.8} ZEC",
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
                return Err(NozyError::InvalidOperation(format!(
                    "Failed to read input: {}",
                    e
                )));
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
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.as_bytes().to_vec())
                }
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
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.as_bytes().to_vec())
                }
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

            use nozy::paths::get_wallet_data_dir;
            use std::fs;
            let notes_path = get_wallet_data_dir().join("notes.json");
            let balance_before = if notes_path.exists() {
                match fs::read_to_string(notes_path) {
                    Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
                        Ok(parsed) => parsed
                            .as_array()
                            .unwrap_or(&vec![])
                            .iter()
                            .filter_map(|n| n.get("value").and_then(|v| v.as_u64()))
                            .sum::<u64>(),
                        Err(_) => 0,
                    },
                    Err(_) => 0,
                }
            } else {
                0
            };

            println!("Processing...");

            let pilot = nozy::PilotSendOptions::for_send();
            match build_and_broadcast_transaction(
                &zebra_client,
                &spendable_notes,
                &actual_recipient,
                amount_zatoshis,
                Some(fee_zatoshis),
                memo_bytes_opt.as_deref(),
                enable_broadcast,
                &config.zebra_url,
                pilot,
            )
            .await
            {
                Ok(txid) => {
                    let amount_with_fee = amount_zatoshis + fee_zatoshis;
                    let balance_after = balance_before.saturating_sub(amount_with_fee);

                    use nozy::privacy_ui::show_privacy_badge;
                    show_privacy_badge();

                    println!("\n✅ Transaction sent successfully!");
                    println!("🆔 Transaction ID: {}", txid);
                    println!("{}", "=".repeat(60));
                    println!("  Amount sent: {:.8} ZEC", amount);
                    println!(
                        "  Fee paid:    {:.8} ZEC",
                        fee_zatoshis as f64 / 100_000_000.0
                    );
                    println!(
                        "  Total spent: {:.8} ZEC",
                        amount_with_fee as f64 / 100_000_000.0
                    );
                    println!(
                        "  Remaining:   {:.8} ZEC",
                        balance_after as f64 / 100_000_000.0
                    );
                    println!();
                    println!("🛡️  This transaction is private and untraceable.");
                    println!("🔒 Privacy is enforced by NozyWallet.");
                    println!("{}", "=".repeat(60));
                    println!("\n💡 Run 'nozy history' to view transaction details");
                }
                Err(e) => {
                    println!("❌ Failed to send: {}", e);
                    handle_insufficient_funds_error(&e);
                }
            }
        }

        Commands::Balance { json } => {
            let as_json = json || cli.json;
            if as_json {
                match nozy::balance_to_json() {
                    Ok(b) => {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&b).map_err(|e| {
                                NozyError::InvalidOperation(format!("json encode: {e}"))
                            })?
                        );
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                        std::process::exit(4);
                    }
                }
            } else {
                use nozy::paths::get_wallet_data_dir;
                use nozy::wallet_balance_snapshot;

                let notes_path = get_wallet_data_dir().join("notes.json");
                let snapshot = match wallet_balance_snapshot() {
                    Ok(snapshot) => snapshot,
                    Err(e) => {
                        eprintln!("❌ Failed to read wallet balance: {}", e);
                        return Ok(());
                    }
                };

                println!("💰 Balance Information");
                println!("{}", "=".repeat(50));
                println!(
                    "   Confirmed: {:.8} ZEC ({} unspent notes)",
                    snapshot.confirmed_zatoshis as f64 / 100_000_000.0,
                    snapshot.unspent_note_count
                );

                if snapshot.pending_zatoshis > 0 {
                    println!(
                        "   Pending:   -{:.8} ZEC",
                        snapshot.pending_zatoshis as f64 / 100_000_000.0
                    );
                    println!(
                        "   Available: {:.8} ZEC",
                        snapshot.available_zatoshis as f64 / 100_000_000.0
                    );
                    println!(
                        "\n   💡 Pending transactions reduce available balance until confirmed"
                    );
                } else {
                    println!(
                        "   Available: {:.8} ZEC",
                        snapshot.available_zatoshis as f64 / 100_000_000.0
                    );
                }

                if !notes_path.exists() {
                    println!("\n   ⚠️  Run 'sync' to update your balance.");
                }
            }
        }

        Commands::Profile { command } => match command {
            ProfileCommand::List => {
                print_profile_list()?;
            }
            ProfileCommand::Switch { profile_id } => {
                nozy::set_active_wallet_profile(&profile_id)?;
                println!("✅ Active wallet profile set to: {profile_id}");
                if !nozy::profile_has_wallet(&profile_id) {
                    println!("⚠️  This profile does not have a wallet yet.");
                }
            }
        },

        Commands::TestnetWallet { command } => match command {
            TestnetWalletCommand::New { name, rpc_url } => {
                let profile = nozy::create_new_profile(Some(&name))?;
                configure_ironwood_testnet(&mut config, &rpc_url)?;

                println!("🔐 Creating dedicated Ironwood testnet wallet...");
                println!("   Profile: {} ({})", profile.name, profile.id);
                println!("   RPC:     {}", rpc_url);

                let mut wallet = HDWallet::new()?;
                save_wallet_to_active_profile(&mut wallet).await?;

                println!("🎉 Testnet wallet created successfully!");
                println!("📝 Mnemonic: {}", wallet.get_mnemonic());
                println!();
                println!("⚠️  This is a testnet wallet. Keep it separate from mainnet funds.");

                match wallet.generate_orchard_address(0, 0, NetworkType::Test) {
                    Ok(address) => {
                        println!("📍 Testnet Orchard address:");
                        println!("{}", address);
                        println!("\nNext:");
                        println!("  1. Fund this address with tZEC");
                        println!("  2. nozy testnet-wallet status");
                        println!("  3. nozy --testnet sync --to-tip");
                        println!("  4. nozy --testnet ironwood plan");
                    }
                    Err(e) => println!("❌ Failed to generate testnet address: {}", e),
                }
            }
            TestnetWalletCommand::Restore { name, rpc_url } => {
                use dialoguer::Input;

                let profile = nozy::create_new_profile(Some(&name))?;
                configure_ironwood_testnet(&mut config, &rpc_url)?;

                println!("🔐 Restoring dedicated Ironwood testnet wallet...");
                println!("   Profile: {} ({})", profile.name, profile.id);
                println!("   RPC:     {}", rpc_url);

                let mnemonic: String = Input::new()
                    .with_prompt("Enter your testnet wallet mnemonic")
                    .with_initial_text("")
                    .interact_text()
                    .map_err(|e| {
                        NozyError::InvalidOperation(format!("Mnemonic input error: {}", e))
                    })?;

                let mut wallet = HDWallet::from_mnemonic(&mnemonic)?;
                save_wallet_to_active_profile(&mut wallet).await?;

                println!("✅ Testnet wallet restored and saved.");
                match wallet.generate_orchard_address(0, 0, NetworkType::Test) {
                    Ok(address) => {
                        println!("📍 Testnet Orchard address:");
                        println!("{}", address);
                        println!("\nNext:");
                        println!("  1. nozy --testnet sync --to-tip");
                        println!("  2. nozy --testnet ironwood plan");
                        println!("  3. nozy --testnet ironwood migrate");
                    }
                    Err(e) => println!("❌ Failed to generate testnet address: {}", e),
                }
            }
            TestnetWalletCommand::Use {
                profile_id,
                rpc_url,
            } => {
                nozy::set_active_wallet_profile(&profile_id)?;
                configure_ironwood_testnet(&mut config, &rpc_url)?;

                println!("✅ Active profile configured as Ironwood testnet wallet.");
                println!("   Profile: {}", profile_id);
                println!("   Network: testnet");
                println!("   RPC:     {}", rpc_url);
                if !nozy::profile_has_wallet(&profile_id) {
                    println!("⚠️  This profile does not have a wallet yet.");
                }
                println!("\nNext:");
                println!("  nozy --testnet receive");
                println!("  nozy --testnet sync --to-tip");
                println!("  nozy --testnet ironwood plan");
            }
            TestnetWalletCommand::Status => {
                let active_id = nozy::active_profile_id();
                let profiles = nozy::list_wallet_profiles()?;
                let active_profile = active_id
                    .as_deref()
                    .and_then(|id| profiles.iter().find(|p| p.id == id));

                println!("Ironwood testnet wallet status");
                println!("{}", "=".repeat(50));
                if let Some(profile) = active_profile {
                    println!("  Active profile: {} ({})", profile.name, profile.id);
                    println!(
                        "  Wallet file:    {}",
                        if nozy::profile_has_wallet(&profile.id) {
                            "present"
                        } else {
                            "missing"
                        }
                    );
                } else {
                    println!("  Active profile: (none)");
                }
                println!("  Network:       {}", config.network);
                println!("  Zebra RPC:     {}", config.zebra_url);
                println!(
                    "  Ready shape:   {}",
                    if config.network == "testnet" && config.zebra_url.contains(":18232") {
                        "testnet/local RPC configured"
                    } else {
                        "run `nozy testnet-wallet use --profile-id <id>`"
                    }
                );
                println!("\nUseful commands:");
                println!("  nozy profile list");
                println!("  nozy --testnet receive");
                println!("  nozy --testnet sync --to-tip");
                println!("  nozy --testnet ironwood plan");
                println!("  nozy --testnet ironwood migrate");
            }
        },

        Commands::Info => {
            let (wallet, _storage) = load_wallet().await?;
            println!("Wallet loaded successfully!");
            println!(
                "Mnemonic: {} (masked for security)",
                display_mnemonic_safe(&wallet.get_mnemonic())
            );
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
                    }
                    BackendKind::Crosslink => {
                        println!("  Backend: Crosslink (experimental)");
                        if config.crosslink_url.is_empty() {
                            println!(
                                "  Crosslink URL: {} (using zebra_url as fallback)",
                                config.zebra_url
                            );
                        } else {
                            println!("  Crosslink URL: {}", config.crosslink_url);
                        }
                    }
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
                let normalized =
                    if remote_url.starts_with("http://") || remote_url.starts_with("https://") {
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
                        eprintln!(
                            "❌ Invalid protocol: {}. Use 'grpc' or 'jsonrpc'",
                            protocol_str
                        );
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
                    }
                    BackendKind::Crosslink => {
                        println!("  Backend: Crosslink (experimental) ⚠️");
                    }
                }
                println!("  Zebra URL: {}", config.zebra_url);
                if !config.crosslink_url.is_empty() {
                    println!("  Crosslink URL: {}", config.crosslink_url);
                }
                println!("  Protocol: {:?}", config.protocol);
                let is_local = config.zebra_url.contains("127.0.0.1")
                    || config.zebra_url.contains("localhost");
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
                println!(
                    "    nozy config --set-crosslink-url <url>        # Set Crosslink node URL"
                );
                println!("    nozy config --show-backend                  # Show backend info");
                println!("  Node URL:");
                println!(
                    "    nozy config --use-local                     # Connect to local Zebra node"
                );
                println!(
                    "    nozy config --use-remote <host:port>         # Connect to remote node"
                );
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
                }
            };

            println!(
                "🔗 Testing {} node connection...",
                match config.backend {
                    nozy::BackendKind::Zebra => "Zebra-family",
                    nozy::BackendKind::Crosslink => "Crosslink",
                }
            );
            println!("📡 Connecting to: {}", test_url);
            println!();

            match client.test_connection().await {
                Ok(_) => {
                    println!();
                    println!("🎉 Connection successful!");
                    let node_kind = client.detect_chain_node_kind().await;
                    let is_local = test_url.contains("127.0.0.1") || test_url.contains("localhost");
                    if is_local {
                        println!(
                            "✅ NozyWallet is connected to your local {} node",
                            node_kind.label()
                        );
                    } else {
                        println!(
                            "✅ NozyWallet is connected to the remote {} node",
                            node_kind.label()
                        );
                    }

                    match client.probe_wallet_treestate().await {
                        Ok(()) => println!("✅ Wallet RPC probe: z_gettreestate (Orchard) OK"),
                        Err(e) => {
                            println!("⚠️  Wallet RPC probe failed: {}", e);
                            println!("   Shielded sync/send need z_gettreestate at chain tip.");
                        }
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
                            if let Some(subver) = info.get("subversion") {
                                println!("   Subversion: {:?}", subver);
                            }
                        }
                        Err(_) => {}
                    }
                }
                Err(e) => {
                    println!("❌ Connection failed!");
                    println!("   Error: {}", e);
                    println!();
                    let is_local = test_url.contains("127.0.0.1") || test_url.contains("localhost");
                    if is_local {
                        println!("💡 Troubleshooting steps for local node:");
                        println!("   1. Make sure Zebrad or Zakurad is running on this PC");
                        println!("   2. Check if RPC is enabled in the node config");
                        println!("   3. Zakura: disable cookie auth for lightwalletd, or set ZAKURA_RPC_COOKIE");
                        println!("   4. Verify it is listening on {}", test_url);
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
                println!(
                    "   Spend Parameters: {}",
                    if proving_status.spend_params {
                        "✅"
                    } else {
                        "❌"
                    }
                );
                println!(
                    "   Output Parameters: {}",
                    if proving_status.output_params {
                        "✅"
                    } else {
                        "❌"
                    }
                );
                println!(
                    "   Spend Verifying Key: {}",
                    if proving_status.spend_vk {
                        "✅"
                    } else {
                        "❌"
                    }
                );
                println!(
                    "   Output Verifying Key: {}",
                    if proving_status.output_vk {
                        "✅"
                    } else {
                        "❌"
                    }
                );
                println!(
                    "   Can Prove: {}",
                    if proving_status.can_prove {
                        "✅"
                    } else {
                        "❌"
                    }
                );

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
                    }
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
                .with_prompt(
                    "Export anonymized data for development insights? (completely anonymous)",
                )
                .interact()
                .unwrap_or(false)
            {
                match analytics.export_anonymized() {
                    Ok(anonymized_json) => {
                        std::fs::write("anonymized_analytics.json", &anonymized_json).map_err(
                            |e| {
                                NozyError::Storage(format!(
                                    "Failed to write anonymized analytics: {}",
                                    e
                                ))
                            },
                        )?;
                        println!("✅ Anonymized data exported to anonymized_analytics.json");
                        println!("💡 This data contains NO personal information");
                        println!(
                            "💡 You can share this with developers to help improve NozyWallet"
                        );
                    }
                    Err(e) => {
                        println!("❌ Failed to export anonymized data: {}", e);
                    }
                }
            }
        }

        Commands::History => {
            use nozy::load_config;
            use nozy::transaction_history::SentTransactionStorage;

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
                        println!(
                            "   Block: {} ({} confirmations)",
                            block_height, tx.confirmations
                        );
                    } else {
                        println!("   Status: Pending in mempool");
                    }

                    if let Some(broadcast_at) = tx.broadcast_at {
                        println!(
                            "   Broadcast: {}",
                            broadcast_at.format("%Y-%m-%d %H:%M:%S UTC")
                        );
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
            use nozy::load_config;
            use nozy::transaction_history::SentTransactionStorage;

            let config = load_config();
            let zebra_client = ZebraClient::from_config(&config);
            let tx_storage = SentTransactionStorage::new()?;

            if let Some(txid_str) = txid {
                println!(
                    "🔍 Checking confirmation status for transaction: {}",
                    txid_str
                );

                match tx_storage
                    .check_transaction_confirmation(&zebra_client, &txid_str)
                    .await
                {
                    Ok(updated) => {
                        if updated {
                            println!("✅ Transaction confirmed! Status updated.");

                            if let Some(tx) = tx_storage.get_transaction(&txid_str) {
                                if let Some(block_height) = tx.block_height {
                                    println!(
                                        "   Block: {} ({} confirmations)",
                                        block_height, tx.confirmations
                                    );
                                }
                            }
                        } else {
                            println!("⏳ Transaction still pending in mempool.");
                        }
                    }
                    Err(e) => {
                        println!("❌ Error checking transaction: {}", e);
                    }
                }
            } else {
                println!("🔍 Checking all pending transactions...");

                match tx_storage
                    .check_all_pending_transactions(&zebra_client)
                    .await
                {
                    Ok(updated_count) => {
                        if updated_count > 0 {
                            println!(
                                "✅ Updated {} transaction(s) to confirmed status.",
                                updated_count
                            );
                        } else {
                            println!("⏳ No transactions confirmed yet. All still pending.");
                        }

                        let conf_updated = tx_storage.update_confirmations(&zebra_client).await?;
                        if conf_updated > 0 {
                            println!(
                                "📊 Updated confirmation counts for {} transaction(s).",
                                conf_updated
                            );
                        }
                    }
                    Err(e) => {
                        println!("❌ Error checking transactions: {}", e);
                    }
                }
            }
        }

        Commands::Status {
            watch,
            interval,
            json,
        } => {
            let as_json = json || cli.json;
            if watch {
                nozy::run_status_tui(&config, interval).await?;
                return Ok(());
            }
            if as_json {
                let zebra_client = ZebraClient::from_config(&config);
                let sync = nozy::gather_sync_status(&zebra_client, &config).await;
                let sync_json = nozy::sync_to_json(&sync);
                let balance = nozy::balance_to_json().ok();
                #[derive(serde::Serialize)]
                struct StatusJson {
                    network: String,
                    zebra_url: String,
                    sync: nozy::LiteSyncJson,
                    balance: Option<nozy::LiteBalanceJson>,
                }
                let out = StatusJson {
                    network: config.network.clone(),
                    zebra_url: config.zebra_url.clone(),
                    sync: sync_json,
                    balance,
                };
                println!(
                    "{}",
                    serde_json::to_string_pretty(&out).map_err(|e| {
                        NozyError::InvalidOperation(format!("json encode: {e}"))
                    })?
                );
                return Ok(());
            }

            use nozy::load_config;
            use nozy::nu6_1_check;
            use nozy::paths::get_wallet_data_dir;
            use nozy::transaction_history::SentTransactionStorage;

            println!("🔍 Checking system status...");
            println!();

            let config = load_config();
            let zebra_client = ZebraClient::from_config(&config);
            match zebra_client.get_block_count().await {
                Ok(height) => {
                    nu6_1_check::display_nu6_1_status(Some(height));
                }
                Err(_) => {
                    nu6_1_check::display_nu6_1_status(None);
                }
            }

            if let Err(e) = nu6_1_check::verify_nu6_1_compatibility() {
                println!("⚠️  NU 6.1 compatibility check: {}", e);
            }

            println!();

            println!("📊 NozyWallet Status Dashboard");
            println!("{}", "=".repeat(60));

            println!("\n🔐 Wallet:");
            match load_wallet().await {
                Ok((wallet, _storage)) => {
                    println!(
                        "   Mnemonic: {} (masked for security)",
                        display_mnemonic_safe(&wallet.get_mnemonic())
                    );
                }
                Err(e) => {
                    println!("   (unlock skipped: {})", e);
                    println!(
                        "   💡 Run `nozy status` in your terminal to enter your wallet password"
                    );
                }
            }

            println!("\n🔗 Connection:");
            match config.backend {
                nozy::BackendKind::Zebra => {
                    println!("   Backend: Zebra (standard)");
                    println!("   URL: {}", config.zebra_url);
                }
                nozy::BackendKind::Crosslink => {
                    println!("   Backend: Crosslink (experimental) ⚠️");
                    if config.crosslink_url.is_empty() {
                        println!("   URL: {} (using zebra_url)", config.zebra_url);
                    } else {
                        println!("   URL: {}", config.crosslink_url);
                    }
                }
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
                }
                Err(e) => {
                    println!("   ❌ Not connected: {}", e);
                    use nozy::nu6_1_check;
                    nu6_1_check::display_nu6_1_status(None);
                }
            }

            println!("\n💰 Balance:");
            match nozy::wallet_balance_snapshot() {
                Ok(snapshot) => {
                    println!(
                        "   Confirmed: {:.8} ZEC ({} unspent notes)",
                        snapshot.confirmed_zatoshis as f64 / 100_000_000.0,
                        snapshot.unspent_note_count
                    );
                    if snapshot.pending_zatoshis > 0 {
                        println!(
                            "   Available: {:.8} ZEC (pending: -{:.8} ZEC)",
                            snapshot.available_zatoshis as f64 / 100_000_000.0,
                            snapshot.pending_zatoshis as f64 / 100_000_000.0
                        );
                    }
                }
                Err(e) => {
                    println!("   ❌ Failed to read balance: {}", e);
                }
            }
            let notes_path = get_wallet_data_dir().join("notes.json");
            if !notes_path.exists() {
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

            let sync_snapshot = nozy::gather_sync_status(&zebra_client, &config).await;
            nozy::print_sync_status(&sync_snapshot);

            println!("\n{}", "=".repeat(60));
        }

        Commands::Health {
            max_scan_gap,
            require_lwd,
            require_ironwood_rpc,
            json,
        } => {
            let as_json = json || cli.json;
            let zebra_client = ZebraClient::from_config(&config);
            let report = nozy::gather_health_report(
                &zebra_client,
                &config,
                max_scan_gap,
                require_lwd,
                require_ironwood_rpc,
            )
            .await;
            if as_json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|e| {
                        NozyError::InvalidOperation(format!("json encode: {e}"))
                    })?
                );
            } else {
                nozy::print_health_human(&report);
            }
            std::process::exit(report.exit_code as i32);
        }

        Commands::Tui { interval } => {
            nozy::run_status_tui(&config, interval).await?;
        }

        Commands::Lwd { command } => {
            use nozy::paths::get_wallet_data_dir;
            use nozy::sync_status::resolve_lightwalletd_url;

            let lwd_url = resolve_lightwalletd_url();
            let db_path = get_wallet_data_dir().join("lwd_compact.sqlite");
            std::fs::create_dir_all(get_wallet_data_dir()).map_err(|e| {
                NozyError::Storage(format!("Failed to create wallet data dir: {}", e))
            })?;

            let mut client = zeaking::lwd::connect_lightwalletd(&lwd_url)
                .await
                .map_err(|e| NozyError::InvalidOperation(format!("lightwalletd: {}", e)))?;
            let tip = zeaking::lwd::chain_tip_height(&mut client)
                .await
                .map_err(|e| NozyError::InvalidOperation(format!("chain tip: {}", e)))?;

            let store = zeaking::lwd::LwdCompactStore::open(&db_path)
                .map_err(|e| NozyError::Storage(format!("open {}: {}", db_path.display(), e)))?;

            match command {
                LwdCommand::Prune => {
                    let before = store
                        .max_compact_height()
                        .map_err(|e| NozyError::Storage(format!("max compact height: {}", e)))?;
                    let pruned = zeaking::lwd::prune_stale_compact_cache(&store, tip)
                        .map_err(|e| NozyError::Storage(format!("prune: {}", e)))?;
                    let after = store
                        .max_compact_height()
                        .map_err(|e| NozyError::Storage(format!("max compact height: {}", e)))?;
                    println!("🧹 LWD compact cache prune");
                    println!("   URL: {}", lwd_url);
                    println!("   DB:  {}", db_path.display());
                    println!("   LWD tip: {}", tip);
                    if let Some(h) = before {
                        println!("   Max height before: {}", h);
                    }
                    if pruned > 0 {
                        println!("   Removed {} stale block(s) above tip", pruned);
                    } else {
                        println!("   No stale blocks above tip");
                    }
                    if let Some(h) = after {
                        println!("   Max height after:  {}", h);
                    } else {
                        println!("   Max height after:  (empty)");
                    }
                }
                LwdCommand::SyncToTip { start_floor } => {
                    let stats = zeaking::lwd::sync_compact_to_tip_with_options(
                        &mut client,
                        &store,
                        zeaking::lwd::SyncCompactToTipOptions {
                            start_floor,
                            ..Default::default()
                        },
                    )
                    .await
                    .map_err(|e| NozyError::InvalidOperation(format!("compact-to-tip: {}", e)))?;
                    println!("✅ LWD compact sync to tip");
                    println!("   URL: {}", lwd_url);
                    println!("   DB:  {}", db_path.display());
                    println!("   Chain tip: {}", stats.chain_tip);
                    println!(
                        "   Range: {} - {} (requested start {})",
                        stats.range_start_effective, stats.range_end, stats.range_start_requested
                    );
                    println!("   Blocks written: {}", stats.blocks_written);
                    if stats.already_at_tip {
                        println!("   Already at tip");
                    }
                }
            }
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
                            println!(
                                "   Created: {}",
                                entry.created_at.format("%Y-%m-%d %H:%M:%S UTC")
                            );
                            if let Some(last_used) = entry.last_used {
                                println!(
                                    "   Last used: {} ({} times)",
                                    last_used.format("%Y-%m-%d %H:%M:%S UTC"),
                                    entry.usage_count
                                );
                            } else {
                                println!("   Never used");
                            }
                        }
                    }
                }

                AddressBookCommand::Add {
                    name,
                    address,
                    notes,
                } => match address_book.add_address(name.clone(), address.clone(), notes) {
                    Ok(()) => {
                        println!("✅ Added '{}' to address book", name);
                        println!("   Address: {}", address);
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to add address: {}", e);
                    }
                },

                AddressBookCommand::Remove { name } => match address_book.remove_address(&name) {
                    Ok(true) => {
                        println!("✅ Removed '{}' from address book", name);
                    }
                    Ok(false) => {
                        eprintln!("❌ Address '{}' not found in address book", name);
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to remove address: {}", e);
                    }
                },

                AddressBookCommand::Get { name } => match address_book.get_address(&name) {
                    Some(entry) => {
                        println!("📌 {}", entry.name);
                        println!("   Address: {}", entry.address);
                        if let Some(notes) = &entry.notes {
                            println!("   Notes: {}", notes);
                        }
                        println!(
                            "   Created: {}",
                            entry.created_at.format("%Y-%m-%d %H:%M:%S UTC")
                        );
                        if let Some(last_used) = entry.last_used {
                            println!(
                                "   Last used: {} ({} times)",
                                last_used.format("%Y-%m-%d %H:%M:%S UTC"),
                                entry.usage_count
                            );
                        } else {
                            println!("   Never used");
                        }
                    }
                    None => {
                        eprintln!("❌ Address '{}' not found in address book", name);
                    }
                },

                AddressBookCommand::Search { query } => {
                    let results = address_book.search_addresses(&query);

                    if results.is_empty() {
                        println!("🔍 No addresses found matching '{}'", query);
                    } else {
                        println!(
                            "🔍 Search results for '{}' ({} found)",
                            query,
                            results.len()
                        );
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
            let zeaking = Zeaking::new(get_zeaking_index_dir(), block_source, block_parser)
                .await
                .map_err(|e| NozyError::Storage(format!("Zeaking error: {}", e)))?;

            match command {
                ZeakingCommand::Sync => {
                    println!("🔄 Syncing Zeaking index to chain tip...");
                    match zeaking.sync_to_tip().await {
                        Ok(stats) => {
                            println!("✅ Index sync complete!");
                            println!("   Blocks indexed: {}", stats.blocks_indexed);
                            println!("   Transactions indexed: {}", stats.transactions_indexed);
                            println!(
                                "   Height range: {} - {}",
                                stats.start_height, stats.end_height
                            );
                            if stats.errors > 0 {
                                println!("   ⚠️  Errors: {}", stats.errors);
                            }
                            zeaking.save_index().map_err(|e| {
                                NozyError::Storage(format!("Failed to save index: {}", e))
                            })?;
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to sync index: {}", e);
                        }
                    }
                }
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
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to index blocks: {}", e);
                        }
                    }
                }
                ZeakingCommand::Stats => {
                    let stats = zeaking.get_stats();
                    println!("📊 Zeaking Index Statistics");
                    println!("{}", "=".repeat(60));
                    println!("   Blocks indexed: {}", stats.blocks_indexed);
                    println!("   Transactions indexed: {}", stats.transactions_indexed);
                    println!(
                        "   Height range: {} - {}",
                        stats.start_height, stats.end_height
                    );
                    println!(
                        "   Last indexed height: {}",
                        zeaking.get_last_indexed_height()
                    );
                }
                ZeakingCommand::Block { height } => match zeaking.get_block(height) {
                    Some(block) => {
                        println!("📦 Block #{}", height);
                        println!("{}", "=".repeat(60));
                        println!("   Hash: {}", block.hash);
                        println!(
                            "   Time: {}",
                            chrono::DateTime::<chrono::Utc>::from_timestamp(block.time, 0)
                                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                                .unwrap_or_else(|| "Unknown".to_string())
                        );
                        println!("   Size: {} bytes", block.size);
                        println!("   Transactions: {}", block.tx_count);
                        println!("   Orchard actions: {}", block.orchard_action_count);
                        println!(
                            "   Indexed at: {}",
                            block.indexed_at.format("%Y-%m-%d %H:%M:%S UTC")
                        );
                    }
                    None => {
                        println!("❌ Block {} not found in index", height);
                        println!(
                            "   Use 'nozy zeaking index --start {} --end {}' to index it",
                            height, height
                        );
                    }
                },
                ZeakingCommand::Transaction { txid } => match zeaking.get_transaction(&txid) {
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
                        println!(
                            "   Indexed at: {}",
                            tx.indexed_at.format("%Y-%m-%d %H:%M:%S UTC")
                        );
                    }
                    None => {
                        println!("❌ Transaction {} not found in index", txid);
                    }
                },
                ZeakingCommand::FindNullifier { nullifier } => {
                    match zeaking.find_transaction_by_nullifier(&nullifier) {
                        Some(tx) => {
                            println!("🔍 Found transaction for nullifier {}", nullifier);
                            println!("   Transaction ID: {}", tx.txid);
                            println!("   Block height: {}", tx.block_height);
                        }
                        None => {
                            println!("❌ No transaction found for nullifier {}", nullifier);
                        }
                    }
                }
            }
        }

        Commands::PrivacyNetwork { command } => {
            use nozy::privacy_network::proxy::ProxyConfig;
            use nozy::privacy_network::{I2PProxy, TorProxy};

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
                }
                PrivacyNetworkCommand::TestTor => {
                    println!("🔍 Testing Tor connection...");
                    let tor = TorProxy::new(None);
                    match tor.test_tor_connection().await {
                        Ok(true) => {
                            println!("✅ Tor is working!");
                            if let Ok(ip) = tor.get_tor_ip().await {
                                println!("   Your IP through Tor: {}", ip);
                            }
                        }
                        Ok(false) => {
                            println!("❌ Tor connection failed");
                            println!("   Please check if Tor is running on 127.0.0.1:9050");
                        }
                        Err(e) => {
                            println!("❌ Error testing Tor: {}", e);
                        }
                    }
                }
                PrivacyNetworkCommand::TestI2p => {
                    println!("🔍 Testing I2P connection...");
                    let i2p = I2PProxy::new(None);
                    match i2p.test_i2p_connection().await {
                        Ok(true) => {
                            println!("✅ I2P is working!");
                        }
                        Ok(false) => {
                            println!("❌ I2P connection failed");
                            println!("   Please check if I2P router is running on 127.0.0.1:4444");
                        }
                        Err(e) => {
                            println!("❌ Error testing I2P: {}", e);
                        }
                    }
                }
                PrivacyNetworkCommand::GetIp => {
                    let proxy = ProxyConfig::auto_detect().await;
                    if proxy.enabled {
                        println!("🔍 Getting IP through privacy network...");
                        if let Ok(client) = proxy.create_client() {
                            match client
                                .get("https://api.ipify.org?format=json")
                                .timeout(std::time::Duration::from_secs(15))
                                .send()
                                .await
                            {
                                Ok(response) => {
                                    if let Ok(json) = response.json::<serde_json::Value>().await {
                                        if let Some(ip) = json.get("ip").and_then(|v| v.as_str()) {
                                            println!("✅ Your IP: {}", ip);
                                            println!("   Network: {:?}", proxy.network);
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!("❌ Failed to get IP: {}", e);
                                }
                            }
                        }
                    } else {
                        println!("❌ No privacy network available");
                    }
                }
            }
        }

        Commands::Swap { command } => {
            use nozy::cli_helpers::load_wallet;
            use nozy::monero::MoneroWallet;
            use nozy::privacy_network::proxy::ProxyConfig;
            use nozy::swap::{SwapDirection, SwapEngine, SwapService};

            let proxy = ProxyConfig::auto_detect().await;
            if !proxy.enabled {
                println!("❌ Privacy network (Tor/I2P) required for swaps!");
                println!("   Please start Tor or I2P before continuing.");
                return Ok(());
            }

            println!("🛡️  Privacy network active: {:?}", proxy.network);

            let (zcash_wallet, _) = load_wallet().await?;

            let swap_service = SwapService::new(None, None, Some(proxy.clone())).map_err(|e| {
                NozyError::InvalidOperation(format!("Failed to create swap service: {}", e))
            })?;

            match command {
                SwapCommand::XmrToZec { amount } => {
                    let monero_wallet = MoneroWallet::new(None, None, None, Some(proxy)).ok();

                    let mut swap_engine =
                        SwapEngine::new(swap_service, monero_wallet, Some(zcash_wallet))?;

                    swap_engine
                        .execute_swap(SwapDirection::XmrToZec, amount)
                        .await?;
                }
                SwapCommand::ZecToXmr { amount } => {
                    let monero_wallet = MoneroWallet::new(None, None, None, Some(proxy)).ok();

                    let mut swap_engine =
                        SwapEngine::new(swap_service, monero_wallet, Some(zcash_wallet))?;

                    swap_engine
                        .execute_swap(SwapDirection::ZecToXmr, amount)
                        .await?;
                }
                SwapCommand::Status { swap_id } => {
                    let swap_engine = SwapEngine::new(swap_service, None, Some(zcash_wallet))?;

                    match swap_engine.check_swap_status(&swap_id).await {
                        Ok(status) => {
                            println!("📊 Swap Status: {}", swap_id);
                            println!("   Status: {:?}", status.status);
                            println!("   Progress: {:.1}%", status.progress * 100.0);
                            if let Some(txid) = status.txid {
                                println!("   Transaction ID: {}", txid);
                            }
                        }
                        Err(e) => {
                            println!("❌ Failed to get swap status: {}", e);
                        }
                    }
                }
                SwapCommand::List => {
                    use nozy::bridge::SwapStorage;
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
                            println!(
                                "   Created: {}",
                                chrono::DateTime::<chrono::Utc>::from_timestamp(
                                    swap.created_at as i64,
                                    0
                                )
                                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                                .unwrap_or_else(|| "Unknown".to_string())
                            );
                            if let Some(txid) = &swap.txid {
                                println!("   TXID: {}", txid);
                            }
                        }
                    }
                }
                SwapCommand::Churn { times, ring_size } => {
                    println!("🔄 Churning Monero outputs...");
                    println!("   Times: {}", times);
                    println!("   Ring Size: {}", ring_size);
                    println!();

                    let monero_wallet = MoneroWallet::new(None, None, None, Some(proxy))?;

                    use nozy::bridge::ChurnManager;
                    let churn_manager = ChurnManager::new(monero_wallet);
                    churn_manager
                        .churn_outputs(times, ring_size, Some(300))
                        .await?;
                }
            }
        }

        #[cfg(feature = "secret-network")]
        Commands::Shade { command } => {
            use nozy::load_config;
            use nozy::privacy_network::proxy::ProxyConfig;
            use nozy::secret::snip20::shade_tokens;
            use nozy::secret::SecretWallet;
            use nozy::secret_keys::SecretKeyDerivation;

            let config = load_config();
            let proxy = ProxyConfig::auto_detect().await;

            let hd_wallet = load_wallet().await.ok();

            match command {
                ShadeCommand::Balance { address, token } => {
                    let wallet_address = if let Some(addr) = address {
                        addr
                    } else if let Some((ref wallet, _)) = hd_wallet.as_ref() {
                        match wallet.generate_secret_address(0, 0) {
                            Ok(addr) => {
                                println!("📍 Using Secret Network address from wallet: {}", addr);
                                addr
                            }
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
                        }
                        Err(e) => {
                            println!("   SCRT: Error - {}", e);
                        }
                    }

                    if let Some(token_contract) = token {
                        match wallet.get_token_balance(&token_contract).await {
                            Ok((balance, info)) => {
                                println!("   {}: {:.6}", info.symbol, balance);
                            }
                            Err(e) => {
                                println!("   Token: Error - {}", e);
                            }
                        }
                    } else {
                        println!("\n   Common Shade Tokens:");
                        for (_name, contract) in
                            [("SHD", shade_tokens::SHD), ("SILK", shade_tokens::SILK)]
                        {
                            match wallet.get_token_balance(contract).await {
                                Ok((balance, info)) => {
                                    if balance > 0.0 {
                                        println!("      {}: {:.6}", info.symbol, balance);
                                    }
                                }
                                Err(_) => {}
                            }
                        }
                    }
                }

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
                        }
                        Err(e) => {
                            println!("   Error: {}", e);
                        }
                    }
                }

                ShadeCommand::Send {
                    recipient,
                    amount,
                    token,
                    memo,
                } => {
                    println!("💸 Sending Shade Token");
                    println!("{}", "=".repeat(60));
                    println!("   Token: {}", token);
                    println!("   Recipient: {}", recipient);
                    println!("   Amount: {}", amount);

                    let (hd_wallet, _storage) = load_wallet().await?;

                    // Derive Secret Network address and key pair
                    let secret_address = hd_wallet.generate_secret_address(0, 0)?;
                    let key_derivation = SecretKeyDerivation::new(hd_wallet);
                    let key_pair = key_derivation.derive_key_pair(
                        &nozy::secret_keys::SecretDerivationPath {
                            account: 0,
                            change: 0,
                            index: 0,
                        },
                    )?;

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
                        }
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
                        println!(
                            "\nType 'SEND' (all caps) to confirm, or anything else to cancel:"
                        );
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
                        return Err(NozyError::InvalidOperation(format!(
                            "Failed to read input: {}",
                            e
                        )));
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
                        }
                        Err(e) => {
                            println!("\n❌ Failed to send transaction: {}", e);
                            println!("\n💡 Common issues:");
                            println!("   - Insufficient balance");
                            println!("   - Network connectivity");
                            println!("   - Invalid contract address");
                            println!("   - Invalid recipient address");
                        }
                    }
                }

                ShadeCommand::Receive { account, index } => {
                    let (wallet, _storage) = load_wallet().await?;

                    match wallet.generate_secret_address(account, index) {
                        Ok(address) => {
                            println!("📍 Your Secret Network address:");
                            println!("{}", address);
                            println!("\n💡 Share this address to receive SCRT and Shade tokens.");
                            println!("   Derivation path: m/44'/529'/{}/0/{}", account, index);
                        }
                        Err(e) => {
                            println!("❌ Failed to generate address: {}", e);
                        }
                    }
                }

                ShadeCommand::History => {
                    use nozy::secret::SecretTransactionStorage;

                    let tx_storage = SecretTransactionStorage::new()?;
                    let all_txs = tx_storage.get_all_transactions();

                    if all_txs.is_empty() {
                        println!("📝 No Secret Network transaction history found.");
                        println!("   Transactions will appear here after you send tokens.");
                    } else {
                        println!(
                            "📜 Secret Network Transaction History ({} transactions)",
                            all_txs.len()
                        );
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
                                println!(
                                    "   Block: {} ({} confirmations)",
                                    block_height, tx.confirmations
                                );
                            } else {
                                println!("   Status: Pending in mempool");
                            }

                            if let Some(broadcast_at) = tx.broadcast_at {
                                println!(
                                    "   Broadcast: {}",
                                    broadcast_at.format("%Y-%m-%d %H:%M:%S UTC")
                                );
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
                }

                ShadeCommand::Status { txid } => {
                    use nozy::load_config;
                    use nozy::secret::{SecretRpcClient, SecretTransactionStorage};

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
                                            println!(
                                                "   Block: {} ({} confirmations)",
                                                block_height, tx.confirmations
                                            );
                                        }
                                        if let Some(error) = &tx.error {
                                            println!("   Error: {}", error);
                                        }
                                    }
                                } else {
                                    println!("⏳ Transaction still pending in mempool.");
                                }
                            }
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
                            println!(
                                "✅ Updated {} transaction(s) to confirmed status.",
                                updated_count
                            );
                        } else {
                            println!("⏳ No transactions confirmed yet. All still pending.");
                        }

                        let conf_updated = tx_storage.update_confirmations(&rpc).await?;
                        if conf_updated > 0 {
                            println!(
                                "📊 Updated confirmation counts for {} transaction(s).",
                                conf_updated
                            );
                        }
                    }
                }

                ShadeCommand::ListTokens => {
                    println!("🎨 Shade Protocol Tokens");
                    println!("{}", "=".repeat(60));
                    println!("\n   Mainnet Token Contracts:");
                    println!("   SHD (Shade): {}", shade_tokens::SHD);
                    println!("   SILK: {}", shade_tokens::SILK);
                    println!("\n💡 Use 'nozy shade balance' to check balances");
                    println!("💡 Use 'nozy shade receive' to generate your address");
                }
            }
        }

        Commands::Monero { command } => {
            use nozy::monero::MoneroWallet;
            use nozy::privacy_network::proxy::ProxyConfig;

            let proxy = ProxyConfig::auto_detect().await;

            match command {
                MoneroCommand::Balance {
                    rpc_url,
                    username,
                    password,
                } => {
                    let wallet = MoneroWallet::new(rpc_url, username, password, Some(proxy))?;

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
                }

                MoneroCommand::Send {
                    recipient,
                    amount,
                    rpc_url,
                    username,
                    password,
                } => {
                    let wallet = MoneroWallet::new(rpc_url, username, password, Some(proxy))?;

                    if !wallet.is_connected().await {
                        return Err(NozyError::NetworkError(
                            "Failed to connect to Monero wallet RPC. Make sure monero-wallet-rpc is running.".to_string()
                        ));
                    }

                    wallet.validate_address(&recipient)?;
                    wallet.validate_amount(amount)?;

                    let balance = wallet.get_balance_xmr().await?;
                    if amount > balance {
                        let error = NozyError::InsufficientFunds(format!(
                            "Insufficient balance: {:.12} XMR available, {:.12} XMR requested",
                            balance, amount
                        ));
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
                            println!(
                                "💡 Use 'nozy monero status --txid {}' to check status",
                                txid
                            );
                        }
                        Err(e) => {
                            println!();
                            println!("❌ Transaction failed: {}", e);
                        }
                    }
                }

                MoneroCommand::Receive {
                    account_index,
                    rpc_url,
                    username,
                    password,
                } => {
                    let wallet = MoneroWallet::new(rpc_url, username, password, Some(proxy))?;

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
                }

                MoneroCommand::History => {
                    let wallet = MoneroWallet::new(None, None, None, Some(proxy))?;

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
                            println!(
                                "   Created: {}",
                                tx.created_at.format("%Y-%m-%d %H:%M:%S UTC")
                            );
                            if let Some(broadcast) = tx.broadcast_at {
                                println!(
                                    "   Broadcast: {}",
                                    broadcast.format("%Y-%m-%d %H:%M:%S UTC")
                                );
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
                }

                MoneroCommand::Status { txid } => {
                    let wallet = MoneroWallet::new(None, None, None, Some(proxy))?;

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
                            println!(
                                "   Created: {}",
                                tx.created_at.format("%Y-%m-%d %H:%M:%S UTC")
                            );
                            if let Some(broadcast) = tx.broadcast_at {
                                println!(
                                    "   Broadcast: {}",
                                    broadcast.format("%Y-%m-%d %H:%M:%S UTC")
                                );
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
                        let pending: Vec<_> = transactions
                            .iter()
                            .filter(|tx| {
                                matches!(tx.status, nozy::MoneroTransactionStatus::Pending)
                            })
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
                                println!(
                                    "✅ Updated {} transaction(s) to confirmed status.",
                                    updated_count
                                );
                            } else {
                                println!("⏳ No transactions confirmed yet. All still pending.");
                            }

                            let conf_updated = wallet.update_confirmations().await?;
                            if conf_updated > 0 {
                                println!(
                                    "📊 Updated confirmation counts for {} transaction(s).",
                                    conf_updated
                                );
                            }
                        }
                    }
                }
            }
        }

        Commands::Ironwood { command } => {
            use nozy::ironwood::{
                display_ironwood_status, execute_orchard_migration,
                execute_orchard_migration_broadcast, execute_orchard_note_split,
                fetch_pool_balances, is_ironwood_active, nu6_3_activation_height,
                plan_orchard_migration_at, plan_orchard_note_split_outputs,
                refresh_orchard_migration_schedule_at, save_orchard_migration_plan_at,
                IronwoodWalletStatus, MigrationReadinessState,
            };
            use nozy::load_wallet_notes;
            use nozy::max_serialized_witness_lag_blocks;
            use nozy::shielded_pool::ShieldedPool;
            use nozy::ZebraClient;

            let zebra_client = ZebraClient::from_config(&config);
            let chain_tip = zebra_client.get_block_count().await.unwrap_or(0);
            let testnet = config.network.eq_ignore_ascii_case("testnet");
            let activation_height = nu6_3_activation_height(testnet);
            let ironwood_active = is_ironwood_active(chain_tip, testnet);
            let migration_schedule_tip = if ironwood_active {
                chain_tip
            } else {
                activation_height
                    .map(|height| height.saturating_sub(1))
                    .unwrap_or(chain_tip)
            };

            match command {
                IronwoodCommand::Status => {
                    let pools = fetch_pool_balances(&zebra_client).await.unwrap_or_default();
                    let notes = load_wallet_notes().unwrap_or_default();
                    let orchard_zat: u64 = notes
                        .iter()
                        .filter(|n| !n.spent && n.pool == ShieldedPool::Orchard)
                        .map(|n| n.value)
                        .sum();
                    let ironwood_zat: u64 = notes
                        .iter()
                        .filter(|n| !n.spent && n.pool == ShieldedPool::Ironwood)
                        .map(|n| n.value)
                        .sum();

                    let mut blockers = Vec::new();
                    if activation_height.is_none() {
                        blockers.push(
                            "NU6.3 activation height is not configured for this network yet"
                                .to_string(),
                        );
                    } else if !ironwood_active {
                        let remaining = activation_height
                            .map(|height| height.saturating_sub(chain_tip))
                            .unwrap_or(0);
                        blockers.push(format!(
                            "Planning only: Ironwood activates at height {} in ~{remaining} blocks \
                             (target {}). After activation Orchard sends freeze except migration.",
                            activation_height.unwrap_or(0),
                            if testnet {
                                nozy::NU6_3_TESTNET_ACTIVATION_TARGET
                            } else {
                                nozy::NU6_3_MAINNET_ACTIVATION_TARGET
                            }
                        ));
                    }
                    if ironwood_active && orchard_zat > 0 {
                        blockers.push(
                            "Orchard notes remain — run `nozy ironwood migrate` for turnstile migration"
                                .to_string(),
                        );
                    }
                    if ironwood_active && ironwood_zat == 0 && orchard_zat == 0 {
                        blockers.push(
                            "No unspent shielded notes — receive ZEC or run `nozy sync --to-tip` to index Ironwood outputs"
                                .to_string(),
                        );
                    }

                    let wallet_ready = ironwood_active
                        && ironwood_zat > 0
                        && orchard_zat == 0
                        && blockers.is_empty();

                    let status = IronwoodWalletStatus {
                        chain_tip,
                        ironwood_active,
                        nu6_3_activation_height: activation_height,
                        pools,
                        orchard_notes_unspent: orchard_zat,
                        ironwood_notes_unspent: ironwood_zat,
                        migration_recommended: ironwood_active && orchard_zat > 0,
                        wallet_ready,
                        blockers,
                    };
                    display_ironwood_status(&status);
                }
                IronwoodCommand::Plan { save } => {
                    let plan = plan_orchard_migration_at(ironwood_active, migration_schedule_tip)?;
                    println!("🌲 Ironwood migration plan");
                    println!(
                        "   Orchard notes to migrate: {}",
                        plan.orchard_notes_to_migrate
                    );
                    println!("   Total: {} zatoshis", plan.total_zatoshis);
                    if !ironwood_active {
                        println!(
                            "   Ironwood active: no (tip {}, activation {})",
                            chain_tip,
                            activation_height
                                .map(|h| h.to_string())
                                .unwrap_or_else(|| "unknown".to_string())
                        );
                        println!("   Schedule starts at the first ZIP 318 bucket after activation");
                    }
                    println!("   ZIP 318 mode: scheduled background migration");
                    println!(
                        "   Anchor bucket interval: {} blocks",
                        plan.zip318.anchor_bucket_interval_blocks
                    );
                    println!(
                        "   Note split required: {}",
                        if plan.zip318.note_split_required {
                            "yes"
                        } else {
                            "no"
                        }
                    );
                    println!(
                        "   Canonical transfer count: {}",
                        plan.zip318.total_transfer_count
                    );
                    println!(
                        "   Next anchor bucket: {}",
                        plan.zip318
                            .next_anchor_bucket_height
                            .map(|h| h.to_string())
                            .unwrap_or_else(|| "none".to_string())
                    );
                    println!("   Same-denomination cap per bucket: {}", plan.zip318.k_max);
                    for d in &plan.zip318.denomination_transfers {
                        println!("     • {} × {} zat", d.count, d.value_zat);
                    }
                    if !plan.zip318.scheduled_transfers.is_empty() {
                        println!();
                        println!("   Draft schedule preview:");
                        for transfer in plan.zip318.scheduled_transfers.iter().take(12) {
                            println!(
                                "     • #{} {} zat at bucket {} (slot {})",
                                transfer.sequence,
                                transfer.value_zat,
                                transfer.anchor_bucket_height,
                                transfer.bucket_slot
                            );
                        }
                        if plan.zip318.scheduled_transfers.len() > 12 {
                            println!(
                                "     • ... {} more scheduled transfers",
                                plan.zip318.scheduled_transfers.len() - 12
                            );
                        }
                    }
                    println!();
                    println!("   Source notes:");
                    for t in &plan.transfers {
                        println!(
                            "     • {} zat (note {}, height {})",
                            t.value_zat, t.nullifier_hex, t.block_height
                        );
                    }
                    if save {
                        let (schedule, path) = save_orchard_migration_plan_at(
                            ironwood_active,
                            migration_schedule_tip,
                        )?;
                        println!();
                        println!(
                            "   ✅ Draft schedule saved: {} ({} transfers)",
                            path.display(),
                            schedule.transfers.len()
                        );
                    }
                }
                IronwoodCommand::Preflight => {
                    let notes = load_wallet_notes().unwrap_or_default();
                    let orchard_notes: Vec<_> = notes
                        .iter()
                        .filter(|n| !n.spent && n.pool == ShieldedPool::Orchard)
                        .collect();
                    let orchard_zat: u64 = orchard_notes.iter().map(|n| n.value).sum();
                    let witness_lag = max_serialized_witness_lag_blocks(&notes, chain_tip);
                    let plan = plan_orchard_migration_at(ironwood_active, migration_schedule_tip)?;
                    let refreshed = refresh_orchard_migration_schedule_at(
                        ironwood_active,
                        migration_schedule_tip,
                    )?;
                    let orchard_values: Vec<u64> = orchard_notes.iter().map(|n| n.value).collect();
                    let migration_fee = nozy::fee_policy::estimate_orchard_send_fee_zatoshis(
                        None,
                        nozy::fee_policy::NOZY_WALLET_PRIORITY_FEE,
                    );
                    let readiness = nozy::ironwood::migration::assess_orchard_migration_readiness_with_spendability(
                        ironwood_active,
                        chain_tip,
                        &plan,
                        refreshed.as_ref().map(|(schedule, _, _)| schedule),
                        Some(witness_lag),
                        Some(nozy::MAX_SEND_WITNESS_LAG_BLOCKS),
                        Some(&orchard_values),
                        Some(migration_fee),
                    );

                    println!("🌲 Ironwood migration preflight");
                    println!("   Network: {}", config.network);
                    println!("   Chain tip: {}", chain_tip);
                    println!(
                        "   Activation height: {}",
                        activation_height
                            .map(|h| h.to_string())
                            .unwrap_or_else(|| "unknown".to_string())
                    );
                    if let Some(height) = activation_height {
                        println!(
                            "   Blocks until activation: {}",
                            height.saturating_sub(chain_tip)
                        );
                    }
                    println!(
                        "   Ironwood active: {}",
                        if ironwood_active { "yes" } else { "no" }
                    );
                    println!(
                        "   Orchard migration balance: {} zat across {} notes",
                        orchard_zat,
                        orchard_notes.len()
                    );
                    println!("   Max witness lag: {} blocks", witness_lag);
                    println!("   ZIP 318 transfers: {}", plan.zip318.total_transfer_count);
                    println!(
                        "   Next anchor bucket: {}",
                        plan.zip318
                            .next_anchor_bucket_height
                            .map(|h| h.to_string())
                            .unwrap_or_else(|| "none".to_string())
                    );
                    match &refreshed {
                        Some((schedule, path, expired)) if *expired > 0 => {
                            println!(
                                "   Schedule refresh: rebuilt {} expired/missed transfers at {}",
                                expired,
                                path.display()
                            );
                            println!("   Schedule transfers: {}", schedule.transfers.len());
                        }
                        Some((schedule, path, _)) => {
                            println!(
                                "   Schedule refresh: current schedule OK at {} ({} transfers)",
                                path.display(),
                                schedule.transfers.len()
                            );
                        }
                        None => {
                            println!("   Schedule refresh: no saved schedule yet");
                        }
                    }
                    println!("   Readiness state: {}", readiness.state.label());
                    if let Some(transfer) = readiness.next_eligible_transfer.as_ref() {
                        println!(
                            "   Next eligible transfer: #{} {} zat at bucket {}",
                            transfer.sequence, transfer.value_zat, transfer.anchor_bucket_height
                        );
                    } else if let Some(transfer) = readiness.next_waiting_transfer.as_ref() {
                        println!(
                            "   Next waiting transfer: #{} {} zat at bucket {}",
                            transfer.sequence, transfer.value_zat, transfer.anchor_bucket_height
                        );
                    } else if let Some(transfer) = readiness.active_presigned_transfer.as_ref() {
                        println!(
                            "   Presigned transfer: #{} {} zat (txid {})",
                            transfer.sequence,
                            transfer.value_zat,
                            transfer.prepared_txid.as_deref().unwrap_or("unknown")
                        );
                    }
                    if let Some(validation) = readiness.validation.as_ref() {
                        println!(
                            "   Schedule validation: {}",
                            if validation.valid {
                                "ok"
                            } else {
                                "needs rebuild"
                            }
                        );
                        if validation.expired_transfer_count > 0 {
                            println!(
                                "   Expired/missed windows: {}",
                                validation.expired_transfer_count
                            );
                        }
                        if validation.stale_presigned_count > 0 {
                            println!(
                                "   Stale presigned txs: {}",
                                validation.stale_presigned_count
                            );
                        }
                    }
                    for blocker in &readiness.blockers {
                        println!("   Blocker: {blocker}");
                    }

                    // Safer migration priorities (see SAFE_MIGRATION_NETWORK_PRIVACY_FORUM_POST.md)
                    let privacy = nozy::ironwood::assess_migration_network_privacy(
                        &config.zebra_url,
                        &nozy::ironwood::MigrationNetworkPrivacyOpts::default(),
                    )
                    .await;
                    println!();
                    println!("   Priority 1 — network privacy (IP):");
                    println!(
                        "      Zebra URL local: {}",
                        if privacy.zebra_url_local { "yes" } else { "no" }
                    );
                    println!(
                        "      Tor/I2P proxy detected: {}",
                        if privacy.privacy_proxy_detected {
                            privacy.privacy_proxy_label.as_deref().unwrap_or("yes")
                        } else {
                            "no"
                        }
                    );
                    if privacy.allowed {
                        println!(
                            "      Broadcast policy: allowed ({})",
                            privacy.mode.map(|m| m.label()).unwrap_or("ok")
                        );
                    } else {
                        println!(
                            "      Broadcast policy: blocked until local node, Tor/I2P, \
                             --attest-private-network, or --force-clearnet"
                        );
                    }
                    for warning in &privacy.warnings {
                        println!("      Warning: {warning}");
                    }
                    for blocker in &privacy.blockers {
                        println!("      Blocker: {blocker}");
                    }

                    let bucket = nozy::ironwood::previous_zip318_anchor_boundary(chain_tip);
                    let local_in_bucket = refreshed
                        .as_ref()
                        .map(|(schedule, _, _)| {
                            schedule
                                .transfers
                                .iter()
                                .filter(|t| t.anchor_bucket_height == bucket)
                                .count()
                        })
                        .unwrap_or(0);
                    let cover = nozy::ironwood::assess_migration_cover_traffic(
                        chain_tip,
                        local_in_bucket,
                        nozy::ironwood::ZIP318_DEFAULT_K_MAX,
                    );
                    println!("   Priority 2 — shared cover traffic:");
                    println!(
                        "      Current ZIP 318 bucket: {} ({} local transfer(s), k_max={})",
                        cover.current_bucket_height, cover.local_transfers_in_bucket, cover.k_max
                    );
                    for note in &cover.notes {
                        println!("      Note: {note}");
                    }
                    for warning in &cover.warnings {
                        println!("      Warning: {warning}");
                    }

                    let amount_timing = nozy::ironwood::amount_timing_status();
                    println!("   Priority 3 — amount/timing algorithm:");
                    println!("      Active: {}", amount_timing.active.label());
                    println!("      Planned: {}", amount_timing.planned.label());
                    for note in &amount_timing.notes {
                        println!("      Note: {note}");
                    }

                    println!();
                    match readiness.state {
                        MigrationReadinessState::PlanningOnly => {
                            println!("   Result: planning-only until NU6.3 activation");
                        }
                        MigrationReadinessState::NoOrchardNotes => {
                            println!("   Result: no Orchard notes need migration");
                        }
                        MigrationReadinessState::SplitRequired => {
                            println!(
                                "   Result: ZIP 318 note-splitting phase required before prebuild"
                            );
                        }
                        MigrationReadinessState::ReadyToPrebuild => {
                            println!("   Result: ready to attempt locked V6 prebuild");
                        }
                        MigrationReadinessState::WaitingForWindow => {
                            println!("   Result: waiting for next ZIP 318 anchor bucket");
                        }
                        MigrationReadinessState::PresignedWaitingForBroadcast => {
                            println!("   Result: presigned turnstile transaction waiting for bucket/window");
                        }
                        MigrationReadinessState::ReadyToBroadcast => {
                            println!(
                                "   Result: ready to broadcast presigned turnstile transaction"
                            );
                        }
                        MigrationReadinessState::Blocked => {
                            println!("   Result: blocked; resolve blockers above");
                        }
                    }
                }
                IronwoodCommand::Migrate => {
                    if !ironwood_active {
                        return Err(NozyError::InvalidOperation(format!(
                            "Ironwood (NU6.3) is not active on this network yet (tip {}, activation {}). \
                             The V6 Orchard-to-Ironwood transaction builder is only valid after activation.",
                            chain_tip,
                            activation_height
                                .map(|h| h.to_string())
                                .unwrap_or_else(|| "unknown".to_string())
                        )));
                    }

                    if let Some((_schedule, path, expired)) =
                        refresh_orchard_migration_schedule_at(ironwood_active, chain_tip)?
                    {
                        if expired > 0 {
                            println!(
                                "♻️  Rebuilt Ironwood schedule after {} expired/missed transfer windows",
                                expired
                            );
                            println!("   Schedule: {}", path.display());
                        }
                    }

                    println!("🔐 Unlocking wallet for Ironwood migration prebuild...");
                    let (wallet, _storage) = load_wallet().await?;
                    println!("🔍 Loading spendable Orchard notes for turnstile prebuild...");
                    let spendable_notes = scan_notes_for_sending(wallet, &config.zebra_url).await?;
                    let result = execute_orchard_migration(
                        &config.zebra_url,
                        ironwood_active,
                        &spendable_notes,
                    )
                    .await?;

                    if let Some(prepared) = result.prepared {
                        println!("✅ Prebuilt locked V6 turnstile transaction");
                        println!("   Sequence: {}", prepared.sequence);
                        println!("   Value:    {} zat", prepared.value_zat);
                        println!("   TXID:     {}", prepared.txid);
                        println!("   Source:   {}", prepared.source_nullifier_hex);
                        println!("   Prepared at height: {}", prepared.prepared_at_height);
                        println!("   Expires at height:  {}", prepared.expires_at_height);
                        if let Some(path) = result.schedule_path {
                            println!("   Schedule: {}", path.display());
                        }
                        println!(
                            "   Next: run `nozy ironwood broadcast` when the ZIP 318 bucket window is open"
                        );
                    } else {
                        match result.readiness_state {
                            MigrationReadinessState::NoOrchardNotes => {
                                println!("✅ No Orchard notes require Ironwood migration.");
                            }
                            MigrationReadinessState::SplitRequired => {
                                println!("🧩 ZIP 318 note splitting is required before migration prebuild.");
                                if let Some(path) = result.schedule_path {
                                    println!("   Schedule: {}", path.display());
                                }
                                for blocker in &result.blockers {
                                    println!("   Blocker: {blocker}");
                                }
                            }
                            MigrationReadinessState::WaitingForWindow => {
                                println!("⏳ Waiting for the next ZIP 318 anchor bucket before prebuild.");
                                if let Some(path) = result.schedule_path {
                                    println!("   Schedule: {}", path.display());
                                }
                            }
                            MigrationReadinessState::PresignedWaitingForBroadcast => {
                                println!(
                                    "🔒 A locked V6 turnstile transaction is already presigned."
                                );
                                if let Some(path) = result.schedule_path {
                                    println!("   Schedule: {}", path.display());
                                }
                                for blocker in &result.blockers {
                                    println!("   Blocker: {blocker}");
                                }
                            }
                            MigrationReadinessState::ReadyToBroadcast => {
                                println!(
                                    "📡 Presigned turnstile transaction is ready for broadcast."
                                );
                                if let Some(path) = result.schedule_path {
                                    println!("   Schedule: {}", path.display());
                                }
                                println!("   Run: nozy ironwood broadcast");
                            }
                            MigrationReadinessState::PlanningOnly
                            | MigrationReadinessState::ReadyToPrebuild
                            | MigrationReadinessState::Blocked => {
                                println!(
                                    "⚠️  Migration did not prebuild a transaction ({})",
                                    result.readiness_state.label()
                                );
                                for blocker in &result.blockers {
                                    println!("   Blocker: {blocker}");
                                }
                            }
                        }
                    }
                }
                IronwoodCommand::Broadcast {
                    dry_run,
                    wait_confirm,
                    attest_private_network,
                    force_clearnet,
                } => {
                    if !ironwood_active {
                        return Err(NozyError::InvalidOperation(format!(
                            "Ironwood (NU6.3) is not active on this network yet (tip {}, activation {}).",
                            chain_tip,
                            activation_height
                                .map(|h| h.to_string())
                                .unwrap_or_else(|| "unknown".to_string())
                        )));
                    }

                    if let Some((_schedule, path, expired)) =
                        refresh_orchard_migration_schedule_at(ironwood_active, chain_tip)?
                    {
                        if expired > 0 {
                            println!(
                                "♻️  Rebuilt Ironwood schedule after {} expired/missed transfer windows",
                                expired
                            );
                            println!("   Schedule: {}", path.display());
                        }
                    }

                    let network_privacy = nozy::ironwood::MigrationNetworkPrivacyOpts {
                        attest_private_network,
                        force_clearnet,
                        broadcast_via_nym_mixnet: config.privacy_network.broadcast_via_nym_mixnet,
                    };
                    let result = execute_orchard_migration_broadcast(
                        &config.zebra_url,
                        ironwood_active,
                        dry_run,
                        wait_confirm,
                        &network_privacy,
                    )
                    .await?;

                    if dry_run {
                        println!("🧪 Dry run — presigned turnstile broadcast plan");
                    } else if result.blockers.is_empty() || result.confirmed {
                        println!("✅ Presigned turnstile transaction broadcast");
                    } else {
                        println!("⚠️  Turnstile broadcast did not complete");
                    }
                    println!("   Sequence: {}", result.sequence);
                    println!("   TXID:     {}", result.txid);
                    println!("   Height:   {}", result.broadcast_at_height);
                    println!("   Schedule: {}", result.schedule_path.display());
                    if result.confirmed {
                        println!("   Confirmed: yes");
                    } else if wait_confirm && !dry_run {
                        println!("   Confirmed: pending (run sync and preflight again)");
                    }
                    for blocker in &result.blockers {
                        println!("   Blocker: {blocker}");
                    }
                }
                IronwoodCommand::Split { dry_run } => {
                    if !ironwood_active {
                        return Err(NozyError::InvalidOperation(format!(
                            "Ironwood (NU6.3) is not active on this network yet (tip {}, activation {}). \
                             ZIP 318 note splitting is only valid after activation.",
                            chain_tip,
                            activation_height
                                .map(|h| h.to_string())
                                .unwrap_or_else(|| "unknown".to_string())
                        )));
                    }

                    println!("🔐 Unlocking wallet for ZIP 318 note split...");
                    let (wallet, _storage) = load_wallet().await?;
                    println!("🔍 Loading spendable Orchard notes...");
                    let spendable_notes = scan_notes_for_sending(wallet, &config.zebra_url).await?;

                    if dry_run {
                        let spend_note = spendable_notes
                            .iter()
                            .filter(|note| !note.orchard_note.spent)
                            .filter(|note| {
                                nozy::ironwood::note_requires_canonical_split(
                                    note.orchard_note.value,
                                )
                            })
                            .max_by_key(|note| note.orchard_note.value)
                            .ok_or_else(|| {
                                NozyError::InvalidOperation(
                                    "No Orchard note requires canonical splitting.".to_string(),
                                )
                            })?;
                        let (outputs, fee) =
                            plan_orchard_note_split_outputs(spend_note.orchard_note.value)?;
                        println!("🧪 Dry run — ZIP 318 note split plan");
                        println!(
                            "   Source note: {} zat ({})",
                            spend_note.orchard_note.value,
                            hex::encode(spend_note.orchard_note.nullifier.to_bytes())
                        );
                        println!("   Fee:         {fee} zat");
                        println!("   Outputs:");
                        for (index, value) in outputs.iter().enumerate() {
                            println!("     • #{} {value} zat", index + 1);
                        }
                        println!("   Run without --dry-run to build, prove, and broadcast.");
                    } else {
                        let result = execute_orchard_note_split(
                            &config.zebra_url,
                            ironwood_active,
                            &spendable_notes,
                            true,
                        )
                        .await?;
                        println!("✅ Orchard note split broadcast");
                        println!(
                            "   Source:  {} zat ({})",
                            result.source_value_zat, result.source_nullifier_hex
                        );
                        println!("   Fee:     {} zat", result.fee_zat);
                        println!("   TXID:    {}", result.txid);
                        println!("   Outputs:");
                        for (index, value) in result.output_values_zat.iter().enumerate() {
                            println!("     • #{} {value} zat", index + 1);
                        }
                        if result.note_split_still_required {
                            println!(
                                "   Next: run `nozy sync --to-tip`, then `nozy ironwood split` again if preflight still reports split-required."
                            );
                        } else {
                            println!(
                                "   Next: run `nozy sync --to-tip`, `nozy ironwood plan --save`, then `nozy ironwood preflight`."
                            );
                        }
                    }
                }
            }
        }

        Commands::Nu61 => {
            use nozy::load_config;
            use nozy::nu6_1_check;
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
                }
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
