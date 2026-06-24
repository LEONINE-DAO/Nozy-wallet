# CLI balance and NoteIndex v2 — paper & lecture notes

**Status:** Fixed June 2026 (CLI); desktop / extension reuse planned  
**Audience:** Technical paper, lecture slides, operator docs  
**Bug ID:** [BUG-2026-012](../issues/bugs/2026-06-cli-balance-v2-noteindex.md)

**Related:**

| Doc | Role |
|-----|------|
| [`MAINNET_SEND_READINESS_EVIDENCE.md`](MAINNET_SEND_READINESS_EVIDENCE.md) | Run C — sync/send; mark-spent warning surfaced v2 parser drift |
| [`bugs/2026-06-cli-balance-v2-noteindex.md`](../issues/bugs/2026-06-cli-balance-v2-noteindex.md) | Formal bug writeup |
| [`bugs/2026-06-history-empty-despite-balance.md`](../issues/bugs/2026-06-history-empty-despite-balance.md) | Same pattern: one surface correct, another stale |

---

## Executive summary (one slide)

1. **`notes.json` migrated to v2 `NoteIndex`** for fast load, spent marking, and witness metadata — but some CLI paths still assumed a **legacy JSON array**.
2. **`nozy balance` showed 0 ZEC** on v2 wallets while api-server showed the real shielded total — operators could not trust the CLI for send decisions.
3. **Fix:** one Rust helper — `wallet_balance_snapshot()` — reads cache via `load_wallet_notes()`, excludes spent notes, subtracts pending sends for **available** balance.
4. **Lesson:** after a storage format migration, **grep for hand-rolled JSON parsers**; one canonical load path per entity (`NoteIndex` for notes).
5. **Rollout:** CLI first; api-server / desktop / extension will adopt the same snapshot when those surfaces are updated.

---

## What balance means in NozyWallet

NozyWallet is **Orchard shielded-first**. Balance is **not** from Zebrad RPC — it is derived locally:

```text
compact sync / scan  →  notes in notes.json (NoteIndex v2)
                              ↓
                    unspent note values (zatoshis)
                              ↓
              − pending outbound (amount + fee)
                              ↓
                    available to send (ZEC)
```

| Term | Definition |
|------|------------|
| **Confirmed** | Sum of `value` on notes where `spent == false` |
| **Pending** | Sum of `amount + fee` for txs still in mempool (local `SentTransactionStorage`) |
| **Available** | Confirmed − pending (what send preflight should use) |

Zebrad does not provide a wallet balance RPC for Orchard notes on the operator stack we target.

---

## The bug — what CLI was doing wrong

### Symptom (mainnet, June 2026)

During send-readiness testing:

- `nozy sync --to-tip` completed; balance after sync **0.0064 ZEC** (logged by sync).
- `curl http://localhost:3000/api/balance` returned the correct zatoshis.
- **`nozy balance` printed 0.00000000 ZEC** (or omitted unspent notes).

Run C also logged a **mark-spent warning** after broadcast — same root class: code still treating v2 `notes.json` as a legacy array.

### Broken code path (before fix)

`Commands::Balance` in `src/main.rs` did approximately:

```text
read notes.json as string
  → parse as serde_json::Value
  → require top-level JSON array
  → sum every object's "value" field (no spent check)
```

### Why that fails on v2

v2 `NoteIndex` on disk looks like:

```json
{
  "version": 2,
  "notes": [ ... ],
  "nullifier_index": { ... },
  "height_index": { ... }
}
```

There is **no top-level array**. `.as_array()` → empty → **sum = 0**.

On **legacy** array files, summing all `value` fields without `spent` could **overstate** balance after sends.

### Correct path (already existed)

The send pipeline and api-server already used:

- `load_wallet_notes()` → `NoteIndex::load_from_file` (v2 + legacy migration)
- `wallet_unspent_balance_zatoshis()` → filter `!note.spent`, then sum

CLI simply had not been wired to this path.

---

## The fix — `wallet_balance_snapshot()`

**Location:** `src/cli_helpers.rs` (exported from `nozy` crate).

**API:**

