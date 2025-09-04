use nozy::{
    HDWallet, AddressManager, ZebraClient, ZcashTransactionBuilder, 
    ZcashKeyDerivation, NoteScanner, NoteStorage
};
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 NozyWallet - Send ZEC Transaction\n");

    
    let password = "your_wallet_password";
    let hd_wallet = HDWallet::new(password)?;
    let address_manager = AddressManager::new();
    let zebra_client = ZebraClient::new("http://127.0.0.1:18232".to_string());
    let mut transaction_builder = ZcashTransactionBuilder::new();
    let key_derivation = ZcashKeyDerivation::new(hd_wallet.clone());
    let note_storage = NoteStorage::new("nozy_storage".to_string())?;

    
    transaction_builder.set_zebra_client(zebra_client.clone());
    transaction_builder.set_key_derivation(key_derivation.clone());

    let mut note_scanner = NoteScanner::new();
    note_scanner.set_zebra_client(zebra_client.clone());
    note_scanner.set_note_storage(note_storage.clone());
    note_scanner.set_key_derivation(key_derivation.clone());

    transaction_builder.setup_note_scanner()?;

    
    print!("Enter source address: ");
    io::stdout().flush()?;
    let mut source_address = String::new();
    io::stdin().read_line(&mut source_address)?;
    let source_address = source_address.trim();

    print!("Enter destination address: ");
    io::stdout().flush()?;
    let mut dest_address = String::new();
    io::stdin().read_line(&mut dest_address)?;
    let dest_address = dest_address.trim();

    print!("Enter amount in ZEC: ");
    io::stdout().flush()?;
    let mut amount_input = String::new();
    io::stdin().read_line(&mut amount_input)?;
    let amount: f64 = amount_input.trim().parse()?;

    println!("\n📊 Checking wallet balance...");
    
    let balance = note_scanner.get_address_balance(source_address, password)?;
    let balance_zec = balance as f64 / 100_000_000.0;
    
    println!("💰 Available balance: {:.8} ZEC", balance_zec);
    
    if balance < (amount * 100_000_000.0) as u64 {
        println!("❌ Insufficient funds!");
        return Ok(());
    }

    println!("\n🔍 Scanning for spendable notes...");
    
    
    let spendable_notes = note_scanner.get_spendable_notes(source_address, password)?;
    println!("📝 Found {} spendable notes", spendable_notes.len());

    if spendable_notes.is_empty() {
        println!("❌ No spendable notes found!");
        return Ok(());
    }

    println!("\n🏗️ Building transaction...");
    
    
    let transaction = transaction_builder.build_send_transaction(
        source_address,
        dest_address,
        amount,
        password,
    )?;

    println!("✅ Transaction built successfully!");
    println!("🆔 Transaction hex: {}", transaction.hex);
    println!("📏 Transaction size: {} bytes", transaction.hex.len() / 2);

    
    print!("\n❓ Confirm transaction? (y/N): ");
    io::stdout().flush()?;
    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm)?;
    
    if confirm.trim().to_lowercase() != "y" {
        println!("❌ Transaction cancelled");
        return Ok(());
    }

    println!("\n📡 Broadcasting transaction...");
    
    
    if let Some(client) = transaction_builder.get_zebra_client() {
        match client.broadcast_transaction(&hex::decode(&transaction.hex)?).await {
            Ok(txid) => {
                println!("✅ Transaction broadcast successfully!");
                println!("🆔 Transaction ID: {}", txid);
                println!("🔗 View on explorer: https://explorer.zcha.in/tx/{}", txid);
                
                
                
                let parsed_transaction = nozy::block_parser::ParsedTransaction {
                    txid: txid.clone(),
                    version: 5,
                    orchard_actions: vec![],
                    sapling_spends: vec![],
                    sapling_outputs: vec![],
                    transparent_inputs: vec![],
                    transparent_outputs: vec![],
                };
                note_storage.store_transaction(parsed_transaction, 0, chrono::Utc::now())?;
                println!("💾 Transaction saved to local storage");
            }
            Err(e) => {
                println!("❌ Failed to broadcast transaction: {}", e);
            }
        }
    } else {
        println!("❌ Zebra client not available");
    }

    Ok(())
} 