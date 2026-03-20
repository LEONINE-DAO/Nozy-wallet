# Creating a Wallet

NozyWallet supports creating new wallets or restoring existing ones from a mnemonic phrase.

## Create a New Wallet

To create a new wallet:

```bash
cargo run --bin nozy new
```

This command will:
- Generate a new 24-word BIP39 mnemonic phrase
- Create an encrypted wallet file
- Optionally set a password for security
- Display the mnemonic phrase (SAVE THIS!)

**⚠️ CRITICAL: Save your mnemonic phrase!**
Write down the 24-word mnemonic phrase displayed. You'll need it to restore your wallet if you lose access.

## Restore from Mnemonic

If you already have a wallet mnemonic, you can restore it:

```bash
cargo run --bin nozy restore
```

Enter your 24-word mnemonic phrase when prompted. The wallet will be restored and saved to `wallet_data/wallet.dat`.

## Password Protection

When creating or restoring a wallet, you'll be prompted to set a password. This password:
- Encrypts your wallet file using AES-256-GCM
- Is required to unlock the wallet for operations
- Uses Argon2 for secure password hashing
- Never stored in plain text

## Wallet Storage

Wallets are stored in the `wallet_data/` directory:
- Default location: `wallet_data/wallet.dat`
- Encrypted with your password
- Can be backed up and restored

## Next Steps

- [Quick Start Guide](../getting-started/quick-start.md) - Get started with your wallet
- [Sending ZEC](sending-zec.md) - Learn how to send transactions
- [Command Reference](command-reference.md) - Complete command documentation
