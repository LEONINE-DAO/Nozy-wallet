# Quick Start Guide

Get up and running with NozyWallet in 5 minutes!

## Overview

This quick start guide will help you:
1. Install NozyWallet
2. Create your first wallet
3. Generate a receiving address
4. Sync your wallet

## Step 1: Installation (2 minutes)

### Desktop App (Recommended)

1. **Download** from [GitHub Releases](https://github.com/LEONINE-DAO/Nozy-wallet/releases/latest)
2. **Install** the `.exe` (Windows), `.dmg` (macOS), or `.AppImage` (Linux)
3. **Launch** NozyWallet

### CLI (Alternative)

```bash
# Clone repository
git clone https://github.com/LEONINE-DAO/Nozy-wallet.git
cd Nozy-wallet

# Build
cargo build --release

# Binary will be in target/release/nozy (or nozy.exe on Windows)
```

## Step 2: Create Wallet (1 minute)

### Desktop App

1. Launch NozyWallet
2. Click "Create New Wallet"
3. **Save your 24-word mnemonic phrase** - Write it down on paper!
4. Set a password (optional but recommended)
5. Click "Create"

### CLI

```bash
nozy new
```

Follow the prompts to create your wallet and save your mnemonic phrase.

**⚠️ CRITICAL:** Your mnemonic phrase is the ONLY way to recover your wallet. Store it securely offline!

## Step 3: Generate Address (30 seconds)

### Desktop App

1. Click "Receive" tab
2. Your Orchard address will be displayed
3. Copy the address or show QR code

### CLI

```bash
nozy addresses
```

**Example output:**
```
Generating Orchard address...

Address 0: u1testaddress1234567890abcdefghijklmnopqrstuvwxyz...
```

**Copy this address** - You'll use it to receive ZEC.

## Step 4: Sync Wallet (1-2 minutes)

### Desktop App

1. Click "Sync" button
2. Wait for sync to complete
3. Your balance will appear

### CLI

```bash
# Sync from recent blocks (faster)
nozy sync

# Or sync from specific height
nozy sync --start-height 3000000
```

**First sync may take time** - The wallet needs to scan the blockchain for your notes.

## What's Next?

### Send Your First Transaction

1. **Get ZEC** - Receive ZEC to your address
2. **Wait for confirmations** - Usually 1-2 blocks (~75 seconds each)
3. **Sync wallet** - Run sync to see incoming funds
4. **Send ZEC** - Use "Send" to create a transaction

See [Sending ZEC](user-guide/sending-zec.md) for detailed instructions.

### Learn More

- [Creating Your First Wallet](creating-wallet.md) - Detailed wallet creation guide
- [Restoring from Seed Phrase](restoring-wallet.md) - How to restore a wallet
- [Wallet Management](user-guide/wallet-management.md) - Advanced wallet operations
- [Security Best Practices](security/best-practices.md) - Keep your wallet secure

## Common Questions

**Q: Do I need my own Zebra node?**  
A: No! You can use a public Zebra node. Configure the node URL in settings if needed.

**Q: How long does first sync take?**  
A: Depends on blockchain height. Recent blocks (last 1000) take ~1-2 minutes. Full sync takes longer.

**Q: Can I use transparent addresses?**  
A: No. NozyWallet only supports shielded (Orchard) addresses for complete privacy.

**Q: What if I lose my mnemonic?**  
A: Without your mnemonic phrase, you cannot recover your wallet. Always backup your mnemonic securely!

**Q: Is my wallet safe?**  
A: Yes! Your wallet file is encrypted with your password, and your private keys never leave your device.

## Getting Help

- **Documentation**: Browse the full documentation
- **Troubleshooting**: Check [Common Issues](troubleshooting/common-issues.md)
- **GitHub Issues**: Report bugs or ask questions
- **Community**: Join our Discord (if available)

## Ready to Use!

You're now ready to use NozyWallet! Remember:

1. ✅ **Backup your mnemonic** - Store it securely offline
2. ✅ **Use strong password** - Protect your wallet file
3. ✅ **Verify addresses** - Double-check addresses before sending
4. ✅ **Start small** - Test with small amounts first
5. ✅ **Stay private** - All transactions are automatically private!

Happy private transacting! 🦋
