use nozy::paths::get_wallet_data_dir;
use nozy::WalletStorage;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Wallet Diagnostic Tool");
    println!("========================\n");

    let wallet_path = get_wallet_data_dir().join("wallet.dat");

    if !wallet_path.exists() {
        println!("❌ No wallet found at: {}", wallet_path.display());
        return Ok(());
    }

    let metadata = fs::metadata(&wallet_path)?;
    println!("📁 Wallet File Info:");
    println!("   Path: {}", wallet_path.display());
    println!("   Size: {} bytes", metadata.len());
    println!("   Created: {:?}", metadata.created());
    println!("   Modified: {:?}", metadata.modified());
    println!();

    // Try to read the encrypted file (it's stored as hex string)
    let hex_string = fs::read_to_string(&wallet_path)?;
    println!("🔐 Encrypted File Analysis:");
    println!("   Hex string length: {} characters", hex_string.len());

    // Try to decode as hex
    match hex::decode(&hex_string.trim()) {
        Ok(decoded) => {
            println!("   ✅ File is hex-encoded");
            println!("   Decoded size: {} bytes", decoded.len());

            // The format is: [16 bytes salt][12 bytes nonce][ciphertext]
            if decoded.len() >= 28 {
                let salt = &decoded[0..16];
                let nonce = &decoded[16..28];
                println!("   Salt (first 8 bytes): {}", hex::encode(&salt[0..8]));
                println!("   Nonce (first 8 bytes): {}", hex::encode(&nonce[0..8]));
                println!("   Ciphertext size: {} bytes", decoded.len() - 28);
                println!("   ✅ File structure appears valid");
            } else {
                println!("   ❌ Decoded data too small - file might be corrupted!");
            }
        }
        Err(e) => {
            println!("   ❌ Failed to decode hex: {}", e);
            println!("   File might be corrupted or in wrong format");
        }
    }

    println!();
    println!("🧪 Testing Common Passwords:");
    let common_passwords = vec![
        "",
        "password",
        "Password",
        "PASSWORD",
        "123456",
        "nozy",
        "Nozy",
        "nozywallet",
        "NozyWallet",
        "NOZYWALLET",
    ];

    let storage = WalletStorage::with_xdg_dir();
    let mut tried = 0;

    for pwd in common_passwords {
        tried += 1;
        print!(
            "   Test {}: {:?} ... ",
            tried,
            if pwd.is_empty() { "(empty)" } else { pwd }
        );

        match storage.load_wallet(pwd).await {
            Ok(_) => {
                println!("✅ SUCCESS!");
                println!(
                    "\n🎉 Found the password: {:?}",
                    if pwd.is_empty() {
                        "(empty - no password)"
                    } else {
                        pwd
                    }
                );
                return Ok(());
            }
            Err(e) => {
                let error_str = e.to_string();
                if error_str.contains("Decryption failed") {
                    println!("❌");
                } else if error_str.contains("Invalid password") {
                    println!("⚠️  (decrypted but hash mismatch)");
                } else {
                    println!("❌ Error");
                }
            }
        }
    }

    println!("\n❌ None of the common passwords worked.");
    println!("\n💡 Diagnosis:");
    println!("   The wallet file structure appears valid.");
    println!("   The encryption password doesn't match any common variations.");
    println!("\n🔧 Solution:");
    println!("   You need to restore from your mnemonic phrase:");
    println!("   ./target/release/nozy.exe restore");
    println!("\n⚠️  Without the mnemonic, the wallet cannot be recovered.");

    Ok(())
}
