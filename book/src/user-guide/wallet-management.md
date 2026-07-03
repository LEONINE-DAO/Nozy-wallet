# Wallet Management

Manage wallets and profiles in NozyWallet.

## Single wallet vs profiles

- **CLI** traditionally uses one active data directory under the platform Nozy data path.
- **Desktop** supports **multiple profiles** — each with its own mnemonic, `wallet.dat`, notes, and sync state.

Profiles live under:

- Windows: `%APPDATA%\nozy\nozy\data\profiles\{profile-id}\`
- Manifest tracks active profile id

## Desktop: switch profile

1. **Lock** the current wallet (or quit).
2. Welcome screen lists profiles with labels.
3. Select profile → **Unlock** with that profile’s password.

Creating a new wallet on Welcome adds a **new profile** without removing others.

## Create additional wallet

Welcome → **Create new wallet** → new password + new mnemonic. Back up the new mnemonic separately.

## Rename / organize

Use profile labels shown on Welcome (implementation may use profile name from manifest). Keep a personal map of which mnemonic belongs to which label.

## Lock and password

- **Lock** clears session keys from memory; data on disk stays encrypted.
- **Change password** (Settings → Security) re-encrypts storage — ensure you have mnemonic backup first.

## Delete a profile

No one-click delete in all builds — advanced users remove the profile folder under `profiles/` while app is closed. **Only if mnemonic is backed up.**

## CLI wallet path

Config and data paths: [Backup & Recovery](backup-recovery.md).

## Related

- [Creating Your First Wallet](../getting-started/creating-wallet.md)
- [Backup Strategies](../security/backup-strategies.md)
