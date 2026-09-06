# ZIP 318 twin balance — case breakdown

**Status:** **Recovered on operator profile** (2026-08-28); code hardening in [PR #277](https://github.com/LEONINE-DAO/Nozy-wallet/pull/277); merge fix shipped in [PR #198](https://github.com/LEONINE-DAO/Nozy-wallet/pull/198).  
**Date:** 2026-08-28  
**Profile:** mainnet `5a5c81bda1fa6343` (Wallet 1)  
**Related:** [`ZIP318_TWIN_BALANCE_RECOVERY.md`](ZIP318_TWIN_BALANCE_RECOVERY.md) · [`MAINNET_IRONWOOD_MIGRATION_EVIDENCE.md`](MAINNET_IRONWOOD_MIGRATION_EVIDENCE.md) · [`IRONWOOD_WALLET_READINESS.md`](IRONWOOD_WALLET_READINESS.md)

This document is the **postmortem + fix narrative** for “half balance” after Ironwood / ZIP 318 prep — what was reported, what was wrong, how we proved it, and what we changed.

---

## Executive summary

| Question | Answer |
|----------|--------|
| Were funds stolen? | **No.** ZEC remained on-chain in the wallet’s keys; the app **under-counted unspent notes**. |
| What broke? | A **collapsed ZIP 318 twin** in `notes.json` plus incremental sync paths that could **persist an under-counted cache** without a full rescan. |
| How was it fixed for the operator? | `nozy sync --start-height 3428000 --to-tip` rebuilt the cache; balance went **0.0199 → 0.0419 ZEC**. |
| How do we prevent recurrence? | Nullifier-keyed merge (PR #198), refuse-to-persist guard, `notes-doctor`, bounded incremental sync hardening (PR #277). |

---

## Reported symptom

**User expectation (2026-07-21 desktop screenshot):** **0.0445 ZEC** available on mainnet Wallet 1 (`u1qr0zfsta9p…9c70yhwt`).

**Observed before recovery (2026-08-28 CLI):**

| Metric | Value |
|--------|------:|
| Confirmed | **0.01990000 ZEC** (8 unspent notes) |
| Available | **0.01850000 ZEC** |
| Pending lock | 0.00140000 ZEC |
| `notes-doctor` equal-value groups | **0** (+0 twin/extra notes) |

Gap vs screenshot: **~0.0246 ZEC** (~55% of displayed balance) — consistent with **one half of a split note pair missing from cache**.

**Misleading signals we ruled out:**

- Wrong extension video (0.0927 ZEC) — different session; disregarded.
- “Funds at risk” Ironwood banner — migration **spendability** warning, not loss.
- Phantom `sent_transactions.json` rows (`utest…` on mainnet) — corrupt **history**, not the main ~0.02 ZEC gap.
- Large extension-only sends to address-book “Tman” — real txs on another timeline; not the Jul 21 → Aug 28 half-balance regression.

---

## Root cause (three layers)

### Layer 1 — ZIP 318 creates **twin notes**

Ironwood prep (`nozy ironwood split`) emits **multiple equal-value Orchard notes in one transaction** (canonical denominations). Each note has a **distinct nullifier** but may share `(txid, block_height, value)`.

Example split heights on this profile (from migration evidence):

| TXID (prefix) | Height | Role |
|---------------|--------|------|
| `9b3b2f81…` | 3,428,651 | ZIP 318 split A |
| `e0e233e8…` | 3,428,662 | ZIP 318 split B |

If the wallet keeps **only one** of two 2,000,000-zat twins, balance drops ~50%.

### Layer 2 — Historical merge keyed on the wrong identity

Before PR #198, scan merge could treat equal-value notes as duplicates when keyed on `(txid, height, value)` instead of **nullifier**. One twin was dropped → **halved balance**.

**Fix (merged):** `merge_scanned_notes` in `src/notes.rs` keys on **nullifier** (fallback: `note_bytes`). Unit test `merge_keeps_equal_value_zip318_twin_notes` locks this behavior.

### Layer 3 — Stale cache + incremental sync edge cases

Even with correct merge logic, an **already-collapsed** `notes.json` stays wrong until rescan. Additionally, desktop/API sync uses **bounded height chunks** (`end_height`). Prior behavior could:

- rewind witness-resume floor during chunked catch-up,
- run baseline start obfuscation on bounded chunks,
- witness all the way to tip every chunk,

…making it easier to **persist a partial view** without operators noticing.

**Fix (PR #277):** `clamp_incremental_chunk_range`, skip rewind/obfuscation when `end_height` is set, checkpoint `last_scan_height` per chunk, cap witness catch-up to chunk end. New test: `bounded_chunk_skips_obfuscation_and_stays_forward_only`.

---

## Recovery we ran (operator evidence)

**Preconditions:** Close desktop/extension so they do not rewrite `notes.json` mid-scan.

```bash
nozy sync --start-height 3428000 --to-tip
nozy balance
nozy notes-doctor
```

| Step | Result |
|------|--------|
| Scan window | 3,428,000 → tip (~36k blocks, ~77 min, minimal progress UI) |
| New notes found | **+2** (28 total) |
| Confirmed balance | **0.04190000 ZEC** (10 unspent notes) |
| Available | **0.04050000 ZEC** |
| Twin audit | **1** equal-value group (+1 extra note) |
| vs Jul 21 screenshot | Within **~0.0026 ZEC** (fees / pending / sapling) |

**Conclusion:** Missing twin was **re-indexed**; funds were never gone.

---

## Detection tooling

### `nozy notes-doctor`

Surfaces `note_cache_integrity`:

- `equal_value_unspent_groups: 0` **after known splits** → suspect collapsed twin → rescan.
- `equal_value_unspent_groups: 1 (+1 twin/extra notes)` → healthy post-ZIP-318 cache.

### Refuse-to-persist guard

`sync_wallet_notes` calls `missing_scanned_nullifiers_after_merge` after merge. If scan found nullifiers that merge dropped, sync **errors** instead of saving a halved cache (message points at `nozy sync --start-height … --to-tip`).

---

## Fix stack (what shipped / shipping)

| Layer | Change | Where |
|-------|--------|-------|
| **Merge identity** | Nullifier-keyed merge; twin unit tests | PR #198 · `src/notes.rs`, `src/wallet_sync.rs` |
| **Integrity audit** | `notes-doctor`, `note_cache_integrity` | `src/notes.rs`, CLI |
| **Persist guard** | Refuse save if scanned nullifiers missing | `src/wallet_sync.rs` |
| **Incremental hardening** | Forward-only bounded chunks, per-chunk checkpoint | PR #277 · `src/wallet_sync.rs` |
| **Operator runbook** | Recovery steps | [`ZIP318_TWIN_BALANCE_RECOVERY.md`](ZIP318_TWIN_BALANCE_RECOVERY.md) |

---

## Case matrix (for support / grant readers)

### Case A — “Half balance after split” (this incident)

**Trigger:** ZIP 318 split on mainnet; incremental sync without full rebuild.  
**Signature:** ~50% of expected ZEC; `notes-doctor` shows 0 equal-value groups after splits.  
**Recovery:** Rescan from before split height; refresh desktop UI.  
**Prevention:** PR #198 + #277; run `notes-doctor` after splits.

### Case B — “0 ZEC after incremental sync” (related, documented in Ironwood readiness)

**Trigger:** Incremental range finds 0 new notes; old path replaced cache incorrectly.  
**Signature:** CLI 0 balance but schedule/preflight still shows notes.  
**Recovery:** Rescan from note birth height.  
**Prevention:** Shared `merge_scanned_notes` + `save_wallet_notes` path (Ironwood readiness § incremental sync case).

### Case C — “Funds at risk” banner

**Trigger:** NU6.3 / Ironwood activation; Orchard notes not migrated.  
**Signature:** UI warning; Orchard pool still holds zatoshis.  
**Not a balance bug.** Use `nozy ironwood migrate` path — separate from twin collapse.

---

## Timeline (this profile)

| When | Event |
|------|-------|
| 2026-06–07 | Ironwood / Orchard testing; small sends |
| 2026-07-21 | Desktop shows **0.0445 ZEC** (baseline screenshot) |
| 2026-07-30 | Mainnet Orchard→Ironwood turnstile confirmed (`ea2fa4e6…` @ 3,430,663) |
| 2026-08-28 | User reports ~half balance; CLI **0.0199 ZEC** |
| 2026-08-28 | Resync `3428000→tip` → **0.0419 ZEC**; PR #277 opened |

---

## What we tell users

1. **Your ZEC is likely still yours** if keys are intact — check `notes-doctor` before assuming loss.
2. **Half balance after Ironwood splits** is a known cache class; full rescan from split height fixes most cases.
3. **Always sync UI after CLI recovery** (desktop / extension “Sync to tip”).
4. **Do not trust a single surface** (extension Activity vs desktop `sent_transactions.json`) for accounting — chain + `notes.json` + CLI are source of truth.

---

## Open follow-ups

| ID | Item | Status |
|----|------|--------|
| T1 | Merge PR #277 incremental hardening | Open |
| T2 | Desktop sync progress UI for long `--to-tip` rescans | Open |
| T3 | Auto-suggest rescan height when `notes-doctor` sees post-split 0 twin groups | Nice-to-have |
| T4 | Reconcile extension Activity with desktop history store | Separate track |

---

## References

- ZIP 318 draft: https://github.com/zcash/zips/pull/1317  
- Forum: [Ironwood wallets update](https://forum.zcashcommunity.com/t/ironwood-is-here-updated-wallets-libraries-aug-20/56557)  
- Security audit lead L17 (twin merge): [`security-audit/LEADS.md`](security-audit/LEADS.md)
