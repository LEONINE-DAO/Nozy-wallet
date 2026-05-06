use nozy::{HDWallet, NoteScanner, ZebraClient};
use zcash_protocol::consensus::NetworkType;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 NozyWallet - Production Features Test\n");

    let hd_wallet = HDWallet::new()?;
    let zebra_client = ZebraClient::new("http://127.0.0.1:8232".to_string());
    let mut note_scanner = NoteScanner::new(hd_wallet.clone(), zebra_client.clone());

    println!("✅ Wallet created successfully");
    println!("📝 Mnemonic: {}", hd_wallet.get_mnemonic());

    let network = NetworkType::Main;
    let mut addresses = Vec::new();
    for i in 0..5 {
        let addr = hd_wallet.generate_orchard_address(0, i, network)?;
        addresses.push(addr);
    }
    println!("🏠 Generated {} addresses:", addresses.len());
    for (i, addr) in addresses.iter().enumerate() {
        println!("  {}: {}", i + 1, addr);
    }

    println!("\n🔗 Testing Zebra connection...");
    match zebra_client.get_block_count().await {
        Ok(height) => {
            println!("✅ Connected to Zebra node");
            println!("📊 Current block height: {}", height);
        }
        Err(e) => {
            println!("❌ Failed to connect to Zebra: {}", e);
            return Ok(());
        }
    }

    println!("\n🔍 Testing note scanning...");
    let tip_height = match zebra_client.get_block_count().await {
        Ok(h) => h,
        Err(_) => 3_066_071,
    };
    let start_height = tip_height.saturating_sub(100);

    match note_scanner
        .scan_notes(Some(start_height), Some(tip_height))
        .await
    {
        Ok((result, spendable)) => {
            println!("✅ Note scanning completed");
            println!("📊 Total notes found: {}", result.notes.len());
            println!("💰 Total balance: {} ZAT", result.total_balance);
            println!("💸 Spendable notes: {}", spendable.len());

            if result.total_balance > 0 {
                println!("🎉 Found ZEC in wallet!");
                for (i, note) in result.notes.iter().enumerate() {
                    if !note.spent {
                        println!(
                            "  Note {}: {} ZAT (Block: {})",
                            i + 1,
                            note.value,
                            note.block_height
                        );
                    }
                }
            } else {
                println!("💡 No ZEC found in scanned blocks");
            }
        }
        Err(e) => {
            println!("❌ Note scanning failed: {}", e);
        }
    }

    println!("\n🎯 All production features tested!");
    Ok(())
}
