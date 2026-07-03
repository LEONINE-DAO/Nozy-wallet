# Transaction History

View sent and received shielded activity in NozyWallet.

## Desktop

**History** tab shows:

- Sent and received transactions
- Block height and status
- Links to [mainnet.zcashexplorer.app](https://mainnet.zcashexplorer.app) for TXIDs
- Refresh control in header

**Home → Recent Activity** shows the latest entries; full list on History.

Sorting is by **block height** (newest first), then timestamp, then txid.

## CLI

```bash
nozy history
```

## What appears in history

| Type | Source |
|------|--------|
| **Sent** | Locally recorded broadcasts (`sent_transactions.json` / profile store) |
| **Received** | Detected during sync when notes match your addresses |

Very old receives may show **Block N** if broadcast timestamp was unavailable (epoch placeholder omitted in UI).

## Confirmations

```bash
nozy check-confirmations -t <txid>
```

Or open explorer link from History.

## Export

Export to CSV/file is roadmap item — copy TXIDs from History or use explorer for now.

## Related

- [Sending ZEC](sending-zec.md)
- [Receiving ZEC](receiving-zec.md)
