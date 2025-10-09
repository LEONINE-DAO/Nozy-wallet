# NozyWallet - Privacy-Focused Zcash Wallet

A modern, privacy-focused Zcash wallet built in Rust with support for Orchard shielded transactions.

## 🚀 Features

### ✅ Implemented Features

- **HD Wallet Support**: Hierarchical deterministic wallet with BIP39 mnemonic support
- **Password Protection**: Argon2-based password hashing for wallet security
- **Address Generation**: Generate Orchard addresses for receiving ZEC
- **Blockchain Integration**: Real-time connection to Zebra RPC node
- **Note Scanning**: Scan blockchain for incoming shielded notes
- **Transaction Building**: Build and broadcast Zcash transactions
- **Backup & Recovery**: Comprehensive wallet backup and recovery system
- **Error Handling**: User-friendly error messages and suggestions
- **CLI Interface**: Interactive command-line interface

### 🔄 In Development

- **Note Commitment Conversion**: Convert NoteCommitment to ExtractedNoteCommitment
- **Unified Address Parsing**: Extract Orchard addresses from unified addresses
- **Merkle Path Construction**: Convert authentication paths to MerkleHashOrchard arrays
- **Bundle Authorization**: Complete transaction authorization and signing

## 📦 Installation

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))
- Zebra RPC node running on `http://127.0.0.1:8232`

### Build from Source

```bash
git clone https://github.com/yourusername/nozywallet.git
cd nozywallet
cargo build --release
```

## 🚀 Quick Start

### 1. Create a New Wallet

```bash
cargo run --bin nozy new
```

This will:
- Generate a new HD wallet with BIP39 mnemonic
- Ask if you want password protection
- Create a sample Orchard address
- Save the wallet to `wallet_data/wallet.dat`

### 2. Restore from Mnemonic

```bash
cargo run --bin nozy restore
```

Enter your 24-word mnemonic phrase to restore your wallet.

### 3. Generate Addresses

```bash
cargo run --bin nozy addresses --count 5
```

Generate multiple Orchard addresses for receiving ZEC.

### 4. Scan for Notes

```bash
cargo run --bin nozy scan --start-height 1000000 --end-height 1000100
```

Scan the blockchain for incoming shielded notes.

### 5. Send ZEC

```bash
cargo run --bin nozy send --recipient "u1..." --amount 0.1
```

Send ZEC to a recipient address (mainnet broadcasting disabled by default for safety).

## 🔧 Configuration

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

### Environment Variables

- `ZEBRA_RPC_URL`: Override default Zebra RPC URL (default: `http://127.0.0.1:8232`)

## 📚 API Documentation

### Core Types

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

### Error Handling

```rust
use nozy::{NozyError, NozyResult};

match result {
    Ok(value) => println!("Success: {:?}", value),
    Err(NozyError::NetworkError(_)) => {
        println!("Network error: {}", error.user_friendly_message());
    },
    Err(NozyError::AddressParsing(_)) => {
        println!("Address error: {}", error.user_friendly_message());
    },
    Err(e) => println!("Other error: {}", e),
}
```

## 🧪 Testing

### Run All Tests

```bash
cargo test
```

### Run Integration Tests

```bash
cargo test -- --ignored
```

### Run Performance Tests

```bash
cargo test performance_tests
```

## 🔒 Security

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
├── transaction_builder.rs # Transaction orchestration
└── tests.rs             # Test suite
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development Setup

```bash
# Clone and build
git clone https://github.com/yourusername/nozywallet.git
cd nozywallet
cargo build

# Run tests
cargo test

# Run with debug output
RUST_LOG=debug cargo run --bin nozy new
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## ⚠️ Disclaimer

This software is provided "as is" without warranty of any kind. Use at your own risk. Always verify transactions before broadcasting to mainnet.

## 🆘 Support

- **Issues**: [GitHub Issues](https://github.com/yourusername/nozywallet/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/nozywallet/discussions)
- **Documentation**: [Wiki](https://github.com/yourusername/nozywallet/wiki)

## 🙏 Acknowledgments

- [Zcash Foundation](https://zfnd.org/) for the Zcash protocol
- [Zebra](https://github.com/ZcashFoundation/zebra) for the RPC node
- [Orchard](https://github.com/zcash/orchard) for shielded transaction support
- [Rust](https://rust-lang.org/) for the amazing language and ecosystem