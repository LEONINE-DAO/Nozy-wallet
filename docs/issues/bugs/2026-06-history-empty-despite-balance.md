# BUG-2026-002: Transaction history empty despite persisted balance

**Status:** Fixed on `master` (unreleased tag at time of writing)  
**Severity:** P1  
**Surface:** api-server  
**Reporter:** Gilmore (VPS testing)  
**GitHub issue:** _(file if/when created)_

---

## Summary

`/api/sync` and `/api/wallet/status` showed correct balance and sync height, but `GET /api/transaction/history` returned `{ "transactions": [], "total": 0 }` — no received deposit appeared.

---

## Root cause

History endpoints read only `SentTransactionStorage` (outgoing txs recorded at broadcast). Received deposits live in persisted `notes.json` and were never merged into history views.

---

## Fix

Added `collect_wallet_transaction_views()` to merge sent records with received deposits grouped by txid from `notes.json`. Endpoints updated:

- `GET /api/transaction/history`
- `GET /api/transaction/{txid}`
- `GET /api/wallet/status` (`total_transactions`)
- `web_read_state`

Responses include `transaction_type: "Received"` or `"Sent"`.

---

## Expected after fix

Detected deposits appear in history with txid, amount, block height, confirmations.

---

## Verification

1. Receive shielded ZEC; sync until balance updates.
2. `GET /api/transaction/history` — at least one `Received` entry.

---

## References

- [`CHANGELOG.md`](../../CHANGELOG.md) — [Unreleased]
