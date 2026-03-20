# Core Types

## HDWallet

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

## ZebraClient

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

## OrchardTransactionBuilder

Builder for creating Orchard transactions with proving support.

```rust
use nozy::orchard_tx::OrchardTransactionBuilder;

// Create builder with proving parameters
let mut builder = OrchardTransactionBuilder::new_async(true).await?;

// Check proving status
let status = builder.get_proving_status();

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

## OrchardProvingManager

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

// Get proving parameters
let spend_params = manager.get_proving_params("spend")?;
let output_params = manager.get_proving_params("output")?;
```
