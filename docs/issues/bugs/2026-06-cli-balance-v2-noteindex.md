# BUG-2026-012: CLI balance shows 0 on v2 `notes.json` (legacy array parser)

**Status:** Fixed on `master` (unreleased tag at time of writing)  
**Severity:** P1  
**Surface:** cli (`nozy balance`, `nozy status`)  
**Reporter:** Internal (mainnet send-readiness testing, June 2026)  
**GitHub issue:** _(file if/when created)_

---

## Summary

After sync and successful mainnet sends, **`nozy balance`** and the balance section of **`nozy status`** could report **0.00000000 ZEC** even though `notes.json` held unspent Orchard notes and api-server `/api/balance` showed the correct amount.

---

## Environment

| Field | Value |
|-------|--------|
| Nozy version | `master` post send-readiness work (June 2026) |
| OS | Windows host, WSL Zebrad |
| Network | mainnet |
| `notes.json` format | **v2 `NoteIndex`** (migrated from legacy array) |

---

## Steps to reproduce

1. Sync wallet until `notes.json` is written in **v2** format (`version: 2`, `nullifier_index`, etc.).
2. Confirm api-server balance is correct: `curl http://localhost:3000/api/balance`.
3. Run `cargo run --release --bin nozy -- balance`.

---

## Expected behavior

CLI shows the same unspent Orchard total as api-server: sum of **unspent** notes from `notes.json`, minus pending outbound sends for **available** balance.

---

## Actual behavior

CLI printed **0 ZEC** (or wrong total) because it parsed `notes.json` as a **top-level JSON array** of note objects. v2 files are a **JSON object** (`NoteIndex`), so `.as_array()` returned nothing and the sum was 0.

Legacy array wallets could also **over-count** balance by summing **all** `value` fields without checking `spent: true`.

---

## Root cause

**Two balance code paths** after the NoteIndex v2 migration:

| Path | Used by | Behavior |
|------|---------|----------|
| `load_wallet_notes()` → `wallet_unspent_balance_zatoshis()` | api-server, `cached_unspent_balance_zatoshis()`, send pipeline | Correct: v2 + legacy migration, excludes spent notes |
| Manual `serde_json::Value` / `Vec` parse of `notes.json` | `nozy balance`, `nozy status` balance section | Broken on v2; ignored `spent` on legacy |

This is the same **“dual parser”** class of bug as post-send `mark_wallet_notes_spent` (fixed separately via `NoteIndex::load_from_file` / `save_to_file`).

---

## Fix

1. Added **`WalletBalanceSnapshot`** and **`wallet_balance_snapshot()`** in `src/cli_helpers.rs`:
   - `confirmed_zatoshis` — unspent notes from cache  
   - `pending_zatoshis` — in-flight sends (amount + fee) from `SentTransactionStorage`  
   - `available_zatoshis` — confirmed − pending  
   - `unspent_note_count`

2. **`nozy balance`** and **`nozy status`** now call `wallet_balance_snapshot()` instead of hand-parsing JSON.

3. Exported from `nozy` crate for reuse by api-server / desktop later (**CLI-first rollout**).

---

## Expected after fix

```text
💰 Balance Information
==================================================
   Confirmed: 0.00640000 ZEC (2 unspent notes)
   Available: 0.00640000 ZEC
```

With a pending send:

```text
   Confirmed: 0.00640000 ZEC (2 unspent notes)
   Pending:   -0.00020000 ZEC
   Available: 0.00620000 ZEC
```

---

## Verification

1. Wallet with v2 `notes.json` and known balance.
2. `nozy balance` matches `curl …/api/balance` (confirmed total).
3. `nozy status` balance section matches.
4. After broadcasting a send (pending in mempool), **Available** decreases by amount + fee.

---

## References

- Paper / lecture: [`../../reference/CLI_BALANCE_NOTEINDEX.md`](../../reference/CLI_BALANCE_NOTEINDEX.md)
- Mainnet run C (symptom context): [`../../reference/MAINNET_SEND_READINESS_EVIDENCE.md`](../../reference/MAINNET_SEND_READINESS_EVIDENCE.md) — Run C mark-spent warning (related v2 parser issue)
- Code: `src/cli_helpers.rs` (`wallet_balance_snapshot`), `src/main.rs` (`Commands::Balance`, `Commands::Status`)
