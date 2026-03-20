# Troubleshooting

Common issues and solutions for NozyWallet.

## Zebra Connection Issues

### Problem: "Zebra connection failed" or "Network error"

**Solutions:**

1. **Check if Zebra is running:**
   ```bash
   # Check if Zebra process is running
   ps aux | grep zebrad
   ```

2. **Start Zebra with RPC enabled:**
   ```bash
   zebrad start --rpc.bind-addr 127.0.0.1:8232
   ```

3. **Test connection:**
   ```bash
   cargo run --bin nozy test-zebra
   ```

## Wallet Issues

### Problem: "Wallet not found" or "Failed to load wallet"

**Solutions:**

1. **Check wallet file exists:**
   ```bash
   ls wallet_data/wallet.dat
   ```

2. **Create a new wallet:**
   ```bash
   cargo run --bin nozy new
   ```

3. **Restore from mnemonic:**
   ```bash
   cargo run --bin nozy restore
   ```

## Transaction Issues

### Problem: "No spendable notes found"

**Solutions:**

1. **Scan for notes:**
   ```bash
   cargo run --bin nozy scan --start-height <START> --end-height <END>
   ```

2. **Check your balance:**
   ```bash
   cargo run --bin nozy list-notes
   ```

### Problem: "Insufficient funds"

**Solutions:**

1. **Verify your balance:**
   ```bash
   cargo run --bin nozy list-notes
   ```

2. **Ensure you've scanned for notes:**
   ```bash
   cargo run --bin nozy scan
   ```

## Address Issues

### Problem: "Invalid recipient address"

**Solutions:**

1. **Ensure you're using a Unified Address** (starts with `u1...` or `utest1...` for testnet)
2. **Verify the address is correct** - copy it carefully
3. **Check network match** - testnet vs mainnet addresses

## Proving Issues

### Problem: "Proving parameters not found"

**Solutions:**

1. **Download proving parameters:**
   ```bash
   cargo run --bin nozy proving --download
   ```

2. **Check status:**
   ```bash
   cargo run --bin nozy proving --status
   ```

## Getting Help

If you're still experiencing issues:

- Check the [GitHub Issues](https://github.com/LEONINE-DAO/Nozy-wallet/issues)
- Ask in [GitHub Discussions](https://github.com/LEONINE-DAO/Nozy-wallet/discussions)
- Review the [Command Reference](command-reference.md)
