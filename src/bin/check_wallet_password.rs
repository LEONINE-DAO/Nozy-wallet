use nozy::paths::get_wallet_data_dir;
use nozy::WalletStorage;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Wallet Password Diagnostic Tool\n");

    let wallet_path = get_wallet_data_dir().join("wallet.dat");

    if !wallet_path.exists() {
        println!("❌ No wallet found at: {}", wallet_path.display());
        println!("   Create a wallet first with: cargo run --bin nozy -- new");
        return Ok(());
    }

    println!("✅ Wallet file found at: {}", wallet_path.display());
    println!(
        "   File size: {} bytes\n",
        fs::metadata(&wallet_path)?.len()
    );

    let storage = WalletStorage::with_xdg_dir();

    // Try to load with empty password
    println!("🔐 Testing password status...");
    match storage.load_wallet("").await {
        Ok(_) => {
            println!("✅ Wallet loaded successfully with EMPTY password");
            println!("   This wallet has NO password protection.");
            println!("   You can use an empty password or any password to unlock it.");
        }
        Err(e) => {
            println!("❌ Failed to load with empty password: {}", e);
            println!("   This wallet IS password protected.");
            println!("   You need to enter the correct password.");

            // Check if it's a decryption error or password verification error
            let error_msg = e.to_string();
            if error_msg.contains("decrypt") || error_msg.contains("deserialize") {
                println!("\n💡 The wallet file is encrypted with a password.");
                println!("   You must use the password you set when creating the wallet.");
            } else if error_msg.contains("Invalid password") {
                println!("\n💡 The password verification failed.");
                println!("   Make sure you're entering the exact password you used when creating the wallet.");
            }
        }
    }

    println!("\n📝 Troubleshooting:");
    println!("   1. If you forgot your password, you can restore from your mnemonic:");
    println!("      cargo run --bin nozy -- restore");
    println!("   2. Make sure you're entering the password exactly as you set it");
    println!("   3. Check for typos, case sensitivity, and special characters");

    Ok(())
}
