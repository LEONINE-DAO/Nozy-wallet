# NU 6.1 Support

NozyWallet is fully compatible with Zcash Network Upgrade 6.1 (NU 6.1).

## What is NU 6.1?

Network Upgrade 6.1 is the latest Zcash network upgrade that brings:
- **Protocol Version 170140** - Latest protocol features
- **Activation Height** - Block 3,146,400 (November 23, 2025)
- **New Features** - ZIP 271, ZIP 1016 support
- **Performance Improvements** - Better efficiency and scalability

## NozyWallet Compatibility

✅ **Fully Compatible** - NozyWallet is ready for NU 6.1:
- Protocol version 170140 support
- Latest libraries (`zcash_protocol 0.6.2+`, `zcash_primitives 0.24.1+`)
- All NU 6.1 features supported
- Automatic upgrade handling

## Checking NU 6.1 Status

### CLI

**Check NU 6.1 status:**
```bash
nozy nu61
```

**Or check wallet status:**
```bash
nozy status
```

**Example output:**
```
✅ NU 6.1 Support: Enabled
Protocol Version: 170140
Activation Height: 3,146,400
Current Height: 3,147,200
Status: Active
```

## What Changed?

### For Users

**No action required** - NozyWallet automatically:
- ✅ Uses latest protocol version
- ✅ Supports new features
- ✅ Handles upgrade seamlessly
- ✅ Maintains compatibility

**Benefits:**
- Better performance
- Improved privacy
- Enhanced features
- Future-proof wallet

### For Developers

**Technical changes:**
- Updated protocol version
- New ZIP support
- Enhanced libraries
- Improved APIs

## Upgrade Timeline

### Before Activation (Block < 3,146,400)
- Wallet uses pre-NU 6.1 protocol
- All features work normally
- Upgrade preparation in place

### After Activation (Block >= 3,146,400)
- Wallet automatically uses NU 6.1
- New features activated
- Full compatibility maintained

### Current Status

**Check current status:**
```bash
nozy nu61
```

This shows:
- Current block height
- Activation status
- Protocol version in use

## Features Enabled

### ZIP 271 Support

ZIP 271 brings:
- Enhanced transaction formats
- Better compatibility
- Improved privacy features

### ZIP 1016 Support

ZIP 1016 enables:
- New transaction types
- Enhanced functionality
- Better interoperability

## Migration Guide

### For Existing Wallets

**No migration needed** - Existing wallets work automatically:
- ✅ Old wallets compatible
- ✅ Automatic upgrade
- ✅ No manual steps required
- ✅ All features available

### For New Wallets

**Standard creation process** - New wallets use NU 6.1 by default:
- ✅ Latest protocol from start
- ✅ All features enabled
- ✅ Future-proof setup

## Troubleshooting

### "Protocol version mismatch" Error

**Problem:** Wallet and node use different protocol versions.

**Solutions:**
1. **Update NozyWallet** - Ensure latest version
2. **Update Zebra node** - Ensure node supports NU 6.1
3. **Check network** - Verify mainnet/testnet matches
4. **Sync wallet** - Run sync to update

### "NU 6.1 not active" Warning

**Problem:** Network upgrade hasn't activated yet.

**Solutions:**
- Wait for activation height (block 3,146,400)
- Check current block height
- Verify activation date (November 23, 2025)

## Learn More

- [Zcash NU 6.1 Documentation](https://zcash.github.io/zcash/) - Official Zcash documentation
- [ZIP 271](https://zips.z.cash/zip-0271) - ZIP 271 specification
- [ZIP 1016](https://zips.z.cash/zip-1016) - ZIP 1016 specification
- [Network Upgrades](https://zcash.readthedocs.io/en/latest/rtd_pages/network_upgrade.html) - Zcash network upgrade guide
