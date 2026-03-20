# Command Reference

Complete reference for all NozyWallet CLI commands.

## Basic Commands

### `nozy new`
Create a new wallet with a fresh mnemonic phrase.

```bash
cargo run --bin nozy new
```

Generates a new 24-word mnemonic phrase and creates an encrypted wallet file.

### `nozy restore`
Restore an existing wallet from a mnemonic phrase.

```bash
cargo run --bin nozy restore
```

Prompts for your 24-word mnemonic phrase and creates a new wallet file.

### `nozy info`
Display wallet information.

```bash
cargo run --bin nozy info
```

Shows wallet mnemonic phrase and confirmation that wallet loaded successfully.

## Address Management

### `nozy addresses`
Generate new receiving addresses.

```bash
# Generate 1 address (default)
cargo run --bin nozy addresses

# Generate multiple addresses
cargo run --bin nozy addresses --count 5
```

## Transaction Commands

### `nozy scan`
Scan the blockchain for notes (received ZEC).

```bash
# Scan recent blocks
cargo run --bin nozy scan

# Scan from specific height
cargo run --bin nozy scan --start-height 2800000

# Scan specific range
cargo run --bin nozy scan --start-height 2800000 --end-height 2900000
```

### `nozy send`
Send ZEC to another address.

```bash
cargo run --bin nozy send --recipient <ADDRESS> --amount <AMOUNT>
```

**Note:** Amount is in zatoshis (1 ZEC = 100,000,000 zatoshis).

### `nozy list-notes`
Display all stored notes and their details.

```bash
cargo run --bin nozy list-notes
```

## Network Commands

### `nozy test-zebra`
Test connection to Zebra RPC node.

```bash
cargo run --bin nozy test-zebra
```

## Proving Commands

### `nozy proving --status`
Check proving parameters status.

```bash
cargo run --bin nozy proving --status
```

### `nozy proving --download`
Download proving parameters.

```bash
cargo run --bin nozy proving --download
```

## Next Steps

- [Sending ZEC](sending-zec.md) - Detailed guide for sending transactions
- [Troubleshooting](troubleshooting.md) - Common issues and solutions
