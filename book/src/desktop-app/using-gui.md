# Using the GUI

Overview of the NozyWallet desktop interface (Tauri app).

## Navigation

| Tab / area | Purpose |
|------------|---------|
| **Home** | Balance, recent activity, block sync strip, quick actions |
| **Send** | Recipient, amount, memo, contact picker |
| **Receive** | Unified Orchard address, copy |
| **History** | Sent and received transactions, explorer links |
| **Contacts** | Address book (add, edit, use in Send) |
| **Settings** | Network, security, backup, proving |

Header: **Sync** button, wallet lock, profile context.

## Home

- **Balance** — spendable shielded ZEC after sync.
- **Recent Activity** — latest txs; links to block explorer (`mainnet.zcashexplorer.app`).
- **Block sync panel** — blocks behind tip; detects stalled node if tip stops moving.

Background sync may run while unlocked (`useWalletAutoSync`).

## Send

1. Open **Send** tab (not only Home shortcut — full form on Send page).
2. Enter **recipient** (`u1…` mainnet Orchard unified) or pick from **Contacts**.
3. Amount in ZEC; optional memo.
4. Confirm password if locked.
5. Wait for prove + broadcast — first send can take several minutes (toast explains).

**Keystone (mainnet):** When enabled under **Settings → Keystone**, Send uses PCZT signing — prepare transaction, scan QR on Keystone, paste signed PCZT, broadcast. See [Keystone Hardware Wallet](../security/keystone-hardware-wallet.md).

Send is blocked if witness lag too high or node unreachable — sync first.

## Receive

Copy unified address. Only Orchard shielded receives are supported.

## History

- Sticky header with refresh.
- Sent vs received counts.
- **View on explorer** per transaction.

## Contacts

Add name + shielded address inline. Only `u1` / `zs1` style addresses accepted.

## Settings

| Section | Items |
|---------|--------|
| **Network** | `zebra_url`, Test Connection |
| **Security** | Password change, seed reveal (when enabled), auto-lock toggles |
| **Keystone** | UFVK export, enable air-gapped PCZT signing (mainnet) — [guide](../security/keystone-hardware-wallet.md) |
| **Backup** | Export / restore backup file |
| **Proving** | Parameter download status |

## Sync behavior

**Sync** runs catch-up until scan gap closes. “Synced successfully” should align with tip — not merely one RPC ping. See sync helpers in app (`syncWalletToTip`, honest status messages).

## Lock / unlock

Lock clears in-memory session. Unlock required for send, seed reveal, and some settings.

## What not to do

- Don’t use **browser tab** at `localhost:5173` for real funds.
- Don’t skip mnemonic backup at creation.
- Don’t send while node tip is behind wallet scan height (node still syncing).

## Related

- [Sending ZEC](../user-guide/sending-zec.md)
- [Keystone Hardware Wallet](../security/keystone-hardware-wallet.md)
- [Troubleshooting](troubleshooting.md)
- [Desktop README](../../../desktop-client/README.md)
