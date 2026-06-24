# BUG-2026-001: Send rescanned ~50k blocks despite synced wallet

**Status:** Fixed on `master` (unreleased tag at time of writing)  
**Severity:** P1  
**Surface:** api-server  
**Reporter:** Gilmore (VPS + self-hosted Zebrad testing)  
**GitHub issue:** _(file if/when created)_

---

## Summary

After sync showed healthy state (balance ~0.0025 ZEC, `last_scan_height` slightly behind tip), `POST /api/transaction/send` immediately began scanning ~50,025 blocks instead of reusing cached spendable notes.

---

## Root cause

`scan_notes_for_sending()` always rewound **50,000 blocks** from `last_scan_height` before building a transaction — a pre-persistence safety net that ignored cached `notes.json` and persisted Orchard witnesses.

---

## Fix

Send loads spendable notes directly from `notes.json` via `load_spendable_notes_from_wallet()`. Witness catch-up to chain tip happens at spend-build time only. Fallback scan uses incremental bounds (100-block reorg rewind), not a fixed 50k rewind.

**Code:** `src/notes.rs` — `load_spendable_notes_from_wallet`

---

## Expected after fix

Send reuses existing wallet state; at most a small incremental scan when cache is empty or witnesses missing.

---

## Verification

1. Sync wallet to healthy state with known balance.
2. `POST /api/transaction/send` — should not log 50k-block scan.
3. Send completes using cached note + witness catch-up.

---

## References

- [`CHANGELOG.md`](../../CHANGELOG.md) — [Unreleased]
- [`FORUM_ROADMAP_UPDATE_2026-06-20.md`](../FORUM_ROADMAP_UPDATE_2026-06-20.md)
