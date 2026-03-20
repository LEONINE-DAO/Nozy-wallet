# Sending ZEC - Step-by-Step Guide

Follow these steps to send your first shielded ZEC transaction with NozyWallet!

## Step 1: Create or Restore Your Wallet

### Option A: Create New Wallet
```bash
cargo run --bin nozy new
```

**You'll be prompted for:**
- Password (optional but recommended)
- Password confirmation

**⚠️ CRITICAL: Save your mnemonic phrase!**
Write down the 24-word mnemonic phrase displayed. You'll need it to restore your wallet.

### Option B: Restore Existing Wallet
```bash
cargo run --bin nozy restore
```

Enter your 24-word mnemonic phrase when prompted.

## Step 2: Get Your Receiving Address

```bash
cargo run --bin nozy addresses
```

**This will display:**
- Your Unified Address (starts with `u1...`)
- Your Orchard Address (starts with `orchard1...`)

**📋 Copy your Unified Address** - you'll need it to receive test ZEC.

## Step 3: Get Test ZEC (Testnet Only)

If you're on testnet, you can get free test ZEC from a faucet:

**Zcash Testnet Faucet:**
- Visit: https://faucet.testnet.z.cash/
- Enter your Unified Address from Step 2
- Request test ZEC (usually 0.1 or 1.0 ZEC)

**Wait for confirmation:**
The faucet usually takes 5-15 minutes to send the funds and for your Zebra node to sync.

## Step 4: Scan for Notes

After receiving ZEC, scan the blockchain to detect your notes:

```bash
cargo run --bin nozy scan --start-height 2800000 --end-height 2900000
```

**Adjust heights based on:**
- Current block height: Run `cargo run --bin quick_test` to see your node's current height
- Start height: Set slightly before you expect your transaction (e.g., current - 1000)
- End height: Current block height

## Step 5: List Your Notes

Verify you have spendable notes:

```bash
cargo run --bin nozy list-notes
```

## Step 6: Send ZEC!

Now you're ready to send! Use this command:

```bash
cargo run --bin nozy send --to <RECIPIENT_ADDRESS> --amount <AMOUNT_IN_ZATOSHIS>
```

### Example: Send 0.001 ZEC

```bash
cargo run --bin nozy send --to u1test1234567890abcdefghijklmnop... --amount 100000
```

**⚠️ Important:**
- Amount is in **zatoshis** (1 ZEC = 100,000,000 zatoshis)
- 0.001 ZEC = 100,000 zatoshis
- 0.01 ZEC = 1,000,000 zatoshis
- 0.1 ZEC = 10,000,000 zatoshis
- 1.0 ZEC = 100,000,000 zatoshis

## Amount Conversion Table

| ZEC | Zatoshis | Command Example |
|-----|----------|----------------|
| 0.0001 | 10,000 | `--amount 10000` |
| 0.001 | 100,000 | `--amount 100000` |
| 0.01 | 1,000,000 | `--amount 1000000` |
| 0.1 | 10,000,000 | `--amount 10000000` |
| 1.0 | 100,000,000 | `--amount 100000000` |
| 10.0 | 1,000,000,000 | `--amount 1000000000` |

## Troubleshooting

### Error: "No spendable notes found"
**Solution:** Run the `scan` command with the correct block range.

### Error: "Failed to connect to Zebra"
**Solution:** Verify your Zebra node is running and synced.

### Error: "Invalid recipient address"
**Solution:** Make sure you're using a valid Zcash Unified Address (starts with `u1...` or `utest1...` for testnet).

### Error: "Insufficient funds"
**Solution:** Check your balance with `cargo run --bin nozy list-notes`

## Next Steps

- [Command Reference](command-reference.md) - Complete command documentation
- [Troubleshooting](troubleshooting.md) - Common issues and solutions
