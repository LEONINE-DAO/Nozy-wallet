# BUG-2026-013: CLI `sync` wipes v2 `notes.json` (legacy parser bypass)

**Status:** Fixed on `master` (post v2.3.6.6)  
**Severity:** P0  
**Surface:** cli (`nozy sync`, `nozy sync --to-tip`)  
**Reporter:** Internal (mainnet smoke test, June 2026)  
**GitHub issue:** _(file if/when created)_

---

## Summary

After `wallet_sync` landed in v2.3.6.2, **api-server** and **desktop** used `sync_wallet_notes()` with v2 `NoteIndex` load/merge/save. The **CLI `sync` command kept a separate code path** that parsed `notes.json` as a legacy top-level JSON array (`unwrap_or_default()` on failure). On v2 files this silently loaded **zero notes**, merged an incremental scan, and **overwrote the cache**—often with `[]`—while history and pending sends still showed prior activity.

---

## Environment

| Field | Value |
|-------|--------|
| Nozy version | v2.3.6.5 release (pre-fix); `master` post June 2026 smoke test |
| OS | Windows host, WSL Zebrad (`http://172.20.199.206:8232`) |
| Network | mainnet |
| `notes.json` format | **v2 `NoteIndex`** (10 notes, ~45 KB after recovery rescan) |

---

## Steps to reproduce (pre-fix)

1. Sync until `notes.json` is v2 with unspent Orchard notes (`nozy balance` shows confirmed > 0).
2. Run `nozy sync --to-tip` (incremental tail only, e.g. 5–24 blocks).
3. Observe sync output: `Balance: 0.00000000 ZEC`, `Found 0 new notes`.
4. Run `nozy balance` → **0 confirmed**, 0 unspent notes; `notes.json` shrunk to empty v2 shell (`notes: []`).

---

## Expected behavior

Incremental sync **merges** newly scanned notes into the existing cache and persists via `NoteIndex::save_to_file`. Confirmed balance unchanged when the tail scan finds nothing new.

---

## Actual behavior

CLI sync **replaced** the v2 cache with an empty or scan-only legacy array because:

| Step | Broken CLI path | Correct `sync_wallet_notes` path |
|------|-----------------|----------------------------------|
| Load cache | `serde_json::from_str::<Vec<_>>` + `unwrap_or_default()` | `load_wallet_notes()` → `NoteIndex::load_from_file` |
| v2 file | Parse fails → **empty vec** | Loads all notes + indexes |
| Persist | `fs::write` legacy array | `save_wallet_notes` → v2 atomic save |
| Empty cache + `--to-tip` | Scans only `last_scan+1…tip` | `apply_empty_cache_backfill` rescans from mainnet floor |

---

## Operator impact (June 2026 smoke test)

| Step | Result |
|------|--------|
| Pre-sync status | 0.0013 ZEC confirmed, 2 notes; 24 blocks behind tip |
| `sync --to-tip` (24 blk) | Reported balance **0**; cache **wiped** |
| Recovery | `sync --start-height 3370000 --to-tip` → 10 notes, 0.01065242 ZEC scanned |
| Post-fix incremental sync | Notes **preserved** (10 notes, balance unchanged) |

Send was blocked afterward (pending outbound > confirmed)—separate from this bug.

---

## Fix (master)

1. **CLI `Commands::Sync`** delegates to `sync_wallet_notes()` (same as api-server / desktop).
2. **`apply_empty_cache_backfill`**: empty cache + plain `sync --to-tip` rescans from mainnet default floor (3_050_000), not only the incremental tail.
3. **`sync_wallet_notes` persist guard**: refuse to save if merge would replace a non-empty cache with zero notes.
4. **`NoteIndex::load_from_file`**: recover notes from a corrupt v2 wrapper via the `notes` JSON field when full deserialize fails.
5. **CLI send path**: pre-send balance uses `wallet_balance_snapshot()` (related dual-parser class as BUG-2026-012).

---

## Verification

1. Wallet with v2 `notes.json` and known note count.
2. `nozy sync --to-tip` → note count unchanged; file stays v2 with `nullifier_index`.
3. Empty cache + `sync --to-tip` → backfill scan from ≥3_050_000, not “already synced at zero balance”.
4. `nozy balance` matches api-server `/api/balance` before and after incremental sync.

**Recovery if already wiped:**  
`nozy sync --start-height 3370000 --to-tip` (or `--start-height 3050000 --to-tip` for full Orchard floor).

---

## References

- White paper: §1.6, §3.3, §3.4, §6, §8, Appendix A (`BUG-2026-013`)
- Related: [`2026-06-cli-balance-v2-noteindex.md`](2026-06-cli-balance-v2-noteindex.md) (BUG-2026-012, same dual-parser class)
- Code: `src/main.rs` (`Commands::Sync`), `src/wallet_sync.rs`, `src/note_index.rs`
