# First-Time Setup

Walkthrough from install to first synced wallet.

## 1. Install the app

See [Installation](installation.md). Start with `cargo tauri dev` or the release installer.

## 2. Connect Zebrad

Before creating a wallet, ensure your node works:

```powershell
nozy test-zebra
```

Or after wallet exists: **Settings → Network** → set URL → **Test Connection**.

WSL on Windows: use WSL IP in URL ([connectivity guide](../../../docs/reference/ZEBRAD_NOZYWALLET_CONNECTIVITY.md)).

## 3. Create or restore

**Welcome screen options:**

| Action | When |
|--------|------|
| **Create new wallet** | First time on this profile |
| **Restore wallet** | Have 24-word phrase |
| **Restore from backup** | Have encrypted backup file |
| **Unlock** | Wallet already on disk |

### Create

1. Choose a strong password.
2. **Write down 24 words** — only chance shown at creation.
3. Confirm mnemonic if prompted.
4. Land on Home.

### Restore

1. Enter 24 words in order.
2. Set password.
3. Sync to tip.

## 4. Multi-wallet profiles

Welcome lists existing **profiles** if any. Select a profile before unlock. Each profile has separate keys and data.

Create additional profiles from Welcome (create/restore flow) without deleting others.

## 5. Initial sync

1. Home → **Sync** (header or layout).
2. Wait until block sync strip shows caught up.
3. Balance updates when notes are indexed.

CLI equivalent: `nozy sync --to-tip`.

## 6. Proving parameters

First shielded send may prompt or auto-download Orchard params. Settings → Proving, or:

```bash
nozy proving --download
```

## 7. Receive test (optional)

**Receive** tab → copy unified address (`u1…`) → fund from faucet or exchange (shielded withdrawal).

## Checklist

- [ ] Zebrad RPC verified
- [ ] Mnemonic backed up offline
- [ ] Sync to tip complete
- [ ] Using desktop window (not browser dev URL)

Next: [Using the GUI](using-gui.md) | [Troubleshooting](troubleshooting.md)
