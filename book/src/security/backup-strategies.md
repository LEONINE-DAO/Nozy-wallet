# Backup Strategies

NozyWallet is **self-custodial**. If you lose your mnemonic and have no backup, funds are **unrecoverable**.

## Strategy 1 — Mnemonic only (minimum)

- Write the **24-word phrase** at creation.
- Store in **two physically separate** safe locations (e.g. home safe + bank box).
- Never type it into websites, chat, or cloud notes.

**Pros:** Works everywhere (CLI, desktop, future mobile).  
**Cons:** Single point of failure if one copy is lost or stolen.

## Strategy 2 — Mnemonic + encrypted file backup

- Keep mnemonic as primary.
- Periodically **export encrypted backup** from desktop (or CLI backup commands when available).
- Store backup file on encrypted USB or offline media — **not** the same bag as the paper mnemonic.

**Pros:** Faster restore on same machine; includes local note index state.  
**Cons:** File still needs your wallet password; less portable than mnemonic alone.

## Strategy 3 — Test restore

Before holding meaningful funds:

1. Create a **test wallet** or profile.
2. Record mnemonic.
3. Delete profile or use a second machine.
4. **Restore** and sync.
5. Confirm addresses match.

Do this once per major version upgrade if you change backup tooling.

## Operational rules

| Do | Don't |
|----|-------|
| Verify words in order | Share seed with “support” or DMs |
| Use strong wallet password | Reuse exchange passwords |
| Label which profile is which (multi-wallet) | Store seed in password managers synced to cloud |
| Re-backup after password change | Assume `wallet.dat` copy alone replaces mnemonic |

## Multi-wallet profiles

Desktop supports **multiple profiles** under one install. Each profile has its own mnemonic and data folder. Back up **each** profile you use — switching profile does not merge backups.

## Related

- [Backup & Recovery](../user-guide/backup-recovery.md)
- [Private Key Management](key-management.md)
- [Wallet Storage](wallet-storage.md)
