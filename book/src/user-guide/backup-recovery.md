# Backup & Recovery

Your wallet can be recovered from a **24-word mnemonic** or from an **encrypted backup file**. Treat both like cash — anyone with the phrase or backup + password controls your funds.

## What to back up

| Method | What it restores | Best for |
|--------|------------------|----------|
| **Mnemonic (24 words)** | Full wallet on any device | Primary backup; disaster recovery |
| **Encrypted backup file** | Wallet file + local state | Same-machine restore, migration |
| **Profile folder** | One wallet profile’s data | Advanced / multi-wallet operators |

Mnemonic is **required**. File backup is optional but convenient for desktop users who use **Export backup** in Settings.

## Mnemonic backup (recommended)

1. Write down the **24 words** shown at wallet creation (or from Settings → Security → reveal seed, when implemented).
2. Store offline — paper or metal; never in cloud email or screenshots.
3. Verify: restore on a test profile or second device before relying on it.

```bash
# CLI restore
nozy restore
# Enter mnemonic and new password when prompted
```

## Desktop: export and restore

**Export**

1. Unlock the wallet.
2. Settings → **Backup** (or Security).
3. Choose a path and export — creates an encrypted backup via `export_backup`.

**Restore**

1. Welcome screen → **Restore from backup** (or Settings).
2. Select the backup file path (`restore_from_backup`).
3. Unlock with the password used when the backup was created.

After restore you must **unlock again** before send or sync.

## CLI and data paths

| OS | Wallet data | Config |
|----|-------------|--------|
| Windows | `%APPDATA%\nozy\nozy\data\` (per profile) | `%APPDATA%\nozy\nozy\config\config.json` |
| Linux / macOS | XDG data dir under `nozy/nozy/data/` | `~/.config/nozy/config.json` |

Multi-wallet **profiles** live under `{data}/profiles/{id}/`. Switching profiles in the desktop Welcome screen changes the active data directory.

## Recovery checklist

1. Install NozyWallet (CLI or desktop).
2. Point **Zebrad** at a synced node ([Zebra Node Setup](../advanced/zebra-node.md)).
3. **Restore** mnemonic or backup file.
4. **Unlock** with password.
5. **Sync to tip** (`nozy sync --to-tip` or desktop Sync).
6. Confirm balance and notes before sending.

## What backup does *not* include

- Your Zebrad chain data (re-sync the node separately).
- lightwalletd compact DB (re-sync via LWD if you use compact cache).
- On-chain funds — recovery only restores **keys**; you still need chain sync to see balance.

See also: [Backup Strategies](../security/backup-strategies.md), [Backup tutorial](../examples/backup-wallet.md).
