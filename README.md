# NozyWallet - Privacy by Default

**Monero-Level Privacy, Zcash Speed**

NozyWallet is a privacy-first Orchard wallet that enforces complete transaction privacy by default. Unlike other Zcash wallets, NozyWallet **only supports shielded transactions** - making it functionally equivalent to Monero in terms of privacy, but with faster block times and lower fees.

>  **Want to help build the future of private cryptocurrency?** Check out our **[Enhancement Roadmap](ENHANCEMENT_ROADMAP.md)** to see exciting features we're building, including **Desktop GUI**, **Mobile Apps**, **Hardware Wallet Support**, and more! We welcome contributors!

##  NU 6.1 Support - Ready for Network Upgrade 6.1!

**NozyWallet is fully updated and ready for Zcash Network Upgrade 6.1 (NU 6.1)!**

- ✅ **Protocol Version 170140** - Fully compatible with NU 6.1
- ✅ **Activation Height**: Block 3,146,400 (November 23, 2025)
- ✅ **Latest Libraries**: Using `zcash_protocol 0.6.2+` and `zcash_primitives 0.24.1+`
- ✅ **NU 6.1 Features**: ZIP 271, ZIP 1016 support

Check your NU 6.1 status:
```bash
nozy nu61
```

Or see it in your wallet status:
```bash
nozy status
```

##  Privacy Guarantees

- ✅ **Every transaction is private** - No transparent transactions possible
- ✅ **Untraceable** - Sender, receiver, and amount are all hidden
- ✅ **Fungible** - No blacklisted or tainted coins
- ✅ **Zero-knowledge proofs** - Cryptographically proven privacy
- ✅ **Privacy by default** - You cannot accidentally compromise your privacy

Just like Monero, but faster and more efficient.

## Features

###  Implemented Features

- **NU 6.1 Support**: Fully compatible with Zcash Network Upgrade 6.1 (protocol version 170140)
- **HD Wallet Support**: Hierarchical deterministic wallet with BIP39 mnemonic support
- **Password Protection**: Argon2-based password hashing for wallet security
- **Address Generation**: Generate Orchard addresses for receiving ZEC
- **Blockchain Integration**: Real-time connection to Zebra RPC node
- **Note Scanning**: Scan blockchain for incoming shielded notes
- **Transaction Building**: Build and broadcast Zcash transactions
- **Backup & Recovery**: Comprehensive wallet backup and recovery system
- **Error Handling**: User-friendly error messages and suggestions
- **CLI Interface**: Interactive command-line interface
- **Orchard Proving**: Complete proving parameters management system
- **Note Commitment Conversion**: Convert NoteCommitment to ExtractedNoteCommitment
- **Unified Address Parsing**: Extract Orchard addresses from unified addresses
- **Merkle Path Construction**: Convert authentication paths to MerkleHashOrchard arrays
- **Bundle Authorization Framework**: Complete transaction authorization framework

###  Upcoming Features & Roadmap

>  **Full Roadmap Available**: See **[ENHANCEMENT_ROADMAP.md](ENHANCEMENT_ROADMAP.md)** for complete development plans, priorities, and implementation details.

NozyWallet is actively being developed with exciting features on the horizon! We're building the most private and user-friendly Zcash wallet.

** High-Priority Features (Looking for Contributors!):**

