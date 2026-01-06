# NozyWallet Troubleshooting Guide

## Quick Reference

**Common Error Messages and Quick Fixes:**

| Error Message | Quick Fix | Section |
|--------------|-----------|---------|
| "Connection failed to http://127.0.0.1:8232: Is Zebra running?" | Start Zebra: `zebrad start --rpc.bind-addr 127.0.0.1:8232` | [1.1](#1-zebra-connection-issues) |
| "Request timeout" | Check Zebra sync status, wait for sync to complete | [1.2](#problem-request-timeout-to-http1270018232-the-node-may-be-slow-or-overloaded) |
| "Invalid JSON response" | Restart Zebra, check version compatibility | [1.3](#problem-invalid-json-response-from-http1270018232-or-rpc-error) |
| "HTTP error 404/500" | Verify RPC configuration in Zebra config file | [1.4](#problem-http-error-404-or-http-error-500-from-zebra) |
| "No wallet found" | Create wallet: `cargo run --bin nozy new` | [2.1](#problem-no-wallet-found-or-wallet-load-failed) |
| "Invalid password" | Check password, restore from mnemonic if needed | [2.2](#problem-invalid-password-or-password-verification-failed) |
| "Proving parameters not found" | Download: `cargo run --bin nozy proving --download` | [3.1](#problem-proving-parameters-not-found-or-cannot-prove) |
| "Insufficient funds" | Rescan notes, check spendable balance | [4.1](#problem-insufficient-funds-or-transaction-build-failed) |
| "Invalid address format" | Use unified address (u1...), not transparent (t1...) | [4.2](#problem-invalid-address-format-or-address-parsing-failed) |
| "link.exe not found" (Windows) | Install Visual Studio Build Tools | [6.1](#problem-compilation-failed-or-build-error-or-linkexe-not-found-windows) |
| "Transaction build failed" | Check proving parameters status | [12.1](#problem-transaction-build-failed-or-cannot-create-proofs) |

## Common Issues and Solutions

### 1. Zebra Connection Issues

#### Problem: "Zebra connection failed" or "Connection failed to http://127.0.0.1:8232: Is Zebra running?"

**Symptoms:**
- `cargo run --bin nozy test-zebra` fails
- Error: "Failed to connect to local Zebra node at http://127.0.0.1:8232 after 4 attempts"
- Error: "Connection failed to http://127.0.0.1:8232: Is Zebra running?"
- Timeout errors when scanning or sending

**Solutions:**

1. **Check if Zebra is running:**
   ```bash
   # On Linux/macOS
   ps aux | grep zebrad
   
   # On Windows (PowerShell)
   Get-Process | Where-Object {$_.ProcessName -like "*zebrad*"}
   
   # Check if port 8232 is open
   # Linux/macOS:
   netstat -tlnp | grep 8232
   # or
   lsof -i :8232
   
   # Windows:
   netstat -ano | findstr :8232
   ```

2. **Start Zebra with RPC enabled:**
   ```bash
   # Install Zebra if not already installed
   cargo install zebrad
   
   # Start Zebra with RPC (Linux/macOS)
   zebrad start --rpc.bind-addr 127.0.0.1:8232
   
   # Or run in background
   nohup zebrad start --rpc.bind-addr 127.0.0.1:8232 > zebrad.log 2>&1 &
   ```

3. **Check Zebra configuration:**
   ```bash
   # Check Zebra config file location
   # Linux: ~/.config/zebrad.toml
   # macOS: ~/Library/Application Support/zebrad/zebrad.toml
   # Windows: %APPDATA%\zebrad\zebrad.toml
   
   cat ~/.config/zebrad.toml  # Linux
   # or
   cat ~/Library/Application\ Support/zebrad/zebrad.toml  # macOS
   
   # Ensure RPC is enabled:
   # [rpc]
   # listen_addr = "127.0.0.1:8232"
   ```

4. **Create/Edit Zebra config:**
   ```bash
   # Create config directory if it doesn't exist
   mkdir -p ~/.config  # Linux
   mkdir -p ~/Library/Application\ Support/zebrad  # macOS
   
   # Edit config file
   nano ~/.config/zebrad.toml
   
   # Add or verify:
   [rpc]
   listen_addr = "127.0.0.1:8232"
   ```

5. **Test direct RPC connection:**
   ```bash
   # Test with curl
   curl -H 'content-type: application/json' \
        --data-binary '{"jsonrpc":"2.0","method":"getblockcount","params":[],"id":1}' \
        http://127.0.0.1:8232
   
   # Should return JSON with block count
   # If it fails, Zebra RPC is not accessible
   ```

6. **Check firewall settings:**
   ```bash
   # On Linux (ufw)
   sudo ufw status
   sudo ufw allow 8232/tcp
   
   # On Linux (firewalld)
   sudo firewall-cmd --add-port=8232/tcp --permanent
   sudo firewall-cmd --reload
   
   # On Windows
   # Open Windows Defender Firewall
   # Add inbound rule for port 8232
   # Or run PowerShell as admin:
   New-NetFirewallRule -DisplayName "Zebra RPC" -Direction Inbound -LocalPort 8232 -Protocol TCP -Action Allow
   ```

7. **Check if another process is using port 8232:**
   ```bash
   # Linux/macOS
   sudo lsof -i :8232
   # Kill the process if needed:
   kill -9 <PID>
   
   # Windows
   netstat -ano | findstr :8232
   # Kill the process:
   taskkill /PID <PID> /F
   ```

#### Problem: "Request timeout to http://127.0.0.1:8232. The node may be slow or overloaded."

**Symptoms:**
- Requests to Zebra timeout
- Slow response times
- "Request timeout" errors

**Solutions:**

1. **Check Zebra sync status:**
   ```bash
   # Check if Zebra is still syncing
   zebrad status
   # If syncing, wait for it to complete
   ```

2. **Increase timeout (if using custom Zebra URL):**
   ```bash
   # Check current Zebra URL
   cargo run --bin nozy config --show
   
   # The timeout is built into the client, but you can:
   # - Ensure Zebra is fully synced
   # - Use a faster Zebra node
   # - Check system resources
   ```

3. **Check system resources:**
   ```bash
   # Check CPU and memory usage
   htop  # Linux
   top   # macOS/Linux
   # Windows: Task Manager
   
   # Zebra may be resource-intensive during sync
   ```

4. **Restart Zebra:**
   ```bash
   # Stop Zebra
   pkill zebrad  # Linux/macOS
   # Windows: Task Manager or
   taskkill /IM zebrad.exe /F
   
   # Start fresh
   zebrad start --rpc.bind-addr 127.0.0.1:8232
   ```

#### Problem: "Invalid JSON response from http://127.0.0.1:8232" or "RPC error"

**Symptoms:**
- Error: "Invalid JSON response from http://127.0.0.1:8232"
- Error: "HTTP error 500 from http://127.0.0.1:8232"
- Malformed JSON in responses

**Solutions:**

1. **Check Zebra version compatibility:**
   ```bash
   zebrad --version
   # Ensure you're using Zebra 1.x or later
   # Update if needed:
   cargo install --force zebrad
   ```

2. **Check Zebra RPC endpoint:**
   ```bash
   # Verify RPC is responding correctly
   curl -X POST http://127.0.0.1:8232 \
        -H 'Content-Type: application/json' \
        -d '{"jsonrpc":"2.0","method":"getblockcount","params":[],"id":1}'
   
   # Should return valid JSON
   ```

3. **Check Zebra logs for errors:**
   ```bash
   # Linux/macOS
   tail -f ~/.cache/zebrad/debug.log
   # or
   journalctl -u zebrad -f  # if running as service
   
   # Look for RPC-related errors
   ```

4. **Restart Zebra cleanly:**
   ```bash
   # Stop Zebra gracefully
   pkill -TERM zebrad  # Linux/macOS
   
   # Wait a few seconds, then start
   zebrad start --rpc.bind-addr 127.0.0.1:8232
   ```

5. **Check for Zebra database corruption:**
   ```bash
   # If Zebra keeps failing, database might be corrupted
   # Backup and reset (WARNING: This will require re-sync)
   # Only do this if absolutely necessary
   mv ~/.cache/zebrad ~/.cache/zebrad.backup
   zebrad start --rpc.bind-addr 127.0.0.1:8232
   ```

#### Problem: "HTTP error 404" or "HTTP error 500" from Zebra

**Solutions:**

1. **Verify RPC endpoint URL:**
   ```bash
   # Check current configuration
   cargo run --bin nozy config --show
   
   # Ensure URL is correct:
   # http://127.0.0.1:8232 (not https://)
   # No trailing slash
   ```

2. **Check Zebra RPC configuration:**
   ```bash
   # Verify RPC is enabled in config
   grep -A 2 "\[rpc\]" ~/.config/zebrad.toml
   
   # Should show:
   # [rpc]
   # listen_addr = "127.0.0.1:8232"
   ```

3. **Test with different RPC method:**
   ```bash
   # Try a simple RPC call
   curl -X POST http://127.0.0.1:8232 \
        -H 'Content-Type: application/json' \
        -d '{"jsonrpc":"2.0","method":"getblockcount","params":[],"id":1}'
   ```

4. **Reconfigure Zebra RPC:**
   ```bash
   # Stop Zebra
   pkill zebrad
   
   # Edit config
   nano ~/.config/zebrad.toml
   
   # Add/update:
   [rpc]
   listen_addr = "127.0.0.1:8232"
   
   # Start Zebra
   zebrad start
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

#### Problem: "Invalid address format" or "Address parsing failed" or "Invalid recipient address"

**Symptoms:**
- Error: "Invalid recipient address. Must be a valid shielded address (u1...)"
- Error: "Transparent addresses (t1) are not supported"
- Error: "Recipient must include an Orchard receiver"
- Address validation fails

**Solutions:**

1. **Check address format:**
   - Orchard addresses start with `u1` (unified addresses)
   - NozyWallet **only supports shielded addresses** (Orchard)
   - Transparent addresses (t1) are **not supported** for privacy
   - Ensure no extra spaces, newlines, or special characters

2. **Verify address is a unified address:**
   ```bash
   # Generate a valid address to see the format
   cargo run --bin nozy addresses --count 1
   
   # Example valid address format:
   # u1testaddress1234567890abcdefghijklmnopqrstuvwxyz...
   ```

3. **Check for common mistakes:**
   ```bash
   # Remove any whitespace
   echo "u1..." | tr -d ' \n\r\t'
   
   # Check address length (should be ~78 characters for unified addresses)
   echo -n "u1..." | wc -c
   ```

4. **Validate address structure:**
   - Must start with `u1`
   - Must be valid bech32 encoding
   - Must contain Orchard receiver
   - No special characters except base32 characters

5. **Test with a known good address:**
   ```bash
   # Generate your own address first
   cargo run --bin nozy addresses --count 1
   
   # Use that address format as reference
   ```

6. **If using address from another wallet:**
   - Ensure it's a **shielded address** (u1...)
   - Not a transparent address (t1...)
   - Not a legacy address
   - Extract Orchard receiver from unified address if needed

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

### 9. Configuration Issues

#### Problem: "Configuration error" or "Invalid network setting"

**Symptoms:**
- Error when setting configuration
- Network setting not recognized
- Zebra URL not working

**Solutions:**

1. **Check current configuration:**
   ```bash
   cargo run --bin nozy config --show
   ```

2. **Reset to defaults:**
   ```bash
   # Remove config file (will use defaults)
   rm ~/.config/nozy/config.json  # Linux
   rm ~/Library/Application\ Support/com.nozy.nozy/config/config.json  # macOS
   rm %APPDATA%\nozy\config\config.json  # Windows
   ```

3. **Set network correctly:**
   ```bash
   # Must be either "mainnet" or "testnet"
   cargo run --bin nozy config --set-network mainnet
   # or
   cargo run --bin nozy config --set-network testnet
   ```

4. **Set Zebra URL:**
   ```bash
   # Use local Zebra
   cargo run --bin nozy config --use-local
   
   # Or set custom URL
   cargo run --bin nozy config --set-zebra-url http://127.0.0.1:8232
   ```

5. **Verify config file format:**
   ```bash
   # Check JSON is valid
   cat ~/.config/nozy/config.json | python -m json.tool
   ```

### 10. Windows-Specific Issues

#### Problem: "Path not found" or "Permission denied" on Windows

**Solutions:**

1. **Check file paths:**
   ```powershell
   # Windows uses backslashes in paths
   # Config location: %APPDATA%\nozy\config\config.json
   # Data location: %APPDATA%\nozy\data\
   
   # Check if directories exist
   Test-Path $env:APPDATA\nozy\config
   Test-Path $env:APPDATA\nozy\data
   ```

2. **Run PowerShell as Administrator (if needed):**
   ```powershell
   # Right-click PowerShell -> Run as Administrator
   # Then run cargo commands
   ```

3. **Check Windows Defender/Antivirus:**
   - May block wallet file access
   - Add exception for NozyWallet directory
   - Check Windows Defender logs

4. **Use Git Bash instead of PowerShell:**
   ```bash
   # Git Bash handles paths better for some operations
   # Install Git for Windows if needed
   ```

#### Problem: "link.exe not found" or "MSVC linker error" on Windows

**Solutions:**

1. **Install Visual Studio Build Tools:**
   ```powershell
   winget install Microsoft.VisualStudio.2022.BuildTools --override "--quiet --add Microsoft.VisualStudio.Workload.VCTools"
   ```

2. **Use Developer Command Prompt:**
   - Open "Developer Command Prompt for VS 2022"
   - Run cargo commands from there

3. **Set PATH manually:**
   ```powershell
   # Find link.exe location
   Get-ChildItem -Path "C:\Program Files\Microsoft Visual Studio" -Recurse -Filter "link.exe" | Select-Object -First 1
   
   # Add to PATH (replace with actual path)
   $env:PATH += ";C:\Program Files\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.xx.xxxxx\bin\Hostx64\x64"
   ```

### 11. Network and Proxy Issues

#### Problem: "Failed to connect" or "Proxy error"

**Solutions:**

1. **Check network connectivity:**
   ```bash
   # Test internet connection
   ping google.com
   
   # Test Zebra connection
   curl http://127.0.0.1:8232
   ```

2. **Check proxy settings:**
   ```bash
   # If behind a proxy, configure environment variables
   export HTTP_PROXY=http://proxy.example.com:8080
   export HTTPS_PROXY=http://proxy.example.com:8080
   export NO_PROXY=127.0.0.1,localhost
   ```

3. **Disable proxy for local connections:**
   ```bash
   export NO_PROXY=127.0.0.1,localhost,*.local
   ```

4. **Check firewall rules:**
   - Ensure port 8232 is allowed
   - Check both inbound and outbound rules
   - Verify localhost connections are allowed

### 12. Transaction-Specific Errors

#### Problem: "Transaction build failed" or "Cannot create proofs"

**Symptoms:**
- Error: "Cannot create proofs: proving system not ready"
- Transaction building fails
- Proving parameters missing

**Solutions:**

1. **Check proving status:**
   ```bash
   cargo run --bin nozy proving --status
   ```

2. **Download proving parameters:**
   ```bash
   # For testing
   cargo run --bin nozy proving --download
   
   # For production, download real parameters
   # See README.md for instructions
   ```

3. **Verify parameters are in correct location:**
   ```bash
   ls -la orchard_params/
   # Should see:
   # - orchard-spend.params
   # - orchard-output.params
   # - orchard-spend-verifying.key
   # - orchard-output-verifying.key
   ```

#### Problem: "Insufficient funds" when balance shows funds

**Solutions:**

1. **Check actual spendable balance:**
   ```bash
   cargo run --bin nozy balance
   # This shows spendable notes, not just total
   ```

2. **Account for transaction fees:**
   - Transaction requires: amount + fee
   - Check fee estimate:
   ```bash
   cargo run --bin nozy send --recipient "u1..." --amount 0.1
   # Will show fee estimate before sending
   ```

3. **Rescan for notes:**
   ```bash
   # Notes might not be detected yet
   cargo run --bin nozy scan --start-height 2000000 --end-height 2100000
   ```

4. **Check note maturity:**
   - Some notes may not be immediately spendable
   - Wait for confirmations
   - Check transaction history

This troubleshooting guide should help resolve most common issues. For additional help, refer to the GitHub issues or create a new discussion.
