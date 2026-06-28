# BUG-2026-013: Wallet status synced but send fails on stale Orchard witness

**Status:** Fixed in **v2.3.6.7**  
**Severity:** P1  
**Surface:** core + api-server  
**Reporter:** Gilmore (VPS + self-hosted Zebrad testing)  
**GitHub issue:** PR #108

---

## Summary

`/api/wallet/status` reported fully synced (`blocks_behind: 0`, `last_sync_height` equals chain tip), but `POST /api/transaction/send` failed immediately:

```text
Orchard witness is 10772 blocks behind chain tip (witness tip 3379050, chain tip 3389822)
```

Repeated `POST /api/sync` calls kept confirming synced while send continued to fail with the same witness lag.

---

## Root cause

Two separate gaps:

1. **Status endpoint** only compared `last_scan_height` to chain tip — it did not inspect `orchard_witness_tip_height` on cached notes.
2. **Sync path** advanced the RPC scan checkpoint without refreshing witnesses on existing cached notes. Witness catch-up when scan was already at tip only ran when `scan_to_tip` was set; API sync never set that flag.

Incremental block scans update witnesses only for notes rediscovered in the scanned range, not for older spendable notes already in `notes.json`.

---

## Fix

- **`src/wallet_sync.rs`:** refresh all cached note witnesses after each scan; batch witness catch-up (1000 blocks per sync) when RPC scan is at tip.
- **`src/orchard_tx.rs`:** `refresh_cached_witnesses_to_tip`.
- **`api-server`:** `scan_to_tip` when `end_height` omitted; status exposes `witness_lag_blocks`, `witness_fresh_for_send`, `ready_for_send`.

---

## Expected after fix

1. Rebuild and restart `nozywallet-api` from **v2.3.6.7** (or `master` with PR #108).
2. Run `POST /api/sync` until `/api/wallet/status` shows `ready_for_send: true`.
3. Send succeeds without witness-stale guard (lag ≤ 50 blocks).

---

## Verification

1. Wallet with stale witness lag but caught-up scan height.
2. `GET /api/wallet/status` — `blocks_behind: 0` but `witness_lag_blocks` > 0 and `ready_for_send: false`.
3. Repeat `POST /api/sync` — witness lag decreases each call.
4. When `ready_for_send: true`, `POST /api/transaction/send` proceeds past witness guard.
