# Security Features

NozyWallet implements enterprise-grade security to protect your funds and personal information.

## Security Architecture

### Self-Custodial

You maintain **full control** over your funds:

- **Private keys stored locally** - Never leave your device
- **No third-party custody** - You control your keys
- **Encrypted storage** - Wallet files encrypted with strong encryption
- **No cloud sync** - Your wallet data stays on your device

### Encryption

All sensitive data is encrypted:

- **Wallet files** - Encrypted at rest with AES-256-GCM
- **Password hashing** - Argon2 key derivation function
- **Memory protection** - Sensitive data cleared from memory
- **Secure key derivation** - BIP39 + BIP32 hierarchical derivation

## Password Protection

### Strong Password Requirements

- Minimum length requirements
- Recommended: Use a password manager
- Unique password: Don't reuse passwords

### Password Storage

- **Never stored in plaintext** - Passwords are hashed with Argon2
- **Salt protection** - Unique salt per wallet
- **Memory-only** - Passwords never written to disk

## Private Key Management

### Key Generation

- **Cryptographically secure** - Uses secure random number generation
- **BIP39 mnemonic** - 24-word seed phrase
- **BIP32 HD wallets** - Hierarchical deterministic key derivation
- **No key reuse** - New keys for each transaction

### Key Storage

- **Encrypted wallet files** - Keys encrypted with your password
- **Secure file permissions** - Wallet files have restricted access
- **Never transmitted** - Keys never leave your device
- **Memory isolation** - Keys cleared when not in use

## Backup & Recovery

### Seed Phrase Backup

- **24-word mnemonic** - Standard BIP39 phrase
- **Offline storage** - Write down, never store digitally
- **Multiple copies** - Store in secure locations
- **Test recovery** - Verify backup works

See [Backup Strategies](security/backup-strategies.md) for detailed guidance.

## Security Best Practices

### For Users

1. ✅ **Use strong passwords** - Unique, complex passwords
2. ✅ **Backup your seed phrase** - Store securely offline
3. ✅ **Keep software updated** - Get latest security patches
4. ✅ **Verify downloads** - Check SHA256 hashes
5. ✅ **Use your own node** - Maximum privacy and security
6. ✅ **Don't share your mnemonic** - Never share with anyone

### For Developers

1. ✅ **Code audits** - Regular security reviews
2. ✅ **Dependency updates** - Keep dependencies current
3. ✅ **Vulnerability scanning** - Automated security scanning
4. ✅ **Secure coding practices** - Follow Rust security guidelines

## Security Audits

### Self-Audit Status

✅ **Completed** - Comprehensive self-security audit performed

See [Security Audits](security/audits.md) for details.

### Third-Party Audit

🔄 **Planned** - Professional third-party audit scheduled before production release

## Open Source Security

### Transparency

- **Open source code** - Anyone can review
- **Public repository** - Full transparency
- **Community review** - Many eyes on the code
- **Bug bounty** - (If implemented)

### Auditable

- **Git history** - Complete development history
- **Change tracking** - All changes reviewed
- **Code signing** - Verified releases
- **Reproducible builds** - Build from source

## Threat Model

### What We Protect Against

- ✅ **Wallet file theft** - Encrypted, password-protected
- ✅ **Password brute force** - Argon2 slow hashing
- ✅ **Network attacks** - Encrypted connections
- ✅ **Malware** - No key extraction possible without password
- ✅ **Transaction replay** - Unique transactions per block

### What Users Must Protect

- ⚠️ **Mnemonic phrase** - User must secure offline
- ⚠️ **Password** - User must use strong password
- ⚠️ **Device security** - User must secure their device
- ⚠️ **Social engineering** - User must not share credentials

## Reporting Security Issues

If you discover a security vulnerability:

1. **Do not** create a public issue
2. **Email** security team at: (add contact)
3. **Include** details and steps to reproduce
4. **Wait** for response before disclosure

We follow responsible disclosure practices.

## Learn More

- [Security Best Practices](security/best-practices.md)
- [Private Key Management](security/key-management.md)
- [Security Audits](security/audits.md)