| Field | Type | Meaning |
|-------|------|---------|
| `confirmed_zatoshis` | `u64` | Unspent Orchard notes |
| `pending_zatoshis` | `u64` | Outbound mempool txs |
| `available_zatoshis` | `u64` | Spendable now |
| `unspent_note_count` | `usize` | Note count (UX / debug) |

**CLI commands updated:**

- `nozy balance` — full breakdown (confirmed, pending, available, note count)
- `nozy status` — confirmed (+ pending line when relevant)

**Rollout policy:** CLI first. Desktop Tauri `get_balance` and extension popup still use legacy parsers until deliberately ported to `wallet_balance_snapshot()` — documented so paper readers know scope.

---

## Lecture narrative (2–3 minutes)

**Setup:** We shipped NoteIndex v2 so witness catch-up, spent marking, and sync merge stay fast and atomic. Migration runs automatically on load.

**Incident:** On mainnet, sync and api-server looked healthy. Operator runs `nozy balance` before a send — **zero**. That erodes trust faster than a slow proof: users think funds are gone or sync failed.

**Diagnosis:** Two parsers for one file. Any time you migrate storage, search for `read_to_string` + `from_str::<Vec` on `notes.json`. The correct abstraction was already there; CLI was a straggler.

**Fix:** One snapshot function, three numbers operators care about (confirmed / pending / available). Same function will feed other surfaces so we never diverge again.

**Broader lesson:** Wallet ≠ node. Balance is a **local index problem**. A correct Zebrad tip does not fix a wrong JSON parser.

---

## Paper-ready paragraph

NozyWallet persists Orchard notes in a version-2 `NoteIndex` structure in `notes.json`, replacing a flat JSON array for incremental sync and spent-note tracking. After this migration, the CLI `balance` command still parsed the file as a top-level array and summed note values without excluding spent entries. On v2 wallets this produced a zero balance despite a populated cache, while the localhost api-server — which already called `load_wallet_notes()` and `wallet_unspent_balance_zatoshis()` — reported the correct shielded total. We consolidated CLI balance reporting into `wallet_balance_snapshot()`, which returns confirmed, pending, and available zatoshis from the canonical load path and pending transaction store. This illustrates a recurring integration risk when evolving wallet storage: surface-level JSON parsers must be retired in favor of a single library entry point, with CLI validated first before extending desktop and extension companions.

---

## Comparison table (for slides)

| Surface | Before fix | After fix (CLI) | Planned |
|---------|------------|-----------------|---------|
| api-server `/api/balance` | Correct | Correct | Use snapshot for pending/available |
| `nozy balance` | **0 on v2** | **Correct** | — |
| `nozy status` balance | **0 on v2** | **Correct** | — |
| Desktop Tauri `get_balance` | Legacy array parse | Unchanged | Port to snapshot |
| Extension popup | Scan progress only | Unchanged | Poll api-server status |

---

## Verification checklist

```powershell
# 1. Build
cargo build --release --bin nozy

# 2. CLI balance
cargo run --release --bin nozy -- balance

# 3. Compare api-server (if running)
curl http://localhost:3000/api/balance

# 4. Status includes balance section
cargo run --release --bin nozy -- status
```

**Pass:** Confirmed zatoshis match between CLI and api-server on the same wallet data dir (`%APPDATA%\nozy\nozy\data\` on Windows).

---

## Related fixes (same migration theme)

| Issue | Symptom | Fix |
|-------|---------|-----|
| BUG-2026-012 | CLI balance 0 on v2 | `wallet_balance_snapshot()` |
| mark-spent after broadcast | Warning / parse error on v2 | `NoteIndex::load_from_file` in `mark_wallet_notes_spent_from_spendables` |
| BUG-2026-002 | History empty, balance OK | Merge received notes into history views |

Together these show **one storage format, many readers** — all must use `NoteIndex` APIs.

---

## White paper / lesson bullet

**Cache format migrations matter:** v2 NoteIndex vs legacy array caused subtle post-send and balance-display bugs until every read path used `load_wallet_notes()` / `NoteIndex::load_from_file`.
