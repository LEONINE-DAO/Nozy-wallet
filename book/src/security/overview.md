# Security Overview

NozyWallet is designed with security as a top priority. This section covers the security features and best practices.

## Security Features

- **Password Protection**: Argon2-based password hashing
- **Encrypted Storage**: AES-256-GCM encryption for wallet files
- **Private Key Management**: Keys never stored in plain text
- **Secure Key Derivation**: BIP32/BIP39 standard key derivation

## Best Practices

1. **Always use a strong password** when creating or restoring a wallet
2. **Backup your mnemonic phrase** in a secure location
3. **Never share your mnemonic phrase** or private keys
4. **Verify addresses** before sending transactions
5. **Test on testnet first** before using mainnet

## Next Steps

- [Password Protection](password-protection.md) - How passwords secure your wallet
- [Wallet Storage](wallet-storage.md) - How wallet data is encrypted
