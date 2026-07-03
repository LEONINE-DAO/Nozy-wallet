# Backup Your Wallet

Step-by-step: create a durable backup before holding funds on mainnet.

## Before you start

- Wallet created and unlocked once (so you have seen the mnemonic).
- Pen and paper or offline storage ready.

## Option A — Mnemonic (all surfaces)

### Desktop

1. Create wallet on Welcome → **Create new wallet**.
2. Copy the **24 words** to paper when shown.
3. Complete the confirmation step if prompted.
4. Store paper offline.

### CLI

```bash
nozy new
# Save the printed mnemonic immediately
```

## Option B — Encrypted file (desktop)

1. Unlock wallet.
2. Open **Settings** → backup / security section.
3. Choose **Export backup** and pick a folder (e.g. encrypted USB).
4. Note the output path from the success message.

The backup requires your **wallet password** to restore.

## Verify the backup

**Mnemonic test (recommended):**

1. On a second profile or machine, choose **Restore**.
2. Enter the 24 words.
3. Sync to tip and confirm the **same receiving address** as the original wallet.

**File test:**

1. Use a test profile.
2. Restore from backup file.
3. Unlock and sync.

## Checklist

- [ ] 24 words written and stored in two places
- [ ] Optional encrypted export on offline media
- [ ] Test restore completed
- [ ] Zebrad connectivity verified ([own node tutorial](own-node.md))

Next: [Restore from Backup](restore-from-backup.md) | [Backup & Recovery](../user-guide/backup-recovery.md)
