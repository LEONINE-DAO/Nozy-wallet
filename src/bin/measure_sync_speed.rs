use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use nozy::{load_config, HDWallet, NoteScanner, WalletStorage, ZebraClient};
use std::time::{Duration, Instant};

#[derive(Parser)]
#[command(name = "measure-sync-speed")]
#[command(about = "Measure wallet sync/scan speed to estimate full restore time")]
struct Args {
    /// Number of blocks to test (default: 100)
    #[arg(long, default_value = "100")]
    test_blocks: u32,

    /// Starting block height (default: current - test_blocks)
    #[arg(long)]
    start_height: Option<u32>,

    /// Zebra URL (overrides config)
    #[arg(long)]
    zebra_url: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("🚀 NozyWallet Sync Speed Measurement");
    println!("====================================\n");

    // Load wallet
    let (wallet, _storage) = load_wallet().await?;

    // Setup Zebra client
    let mut config = load_config();
    if let Some(url) = args.zebra_url {
        config.zebra_url = url;
    }
    let zebra_client = ZebraClient::from_config(&config);

    // Get current block height
    let current_height = zebra_client.get_block_count().await?;
    println!("📊 Current blockchain height: {}", current_height);

    // Determine test range
    let start_height = args
        .start_height
        .unwrap_or_else(|| current_height.saturating_sub(args.test_blocks));
    let end_height = start_height + args.test_blocks - 1;

    println!(
        "🧪 Testing sync speed on {} blocks ({} to {})",
        args.test_blocks, start_height, end_height
    );
    println!();

    // Measure individual block fetch time
    println!("⏱️  Measuring individual block fetch times...");
    let mut fetch_times = Vec::new();
    let mut parse_times = Vec::new();

