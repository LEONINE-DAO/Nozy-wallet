# NozyWallet Command Help Guide

This guide provides detailed examples and explanations for all NozyWallet commands.

## Table of Contents
- [Basic Commands](#basic-commands)
- [Wallet Management](#wallet-management)
- [Address Management](#address-management)
- [Transaction Commands](#transaction-commands)
- [Network Commands](#network-commands)
- [Utility Commands](#utility-commands)
- [Troubleshooting](#troubleshooting)

---

## Basic Commands

### `nozy new`
Create a new wallet with a fresh mnemonic phrase.

**Usage:**
```bash
nozy new
```

**What it does:**
- Generates a new 24-word mnemonic phrase
- Creates an encrypted wallet file
- Optionally sets a password for security
- Displays the mnemonic phrase (SAVE THIS!)

**Example Output:**
```
🔐 Creating new wallet...
✅ Password protection enabled
🎉 Wallet created successfully!
📝 Mnemonic: abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art
⚠️  IMPORTANT: Save this mnemonic in a safe place!
📍 Sample address: u1test1234567890abcdefghijklmnop...
```

---

### `nozy restore`
Restore an existing wallet from a mnemonic phrase.

**Usage:**
```bash
nozy restore
```

**What it does:**
- Prompts for your 24-word mnemonic phrase
- Creates a new wallet file from the mnemonic
- Sets a password for the restored wallet

**Example:**
```bash
$ nozy restore
Enter your 24-word mnemonic: abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art
Enter password to encrypt wallet: [password]
✅ Wallet restored and saved.
```

---

## Wallet Management

### `nozy info`
Display wallet information and mnemonic phrase.

**Usage:**
```bash
nozy info
```

**What it shows:**
- Wallet mnemonic phrase
- Confirmation that wallet loaded successfully

**Example Output:**
```
Wallet loaded successfully!
Mnemonic: abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art
```

---

## Address Management

### `nozy addresses`
Generate new receiving addresses.

**Usage:**
```bash
# Generate 1 address (default)
nozy addresses

# Generate multiple addresses
nozy addresses --count 5
```

**Examples:**
```bash
# Generate 1 address
nozy addresses

# Generate 5 addresses
nozy addresses --count 5
```

**Example Output:**
```
Generating 5 addresses...
Address 0: u1test1234567890abcdefghijklmnop...
Address 1: u1test2345678901bcdefghijklmnopq...
Address 2: u1test3456789012cdefghijklmnopqr...
Address 3: u1test4567890123defghijklmnopqrs...
Address 4: u1test5678901234efghijklmnopqrst...
```

---

## Transaction Commands

### `nozy scan`
Scan the blockchain for notes (received ZEC).

**Usage:**
```bash
# Scan recent blocks
nozy scan

# Scan from specific height
nozy scan --start-height 2800000

# Scan specific range
nozy scan --start-height 2800000 --end-height 2900000
```

**Examples:**
```bash
# Scan recent blocks (recommended)
nozy scan

# Scan from block 2,800,000 to current
nozy scan --start-height 2800000

# Scan specific range (useful for testing)
nozy scan --start-height 2800000 --end-height 2900000
```

**What it does:**
- Scans blockchain blocks for notes sent to your addresses
- Updates your wallet's note database
- Shows total balance found
- Saves notes to `wallet_data/notes.json`

**Example Output:**
```
Scanning blockchain for notes...
Scan complete!
Total notes found: 2
Total balance: 981453 zatoshis
Unspent notes: 2
Spendable notes: 2
🎉 Found ZEC in your wallet!
💰 Balance: 0.00981453 ZEC
  Note 1: 889996 ZAT (Block: 3077720)
  Note 2: 91457 ZAT (Block: 3077730)
```

---

### `nozy send`
Send ZEC to another address.

**Usage:**
```bash
nozy send --recipient <ADDRESS> --amount <AMOUNT> [--zebra-url <URL>]
```

**Examples:**
```bash
# Send 0.001 ZEC to a unified address
nozy send --recipient u1zhgy24tweexhjcsstya5qqzrus4cgv0amasfv5jp6f3p3qvw265rugn8ref5djg472l5s382mwuffremffr7se6xjlh5exagwg2d6frs --amount 0.001

# Send using testnet
nozy send --recipient u1test1234567890abcdefghijklmnop... --amount 0.001 --zebra-url http://127.0.0.1:18232

# Note: NozyWallet only supports shielded addresses (u1 unified addresses with Orchard receivers)
# Transparent addresses (t1) are not supported for privacy protection
```

**What it does:**
1. Loads your wallet (prompts for password)
2. Scans for spendable notes
3. Builds the transaction
4. Asks for mainnet confirmation
5. Broadcasts the transaction

**Example Output:**
```
Sending 0.001 ZEC to u1zhgy24tweexhjcsstya5qqzrus4cgv0amasfv5jp6f3p3qvw265rugn8ref5djg472l5s382mwuffremffr7se6xjlh5exagwg2d6frs...
🔗 Zebra URL set to: http://127.0.0.1:8232
🚨 MAINNET TRANSACTION DETECTED! 🚨
   This will send REAL ZEC on the mainnet blockchain!
   Do you want to enable mainnet broadcasting? (y/N)
Enter 'yes' to enable: yes
✅ Mainnet broadcasting enabled!
🔎 Scanning recent blocks for spendable notes...
✅ Found 2 spendable notes | Total: 0.00981453 ZEC
🔧 Building transaction...
✅ Transaction built successfully!
🆔 Transaction ID: abc1234567890def...
🚀 Broadcasting transaction...
✅ Transaction broadcast successful!
🌐 Network TXID: def4567890123abc...
🔗 Explorer: https://zcashblockexplorer.com/transactions/def4567890123abc...
```

---

### `nozy list-notes`
Display all stored notes and their details.

**Usage:**
```bash
nozy list-notes
```

**Example Output:**
```
Stored notes:
[
  {
    "note_bytes": [140, 148, 13, 0, 0, 0, 0, 0, 141, 129, 60, 144, 11, 187, 53, 148, 126, 116, 207, 156, 123, 60, 41, 174, 250, 167, 87, 201, 119, 26, 155, 240, 67, 12, 117, 230, 148, 68, 246, 20],
    "value": 889996,
    "address_bytes": [161, 230, 174, 136, 239, 251, 74, 149, 149, 114, 103, 166, 250, 75, 36, 191, 133, 71, 133, 248, 57, 233, 178, 80, 237, 141, 202, 124, 230, 226, 222, 99, 36, 65, 139, 232, 136, 243, 103, 55, 164, 64, 138],
    "nullifier_bytes": [141, 129, 60, 144, 11, 187, 53, 148, 126, 116, 207, 156, 123, 60, 41, 174, 250, 167, 87, 201, 119, 26, 155, 240, 67, 12, 117, 230, 148, 68, 246, 20],
    "block_height": 3077720,
    "txid": "68cc63f31c03dd8f430e64cae835c3efa8f2fd513a8009f8f2cb22d3be800fe6",
    "spent": false,
    "memo": []
  }
]
```

---

## Network Commands

### `nozy test-zebra`
Test connection to a Zebra node.

**Usage:**
```bash
# Test default node (mainnet)
nozy test-zebra

# Test specific node
nozy test-zebra --zebra-url http://127.0.0.1:18232
```

**Examples:**
```bash
# Test mainnet node
nozy test-zebra

# Test testnet node
nozy test-zebra --zebra-url http://127.0.0.1:18232

# Test custom node
nozy test-zebra --zebra-url https://zcash-rpc.example.com
```

**Example Output:**
```
🔗 Testing Zebra node connection...
📡 Connecting to: http://127.0.0.1:8232
✅ Zebra node is ONLINE!
📨 Response: {"result":{"version":1001550,"subversion":"/Zcash:4.7.0/","protocolversion":170013,"localservices":"000000000000000d","timeoffset":0,"connections":8,"networks":[{"name":"ipv4","limited":false,"reachable":true,"proxy":"","proxy_randomize_credentials":false},{"name":"ipv6","limited":false,"reachable":true,"proxy":"","proxy_randomize_credentials":false},{"name":"onion","limited":true,"reachable":false,"proxy":"","proxy_randomize_credentials":false}],"relayfee":0.00001000,"localaddresses":[],"warnings":""},"error":null,"id":"test"}
🎉 Zebra RPC is working correctly!
✅ Ready for mainnet transactions!
```

---

## Utility Commands

### `nozy proving`
Manage Orchard proving parameters.

**Usage:**
```bash
# Check proving status
nozy proving --status

# Download proving parameters
nozy proving --download
```

**Examples:**
```bash
# Check if proving parameters are available
nozy proving --status

# Download proving parameters (for testing)
nozy proving --download
```

**Example Output:**
```
🔧 Orchard Proving Parameters Management
=====================================

📊 Proving Status:
   Spend Parameters: ❌
   Output Parameters: ❌
   Spend Verifying Key: ❌
   Output Verifying Key: ❌
   Can Prove: ❌

💡 Use --download to download placeholder parameters
```

---

## Troubleshooting

### Common Issues

#### 1. "No wallet found" Error
```bash
Error: No wallet found. Use 'nozy new' or 'nozy restore' to create a wallet first.
```
**Solution:** Create or restore a wallet first:
```bash
nozy new
# OR
nozy restore
```

#### 2. "Password input error: IO error: not a terminal"
**Solution:** Run the command in an interactive terminal, not in a script.

#### 3. "No spendable notes found"
**Solution:** Scan for notes first:
```bash
nozy scan --start-height 2800000 --end-height 2900000
```

#### 4. "Failed to connect to Zebra"
**Solutions:**
- Check if Zebra is running: `ps aux | grep zebrad`
- Start Zebra node: `./target/release/zebrad`
- Check port: `netstat -tlnp | grep 8232`

#### 5. "Insufficient funds"
**Solution:** Check your balance:
```bash
nozy list-notes
```

### Getting Help

For any command, use `--help` to see detailed usage information:
```bash
nozy --help                    # Show all commands
nozy send --help              # Show send command help
nozy scan --help              # Show scan command help
```

### Network Configuration

- **Mainnet:** `http://127.0.0.1:8232` (port 8232)
- **Testnet:** `http://127.0.0.1:18232` (port 18232)

Use `--zebra-url` to specify a different endpoint:
```bash
nozy send --recipient <address> --amount <amount> --zebra-url http://127.0.0.1:18232
```

---

## Quick Reference

| Command | Purpose | Example |
|---------|---------|---------|
| `nozy new` | Create wallet | `nozy new` |
| `nozy restore` | Restore wallet | `nozy restore` |
| `nozy addresses` | Generate addresses | `nozy addresses --count 5` |
| `nozy scan` | Find received ZEC | `nozy scan --start-height 2800000` |
| `nozy send` | Send ZEC | `nozy send --recipient u1abc... --amount 0.001` |
| `nozy list-notes` | Show notes | `nozy list-notes` |
| `nozy info` | Wallet info | `nozy info` |
| `nozy test-zebra` | Test node | `nozy test-zebra` |

---

*For more help, visit the [NozyWallet GitHub repository](https://github.com/LEONINE-DAO/Nozy-wallet) or open an issue.*
