# NozyWallet API Documentation

## Overview

NozyWallet provides a comprehensive Rust API for Zcash Orchard wallet operations. This document covers the main types, functions, and usage patterns.

## Core Types

### HDWallet

The main wallet type that handles key generation, address creation, and password protection.

```rust
use nozy::{HDWallet, NozyResult};

// Create a new wallet
let wallet = HDWallet::new()?;

// Create from existing mnemonic
let wallet = HDWallet::from_mnemonic("abandon abandon abandon...")?;

// Set password protection
wallet.set_password("my_secure_password")?;

// Generate Orchard address
let address = wallet.generate_orchard_address(0, 0)?;

// Verify password
let is_valid = wallet.verify_password("my_secure_password")?;
```

### ZebraClient

Client for connecting to Zebra RPC node.

```rust
use nozy::ZebraClient;

let client = ZebraClient::new("http://127.0.0.1:8232".to_string());

// Get block count
let height = client.get_block_count().await?;

// Get block data
let block = client.get_block(1000000).await?;

// Get Orchard tree state
let tree_state = client.get_orchard_tree_state(height).await?;
```

### OrchardTransactionBuilder

Builder for creating Orchard transactions with proving support.

```rust
use nozy::orchard_tx::OrchardTransactionBuilder;

// Create builder with proving parameters
let mut builder = OrchardTransactionBuilder::new_async(true).await?;

// Check proving status
let status = builder.get_proving_status();
println!("Can prove: {}", status.can_prove);

// Build transaction
let tx_data = builder.build_single_spend(
    &zebra_client,
    &spendable_notes,
    "u1...",
    1000000, // amount in zatoshis
    10000,   // fee in zatoshis
    Some(b"memo"),
).await?;
```

### OrchardProvingManager

Manages Orchard proving parameters.

```rust
use nozy::proving::OrchardProvingManager;
use std::path::PathBuf;

let mut manager = OrchardProvingManager::new(PathBuf::from("orchard_params"));

// Initialize and check for existing parameters
manager.initialize().await?;

// Download placeholder parameters (for testing)
manager.download_parameters().await?;

// Check status
let status = manager.get_status();
println!("Status: {}", status.status_message());

// Get proving parameters
let spend_params = manager.get_proving_params("spend")?;
let output_params = manager.get_proving_params("output")?;
```

## Error Handling

### NozyError

Comprehensive error type with user-friendly messages.

```rust
use nozy::{NozyError, NozyResult};

match result {
    Ok(value) => println!("Success: {:?}", value),
    Err(NozyError::NetworkError(msg)) => {
        println!("Network error: {}", msg);
    },
    Err(NozyError::AddressParsing(msg)) => {
        println!("Address parsing error: {}", msg);
    },
    Err(NozyError::Transaction(msg)) => {
        println!("Transaction error: {}", msg);
    },
    Err(NozyError::Storage(msg)) => {
        println!("Storage error: {}", msg);
    },
    Err(NozyError::KeyDerivation(msg)) => {
        println!("Key derivation error: {}", msg);
    },
    Err(NozyError::MerklePath(msg)) => {
        println!("Merkle path error: {}", msg);
    },
    Err(NozyError::Cryptographic(msg)) => {
        println!("Cryptographic error: {}", msg);
    },
    Err(NozyError::InvalidOperation(msg)) => {
        println!("Invalid operation: {}", msg);
    },
}
```

### User-Friendly Error Messages

All errors provide context-aware messages:

```rust
let error = NozyError::NetworkError("Connection failed".to_string());
println!("{}", error.user_friendly_message());
// Output: "Network connection failed. Please check your internet connection and try again."

let error = NozyError::AddressParsing("Invalid format".to_string());
println!("{}", error.user_friendly_message());
// Output: "Invalid address format. Please check the address and try again."
```

## Note Management

### NoteScanner

Scans blockchain for incoming notes.

