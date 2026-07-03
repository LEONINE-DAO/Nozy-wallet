# Transaction Endpoints

## `POST /api/transaction/send`

Shielded Orchard send.

Body (typical):

```json
{
  "recipient": "u1…",
  "amount": 0.0001,
  "memo": "optional",
  "password": "…",
  "zebra_url": "optional override"
}
```

Response: `{ "success": true, "txid": "…" }` or error with message/code.

## Send guards

Same as CLI:

- Witness lag limit
- Sync-to-tip recommendations
- Insufficient funds
- Invalid unified address

## Confirmations

Use Zebrad `getrawtransaction` or explorer; optional CLI `check-confirmations`.

## History

Local sent tx list may be exposed in extended API builds — check handlers. Desktop uses Tauri `get_transaction_history`.

CLI: `nozy history`.

See [Transaction Commands](../cli/transaction-commands.md).
