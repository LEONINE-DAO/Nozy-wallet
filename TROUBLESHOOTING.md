# NozyWallet Troubleshooting Guide

## Common Issues and Solutions

### 1. Zebra Connection Issues

#### Problem: "Zebra connection failed" or "Network error"

**Symptoms:**
- `cargo run --bin nozy test-zebra` fails
- Error messages about network connectivity
- Timeout errors when scanning or sending

**Solutions:**

1. **Check if Zebra is running:**
   ```bash
   # Check if Zebra process is running
   ps aux | grep zebrad
   
   # Or check if port 8232 is open
   netstat -tlnp | grep 8232
   ```

2. **Start Zebra with RPC enabled:**
   ```bash
   # Install Zebra if not already installed
   cargo install zebrad
   
   # Start Zebra with RPC
   zebrad start --rpc.bind-addr 127.0.0.1:8232
   ```

3. **Check Zebra configuration:**
   ```bash
   # Check Zebra config file (~/.config/zebrad.toml)
   cat ~/.config/zebrad.toml
   
   # Ensure RPC is enabled:
   # [rpc]
   # listen_addr = "127.0.0.1:8232"
   ```

4. **Test direct RPC connection:**
   ```bash
   curl -H 'content-type: application/json' \
        --data-binary '{"jsonrpc":"2.0","method":"getblockcount","params":[],"id":1}' \
        http://127.0.0.1:8232
   ```

5. **Check firewall settings:**
   ```bash
   # On Linux
   sudo ufw status
   sudo ufw allow 8232
   
   # On Windows
   # Check Windows Firewall settings
   ```

#### Problem: "Invalid JSON response" or "RPC error"

**Solutions:**

1. **Check Zebra version compatibility:**
   ```bash
   zebrad --version
   # Ensure you're using a recent version
   ```

2. **Restart Zebra:**
   ```bash
   # Stop Zebra
   pkill zebrad
   
   # Start fresh
   zebrad start --rpc.bind-addr 127.0.0.1:8232
   ```

3. **Check Zebra logs:**
   ```bash
   tail -f ~/.cache/zebrad/debug.log
   ```

### 2. Wallet Issues

#### Problem: "No wallet found" or "Wallet load failed"

**Symptoms:**
- Error when running commands that require a wallet
- "No wallet found. Use 'nozy new' or 'nozy restore'"

**Solutions:**

1. **Create a new wallet:**
   ```bash
   cargo run --bin nozy new
   ```

2. **Restore from mnemonic:**
   ```bash
   cargo run --bin nozy restore
   # Enter your 24-word mnemonic phrase
   ```

3. **Check wallet file exists:**
   ```bash
   ls -la wallet_data/
   # Should see wallet.dat file
   ```

4. **Check file permissions:**
   ```bash
   chmod 600 wallet_data/wallet.dat
   ```

#### Problem: "Invalid password" or "Password verification failed"

**Solutions:**

1. **Verify password correctness:**
   - Ensure no typos
   - Check caps lock
   - Try without password if wallet was created without one

2. **Check wallet file integrity:**
   ```bash
   # Backup current wallet
   cp wallet_data/wallet.dat wallet_data/wallet.dat.backup
   
   # Try to restore from mnemonic
   cargo run --bin nozy restore
   ```

3. **Reset wallet (if you have mnemonic):**
   ```bash
   # Remove corrupted wallet
   rm wallet_data/wallet.dat
   
   # Restore from mnemonic
   cargo run --bin nozy restore
   ```

### 3. Proving Parameters Issues

#### Problem: "Proving parameters not found" or "Cannot prove"

**Symptoms:**
- Warning messages about missing proving parameters
- Transaction building fails
- "Can Prove: ❌" in status

**Solutions:**

1. **Download placeholder parameters (for testing):**
   ```bash
   cargo run --bin nozy proving --download
   ```

2. **Check parameters status:**
   ```bash
   cargo run --bin nozy proving --status
   ```