```rust
use nozy::{NoteScanner, SpendableNote};

let mut scanner = NoteScanner::new(wallet, zebra_client);

// Scan for notes
let (result, spendable_notes) = scanner.scan_notes(
    Some(1000000), // start height
    Some(1000100), // end height
).await?;

println!("Found {} notes", result.notes.len());
println!("Total balance: {} zatoshis", result.total_balance);
```

### SpendableNote

Represents a note that can be spent.

```rust
use nozy::SpendableNote;

// SpendableNote contains:
// - orchard_note: OrchardNote (the actual note)
// - spending_key: SpendingKey (for spending)
// - derivation_path: String (BIP32 path)
```

## Storage

### WalletStorage

Handles wallet persistence and backup.

```rust
use nozy::WalletStorage;
use std::path::PathBuf;

let storage = WalletStorage::new(PathBuf::from("wallet_data"));

// Save wallet
storage.save_wallet(&wallet, "password").await?;

// Load wallet
let loaded_wallet = storage.load_wallet("password").await?;

// Create backup
storage.create_backup("backups").await?;

// List backups
let backups = storage.list_backups()?;
```

## Transaction Building

### Complete Transaction Flow

```rust
use nozy::{HDWallet, ZebraClient, OrchardTransactionBuilder, NoteScanner};

#[tokio::main]
async fn main() -> NozyResult<()> {
    // 1. Create wallet
    let wallet = HDWallet::new()?;
    wallet.set_password("secure_password")?;
    
    // 2. Connect to Zebra
    let zebra_client = ZebraClient::new("http://127.0.0.1:8232".to_string());
    
    // 3. Scan for notes
    let mut scanner = NoteScanner::new(wallet, zebra_client.clone());
    let (_, spendable_notes) = scanner.scan_notes(Some(1000000), Some(1000100)).await?;
    
    // 4. Build transaction
    let mut builder = OrchardTransactionBuilder::new_async(true).await?;
    let tx_data = builder.build_single_spend(
        &zebra_client,
        &spendable_notes,
        "u1...", // recipient
        1000000, // amount
        10000,   // fee
        Some(b"Hello from NozyWallet!"),
    ).await?;
    
    // 5. Transaction is ready for broadcasting
    println!("Transaction built: {} bytes", tx_data.len());
    
    Ok(())
}
```

## Proving Parameters

### Managing Proving Parameters

```rust
use nozy::proving::{OrchardProvingManager, OrchardProvingKey};

// Initialize proving manager
let mut manager = OrchardProvingManager::new(PathBuf::from("orchard_params"));
manager.initialize().await?;

// Download parameters (placeholder for testing)
manager.download_parameters().await?;

// Check status
let status = manager.get_status();
if status.can_prove {
    println!("✅ Ready for proving");
    
    // Load proving key
    let proving_key = OrchardProvingKey::from_manager(&manager)?;
    println!("Key info: {}", proving_key.info());
} else {
    println!("❌ Missing parameters: {}", status.status_message());
}
```

## CLI Commands

### Available Commands

```bash
# Create new wallet
cargo run --bin nozy new

# Restore from mnemonic
cargo run --bin nozy restore

# Generate addresses
cargo run --bin nozy addresses --count 5

# Scan for notes
cargo run --bin nozy scan --start-height 1000000 --end-height 1000100

# Send ZEC
cargo run --bin nozy send --recipient "u1..." --amount 0.1

# Manage proving parameters
cargo run --bin nozy proving --status
cargo run --bin nozy proving --download

# Test Zebra connection
cargo run --bin nozy test-zebra --zebra-url http://127.0.0.1:8232

# List stored notes
cargo run --bin nozy list-notes
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_creation() {
        let wallet = HDWallet::new().unwrap();
        assert!(!wallet.get_mnemonic().is_empty());
    }

    #[tokio::test]
    async fn test_zebra_connection() {
        let client = ZebraClient::new("http://127.0.0.1:8232".to_string());
        let result = client.get_block_count().await;
        // Test will pass if Zebra is running
        assert!(result.is_ok() || result.is_err());
    }
}
```