-  **Desktop GUI Application** - Cross-platform desktop app with beautiful UI - **🔄 Migrating to Tauri** for better security & performance - **[See Migration Guide →](TAURI_MIGRATION_GUIDE.md)** | **[See Roadmap →](ENHANCEMENT_ROADMAP.md#15-desktop-gui-application)**
-  **Mobile Applications** - Native iOS and Android apps for on-the-go privacy - **[See Roadmap →](ENHANCEMENT_ROADMAP.md#16-mobile-applications)**
-  **Hardware Wallet Integration** - Ledger, Trezor, and other hardware wallet support - **[See Roadmap →](ENHANCEMENT_ROADMAP.md#8-enhanced-security-features)**
-  **Multi-Signature Support** - Enhanced security with multi-sig transactions - **[See Roadmap →](ENHANCEMENT_ROADMAP.md#8-enhanced-security-features)**

** Other Planned Features:**

-  **Multi-Account Management** - Manage multiple wallets from one interface - **[See Roadmap →](ENHANCEMENT_ROADMAP.md#18-multi-account-management)**
-  **Web Interface** - Browser-based wallet (API server ready!) - **[See Roadmap →](ENHANCEMENT_ROADMAP.md#17-web-interface)**
-  **Address Labeling** - Organize and label addresses - **[See Roadmap →](ENHANCEMENT_ROADMAP.md#11-multi-address-support)**
-  **Enhanced Transaction History** - Detailed tracking with export - **[See Roadmap →](ENHANCEMENT_ROADMAP.md#12-transaction-history)**
-  **Multi-Device Sync** - Sync wallet across devices - **[See Roadmap →](ENHANCEMENT_ROADMAP.md#14-backup-and-recovery)**
-  **Network Monitoring** - Real-time status and fee estimation - **[See Roadmap →](ENHANCEMENT_ROADMAP.md#13-network-monitoring)**

** Want to contribute?** We're actively looking for help with:
- Frontend/UI developers (Desktop GUI, Mobile, Web)
- Mobile developers (iOS/Android)
- UI/UX designers
- Security experts (Hardware wallets, Multi-sig)

Check out our [Contributing Guide](#-contributing) and the **[Enhancement Roadmap](ENHANCEMENT_ROADMAP.md)** to see where you can help!

##  Installation

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))
- Zebra RPC node running on `http://127.0.0.1:8232`

### Build from Source

```bash
git clone https://github.com/LEONINE-DAO/Nozy-wallet.git
cd Nozy-wallet
cargo build --release
```

##  Documentation

- **[Command Help Guide](COMMAND_HELP.md)** - Complete command reference with examples
- **[Send ZEC Guide](SEND_ZEC_GUIDE.md)** - Step-by-step sending guide
- **[API Documentation](API_DOCUMENTATION.md)** - Technical API reference

##  Quick Start

### 1. Create a New Wallet

```bash
cargo run --bin nozy new
```

**Example Output:**
```
 Creating new wallet...
 Wallet created successfully!

  **CRITICAL SECURITY WARNING: MNEMONIC BACKUP**

**Your mnemonic phrase is the ONLY way to recover your wallet. If you lose it, you will PERMANENTLY lose access to all your funds.**

**🔒 Security Best Practices:**
- ✅ **Write it down** on paper (never store digitally)
- ✅ **Store in a secure location** (fireproof safe, bank deposit box)
- ✅ **Never share it** with anyone (not even support staff)
- ✅ **Never take screenshots** or photos of your mnemonic
- ✅ **Never store it online** (cloud, email, notes apps)
- ✅ **Make multiple copies** and store in different secure locations
- ✅ **Verify the backup** by restoring from it (on testnet first)

**⚠️  If someone gets your mnemonic, they can steal ALL your funds. Treat it like cash.**

 Your 24-word mnemonic phrase:
nozy nozy nozy nozy nozy nozy nozy nozy nozy nozy nozy nozy nozy nozy nozy nozy
nozy nozy nozy nozy nozy nozy nozy  about
[24 words total - KEEP THIS SECURE!]

 Set password protection? (y/n): y
Enter password: ********
Confirm password: ********
 Password set successfully

 Generated Orchard address:
u1testaddress1234567890abcdefghijklmnopqrstuvwxyz...

 Wallet saved to: ~/.local/share/nozy/data/wallet.dat
```

This will:
- Generate a new HD wallet with BIP39 mnemonic
- Ask if you want password protection
- Create a sample Orchard address
- Save the wallet to the secure data directory

### 2. Restore from Mnemonic

```bash
cargo run --bin nozy restore
```

**Example Output:**
```
📝 Enter your 24-word 

mnemonic phrase:
> abandon abandon abandon ... [enter all 24 words]

 Enter wallet password (if set): ********
 Wallet restored successfully!
 Your addresses are ready to use.
```

Enter your 24-word mnemonic phrase to restore your wallet.

### 3. Generate Addresses

```bash
cargo run --bin nozy addresses --count 5
```

**Example Output:**
```
 Generating 5 Orchard addresses...

Address 0: u1testaddress1234567890abcdefghijklmnopqrstuvwxyz...
Address 1: u1testaddress2345678901bcdefghijklmnopqrstuvwxyzab...
Address 2: u1testaddress3456789012cdefghijklmnopqrstuvwxyzabc...
Address 3: u1testaddress4567890123defghijklmnopqrstuvwxyzabcd...
Address 4: u1testaddress5678901234efghijklmnopqrstuvwxyzabcde...

 Generated 5 addresses
```

Generate multiple Orchard addresses for receiving ZEC.

### 4. Scan for Notes

```bash
cargo run --bin nozy scan --start-height 1000000 --end-height 1000100
```

**Example Output:**
```
 Scanning blockchain for notes...
 Progress: [████████████████████] 100%

 Scan complete!
 Found 3 notes
 Total balance: 0.50000000 ZEC

Notes found:
  - Note 1: 0.20000000 ZEC (height 1000045)
  - Note 2: 0.20000000 ZEC (height 1000067)
  - Note 3: 0.10000000 ZEC (height 1000098)

 Notes saved to: ~/.local/share/nozy/data/notes.json
```

Scan the blockchain for incoming shielded notes.

### 5. Manage Proving Parameters

```bash
# Check proving status
cargo run --bin nozy proving --status

# Download placeholder parameters (for testing)
cargo run --bin nozy proving --download
```

**Example Output:**
```
🔧 Orchard Proving Parameters Management
=====================================

 Proving Status:
   Spend Parameters: ✅
   Output Parameters: ✅
   Spend Verifying Key: ✅
   Output Verifying Key: ✅
   Can Prove: ✅

 Orchard proving ready (Halo 2 - no external parameters required)
```

Manage Orchard proving parameters for transaction creation.

### 6. Send ZEC

```bash
cargo run --bin nozy send --recipient "u1..." --amount 0.1
```

**Complete Transaction Flow Example:**

```bash
$ cargo run --bin nozy send --recipient "u1testrecipient1234567890abcdefghijklmnopqrstuvwxyz" --amount 0.1

Transaction Summary
============================================================
  Recipient: u1testrecipient1234567890abcdefghijklmnopqrstuvwxyz
  Amount:    0.1 ZEC
  Network:   MAINNET ⚠️

 Estimating transaction fee...
  Fee:       0.00001 ZEC
  Total:     0.10001 ZEC (amount + fee)
  Proving:   ✅ Ready (Halo 2)

 Scanning for spendable notes...
 Progress: [████████████████████] 100%
  Available: 0.50000000 ZEC (from 3 notes)

============================================================
⚠️  WARNING: This will send REAL ZEC on MAINNET!
⚠️  This transaction cannot be undone!

Please confirm:
  1. The recipient address is correct
  2. The amount is correct
  3. You understand this will spend real ZEC

Type 'SEND' (all caps) to confirm, or anything else to cancel:
> SEND

Enter memo (optional, press Enter to skip): 
> Payment for services

 Building transaction...
 Privacy: ✅ Shielded transaction (Orchard)
 Transaction size: 2048 bytes

 Transaction built successfully!

 Broadcasting transaction...
 Transaction broadcast successfully!
 Network TXID: abc123def4567890123456789012345678901234567890123456789012345678
 Transaction saved to history - will track confirmations

 Remaining balance: 0.39999000 ZEC
```

**Mainnet Broadcasting Safety:**

NozyWallet requires explicit confirmation before broadcasting transactions on mainnet to prevent accidental loss of funds:

- **Testnet**: Type `yes` to confirm (safe for testing)
- **Mainnet**: Type `SEND` (all caps) to confirm (sends real ZEC)

**To enable mainnet broadcasting:**

1. **Set network to mainnet:**
   ```bash
   cargo run --bin nozy config --set-network mainnet
   ```

2. **When sending, confirm by typing `SEND`:**
   ```bash
   cargo run --bin nozy send --recipient "u1..." --amount 0.1
   # When prompted, type: SEND
   ```

**For testing (recommended):**

```bash
# Set network to testnet
cargo run --bin nozy config --set-network testnet

# Send on testnet (requires typing 'yes' to confirm)
cargo run --bin nozy send --recipient "u1..." --amount 0.1
```

**Check current network:**

```bash
cargo run --bin nozy config --show
```

## 💡 Real-World Usage Examples

### Complete Wallet Setup and Transaction Flow

Here's a complete example of setting up a wallet and sending a transaction:

```bash
# Step 1: Create a new wallet
$ cargo run --bin nozy new
# [Follow prompts to create wallet and set password]

# Step 2: Generate a receiving address
$ cargo run --bin nozy addresses --count 1
📍 Address 0: u1yourreceivingaddress1234567890abcdefghijklmnopqrstuvwxyz...

# Step 3: Share your address to receive ZEC
# Send the address above to someone who wants to send you ZEC

# Step 4: Scan for incoming notes (after receiving ZEC)
$ cargo run --bin nozy scan --start-height 2000000 --end-height 2100000
# This scans the blockchain for notes sent to your addresses

# Step 5: Check your balance
$ cargo run --bin nozy balance
💰 Balance: 1.50000000 ZEC
📝 Notes: 5 spendable notes

# Step 6: Send ZEC to someone
$ cargo run --bin nozy send --recipient "u1recipientaddress..." --amount 0.5
# [Follow prompts to confirm transaction]

# Step 7: Check transaction history
$ cargo run --bin nozy history
📜 Transaction History:
  TXID: abc123... | Amount: 0.5 ZEC | Status: ✅ Confirmed (3 confirmations)
```

### Automated Transaction Script

For programmatic usage, see the [API Documentation](API_DOCUMENTATION.md) for complete Rust examples:

```rust
use nozy::{HDWallet, ZebraClient, OrchardTransactionBuilder, NoteScanner};
use nozy::WalletStorage;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> nozy::NozyResult<()> {
    // 1. Load wallet
    let storage = WalletStorage::with_xdg_dir();
    let wallet = storage.load_wallet("your_password").await?;
    
    // 2. Connect to Zebra
    let zebra_client = ZebraClient::new("http://127.0.0.1:8232".to_string());
    
    // 3. Scan for notes
    let mut scanner = NoteScanner::new(wallet, zebra_client.clone());
    let tip_height = zebra_client.get_block_count().await?;
    let start_height = tip_height.saturating_sub(10_000);
    let (result, spendable_notes) = scanner.scan_notes(
        Some(start_height),
        Some(tip_height)
    ).await?;
    
    println!("Found {} ZEC in {} notes", 
        result.total_balance as f64 / 100_000_000.0,
        spendable_notes.len()
    );
    
    // 4. Build and send transaction
    let mut builder = OrchardTransactionBuilder::new_async(true).await?;
    let transaction = builder.build_single_spend(
        &zebra_client,
        &spendable_notes,
        "u1recipientaddress...",
        50_000_000, // 0.5 ZEC in zatoshis
        10_000,     // fee in zatoshis
        Some(b"Payment from automated script"),
    ).await?;
    
    // 5. Broadcast transaction
    let txid = builder.broadcast_transaction(&zebra_client, &transaction).await?;
    println!("Transaction sent! TXID: {}", txid);
    
    Ok(())
}
```

For more examples, see **[API_DOCUMENTATION.md](API_DOCUMENTATION.md)**.

### CLI Output Examples

**Example: Wallet Status**
```
$ cargo run --bin nozy status

🔐 NozyWallet Status
============================================================
  Network:        MAINNET
  Backend:        Zebra
  Zebra URL:      http://127.0.0.1:8232
  Connection:     ✅ Connected
  Block Height:   3,146,400
  Wallet:         ✅ Loaded
  Balance:        1.50000000 ZEC
  Notes:          5 spendable notes
  Proving:        ✅ Ready
```

**Example: Transaction History**
```
$ cargo run --bin nozy history

📜 Transaction History
============================================================
  TXID: abc123def456... | To: u1recipient... | Amount: 0.5 ZEC
    Status: ✅ Confirmed (3 confirmations)
    Fee: 0.00001 ZEC
    Memo: Payment for services
    Time: 2025-01-15 14:30:00 UTC

  TXID: def456abc123... | To: u1another... | Amount: 0.2 ZEC
    Status: ⏳ Pending (0 confirmations)
    Fee: 0.00001 ZEC
    Time: 2025-01-15 15:00:00 UTC
```

### Demo Video

**Watch a complete walkthrough:**
- Creating a wallet
- Generating addresses
- Scanning for notes
- Sending a transaction
- Checking transaction history

For video tutorials and demonstrations, check our [GitHub Discussions](https://github.com/LEONINE-DAO/Nozy-wallet/discussions) or [Discord](https://discord.gg/XMmFGvcQ89).

## 🔧 Configuration

### Network Settings

Configure whether to use mainnet (real ZEC) or testnet (test ZEC):

```bash
# Set to mainnet (default)
cargo run --bin nozy config --set-network mainnet

# Set to testnet (for testing)
cargo run --bin nozy config --set-network testnet

# Show current configuration
cargo run --bin nozy config --show
```

**Important Notes:**
- **Mainnet**: Requires typing `SEND` (all caps) to confirm transactions (sends real ZEC)
- **Testnet**: Requires typing `yes` to confirm transactions (safe for testing)
- Default network is `mainnet` - mainnet transactions require explicit confirmation to prevent accidental sends
- Always verify the network before sending transactions

### Zebra Node Setup

1. Install and run Zebra:
```bash
# Install Zebra
cargo install zebrad

# Run Zebra with RPC enabled
zebrad start --rpc.bind-addr 127.0.0.1:8232
```

2. Test connection:
```bash
cargo run --bin nozy test-zebra
```

### Proving Parameters Setup

**Important:** Orchard uses Halo 2 proving system which **does NOT require external proving parameters**. The proving system is built into the Orchard library and works automatically.

**No setup required!** Your wallet is ready for shielded transactions immediately.

**Verify Proving Status:**
   ```bash
   cargo run --bin nozy proving --status
   ```

**Expected Output:**
```
✅ Orchard proving ready (Halo 2 - no external parameters required)
```

**Note:** The `proving --download` command is kept for API compatibility but performs no downloads (Halo 2 doesn't need them). It will display an informational message explaining that no downloads are needed.

**Historical Note:** Older Zcash protocols (Sapling, Sprout) required external proving parameters, but Orchard with Halo 2 has these built-in and requires no external files.

### Configuration File

NozyWallet stores configuration in a JSON file. The location depends on your OS:

- **Linux**: `~/.config/nozy/config.json` (or `$XDG_CONFIG_HOME/nozy/config.json`)
- **macOS**: `~/Library/Application Support/com.nozy.nozy/config/config.json`
- **Windows**: `%APPDATA%\nozy\config\config.json`

**Configuration Options:**

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `zebra_url` | String | `http://127.0.0.1:8232` | Zebra RPC node URL |
| `crosslink_url` | String | `""` (empty) | Crosslink backend URL (falls back to `zebra_url` if empty) |
| `network` | String | `mainnet` | Network: `mainnet` or `testnet` |
| `last_scan_height` | Number (optional) | `null` | Last blockchain height scanned |
| `theme` | String | `dark` | UI theme: `dark` or `light` |
| `backend` | String | `zebra` | Backend type: `zebra` or `crosslink` |

**Example `config.json`:**

```json
{
  "zebra_url": "http://127.0.0.1:8232",
  "crosslink_url": "",
  "network": "mainnet",
  "last_scan_height": 2000000,
  "theme": "dark",
  "backend": "zebra"
}
```

**Modify configuration via CLI:**

```bash
# Set Zebra RPC URL
cargo run --bin nozy config --set-zebra-url http://127.0.0.1:8232

# Set network
cargo run --bin nozy config --set-network mainnet

# Show current configuration
cargo run --bin nozy config --show
```

### Data Directories

NozyWallet stores wallet data and configuration in platform-specific directories:

**Wallet Data Directory** (stores `wallet.dat`, `notes.json`, transaction history, etc.):

- **Linux**: `~/.local/share/nozy/data` (or `$XDG_DATA_HOME/nozy/data`)
- **macOS**: `~/Library/Application Support/com.nozy.nozy/data`
- **Windows**: `%APPDATA%\nozy\data`

**Fallback location** (if XDG directories unavailable):
- `~/.nozy/data` (Linux/macOS) or `%USERPROFILE%\.nozy\data` (Windows)

**Configuration Directory** (stores `config.json`):

- **Linux**: `~/.config/nozy` (or `$XDG_CONFIG_HOME/nozy`)
- **macOS**: `~/Library/Application Support/com.nozy.nozy/config`
- **Windows**: `%APPDATA%\nozy\config`

**Fallback location** (if XDG directories unavailable):
- `~/.nozy/config` (Linux/macOS) or `%USERPROFILE%\.nozy\config` (Windows)

**Important Files:**

- `wallet.dat` - Encrypted wallet file (in data directory)
- `config.json` - Configuration file (in config directory)
- `notes.json` - Scanned notes cache (in data directory)
- `orchard_params/` - Proving parameters directory (in project root)

### Environment Variables

#### CLI Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `ZEBRA_RPC_URL` | `http://127.0.0.1:8232` | Override default Zebra RPC URL |
| `RUST_LOG` | (unset) | Set log level (e.g., `debug`, `info`, `warn`, `error`) |
| `HOME` | (system) | Home directory (used for fallback data paths on Unix) |
| `USERPROFILE` | (system) | User profile directory (used for fallback data paths on Windows) |
| `XDG_CONFIG_HOME` | `~/.config` | XDG config directory (Linux) |
| `XDG_DATA_HOME` | `~/.local/share` | XDG data directory (Linux) |

**Examples:**

```bash
# Enable debug logging
export RUST_LOG=debug
cargo run --bin nozy send --recipient "u1..." --amount 0.1

# Override Zebra RPC URL
export ZEBRA_RPC_URL=http://192.168.1.100:8232
cargo run --bin nozy sync

# Set custom data directory (via HOME)
export HOME=/custom/path
cargo run --bin nozy new
```

#### API Server Environment Variables

The API server (`api-server`) supports additional environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `NOZY_API_KEY` | (unset) | API key for authentication (required in production) |
| `NOZY_RATE_LIMIT_REQUESTS` | `100` | Maximum requests per time window |
| `NOZY_RATE_LIMIT_WINDOW` | `60` | Rate limit time window in seconds |
| `NOZY_PRODUCTION` | (unset) | Set to enable production mode (stricter security) |
| `NOZY_HTTP_PORT` | `3000` | HTTP server port |
| `NOZY_HTTPS_ENABLED` | (unset) | Enable HTTPS (requires SSL certificates) |
| `NOZY_HTTPS_PORT` | `443` | HTTPS server port |
| `NOZY_CORS_ORIGINS` | (unset) | Comma-separated list of allowed CORS origins |
| `NOZY_SSL_CERT_PATH` | (unset) | Path to SSL certificate file (required for HTTPS) |
| `NOZY_SSL_KEY_PATH` | (unset) | Path to SSL private key file (required for HTTPS) |

**API Server Example:**

```bash
# Start API server with authentication
export NOZY_API_KEY=your-secret-api-key
export NOZY_PRODUCTION=1
export NOZY_HTTP_PORT=8080
cd api-server
cargo run

# Enable HTTPS
export NOZY_HTTPS_ENABLED=1
export NOZY_SSL_CERT_PATH=/path/to/cert.pem
export NOZY_SSL_KEY_PATH=/path/to/key.pem
cargo run
```

## 📚 API Documentation

For comprehensive API documentation, see **[API_DOCUMENTATION.md](API_DOCUMENTATION.md)**.

### Quick API Reference

#### Core Types

```rust
use nozy::{HDWallet, ZebraClient, NozyResult, NozyError};

// Create a new wallet
let wallet = HDWallet::new()?;

// Set password protection
wallet.set_password("my_secure_password")?;

// Generate an address
let address = wallet.generate_orchard_address(0, 0)?;

// Connect to Zebra
let client = ZebraClient::new("http://127.0.0.1:8232".to_string());

// Scan for notes
let notes = client.scan_notes(1000000, 1000100).await?;
```

#### Complete Transaction Flow

```rust
use nozy::{
    HDWallet, ZebraClient, OrchardTransactionBuilder, 
    NoteScanner, WalletStorage
};

#[tokio::main]
async fn main() -> nozy::NozyResult<()> {
    // 1. Load wallet
    let storage = WalletStorage::with_xdg_dir();
    let wallet = storage.load_wallet("password").await?;
    
    // 2. Connect to Zebra
    let zebra_client = ZebraClient::new("http://127.0.0.1:8232".to_string());
    
    // 3. Get current block height
    let tip_height = zebra_client.get_block_count().await?;
    println!("Current block height: {}", tip_height);
    
    // 4. Scan for spendable notes
    let mut scanner = NoteScanner::new(wallet, zebra_client.clone());
    let start_height = tip_height.saturating_sub(10_000);
    let (result, spendable_notes) = scanner.scan_notes(
        Some(start_height),
        Some(tip_height)
    ).await?;
    
    println!("Found {} notes with total balance: {:.8} ZEC",
        result.notes.len(),
        result.total_balance as f64 / 100_000_000.0
    );
    
    // 5. Check if we have enough funds
    let amount_zatoshis = 50_000_000; // 0.5 ZEC
    let fee_zatoshis = 10_000; // 0.0001 ZEC
    let total_needed = amount_zatoshis + fee_zatoshis;
    
    if result.total_balance < total_needed {
        return Err(nozy::NozyError::InsufficientFunds(
            format!("Need {} zatoshis, have {}", 
                total_needed, result.total_balance)
        ));
    }
    
    // 6. Build transaction
    let mut builder = OrchardTransactionBuilder::new_async(true).await?;
    let transaction = builder.build_single_spend(
        &zebra_client,
        &spendable_notes,
        "u1recipientaddress...", // recipient
        amount_zatoshis,
        fee_zatoshis,
        Some(b"Payment memo"), // optional memo
    ).await?;
    
    println!("Transaction built: {} bytes", transaction.raw_transaction.len());
    println!("Transaction ID: {}", transaction.txid);
    
    // 7. Broadcast transaction (if enabled)
    // Note: In production, this requires explicit confirmation
    // let txid = builder.broadcast_transaction(&zebra_client, &transaction).await?;
    // println!("Transaction broadcast! TXID: {}", txid);
    
    Ok(())
}
```

#### Error Handling

```rust
use nozy::{NozyError, NozyResult};

match result {
    Ok(value) => println!("Success: {:?}", value),
    Err(NozyError::NetworkError(msg)) => {
        println!("Network error: {}", msg);
        println!("💡 Try checking your Zebra connection");
    },
    Err(NozyError::AddressParsing(msg)) => {
        println!("Address error: {}", msg);
        println!("💡 Verify the address format (should start with 'u1')");
    },
    Err(NozyError::InsufficientFunds(msg)) => {
        println!("Insufficient funds: {}", msg);
        println!("💡 Run 'sync' to update your balance");
    },
    Err(e) => println!("Error: {}", e),
}
```

#### Note Management

```rust
use nozy::{NoteScanner, SpendableNote};

// Scan for notes in a range
let mut scanner = NoteScanner::new(wallet, zebra_client);
let (result, spendable_notes) = scanner.scan_notes(
    Some(2_000_000), // start height
    Some(2_100_000), // end height
).await?;

// Process found notes
for note in &spendable_notes {
    let value = note.orchard_note.note.value().inner();
    println!("Note value: {:.8} ZEC", value as f64 / 100_000_000.0);
    println!("Nullifier: {}", hex::encode(note.orchard_note.nullifier.to_bytes()));
}
```

#### Wallet Storage

```rust
use nozy::WalletStorage;
use std::path::PathBuf;

// Save wallet
let storage = WalletStorage::with_xdg_dir();
storage.save_wallet(&wallet, "password").await?;

// Load wallet
let loaded_wallet = storage.load_wallet("password").await?;

// Create backup
storage.create_backup("backups").await?;
```

### More Examples

For additional examples and detailed API documentation, see:

- **[API_DOCUMENTATION.md](API_DOCUMENTATION.md)** - Complete API reference with examples
- **[SEND_ZEC_GUIDE.md](SEND_ZEC_GUIDE.md)** - Step-by-step sending guide
- **[COMMAND_HELP.md](COMMAND_HELP.md)** - Complete command reference

##  Performance

NozyWallet includes performance optimizations for faster note scanning:

### Parallel Block Fetching

By default, NozyWallet fetches **5 blocks in parallel** during scanning, providing up to **5x speedup** for large scans.

**Configure Parallelism:**
```rust
let mut scanner = NoteScanner::new(wallet, zebra_client);
scanner.set_parallel_blocks(10); // Fetch 10 blocks in parallel (1-50 range)
scanner.enable_block_cache();    // Cache blocks for 1 hour
```

**Performance Impact:**
- Sequential: ~100ms per block = 100 seconds for 1000 blocks
- Parallel (5 blocks): ~20 seconds for 1000 blocks (**5x faster**)
- Parallel (10 blocks): ~12 seconds for 1000 blocks (**8x faster**)

### Block Caching

Blocks are cached for 1 hour to avoid redundant fetches:
- Faster rescans of the same blocks
- Reduced network traffic
- Lower load on Zebra node

**Enable Caching:**
```rust
scanner.enable_block_cache();
```

### Incremental Scanning

Use incremental scanning to only process new blocks:
```rust
let scanner = NoteScanner::with_index_file(wallet, client, &index_path)?;
// Only scans blocks after last_scanned_height
scanner.scan_notes(Some(last_height + 1), None).await?;
```

**📖 For detailed performance documentation, see [PERFORMANCE.md](PERFORMANCE.md)**

##  Testing

NozyWallet includes comprehensive unit tests, integration tests, and end-to-end tests.

### Quick Start

```bash
# Run all unit tests (fast, no external dependencies)
cargo test

# Run integration tests (requires Zebra node)
cargo test --test integration_tests -- --ignored

# Run specific test
cargo test test_wallet_creation

# Run with output
cargo test -- --nocapture
```

### Test Setup

**For Unit Tests:** No setup required - run immediately!

**For Integration Tests:**
1. Install and run Zebra:
```bash
   cargo install zebrad
   zebrad start --network testnet --rpc-listen-addr 127.0.0.1:8232
   ```

2. Set environment variables (optional):
   ```bash
   export ZEBRA_RPC_URL=http://127.0.0.1:8232
   export NOZY_TEST_NETWORK=testnet
   ```

3. Run integration tests:
```bash
   cargo test --test integration_tests -- --ignored
   ```

### Test Coverage

- ✅ **Unit Tests** - Wallet creation, address generation, password protection, error handling
- ✅ **Integration Tests** - Zebra connection, note scanning, transaction building
- ✅ **End-to-End Tests** - Complete workflows (create → scan → send)

### CI/CD

All tests run automatically in CI on every push and pull request:
- **Format Check** - `cargo fmt --check`
- **Linting** - `cargo clippy -- -D warnings`
- **Security Audit** - `cargo audit` (scans for vulnerabilities)
- **Unit Tests** - `cargo test`
- **Build Verification** - `cargo build --release`

**📖 For detailed testing documentation, see [TESTING.md](TESTING.md)**  
**📖 For dependency management and security auditing, see [DEPENDENCY_MANAGEMENT.md](DEPENDENCY_MANAGEMENT.md)**

## 🔒 Security

### Security Audits

**Self-Audit:** ✅ Completed (December 2025)  
**Third-Party Audit:** 🎯 Planned for Production (Q1-Q2 2026)

NozyWallet has completed a comprehensive self-security audit. **A professional third-party security audit is planned before production release** to ensure the highest security standards.

**Audit Status:**
-  Self-audit completed - See [SELF_AUDIT_RESULTS.md](SELF_AUDIT_RESULTS.md)
-  Third-party audit planned - See [SECURITY.md](SECURITY.md) for details
-  Considering: Least Authority, Trail of Bits, or Zcash Foundation recommended auditors
-  Exploring grant funding via Zcash Foundation grants

** For complete security information, see [SECURITY.md](SECURITY.md)**  
** For key management details, see [KEY_MANAGEMENT.md](KEY_MANAGEMENT.md)**

### Password Protection

- Uses Argon2 for password hashing
- Salt is randomly generated for each wallet
- Passwords are never stored in plain text

### Wallet Storage

- Wallets are encrypted with AES-256-GCM
- Encryption key is derived from password
- Backup files are also encrypted

### Private Key Management

- Private keys are never stored in plain text
- Keys are derived from mnemonic using BIP32
- Spending keys are only loaded when needed
- **Memory Security**: Private keys are automatically zeroized after use using the `zeroize` crate
- **Secure Key Handling**: Sensitive data is cleared from memory to prevent memory dumps

## 📁 Project Structure

```
src/
├── main.rs              # CLI application
├── lib.rs               # Library exports
├── error.rs             # Error types and handling
├── hd_wallet.rs         # HD wallet implementation
├── notes.rs             # Note scanning and management
├── storage.rs           # Wallet persistence
├── zebra_integration.rs # Zebra RPC client
├── orchard_tx.rs        # Orchard transaction building
├── proving.rs           # Orchard proving parameters management
├── transaction_builder.rs # Transaction orchestration
└── tests.rs             # Test suite
```

## 📦 Dependency Management

NozyWallet uses modern Rust dependency management with security-focused practices.

### Key Dependencies

**Core Libraries:**
- **clap 4.0** - CLI argument parsing with derive macros
- **tokio 1.0** - Async runtime for Zebra RPC client
- **reqwest 0.12** - HTTP client with SOCKS proxy support (Tor)
- **thiserror 1.0** - Error handling with derive macros
- **anyhow 1.0** - Error handling (in API server)

**Cryptography:**
- **aes-gcm 0.10** - Wallet encryption
- **argon2 0.5** - Password hashing
- **orchard 0.11.0** - Zcash Orchard protocol
- **zcash_primitives 0.24.1** - Zcash cryptographic primitives
- **zcash_protocol 0.6.2** - Zcash protocol support (NU 6.1)

**Zcash Libraries:**
- **orchard 0.11.0** - Orchard shielded transactions
- **zcash_primitives 0.24.1** - Core Zcash primitives
- **zcash_protocol 0.6.2** - Protocol version 170140 (NU 6.1)
- **zcash_address 0.9.0** - Address parsing and encoding

### Security Practices

**Version Pinning:**
- Critical security dependencies are pinned to specific versions
- Security patches are applied via version overrides when needed
- Example: `tracing-subscriber = "0.3.20"` to fix RUSTSEC-2025-0055

**Regular Updates:**
```bash
# Check for outdated dependencies
cargo install cargo-outdated
cargo outdated

# Run security audit
cargo install cargo-audit
cargo audit

# Update dependencies (review changes first!)
cargo update
```

**CI Checks:**
- `Cargo.lock` is verified in CI to ensure reproducible builds
- Security audits run automatically on every PR
- Outdated dependency checks run weekly
- License compliance checks ensure MIT-compatible dependencies

### Dependency Update Workflow

1. **Check for updates:**
   ```bash
   cargo outdated
   ```

2. **Review security advisories:**
   ```bash
   cargo audit
   ```

3. **Update dependencies:**
   ```bash
   # Update specific crate
   cargo update -p crate-name
   
   # Update all dependencies
   cargo update
   ```

4. **Test after updates:**
   ```bash
   cargo test
   cargo build --release
   ```

5. **Commit Cargo.lock:**
   - Always commit `Cargo.lock` to ensure reproducible builds
   - CI will verify `Cargo.lock` is up to date

### Best Practices

- ✅ **Pin security-critical dependencies** - Use exact versions for crypto libraries
- ✅ **Regular security audits** - Run `cargo audit` weekly
- ✅ **Keep Cargo.lock committed** - Ensures reproducible builds
- ✅ **Review dependency updates** - Test thoroughly before merging
- ✅ **Use minimal feature flags** - Only enable needed features (reduces attack surface)
- ✅ **Monitor security advisories** - Subscribe to RustSec advisories

See [Cargo.toml](Cargo.toml) for complete dependency list.

** For detailed dependency management practices, see [DEPENDENCY_MANAGEMENT.md](DEPENDENCY_MANAGEMENT.md).**

##  Contributing

We welcome contributions! NozyWallet is an open-source project, and there are many ways to help:

###  How to Contribute

1. **Fork the repository**
2. **Check the [Enhancement Roadmap](ENHANCEMENT_ROADMAP.md)** - See what features are planned and where help is needed
3. **Create a feature branch** (`git checkout -b feature/amazing-feature`)
4. **Make your changes** - Follow Rust best practices and add tests
5. **Commit your changes** (`git commit -m 'Add amazing feature'`)
6. **Push to the branch** (`git push origin feature/amazing-feature`)
7. **Open a Pull Request** - Describe your changes and reference any related issues

###  Areas Where We Need Help

Based on our [Enhancement Roadmap](ENHANCEMENT_ROADMAP.md), we're particularly looking for contributions in:

** High Priority (Active Development):**

- **🖥️ Desktop GUI Development** - Build cross-platform desktop app with Tauri or GTK
  - See: [Priority 6: Desktop GUI Application](ENHANCEMENT_ROADMAP.md#15-desktop-gui-application)
  - Skills: Frontend (React/Vue), Rust, UI/UX design
  
- **📱 Mobile App Development** - Native iOS and Android applications
  - See: [Priority 6: Mobile Applications](ENHANCEMENT_ROADMAP.md#16-mobile-applications)
  - Skills: React Native, Flutter, Swift, Kotlin, Mobile UI/UX

- **🔐 Security Enhancements** - Hardware wallet support, multi-signature
  - See: [Priority 4: Enhanced Security Features](ENHANCEMENT_ROADMAP.md#8-enhanced-security-features)
  - Skills: Cryptography, Hardware integration, Security auditing

- **🌐 Web Interface** - Frontend for existing API server
  - See: [Priority 6: Web Interface](ENHANCEMENT_ROADMAP.md#17-web-interface)
  - Skills: Frontend frameworks, API integration, Web UI/UX

**Always Welcome:**

- **📊 UI/UX Design** - Design beautiful, privacy-focused interfaces for all platforms
- **🧪 Testing** - Write tests, improve test coverage, integration testing
- **📚 Documentation** - Improve docs, write tutorials, create examples
- **🐛 Bug Fixes** - Help fix issues and improve stability
- **⚡ Performance** - Optimize scanning, transaction building, and operations
- **🔧 Core Features** - Help with blockchain integration, transaction processing (see [Priorities 1-3](ENHANCEMENT_ROADMAP.md))

### Development Setup

```bash
# Clone and build
git clone https://github.com/LEONINE-DAO/Nozy-wallet.git
cd Nozy-wallet
cargo build

# Run tests
cargo test

# Run with debug output
RUST_LOG=debug cargo run --bin nozy new
```

### Getting Started

1. ** Read the [Enhancement Roadmap](ENHANCEMENT_ROADMAP.md)** - Understand priorities and see what needs help
   - **Priority 6** (GUI, Mobile, Web) - Great for frontend/mobile developers!
   - **Priority 4** (Security) - Perfect for security experts!
   - **Priorities 1-3** (Core) - For blockchain/Rust developers!

2. ** Check open issues** on GitHub - Look for "good first issue" or "help wanted" labels

3. ** Join discussions** - [GitHub Discussions](https://github.com/LEONINE-DAO/Nozy-wallet/discussions) or [Discord](https://discord.gg/XMmFGvcQ89)

4. ** Start small** - Fix a bug, improve documentation, or add a test

5. ** Ask questions** - We're happy to help you get started!

** Full Details**: See [ENHANCEMENT_ROADMAP.md](ENHANCEMENT_ROADMAP.md) for:
- Complete feature specifications
- Implementation strategies
- Technology recommendations
- Contributor skill requirements
- Phase-by-phase development plans

##  License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## ⚠️ Disclaimer

This software is provided "as is" without warranty of any kind. Use at your own risk. Always verify transactions before broadcasting to mainnet.

## Support

- **Issues**: [GitHub Issues](https://github.com/LEONINE-DAO/Nozy-wallet/issues)
- **Discussions**: [GitHub Discussions](https://discord.gg/XMmFGvcQ89)
- **Documentation**: [Wiki](https://github.com/LEONINE-DAO/Nozy-wallet/wiki)
- **Roadmap**: [Enhancement Roadmap](ENHANCEMENT_ROADMAP.md) - See what's coming next!

## Acknowledgments

- [Zcash Foundation](https://zfnd.org/) for the Zcash protocol
- [Zebra](https://github.com/ZcashFoundation/zebra) for the RPC node
- [Orchard](https://github.com/zcash/orchard) for shielded transaction support

- [Rust](https://rust-lang.org/) for the amazing language and ecosystem




