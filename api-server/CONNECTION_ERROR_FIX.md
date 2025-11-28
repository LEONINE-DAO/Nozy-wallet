# Fixing Cargo Connection Errors

If you're getting connection errors when running `cargo build --release` in the api-server directory, here are the most common solutions:

## Quick Fixes

### 1. Use Offline Mode (if dependencies are already cached)
If you've built before and dependencies are cached, try building offline:

```powershell
cd api-server
cargo build --release --offline
```

### 2. Increase Network Timeout
If you have a slow connection, increase the timeout:

```powershell
$env:CARGO_NET_TIMEOUT = "300"
cd api-server
cargo build --release
```

### 3. Clear Cache and Retry
Sometimes clearing the cache helps:

```powershell
cd api-server
cargo clean
cargo build --release
```

### 4. Use a Registry Mirror (if behind firewall)
If you're behind a corporate firewall or in a region with restricted access:

1. Create `C:\Users\<YourUsername>\.cargo\config.toml`
2. Add this content:

```toml
[registry]
default = "rsproxy"

[registries.rsproxy]
index = "https://rsproxy.cn/crates.io-index"
```

Then rebuild:

```powershell
cd api-server
cargo build --release
```

## Diagnostic Script

Run the diagnostic script to check your setup:

```powershell
cd api-server
.\fix_connection_error.ps1
```

This will check:
- Network connectivity to crates.io
- Cargo configuration
- Cached dependencies
- Proxy settings
- Cargo registry access

## Common Error Messages and Solutions

### "failed to fetch"
- **Cause**: Network connectivity issue
- **Solution**: Check internet connection, try offline mode if cached

### "timeout"
- **Cause**: Slow connection or network issues
- **Solution**: Increase `CARGO_NET_TIMEOUT` environment variable

### "connection refused"
- **Cause**: Firewall or proxy blocking
- **Solution**: Configure proxy settings or use registry mirror

### "SSL/TLS error"
- **Cause**: Certificate or proxy issues
- **Solution**: Check proxy settings, update certificates

## Check Your Network

Test connectivity to crates.io:

```powershell
Test-NetConnection -ComputerName crates.io -Port 443
```

## Still Having Issues?

1. Check if you're behind a corporate firewall/VPN
2. Verify your internet connection is working
3. Try building from a different network
4. Check Windows Firewall settings for cargo.exe
5. Check antivirus software isn't blocking cargo

## Alternative: Build Without Network

If you have all dependencies cached, you can build completely offline:

```powershell
cd api-server
cargo build --release --offline --frozen
```

The `--frozen` flag ensures Cargo.lock is used exactly as-is without updating.

