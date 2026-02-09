# Creating Your First Wallet

This guide walks you through creating your first NozyWallet.

## Desktop App (Recommended)

### Step 1: Launch NozyWallet

After installation, launch NozyWallet from your applications menu.

### Step 2: Setup Wizard

The setup wizard will guide you through:

1. **Welcome screen** - Introduction to NozyWallet
2. **Create new wallet** - Choose to create a new wallet
3. **Password setup** - Set a strong password (optional but recommended)
4. **Address generation** - Generate your first receiving address
5. **Complete** - You're ready to use NozyWallet!

### Step 3: Backup Your Seed Phrase

**CRITICAL**: You will be shown your 24-word mnemonic phrase. This is the ONLY way to recover your wallet if you lose access.

**⚠️ SECURITY WARNING:**

- ✅ Write it down on paper (never digitally)
- ✅ Store in a secure location (fireproof safe, bank deposit box)
- ✅ Never share it with anyone
- ✅ Never take screenshots or photos
- ✅ Never store online (cloud, email, notes apps)
- ✅ Make multiple copies in different secure locations

### Step 4: Generate Your First Address

After creating your wallet, generate your first receiving address:

- Click "Receive" in the desktop app
- Your Orchard address will be displayed
- You can copy it or show a QR code

## CLI (Command Line)

### Step 1: Create Wallet

```bash
nozy new
```

### Step 2: Follow Prompts

You'll be asked:

1. **Create new wallet?** - Type `y` to confirm
2. **Set password?** - Type `y` to set a password (recommended)
3. **Enter password** - Type your password (hidden)
4. **Confirm password** - Type again to confirm

### Step 3: Save Your Mnemonic

**CRITICAL**: Your 24-word mnemonic phrase will be displayed. 

**Example output:**
```
Creating new wallet...
Wallet created successfully!

**CRITICAL SECURITY WARNING: MNEMONIC BACKUP**

Your mnemonic phrase is the ONLY way to recover your wallet.
If you lose it, you will PERMANENTLY lose access to all your funds.

Your 24-word mnemonic phrase:
abandon abandon abandon ... [24 words total - KEEP THIS SECURE!]

Set password protection? (y/n): y
Enter password: ********
Confirm password: ********
Password set successfully

Generated Orchard address:
u1testaddress1234567890abcdefghijklmnopqrstuvwxyz...
```

### Step 4: Generate Addresses

```bash
# Generate a single address
nozy addresses

# Generate multiple addresses
nozy addresses --count 5
```

## What Happens Next?

After creating your wallet:

1. ✅ **Wallet file created** - Stored securely on your device
2. ✅ **First address generated** - Ready to receive ZEC
3. ✅ **Ready to use** - You can now send and receive

## Next Steps

- [Send ZEC](user-guide/sending-zec.md) - Send your first transaction
- [Receive ZEC](user-guide/receiving-zec.md) - Learn about receiving
- [Backup & Recovery](user-guide/backup-recovery.md) - Secure your wallet

## Troubleshooting

### Wallet Already Exists

If you see "Wallet already exists" error:
- You already have a wallet created
- Use `nozy status` to check wallet info
- Use `nozy restore` to restore from seed phrase

### Password Issues

- **Forgot password**: You can restore from seed phrase (password will be reset)
- **Wrong password**: Double-check your password
- **Password too weak**: Use a stronger password with mixed characters

## Security Reminders

⚠️ **Remember**:
- Your mnemonic phrase is your wallet - protect it!
- Your password protects the wallet file - use a strong one
- Never share your mnemonic or password with anyone
- Store backups in secure, offline locations
