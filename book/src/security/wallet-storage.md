# Wallet Storage

NozyWallet stores wallet data securely using encryption.

## Encryption

- **Algorithm**: AES-256-GCM (Galois/Counter Mode)
- **Key Derivation**: Encryption key derived from password using PBKDF2
- **Storage Location**: `wallet_data/wallet.dat`

## What's Encrypted

- Wallet mnemonic phrase
- Generated addresses
- Private key material
- Note data

## Backup

You can backup your wallet by:
1. **Mnemonic Phrase**: The 24-word phrase can restore your entire wallet
2. **Wallet File**: Copy `wallet_data/wallet.dat` (requires password to decrypt)

## Security Features

- **No Plain Text**: Private keys are never stored in plain text
- **Encrypted at Rest**: All wallet data is encrypted on disk
- **Key Derivation**: Keys are derived from mnemonic using BIP32
- **Secure Deletion**: When possible, sensitive data is securely wiped from memory

## Best Practices

1. **Backup your mnemonic phrase** in a secure location
2. **Use strong passwords** for wallet encryption
3. **Keep backups secure** - encrypted wallet files still require password
4. **Test restoration** - verify you can restore from mnemonic before relying on it
