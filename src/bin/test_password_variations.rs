use nozy::WalletStorage;
use nozy::paths::get_wallet_data_dir;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing Password Variations for 'nozywallet'");
    println!("==============================================\n");
    
    let wallet_path = get_wallet_data_dir().join("wallet.dat");
    
    if !wallet_path.exists() {
        println!("❌ No wallet found at: {}", wallet_path.display());
        return Ok(());
    }
    
    println!("✅ Wallet file found: {} bytes\n", fs::metadata(&wallet_path)?.len());
    
    let storage = WalletStorage::with_xdg_dir();
    
    // Test different variations of "nozywallet"
    let variations = vec![
        "nozywallet",
        "NozyWallet",
        "NOZYWALLET",
        "Nozywallet",
        "nozyWallet",
        " nozywallet",
        "nozywallet ",
        " nozywallet ",
        "nozywallet\n",
        "nozywallet\r",
        "nozywallet\r\n",
    ];
    
    println!("🧪 Testing password variations...\n");
    
    let mut found = false;
    for (i, password) in variations.iter().enumerate() {
        print!("Test {}: {:?} ... ", i + 1, password);
        
        match storage.load_wallet(password).await {
            Ok(_) => {
                println!("✅ SUCCESS!");
                println!("   The correct password is: {:?}", password);
                println!("   (Note any spaces, case differences, or special characters)");
                found = true;
                break;
            },
            Err(e) => {
                let error_str = e.to_string();
                if error_str.contains("Decryption failed") {
                    println!("❌ Wrong password (decryption failed)");
                } else if error_str.contains("Invalid password") {
                    println!("⚠️  Decryption OK, but password hash mismatch");
                    println!("   This suggests the password used for encryption differs from password_hash");
                } else {
                    println!("❌ Error: {}", e);
                }
            }
        }
    }
    
    if !found {
        println!("\n❌ None of the variations worked.");
        println!("\n💡 Possible issues:");
        println!("   1. The password might be completely different");
        println!("   2. There might be hidden characters or encoding issues");
        println!("   3. The wallet file might be corrupted");
        println!("\n🔧 Solutions:");
        println!("   - Restore from mnemonic: ./target/release/nozy.exe restore");
        println!("   - Check if you wrote down the password differently");
        println!("   - The password is case-sensitive and must match exactly");
    }
    
    Ok(())
}
