# Restore from Backup

Recover a wallet from a **mnemonic** or **encrypted backup file**.

## Restore from mnemonic

### Desktop

1. Launch NozyWallet (taskbar window, not browser tab).
2. Welcome → **Restore wallet**.
3. Enter all **24 words** in order.
4. Set a **new password** (can match old password).
5. Unlock → **Sync** to tip.

### CLI

```bash
nozy restore
# Paste mnemonic when prompted
nozy sync --to-tip
nozy balance
```

## Restore from backup file (desktop)

1. Welcome or Settings → **Restore from backup**.
2. Select the `.backup` / wallet backup file path.
3. Enter password used when backup was created.
4. Unlock again (session cleared after restore).
5. Sync to tip before sending.

## Restore on a new computer

1. Install NozyWallet ([Installation](../getting-started/installation.md)).
2. Install or connect to **Zebrad** ([Set Up Your Own Node](own-node.md)).
3. Set `zebra_url` in config or Settings.
4. Restore mnemonic or file.
5. Run `nozy test-zebra` or Settings → Test Connection.
6. Sync until `last_scan_height` matches chain tip.

## Multi-wallet profiles

If you used **multiple profiles** on the old machine:

- Each profile has its own mnemonic.
- Restore creates a new profile entry — name it to match your old label.
- Old profile IDs are not preserved; data paths are new UUIDs.

## Troubleshooting

| Issue | Fix |
|-------|-----|
| Invalid mnemonic | Check word order and BIP39 word list |
| Wrong password on file restore | Use password from backup time |
| Balance zero after restore | Sync to tip; wait for witness refresh |
| NET_001 | Fix Zebrad URL before sync |

See [Backup & Recovery](../user-guide/backup-recovery.md).
