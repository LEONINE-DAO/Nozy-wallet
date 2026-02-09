# Receiving ZEC

Learn how to receive ZEC in your NozyWallet.

## Generating Receiving Addresses

NozyWallet generates Orchard addresses that are private by default. Each address can be used to receive ZEC.

### Desktop App

1. **Open NozyWallet**
2. **Click "Receive" tab**
3. **Your address is displayed** - Copy or show QR code
4. **Generate new address** - Click "New Address" for additional addresses

### CLI

**Generate a single address:**
```bash
nozy addresses
```

**Generate multiple addresses:**
```bash
nozy addresses --count 5
```

**Example output:**
```
Generating 5 Orchard addresses...

Address 0: u1testaddress1234567890abcdefghijklmnopqrstuvwxyz...
Address 1: u1testaddress2345678901bcdefghijklmnopqrstuvwxyzab...
Address 2: u1testaddress3456789012cdefghijklmnopqrstuvwxyzabc...
Address 3: u1testaddress4567890123defghijklmnopqrstuvwxyzabcd...
Address 4: u1testaddress5678901234efghijklmnopqrstuvwxyzabcde...

Generated 5 addresses
```

## Address Types

NozyWallet uses **Orchard addresses** exclusively:
- **Unified Address format** (starts with `u1...`)
- **Orchard-only addresses** (starts with `orchard1...`)
- **Shielded by default** - All transactions are private

## Best Practices

### Address Reuse

**Avoid reusing addresses** for maximum privacy:
- ✅ Generate a new address for each transaction
- ✅ Use different addresses for different purposes
- ✅ Don't reuse addresses that have received funds

### Sharing Addresses

**Safe ways to share:**
- ✅ Copy address text
- ✅ Show QR code
- ✅ Share via secure messaging (encrypted channels)

**Avoid:**
- ❌ Sharing on public platforms
- ❌ Reusing addresses
- ❌ Using transparent addresses (not supported)

## Checking Received Funds

### Desktop App

1. **Sync wallet** - Click "Sync" button
2. **View balance** - Balance updates automatically
3. **Check history** - View incoming transactions

### CLI

**Sync wallet to see new funds:**
```bash
nozy sync
```

**Check balance:**
```bash
nozy balance
```

**View transaction history:**
```bash
nozy history
```

## Receiving Process

### Step 1: Share Your Address

Share your receiving address with the sender:
- Copy address text
- Share QR code
- Send via secure channel

### Step 2: Wait for Transaction

The sender will:
1. Create a transaction to your address
2. Broadcast to Zcash network
3. Transaction is mined into a block (~75 seconds)

### Step 3: Sync Wallet

After the transaction is confirmed:
1. **Sync your wallet** - Run sync command
2. **Scan for notes** - Wallet automatically scans for incoming notes
3. **Balance updates** - Your balance reflects the received funds

### Step 4: Verify Receipt

Check that you received the funds:
- View balance
- Check transaction history
- Verify transaction details

## Transaction Confirmations

### Confirmation Time

- **First block** - ~75 seconds
- **Recommended** - 3-10 confirmations for larger amounts
- **Safe** - 10+ confirmations for maximum security

### Checking Confirmations

**Desktop App:**
- View transaction in history
- Confirmation count shown

**CLI:**
```bash
nozy transaction <txid>
```

## Privacy Considerations

### Receiving Privacy

- ✅ **All transactions are private** - Sender, receiver, amount hidden
- ✅ **Address privacy** - Your address doesn't reveal your identity
- ✅ **Amount privacy** - Transaction amounts are hidden
- ✅ **Fungibility** - All ZEC is identical

### Best Practices

1. **Generate new addresses** - One address per transaction
2. **Don't link addresses** - Avoid patterns that could link addresses
3. **Use memos carefully** - Memos are encrypted but be mindful
4. **Sync regularly** - Keep wallet synced to see all funds

## Troubleshooting

### "No funds received" After Sharing Address

**Possible reasons:**
1. **Transaction not confirmed** - Wait for block confirmation
2. **Wallet not synced** - Run sync command
3. **Wrong network** - Ensure mainnet/testnet matches
4. **Transaction failed** - Check sender's transaction status

**Solutions:**
- Sync wallet: `nozy sync`
- Check balance: `nozy balance`
- Verify address: Confirm address matches what was shared

### "Invalid address" Error When Sharing

**Problem:** Address format is incorrect.

**Solutions:**
- Verify address format (must start with `u1...` or `orchard1...`)
- Ensure address was copied completely
- Check for typos or missing characters

## Next Steps

- [Sending ZEC](sending-zec.md) - Learn how to send transactions
- [Transaction History](transaction-history.md) - View your transactions
- [Address Management](address-management.md) - Manage your addresses
- [Wallet Management](wallet-management.md) - Advanced wallet operations
