# Crosslink Backend Setup Guide

## 🎯 Overview

NozyWallet now supports **Zebra Crosslink** as an alternative backend! This allows you to:
- Use experimental Crosslink testnet features
- Prepare for future PoS/staking capabilities
- Keep NozyWallet ahead of the game

## 📋 Quick Start

### Option 1: CLI Command (Easiest)

```bash
# Switch to Crosslink backend
nozy config --use-crosslink --crosslink-url http://127.0.0.1:8232

# Switch back to standard Zebra
nozy config --use-zebra

# View current backend
nozy config --show-backend
```

### Option 2: Manual Config Edit

Edit your config file (usually `~/.config/nozy/config.json` or `%APPDATA%\nozy\config\config.json`):

```json
{
  "zebra_url": "http://127.0.0.1:8232",
  "network": "mainnet",
  "backend": "crosslink",
  "crosslink_url": "http://127.0.0.1:8232",
  "last_scan_height": null,
  "theme": "dark"
}
```

**Key fields:**
- `"backend": "zebra"` - Use standard Zebra (default)
- `"backend": "crosslink"` - Use Zebra Crosslink
- `"crosslink_url"` - URL for Crosslink node (required when using Crosslink)

## 🔧 Configuration Details

### Backend Options

#### Standard Zebra (`"backend": "zebra"`)
- **Default**: Yes
- **URL**: Uses `zebra_url` field
- **Network**: Mainnet/Testnet (standard Zcash)
- **Use case**: Production wallet operations

#### Zebra Crosslink (`"backend": "crosslink"`)
- **Default**: No
- **URL**: Uses `crosslink_url` field (falls back to `zebra_url` if not set)
- **Network**: Crosslink testnet (experimental)
- **Use case**: Testing Crosslink features, future PoS/staking

### Config File Location

**Windows:**
```
C:\Users\YourName\AppData\Roaming\nozy\config\config.json
```

**Linux/Mac:**
```
~/.config/nozy/config.json
```

## 🚀 Setting Up Crosslink Node

### 1. Build Zebra Crosslink

```bash
git clone https://github.com/ShieldedLabs/zebra-crosslink.git
cd zebra-crosslink
cargo build --release
```

### 2. Run Crosslink Node

```bash
# Start Crosslink node (testnet)
./target/release/zebrad start --network testnet

# Or with custom RPC port
./target/release/zebrad start --network testnet --rpc-listen-addr 127.0.0.1:8233
```

### 3. Configure NozyWallet

```bash
# Set Crosslink URL (if using custom port)
nozy config --use-crosslink --crosslink-url http://127.0.0.1:8233

# Or use same port as Zebra
nozy config --use-crosslink --crosslink-url http://127.0.0.1:8232
```

## 📝 CLI Commands

### View Current Backend

```bash
nozy config --show-backend
```

Output:
```
Current backend: zebra
Zebra URL: http://127.0.0.1:8232
Crosslink URL: (not set)
```

### Switch to Crosslink

```bash
nozy config --use-crosslink --crosslink-url http://127.0.0.1:8232
```

### Switch to Zebra

```bash
nozy config --use-zebra
```

### Set Crosslink URL

```bash
nozy config --set-crosslink-url http://127.0.0.1:8233
```

## ✅ Verification

### Test Connection

```bash
# Test current backend connection
nozy test-zebra

# This will test whichever backend is configured
```

### Check Backend in Use

When you run commands like `sync` or `send`, the wallet will:
- Show which backend it's using
- Connect to the appropriate node
- Use backend-specific features if available

## 🔄 Switching Between Backends

You can switch backends anytime:

```bash
# Switch to Crosslink for testing
nozy config --use-crosslink --crosslink-url http://127.0.0.1:8232

# Run commands (they'll use Crosslink)
nozy sync
nozy balance

# Switch back to Zebra for production
nozy config --use-zebra

# Run commands (they'll use Zebra)
nozy sync
nozy balance
```

## ⚠️ Important Notes

### Testnet vs Mainnet

- **Crosslink is experimental** - Use testnet only
- **Don't use mainnet funds** with Crosslink until it's production-ready
- **Standard Zebra** is safe for mainnet

### Network Compatibility

- Crosslink may use different network parameters
- Transactions on Crosslink won't work on standard Zcash
- Keep separate wallets for Crosslink testing

### Future Features

When Crosslink PoS/staking is ready:
- Vault features will automatically use Crosslink backend
- Staking operations will be available
- Rewards will be calculated from Crosslink network

## 🐛 Troubleshooting

### "Backend not found" Error

Make sure your Crosslink node is running:
```bash
# Check if node is running
curl http://127.0.0.1:8232

# Or check process
ps aux | grep zebrad
```

### "Invalid backend" Error

Check your config file:
```bash
# View current config
cat ~/.config/nozy/config.json

# Make sure backend is "zebra" or "crosslink"
```

### Connection Timeout

1. Verify node is running
2. Check RPC port is correct
3. Ensure firewall allows connections
4. Try `nozy test-zebra` to diagnose

## 📚 Example Workflow

```bash
# 1. Start Crosslink node (in separate terminal)
cd zebra-crosslink
./target/release/zebrad start --network testnet

# 2. Switch NozyWallet to Crosslink
nozy config --use-crosslink --crosslink-url http://127.0.0.1:8232

# 3. Verify connection
nozy test-zebra

# 4. Use wallet with Crosslink
nozy sync
nozy balance
nozy send --recipient u1... --amount 0.1

# 5. Switch back to Zebra when done
nozy config --use-zebra
```

## 🎉 You're Ready!

Your NozyWallet is now configured to use Crosslink! All wallet operations (`sync`, `send`, `balance`, etc.) will automatically use the configured backend.

When Crosslink PoS features are ready, they'll be available automatically through the same backend system.

