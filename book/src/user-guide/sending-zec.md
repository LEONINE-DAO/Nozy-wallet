# Sending ZEC

Send private shielded transactions with NozyWallet.

## Prerequisites

Before sending ZEC, ensure:
- ✅ Wallet is created and unlocked
- ✅ You have ZEC in your wallet (balance > 0)
- ✅ Wallet is synced with the blockchain
- ✅ You have the recipient's address

## Desktop App

### Step 1: Open Send Screen

1. Launch NozyWallet
2. Click "Send" in the navigation menu
3. The send screen will open

### Step 2: Enter Recipient Address

1. **Enter recipient address** in the "To" field
   - Must be a valid Orchard address (starts with `u1...` or `orchard1...`)
   - Double-check the address before sending
   - Use address book if saved

2. **Verify address format** - NozyWallet only supports shielded addresses

### Step 3: Enter Amount

1. **Enter amount** in ZEC
   - Amount must be less than your balance (account for fees)
   - Minimum amount: 0.00001 ZEC (1 zatoshi)

2. **Check balance** - Verify you have sufficient funds

### Step 4: Add Memo (Optional)

1. **Enter memo** - Add a note (up to 512 characters)
2. **Memo is encrypted** - Only the recipient can read it

### Step 5: Review & Send

1. **Review transaction details:**
   - Recipient address
   - Amount
   - Memo (if included)
   - Estimated fee

2. **Confirm transaction** - Click "Send" button

3. **Enter password** - If wallet is password-protected

4. **Wait for confirmation** - Transaction will be built and broadcast

## CLI (Command Line)

### Basic Send Command

```bash
nozy send --to <address> --amount <amount>
```

**Example:**
```bash
nozy send --to u1testaddress1234567890abcdefghijklmnopqrstuvwxyz... --amount 0.1
```

### With Memo

```bash
nozy send --to <address> --amount <amount> --memo "Payment for services"
```

### Full Example

```bash
# Send 0.5 ZEC with a memo
nozy send \
  --to u1testaddress1234567890abcdefghijklmnopqrstuvwxyz... \
  --amount 0.5 \
  --memo "Payment for invoice #12345"
```

## Transaction Details

### What Happens When You Send

1. **Note Selection** - Wallet selects spendable notes to cover the amount
2. **Transaction Building** - Creates a shielded transaction
3. **Proof Generation** - Generates zero-knowledge proofs (may take a few seconds)
4. **Broadcasting** - Sends transaction to the Zcash network
5. **Confirmation** - Transaction is included in a block (~75 seconds)

### Transaction Fees

- **Fee is automatic** - Calculated based on transaction size
- **Typical fee** - 0.00001-0.0001 ZEC (1-10 zatoshis)
- **Fee deducted** - Automatically deducted from your balance

### Confirmation Time

- **Block time** - ~75 seconds per block
- **First confirmation** - Usually 1-2 blocks (~75-150 seconds)
- **Recommended confirmations** - 3-10 blocks for larger amounts

## Verifying Transactions

### Check Transaction Status

**Desktop App:**
- View transaction in "History" tab
- Status will show: Pending, Confirmed, or Failed

**CLI:**
```bash
# View transaction history
nozy history

# Check specific transaction
nozy transaction <txid>
```

### Transaction ID (TxID)

After sending, you'll receive a transaction ID:
- **Share with recipient** - They can verify receipt using the txid
- **Keep for records** - Track your transactions
- **Privacy note** - TxID doesn't reveal amounts or addresses

## Troubleshooting

### "Insufficient funds" Error

**Problem:** Not enough ZEC to cover amount + fees.

**Solutions:**
1. **Check balance** - Verify your available balance
2. **Account for fees** - Leave room for transaction fees
3. **Sync wallet** - Ensure wallet is synced to see all notes
4. **Check unspent notes** - Some notes may not be spendable yet

### "Invalid address" Error

**Problem:** Recipient address format is invalid.

**Solutions:**
1. **Verify address** - Check spelling and format
2. **Use unified address** - Must start with `u1...`
3. **Check network** - Ensure mainnet/testnet matches
4. **Extract Orchard address** - NozyWallet extracts Orchard from unified addresses

### "Transaction build failed" Error

**Problem:** Unable to build the transaction.

**Solutions:**
1. **Check proving parameters** - Ensure parameters are downloaded
2. **Verify balance** - Ensure you have spendable notes
3. **Sync wallet** - Sync may be needed to see notes
4. **Check Zebra connection** - Verify connection to Zebra node

### "Proof generation failed" Error

**Problem:** Unable to generate zero-knowledge proofs.

**Solutions:**
1. **Download proving parameters** - `nozy proving --download`
2. **Check disk space** - Parameters are large (~2GB)
3. **Verify parameters** - Check parameter file integrity
4. **Re-download if needed** - Parameters may be corrupted

## Best Practices

### Before Sending

1. ✅ **Verify recipient address** - Double-check address before sending
2. ✅ **Test with small amount** - Send small test first if unsure
3. ✅ **Check balance** - Ensure sufficient funds including fees
4. ✅ **Sync wallet** - Make sure wallet is up to date

### Security

1. ✅ **Keep private keys secure** - Never share your mnemonic or password
2. ✅ **Verify addresses** - Always verify recipient addresses
3. ✅ **Start small** - Test with small amounts first
4. ✅ **Double-check amounts** - Verify amount before confirming

### Privacy

1. ✅ **Use memos carefully** - Memos are encrypted but still be mindful
2. ✅ **Avoid address reuse** - Generate new addresses for each transaction
3. ✅ **Consider churning** - For maximum privacy, use multiple transactions
4. ✅ **Keep transactions private** - Don't publicly share transaction details

## Advanced: Transaction Options

### Custom Fee (if supported)

```bash
nozy send --to <address> --amount <amount> --fee <fee>
```

### Spend from Specific Notes (if supported)

```bash
nozy send --to <address> --amount <amount> --from-note <note-id>
```

## What Happens Next?

After sending:

1. **Transaction broadcasts** - Sent to Zcash network
2. **Included in block** - Mined into next block (~75 seconds)
3. **Confirmations** - Additional blocks add confirmations
4. **Recipient receives** - Funds appear in recipient's wallet after confirmations

## Next Steps

- [Receiving ZEC](receiving-zec.md) - Learn about receiving transactions
- [Transaction History](transaction-history.md) - View and track transactions
- [Address Management](address-management.md) - Manage your addresses
- [Troubleshooting](../troubleshooting/common-issues.md) - Resolve issues