### Running Tests

```bash
# Run all tests
cargo test

# Run with ignored tests (requires Zebra)
cargo test -- --ignored

# Run specific test module
cargo test tests::performance_tests
```

## Security Considerations

### Password Protection

- Uses Argon2 for password hashing
- Salt is randomly generated
- Passwords are never stored in plain text

### Key Management

- Private keys are derived from mnemonic
- Keys are only loaded when needed
- No keys are stored in plain text

### Storage

- Wallets are encrypted with AES-256-GCM
- Backup files are also encrypted
- Sensitive data is cleared from memory

## Performance

### Benchmarks

```bash
# Run performance tests
cargo test performance_tests

# Run with timing
cargo test performance_tests -- --nocapture
```

### Optimization Tips

1. Use `--release` build for production
2. Keep proving parameters in fast storage
3. Use appropriate scan ranges
4. Consider parallel note scanning

## Troubleshooting

### Common Issues

1. **Zebra Connection Failed**
   - Check if Zebra is running
   - Verify RPC URL and port
   - Check firewall settings

2. **Proving Parameters Missing**
   - Download parameters: `cargo run --bin nozy proving --download`
   - Check file permissions
   - Verify parameter file integrity

3. **Wallet Load Failed**
   - Check password correctness
   - Verify wallet file exists
   - Check file permissions

4. **Transaction Build Failed**
   - Ensure sufficient funds
   - Check address format
   - Verify proving parameters

### Debug Mode

```bash
# Run with debug output
RUST_LOG=debug cargo run --bin nozy new

# Run specific command with debug
RUST_LOG=debug cargo run --bin nozy scan --start-height 1000000
```

## Examples

### Complete Wallet Setup

```rust
use nozy::{HDWallet, WalletStorage, ZebraClient, OrchardTransactionBuilder};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> NozyResult<()> {
    // 1. Create wallet
    let mut wallet = HDWallet::new()?;
    wallet.set_password("my_secure_password")?;
    
    // 2. Save wallet
    let storage = WalletStorage::new(PathBuf::from("wallet_data"));
    storage.save_wallet(&wallet, "my_secure_password").await?;
    
    // 3. Generate address
    let address = wallet.generate_orchard_address(0, 0)?;
    println!("Your address: {}", address);
    
    // 4. Connect to Zebra
    let zebra_client = ZebraClient::new("http://127.0.0.1:8232".to_string());
    
    // 5. Initialize proving
    let mut builder = OrchardTransactionBuilder::new_async(true).await?;
    
    println!("Wallet setup complete!");
    Ok(())
}
```

### Note Scanning and Spending

```rust
use nozy::{NoteScanner, OrchardTransactionBuilder, ZebraClient};

async fn scan_and_spend() -> NozyResult<()> {
    // Load wallet and connect to Zebra
    let wallet = load_wallet().await?;
    let zebra_client = ZebraClient::new("http://127.0.0.1:8232".to_string());
    
    // Scan for notes
    let mut scanner = NoteScanner::new(wallet, zebra_client.clone());
    let (result, spendable_notes) = scanner.scan_notes(Some(1000000), Some(1000100)).await?;
    
    if result.total_balance > 0 {
        println!("Found {} ZEC", result.total_balance as f64 / 100_000_000.0);
        
        // Build transaction
        let mut builder = OrchardTransactionBuilder::new_async(true).await?;
        let tx_data = builder.build_single_spend(
            &zebra_client,
            &spendable_notes,
            "u1...", // recipient
            500000,  // amount (0.005 ZEC)
            10000,   // fee
            Some(b"Payment from NozyWallet"),
        ).await?;
        
        println!("Transaction ready: {} bytes", tx_data.len());
    } else {
        println!("No funds found");
    }
    
    Ok(())
}
```

This API documentation provides comprehensive coverage of NozyWallet's functionality. For more specific examples and advanced usage, refer to the test suite and example binaries.
