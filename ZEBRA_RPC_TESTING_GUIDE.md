# 🔍 Zebra Node RPC Compatibility Testing Guide

This guide will help you verify that your Zebra node is running and compatible with NozyWallet's RPC interface.

## 🚀 Quick Start

### Option 1: PowerShell Script (Recommended for Windows)
```powershell
# Run the comprehensive test script
.\test_zebra_node.ps1

# Or test a specific URL
.\test_zebra_node.ps1 http://your-zebra-node:8232
```

### Option 2: Rust Binaries
```bash
# Quick connectivity test
cargo run --bin quick_test

# Comprehensive diagnostic
cargo run --bin diagnose_zebra

# Test with custom URL
cargo run --bin quick_test http://your-zebra-node:8232
```

### Option 3: Manual Testing
```bash
# Test basic RPC connectivity with curl
curl -H 'content-type: application/json' \
     --data-binary '{"jsonrpc": "2.0", "method": "getblockcount", "params": [], "id":1}' \
     http://127.0.0.1:8232
```

## 📋 What the Tests Check

### 1. Basic Connectivity
- ✅ RPC endpoint accessibility
- ✅ JSON-RPC 2.0 protocol compliance
- ✅ Block count retrieval
- ✅ Node synchronization status

### 2. Standard RPC Methods
- ✅ `getblockcount` - Get current block height
- ✅ `getnetworkinfo` - Network information
- ✅ `getmempoolinfo` - Mempool statistics
- ✅ `gettxoutsetinfo` - UTXO set information
- ✅ `getblocktemplate` - Mining template
- ✅ `estimatefee` - Fee estimation

### 3. Orchard-Specific Functionality
- ✅ `get_orchard_tree_state()` - Orchard tree state
- ✅ `get_note_position()` - Note position calculation
- ✅ `get_authentication_path()` - Merkle path generation

## 🔧 Troubleshooting

### If Tests Fail:

#### 1. Check if Zebra is Running
```bash
# Windows
Get-Process | Where-Object {$_.ProcessName -like '*zebra*'}

# Linux/Mac
ps aux | grep zebrad
```

#### 2. Check Zebra Configuration
Edit `~/.config/zebrad.toml`:
```toml
[rpc]
listen_addr = "127.0.0.1:8232"
```

#### 3. Check Port Accessibility
```bash
# Windows
netstat -an | findstr 8232

# Linux/Mac
netstat -tlnp | grep 8232
```

#### 4. Check Zebra Logs
```bash
# Check debug logs
tail -f ~/.cache/zebrad/debug.log
```

#### 5. Verify Zebra Version
```bash
zebrad --version
```

## 📊 Expected Output

### Successful Test Output:
```
🔍 Zebra Node RPC Compatibility Test
====================================

Testing Zebra node at: http://127.0.0.1:8232

1️⃣ Testing basic RPC connectivity with curl...
✅ RPC endpoint is accessible
   Block count: 1234567
   ✅ Node is synchronized

2️⃣ Testing additional RPC methods...
   Testing getnetworkinfo... ✅
   Testing getmempoolinfo... ✅
   Testing gettxoutsetinfo... ✅

3️⃣ Running Rust-based compatibility tests...
   ✅ Rust tests passed

4️⃣ Running comprehensive diagnostic...
   ✅ Comprehensive tests passed

🎉 Zebra node compatibility test completed!

If all tests passed, your Zebra node is compatible with NozyWallet!
```

### Failed Test Output:
```
❌ Cannot connect to RPC endpoint
Error: Connection refused

Troubleshooting:
1. Is Zebra running? Check with: Get-Process | Where-Object {$_.ProcessName -like '*zebra*'}
2. Is RPC enabled? Check ~/.config/zebrad.toml for:
   [rpc]
   listen_addr = "127.0.0.1:8232"
3. Is port 8232 open? Check with: netstat -an | findstr 8232
```

## 🎯 RPC Method Compatibility

### Standard Zebra RPC Methods (✅ Supported)
- `getblockcount` - Returns current block height
- `getnetworkinfo` - Returns network information
- `getmempoolinfo` - Returns mempool statistics
- `gettxoutsetinfo` - Returns UTXO set information
- `getblocktemplate` - Returns mining template
- `estimatefee` - Returns fee estimates
- `getrawtransaction` - Returns raw transaction data
- `decoderawtransaction` - Decodes raw transactions

### NozyWallet Custom Methods (🔧 Implemented)
- `get_orchard_tree_state()` - Generates tree state from block data
- `get_note_position()` - Calculates note position deterministically
- `get_authentication_path()` - Generates auth path deterministically

## 🔄 Next Steps

1. **Run the tests** using one of the methods above
2. **Fix any issues** using the troubleshooting guide
3. **Verify compatibility** by ensuring all tests pass
4. **Start using NozyWallet** with your Zebra node!

## 📞 Support

If you encounter issues:
1. Check the troubleshooting section above
2. Verify your Zebra node is properly configured
3. Ensure your Zebra version is compatible
4. Check the Zebra documentation for RPC configuration

---

**Note**: The Orchard-specific methods use deterministic algorithms since the exact RPC methods for Orchard tree state are not available in standard Zebra RPC. This provides a working foundation that can be enhanced as more specific RPC methods become available.
