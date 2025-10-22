use crate::error::{NozyError, NozyResult};
use crate::{HDWallet, WalletStorage, NoteScanner, ZebraClient};
use std::path::PathBuf;
use dialoguer::Password;

pub async fn load_wallet() -> NozyResult<(HDWallet, WalletStorage)> {
    let storage = WalletStorage::new(PathBuf::from("wallet_data"));
    
    let wallet_path = std::path::Path::new("wallet_data/wallet.dat");
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

pub async fn scan_notes_for_sending(wallet: HDWallet, zebra_url: &str) -> NozyResult<Vec<crate::SpendableNote>> {
    let zebra_client = ZebraClient::new(zebra_url.to_string());
    let tip_height = match zebra_client.get_block_count().await {
        Ok(h) => h,
        Err(e) => {
            println!("⚠️  Could not fetch tip height from Zebra ({}). Falling back to 10k-block scan ending at default.", e);
            3_066_071
        }
    };
    let start_height = tip_height.saturating_sub(10_000);
    let mut note_scanner = NoteScanner::new(wallet, ZebraClient::new("http://127.0.0.1:8232".to_string()));
    
    match note_scanner.scan_notes(Some(start_height), Some(tip_height)).await {
        Ok((_result, spendable)) => Ok(spendable),
        Err(e) => {
            println!("⚠️  Note scan failed: {}. Proceeding with empty note set.", e);
            Ok(Vec::new())
        }
    }
}

pub async fn build_and_broadcast_transaction(
    zebra_client: &ZebraClient,
    spendable_notes: &[crate::SpendableNote],
    recipient: &str,
    amount_zatoshis: u64,
    fee_zatoshis: u64,
    memo: Option<&[u8]>,
    enable_broadcast: bool,
    zebra_url: &str,
) -> NozyResult<()> {
    use crate::ZcashTransactionBuilder;
    
    let mut tx_builder = ZcashTransactionBuilder::new();
    tx_builder.set_zebra_url(zebra_url);
    
    if enable_broadcast {
        tx_builder.enable_mainnet_broadcast();
    }
    
    let transaction = tx_builder.build_send_transaction(
        zebra_client,
        spendable_notes,
        recipient,
        amount_zatoshis,
        fee_zatoshis,
        memo,
    ).await?;
    
    println!("✅ Transaction built successfully!");
    println!("🆔 Transaction ID: {}", transaction.txid);
    println!("📏 Transaction size: {} bytes", transaction.raw_transaction.len());
    
    if enable_broadcast {
        match tx_builder.broadcast_transaction(&transaction).await {
            Ok(txid) => {
                println!("✅ Transaction broadcast successful!");
                println!("🌐 Network TXID: {}", txid);
                println!("🔗 Explorer: https://zcashblockexplorer.com/transactions/{}", txid);
            },
            Err(e) => {
                println!("❌ Broadcast failed: {}", e);
            }
        }
    } else {
        println!("💡 Transaction ready for broadcast (broadcasting disabled)");
    }
    
    Ok(())
}

pub fn handle_insufficient_funds_error(error: &NozyError) {
    if error.to_string().contains("Insufficient funds") {
        println!("💡 Tip: You need to receive some ZEC first before you can send it.");
        println!("   Use the 'scan' command to check for received notes.");
    }
}
