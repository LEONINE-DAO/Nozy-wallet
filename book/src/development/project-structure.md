# Project Structure

## Directory Layout

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

## Key Components

### Core Wallet

- **`hd_wallet.rs`**: Hierarchical deterministic wallet with BIP39 mnemonic support
- **`storage.rs`**: Encrypted wallet persistence
- **`crypto.rs`**: Cryptographic operations (password hashing, encryption)

### Blockchain Integration

- **`zebra_integration.rs`**: Zebra RPC client
- **`zebra_client.rs`**: Low-level RPC communication
- **`notes.rs`**: Note scanning and management

### Transaction Building

- **`orchard_tx.rs`**: Orchard transaction building
- **`transaction_builder.rs`**: Transaction orchestration
- **`transaction_signer.rs`**: Transaction signing
- **`proving.rs`**: Orchard proving parameters management

### Utilities

- **`error.rs`**: Error types and user-friendly error messages
- **`config.rs`**: Configuration management
- **`paths.rs`**: Path utilities

## Binaries

Located in `src/bin/`:

- **`nozy`**: Main CLI application
- **`send_zec.rs`**: Quick send utility
- **`test_rpc.rs`**: RPC testing utility
- **`check_wallet_password.rs`**: Password verification utility
