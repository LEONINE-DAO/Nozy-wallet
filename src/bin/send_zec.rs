use nozy::{
    HDWallet, ZebraClient, ZcashTransactionBuilder, NoteScanner
};
use std::io::{self, Write};
use std::path::PathBuf;
use dialoguer::Password;
use nozy::{WalletStorage, NozyResult, NozyError};

async fn load_wallet() -> NozyResult<(HDWallet, WalletStorage)> {
    use nozy::paths::get_wallet_data_dir;
    let storage = WalletStorage::with_xdg_dir();
    let wallet_path = get_wallet_data_dir().join("wallet.dat");
    if wallet_path.exists() {
        let password = Password::new()
            .with_prompt("Enter wallet password")
            .interact()
            .map_err(|e| NozyError::InvalidOperation(format!("Password input error: {}", e)))?;
        let wallet = storage.load_wallet(&password).await?;
        Ok((wallet, storage))
    } else {
        Err(NozyError::Storage("No wallet found. Use 'nozy new' or 'nozy restore' to create a wallet first.".to_string()))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 NozyWallet - Send ZEC Transaction\n");

    // Load existing wallet
    let (hd_wallet, _storage) = load_wallet().await?;
    let zebra_client = ZebraClient::new("http://127.0.0.1:8232".to_string());
    let mut transaction_builder = ZcashTransactionBuilder::new();
    let mut note_scanner = NoteScanner::new(hd_wallet.clone(), zebra_client.clone());

    // Set up transaction builder
    transaction_builder.set_zebra_url("http://127.0.0.1:8232");
    transaction_builder.enable_mainnet_broadcast();

    // Get user input
    print!("Enter recipient address: ");
    io::stdout().flush()?;
    let mut recipient = String::new();
    io::stdin().read_line(&mut recipient)?;
    let recipient = recipient.trim().to_string();

    print!("Enter amount in ZEC: ");
    io::stdout().flush()?;
    let mut amount_str = String::new();
    io::stdin().read_line(&mut amount_str)?;
    let amount: f64 = amount_str.trim().parse()?;

    // Optional memo
    print!("Enter memo (optional, press Enter to skip): ");
    io::stdout().flush()?;
    let mut memo_input = String::new();
    io::stdin().read_line(&mut memo_input)?;
    let memo_bytes_opt: Option<Vec<u8>> = {
        let trimmed = memo_input.trim();
        if trimmed.is_empty() { None } else { Some(trimmed.as_bytes().to_vec()) }
    };

    // Scan for spendable notes
    println!("🔎 Scanning for spendable notes...");
    let tip_height = match zebra_client.get_block_count().await {
        Ok(h) => h,
        Err(e) => {
            println!("⚠️  Could not fetch tip height: {}. Using default.", e);
            3_066_071
        }
    };
    let start_height = tip_height.saturating_sub(10_000);
    
    let spendable_notes = match note_scanner.scan_notes(Some(start_height), Some(tip_height)).await {
        Ok((_result, spendable)) => spendable,
        Err(e) => {
            println!("⚠️  Note scan failed: {}. Proceeding with empty note set.", e);
            Vec::new()
        }
    };

    if spendable_notes.is_empty() {
        println!("❌ No spendable notes found!");
        return Ok(());
    }

    println!("✅ Found {} spendable notes", spendable_notes.len());
    for (i, note) in spendable_notes.iter().enumerate() {
        println!("  Note {}: {} ZAT", i + 1, note.orchard_note.value);
    }

    // Build transaction
    let amount_zatoshis = (amount * 100_000_000.0) as u64;
    let fee_zatoshis = 10_000;

    println!("🔧 Building transaction...");
    let transaction = transaction_builder.build_send_transaction(
        &zebra_client,
        &spendable_notes,
        &recipient,
        amount_zatoshis,
        fee_zatoshis,
        memo_bytes_opt.as_deref(),
    ).await?;

    println!("✅ Transaction built successfully!");
    println!("🆔 Transaction ID: {}", transaction.txid);
    println!("📏 Transaction size: {} bytes", transaction.raw_transaction.len());

    // Broadcast transaction
    println!("🚀 Broadcasting transaction...");
    match transaction_builder.broadcast_transaction(&transaction).await {
        Ok(txid) => {
            println!("✅ Transaction broadcast successful!");
            println!("🌐 Network TXID: {}", txid);
            println!("🔗 Explorer: https://zcashblockexplorer.com/transactions/{}", txid);
        },
        Err(e) => {
            println!("❌ Broadcast failed: {}", e);
        }
    }

    Ok(())
}