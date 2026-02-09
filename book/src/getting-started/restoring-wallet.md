# Restoring from Seed Phrase

If you've lost access to your wallet or need to restore it on a new device, you can restore your wallet using your 24-word mnemonic phrase.

## Before You Start

**Required:**
- ✅ Your 24-word mnemonic phrase (seed phrase)
- ✅ Secure location to restore the wallet

**⚠️ CRITICAL SECURITY WARNING:**

Your mnemonic phrase is the ONLY way to recover your wallet. Never:
- ❌ Share it with anyone
- ❌ Store it digitally (screenshots, cloud, email, notes apps)
- ❌ Enter it on any website
- ❌ Lose or destroy it

## Desktop App

### Step 1: Launch NozyWallet

Open the NozyWallet desktop application.

### Step 2: Select Restore

In the setup wizard:
1. Click "Restore from Seed Phrase" or "Import Wallet"
2. You'll be prompted for your mnemonic phrase

### Step 3: Enter Your Mnemonic

1. **Enter all 24 words** - Type each word carefully
2. **Verify spelling** - Each word must be spelled correctly
3. **Check order** - Words must be in the correct order
4. **Use spaces** - Separate words with spaces

### Step 4: Set Password

After entering your mnemonic:
1. **Set a new password** (optional but recommended)
2. **Confirm password**
3. Click "Restore Wallet"

### Step 5: Wait for Restoration

The wallet will:
- Generate all keys from your mnemonic
- Restore your wallet file
- Prepare addresses for use

## CLI (Command Line)

### Step 1: Run Restore Command

```bash
nozy restore
```

### Step 2: Enter Mnemonic

You'll be prompted to enter your 24-word mnemonic phrase:

```
Enter your 24-word mnemonic: abandon abandon abandon ... [enter all 24 words]
```

**Tips:**
- Enter all 24 words on one line, separated by spaces
- Double-check spelling before pressing Enter
- Common mistakes: typos, missing words, wrong order

### Step 3: Set Password

After entering your mnemonic:

```
Enter password to encrypt wallet: [enter password]
Confirm password: [enter again]
```

### Step 4: Wait for Completion

**Expected output:**
```
Restore wallet from mnemonic...
✅ Wallet restored and saved.
```

Your wallet is now restored and ready to use!

## Troubleshooting

### "Invalid mnemonic" Error

**Problem:** The mnemonic phrase is invalid.

**Solutions:**
1. **Check spelling** - Verify each word is spelled correctly
2. **Verify word count** - Must be exactly 24 words
3. **Check order** - Words must be in the original order
4. **Verify BIP39 words** - All words must be from the BIP39 word list

**Common mistakes:**
- Missing words (23 words instead of 24)
- Extra words (25+ words)
- Typos (e.g., "abandn" instead of "abandon")
- Wrong order (words swapped)

### "Wallet already exists" Error

**Problem:** A wallet already exists at the default location.

**Solutions:**
1. **Backup existing wallet** - Move it to a safe location first
2. **Delete existing wallet** - Only if you're sure it's not needed
3. **Use different location** - Specify a custom wallet path (if supported)

### "Invalid checksum" Error

**Problem:** The mnemonic phrase has a checksum error, meaning one or more words are incorrect.

**Solutions:**
1. **Double-check each word** - Verify spelling and order
2. **Use word list** - Check against BIP39 word list if unsure
3. **Re-verify source** - Make sure you copied from the correct backup

## After Restoration

Once your wallet is restored:

1. ✅ **Verify addresses** - Generate addresses and verify they match (if you had previous addresses)
2. ✅ **Check balance** - Sync wallet to see your balance
3. ✅ **Backup again** - Save your mnemonic phrase again in a new secure location
4. ✅ **Update password** - If needed, change to a new password

## Security Best Practices

### After Restoring

1. **Verify the restoration worked** - Check that you can see your balance
2. **Test a small transaction** - Send a small amount to verify functionality
3. **Create new backups** - Store your mnemonic in multiple secure locations
4. **Destroy old backups** - If restoring on a new device, securely delete old wallet files

### Protecting Your Mnemonic

- ✅ **Write on paper** - Physical backup is most secure
- ✅ **Store in safe** - Fireproof safe or bank deposit box
- ✅ **Multiple copies** - Store in different secure locations
- ✅ **Test recovery** - Periodically verify you can restore from backup

## Next Steps

After restoring your wallet:

- [Sync Wallet](user-guide/wallet-management.md#syncing) - Sync to see your balance
- [Send ZEC](user-guide/sending-zec.md) - Send transactions
- [Receive ZEC](user-guide/receiving-zec.md) - Generate receiving addresses
- [Backup & Recovery](user-guide/backup-recovery.md) - Learn about backup strategies