3. **Download real parameters (for production):**
   ```bash
   # Create parameters directory
   mkdir -p orchard_params
   
   # Download from official Zcash sources
   wget https://download.z.cash/downloads/orchard-spend.params -O orchard_params/orchard-spend.params
   wget https://download.z.cash/downloads/orchard-output.params -O orchard_params/orchard-output.params
   wget https://download.z.cash/downloads/orchard-spend-verifying.key -O orchard_params/orchard-spend-verifying.key
   wget https://download.z.cash/downloads/orchard-output-verifying.key -O orchard_params/orchard-output-verifying.key
   ```

4. **Verify parameters:**
   ```bash
   ls -la orchard_params/
   # Should see all 4 files
   ```

5. **Check file permissions:**
   ```bash
   chmod 644 orchard_params/*
   ```

### 4. Transaction Issues

#### Problem: "Insufficient funds" or "Transaction build failed"

**Symptoms:**
- Error when trying to send ZEC
- "Insufficient funds" message
- Transaction building fails

**Solutions:**

1. **Check your balance:**
   ```bash
   cargo run --bin nozy scan --start-height 1000000 --end-height 1000100
   ```

2. **Scan wider range:**
   ```bash
   # Scan more blocks
   cargo run --bin nozy scan --start-height 1000000 --end-height 1001000
   ```

3. **Check for incoming transactions:**
   - Look for recent ZEC deposits
   - Verify addresses are correct
   - Check if notes are spendable

4. **Verify recipient address:**
   ```bash
   # Ensure address is valid Orchard address
   cargo run --bin nozy addresses --count 1
   # Compare with recipient address format
   ```

#### Problem: "Invalid address format" or "Address parsing failed"

**Solutions:**

1. **Check address format:**
   - Orchard addresses start with `u1`
   - Unified addresses are supported
   - Ensure no extra spaces or characters

2. **Generate test address:**
   ```bash
   cargo run --bin nozy addresses --count 1
   # Use this format for testing
   ```

3. **Verify address with Zcash tools:**
   ```bash
   # Use zcash-cli to verify address
   zcash-cli validateaddress "u1..."
   ```

### 5. Note Scanning Issues

#### Problem: "No notes found" or "Scan failed"

**Symptoms:**
- Scan completes but finds no notes
- Error during scanning process
- "No ZEC found" message

**Solutions:**

1. **Check scan range:**
   ```bash
   # Use current block height
   cargo run --bin nozy scan --start-height 3000000 --end-height 3000100
   ```

2. **Verify wallet addresses:**
   ```bash
   cargo run --bin nozy addresses --count 5
   # Check if these addresses received ZEC
   ```

3. **Check Zebra sync status:**
   ```bash
   cargo run --bin nozy test-zebra
   # Ensure Zebra is fully synced
   ```

4. **Try different height ranges:**
   ```bash
   # Scan recent blocks
   cargo run --bin nozy scan --start-height 3050000 --end-height 3050100
   ```

5. **Check for Orchard transactions:**
   - Ensure you're looking for Orchard notes
   - Check if ZEC was sent to Orchard addresses
   - Verify transaction types

### 6. Build and Compilation Issues

#### Problem: "Compilation failed" or "Build error"

**Solutions:**

1. **Update Rust:**
   ```bash
   rustup update
   rustc --version
   # Ensure Rust 1.70+
   ```

2. **Clean and rebuild:**
   ```bash
   cargo clean
   cargo build --release
   ```

3. **Check dependencies:**
   ```bash
   cargo update
   cargo build
   ```

4. **Check for conflicting versions:**
   ```bash
   cargo tree
   # Look for version conflicts
   ```

#### Problem: "Feature not available" or "Unsupported operation"

**Solutions:**

1. **Check feature flags:**
   ```bash
   cargo build --features "full"
   ```

2. **Update dependencies:**
   ```bash
   cargo update
   ```

3. **Check Rust edition:**
   ```toml
   # In Cargo.toml
   edition = "2021"
   ```