    let pb = ProgressBar::new(args.test_blocks as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} blocks | {msg}")
            .unwrap()
            .progress_chars("#>-")
    );

    for height in start_height..=end_height {
        pb.set_message(format!("Block {}", height));

        // Measure block fetch
        let fetch_start = Instant::now();
        let block_hash = match zebra_client.get_block_hash(height).await {
            Ok(hash) => hash,
            Err(e) => {
                pb.println(format!("⚠️  Failed to get block {} hash: {}", height, e));
                pb.inc(1);
                continue;
            }
        };
        let fetch_time = fetch_start.elapsed();
        fetch_times.push(fetch_time);

        // Measure block data fetch
        let parse_start = Instant::now();
        match zebra_client.get_block_by_hash(&block_hash, 2).await {
            Ok(_) => {
                let parse_time = parse_start.elapsed();
                parse_times.push(parse_time);
            }
            Err(e) => {
                pb.println(format!("⚠️  Failed to get block {} data: {}", height, e));
            }
        }

        pb.inc(1);
    }
    pb.finish();

    // Calculate statistics
    if fetch_times.is_empty() {
        println!("❌ No successful block fetches. Check Zebra connection.");
        return Ok(());
    }

    let avg_fetch = fetch_times.iter().sum::<Duration>() / fetch_times.len() as u32;
    let min_fetch = fetch_times.iter().min().unwrap();
    let max_fetch = fetch_times.iter().max().unwrap();

    let avg_parse = if !parse_times.is_empty() {
        parse_times.iter().sum::<Duration>() / parse_times.len() as u32
    } else {
        Duration::ZERO
    };

    let avg_total = avg_fetch + avg_parse;
    let blocks_per_second = 1.0 / avg_total.as_secs_f64();

    println!("\n📊 Block Fetch Performance:");
    println!("  Average: {:?} per block", avg_fetch);
    println!("  Min: {:?}", min_fetch);
    println!("  Max: {:?}", max_fetch);
    println!("  Throughput: {:.2} blocks/second", blocks_per_second);

    if !parse_times.is_empty() {
        println!("\n📊 Block Parse Performance:");
        println!("  Average: {:?} per block", avg_parse);
    }

    // Measure full scan with note decryption
    println!("\n⏱️  Measuring full scan with note decryption...");
    let scan_start = Instant::now();
    let mut scanner = NoteScanner::new(wallet, zebra_client.clone());

    // Test on smaller range for full scan (decryption is expensive)
    let scan_test_blocks = args.test_blocks.min(10);
    let scan_end = start_height + scan_test_blocks - 1;

    match scanner.scan_notes(Some(start_height), Some(scan_end)).await {
        Ok((result, _)) => {
            let scan_time = scan_start.elapsed();
            let scan_blocks_per_sec = scan_test_blocks as f64 / scan_time.as_secs_f64();

            println!("\n📊 Full Scan Performance (with decryption):");
            println!("  Time for {} blocks: {:?}", scan_test_blocks, scan_time);
            println!("  Throughput: {:.2} blocks/second", scan_blocks_per_sec);
            println!("  Notes found: {}", result.notes.len());
        }
        Err(e) => {
            println!("⚠️  Full scan test failed: {}", e);
        }
    }

    // Estimate full restore time
    println!("\n📈 Sync Time Estimates:");
    println!("=====================================");

    // Calculate blocks to scan for new wallet
    let orchard_start = 3_050_000u32; // NU 6.1 activation (approximate)
    let blocks_to_scan = current_height.saturating_sub(orchard_start);

    println!("For a NEW wallet restore:");
    println!(
        "  Blocks to scan: {} (from {} to {})",
        blocks_to_scan, orchard_start, current_height
    );

    // Estimate based on fetch time (optimistic - doesn't include decryption)
    let optimistic_time = Duration::from_secs_f64(blocks_to_scan as f64 * avg_total.as_secs_f64());
    println!(
        "  ⚡ Optimistic estimate (fetch only): {}",
        format_duration(optimistic_time)
    );

    // Estimate based on full scan (includes decryption)
    if !parse_times.is_empty() {
        let full_scan_time_per_block = if scan_test_blocks > 0 {
            scan_start.elapsed().as_secs_f64() / scan_test_blocks as f64
        } else {
            avg_total.as_secs_f64() * 2.0 // Rough estimate: decryption doubles time
        };
        let realistic_time =
            Duration::from_secs_f64(blocks_to_scan as f64 * full_scan_time_per_block);
        println!(
            "  🎯 Realistic estimate (with decryption): {}",
            format_duration(realistic_time)
        );
    }

    // Estimate for incremental sync (last 1000 blocks)
    let incremental_blocks = 1000u32;
    let incremental_time =
        Duration::from_secs_f64(incremental_blocks as f64 * avg_total.as_secs_f64());
    println!(
        "\nFor INCREMENTAL sync (last {} blocks):",
        incremental_blocks
    );
    println!("  Estimated time: {}", format_duration(incremental_time));

    // Performance assessment
    println!("\n💡 Performance Assessment:");
    if blocks_per_second > 10.0 {
        println!("  ✅ EXCELLENT - Sync will be fast");
    } else if blocks_per_second > 5.0 {
        println!("  ✅ GOOD - Sync will be reasonable");
    } else if blocks_per_second > 1.0 {
        println!("  ⚠️  MODERATE - Sync may take hours");
    } else {
        println!("  ❌ SLOW - Sync will take days/weeks");
        println!("  💡 Consider optimizations:");
        println!("     - Use local Zebra node");
        println!("     - Check network connection");
        println!("     - Enable parallel block fetching (future feature)");
    }

    Ok(())
}

async fn load_wallet() -> Result<(HDWallet, WalletStorage), Box<dyn std::error::Error>> {
    use dialoguer::Password;
    use nozy::WalletStorage;

    let storage = WalletStorage::with_xdg_dir();
    let wallet_path = nozy::paths::get_wallet_data_dir().join("wallet.dat");

    if !wallet_path.exists() {
        return Err("No wallet found. Please create a wallet first.".into());
    }

    let password = Password::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Enter wallet password")
        .allow_empty_password(true)
        .interact()?;

    let wallet = storage.load_wallet(&password).await?;
    Ok((wallet, storage))
}

fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    if secs < 60 {
        format!("{} seconds", secs)
    } else if secs < 3600 {
        format!("{} minutes ({} seconds)", secs / 60, secs)
    } else if secs < 86400 {
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        format!("{} hours {} minutes", hours, mins)
    } else {
        let days = secs / 86400;
        let hours = (secs % 86400) / 3600;
        format!("{} days {} hours", days, hours)
    }
}
