# Configuration

## Zebra Node Setup

NozyWallet requires a Zebra RPC node to interact with the Zcash blockchain.

### 1. Install Zebra

```bash
# Install Zebra
cargo install zebrad
```

### 2. Run Zebra with RPC Enabled

```bash
zebrad start --rpc.bind-addr 127.0.0.1:8232
```

### 3. Test Connection

Verify that NozyWallet can connect to your Zebra node:

```bash
cargo run --bin nozy test-zebra
```

## Proving Parameters Setup

Orchard transactions require proving parameters for creating zero-knowledge proofs.

### For Testing: Use Placeholder Parameters

```bash
cargo run --bin nozy proving --download
```

This downloads placeholder parameters suitable for testing.

### For Production: Download Real Parameters

1. Download from: https://download.z.cash/downloads/
2. Place in `orchard_params/` directory:
   - `orchard-spend.params`
   - `orchard-output.params`
   - `orchard-spend-verifying.key`
   - `orchard-output-verifying.key`

### Verify Setup

Check that your proving parameters are correctly configured:

```bash
cargo run --bin nozy proving --status
```

## Environment Variables

You can override default settings using environment variables:

- `ZEBRA_RPC_URL`: Override default Zebra RPC URL (default: `http://127.0.0.1:8232`)

Example:

```bash
export ZEBRA_RPC_URL=http://192.168.1.100:8232
cargo run --bin nozy scan --start-height 1000000 --end-height 1000100
```

## Next Steps

- [Quick Start Guide](quick-start.md) - Create your first wallet
- [User Guides](../user-guides/creating-wallet.md) - Learn how to use NozyWallet