### 7. Performance Issues

#### Problem: Slow scanning or high memory usage

**Solutions:**

1. **Reduce scan range:**
   ```bash
   # Scan smaller ranges
   cargo run --bin nozy scan --start-height 1000000 --end-height 1000100
   ```

2. **Use release build:**
   ```bash
   cargo build --release
   cargo run --release --bin nozy scan
   ```

3. **Check system resources:**
   ```bash
   # Monitor memory usage
   htop
   # or
   top
   ```

4. **Optimize Zebra:**
   ```bash
   # Use faster storage for Zebra
   # Consider SSD for better performance
   ```

### 8. Security Issues

#### Problem: "Security warning" or "Unsafe operation"

**Solutions:**

1. **Check password strength:**
   - Use strong passwords
   - Avoid common patterns
   - Consider using a password manager

2. **Verify wallet encryption:**
   ```bash
   # Check if wallet is encrypted
   file wallet_data/wallet.dat
   # Should show encrypted data
   ```

3. **Check file permissions:**
   ```bash
   chmod 600 wallet_data/wallet.dat
   chmod 700 wallet_data/
   ```

4. **Use secure storage:**
   - Store wallet on encrypted drive
   - Use secure backup locations
   - Avoid cloud storage for sensitive data

## Debug Mode

### Enable Debug Logging

```bash
# Run with debug output
RUST_LOG=debug cargo run --bin nozy new

# Run specific command with debug
RUST_LOG=debug cargo run --bin nozy scan --start-height 1000000
```

### Common Debug Messages

1. **Network Debug:**
   ```
   DEBUG nozy::zebra_integration: Connecting to Zebra at http://127.0.0.1:8232
   DEBUG nozy::zebra_integration: RPC request: getblockcount
   ```

2. **Wallet Debug:**
   ```
   DEBUG nozy::hd_wallet: Generating new wallet
   DEBUG nozy::hd_wallet: Setting password protection
   ```

3. **Transaction Debug:**
   ```
   DEBUG nozy::orchard_tx: Building Orchard transaction
   DEBUG nozy::orchard_tx: Adding spend action
   ```

## Getting Help

### Log Collection

When reporting issues, include:

1. **System information:**
   ```bash
   uname -a
   rustc --version
   cargo --version
   ```

2. **Zebra information:**
   ```bash
   zebrad --version
   cargo run --bin nozy test-zebra
   ```

3. **Wallet status:**
   ```bash
   cargo run --bin nozy proving --status
   ls -la wallet_data/
   ```

4. **Error logs:**
   ```bash
   RUST_LOG=debug cargo run --bin nozy [command] 2>&1 | tee debug.log
   ```

### Community Support

- **GitHub Issues**: Report bugs and feature requests
- **GitHub Discussions**: Ask questions and get help
- **Documentation**: Check README and API docs

### Emergency Recovery

If you lose access to your wallet:

1. **Use mnemonic recovery:**
   ```bash
   cargo run --bin nozy restore
   # Enter your 24-word mnemonic
   ```

2. **Check backups:**
   ```bash
   ls -la wallet_data/
   # Look for backup files
   ```

3. **Verify mnemonic:**
   - Use official Zcash tools
   - Test with small amounts first
   - Double-check word order

## Prevention

### Best Practices

1. **Regular backups:**
   ```bash
   # Create regular backups
   cargo run --bin nozy proving --status
   cp -r wallet_data/ backup_$(date +%Y%m%d)/
   ```

2. **Test with small amounts:**
   - Always test with small ZEC amounts first
   - Verify addresses before sending
   - Use testnet when possible

3. **Keep software updated:**
   ```bash
   # Update regularly
   cargo update
   rustup update
   ```

4. **Secure storage:**
   - Use encrypted storage
   - Keep mnemonic safe
   - Use strong passwords

This troubleshooting guide should help resolve most common issues. For additional help, refer to the GitHub issues or create a new discussion.
