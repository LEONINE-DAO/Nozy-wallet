# Mainnet send readiness — field evidence (paper & lecture)

**Status:** Recorded mainnet runs — June 2026  
**Audience:** Technical paper, Shielded Labs pilot reporting, lecture slides  
**Environment:** Windows host + WSL Zebrad (`http://172.20.199.206:8232`), `nozy` CLI release, wallet in `%APPDATA%\nozy\nozy\data`

**Related:**

| Doc | Role |
|-----|------|
| [`PILOT_EXPIRY_PROVING_LATENCY.md`](PILOT_EXPIRY_PROVING_LATENCY.md) | Two clocks, 5 vs 15 blocks, Gilmore `-25` RCA |
| [`CLI_BALANCE_NOTEINDEX.md`](CLI_BALANCE_NOTEINDEX.md) | CLI balance 0 on v2 NoteIndex (BUG-2026-012), `wallet_balance_snapshot()` |
| [`../issues/bugs/2026-06-cli-balance-v2-noteindex.md`](../issues/bugs/2026-06-cli-balance-v2-noteindex.md) | BUG-2026-012 formal writeup |
| [`../MAINNET_SEND_EXPIRY_TEST.md`](../MAINNET_SEND_EXPIRY_TEST.md) | How to reproduce tests |
| [`ZEBRAD_NOZYWALLET_CONNECTIVITY.md`](ZEBRAD_NOZYWALLET_CONNECTIVITY.md) | Verify Zebrad RPC before sync/send |
| [`../issues/bugs/2026-06-send-expiry-before-broadcast.md`](../issues/bugs/2026-06-send-expiry-before-broadcast.md) | BUG-2026-011 |
| [`../PILOT_MAINNET_EVIDENCE.md`](../PILOT_MAINNET_EVIDENCE.md) | Post-broadcast expiry + speed-up |

---

## Executive summary (lecture slide)

1. **Pilot expiry stays at 5 blocks** (~6 min mempool window) for fast expire-and-replace UX.
2. **Pre-broadcast `-25`** on slow VPS was fixed by late tip refresh + rebuild/retry (BUG-2026-011) — not by stretching expiry to 15 blocks.
3. **Send latency** on operator hardware is dominated by **witness freshness** and **Orchard Halo2 proving**, not recipient wallet type (Zodl vs Nozy).
4. **Sync-to-tip before send** + witness lag guard + proving warm-up reduces unpredictable 7+ minute sends to **~3–3.5 minutes** on the same stack.
5. **Mainnet verification (June 2026):** two successful broadcasts after sync, no `-25`, spent notes persisted in v2 `notes.json`.

---

## Test matrix (all recorded runs)

| # | Date | Phase | Witness lag | Sync | Send time | Broadcast | TXID (prefix) | Notes |
|---|------|-------|-------------|------|-----------|-----------|---------------|-------|
| A | 2026-06-21 | Gilmore (pre-fix) | ~5000+ blk | — | ~12+ min wall | **FAIL `-25`** | `daed46a0…` | Expiry 3385380, tip 3385384 at broadcast |
| B | 2026-06-21 | Post-fix, stale wallet | ~5000 blk | partial | **~419 s** | **PASS** | `e4f0f504…` | First post-fix mainnet; witness catch-up dominated |
| C | 2026-06-22 | Sync + send #1 | 1 blk after sync | 5132 blk / **32 s** | **198.7 s** | **PASS** | `5a03fbd1…` | `notes.json` mark-spent warning (legacy parse) |
| D | 2026-06-22 | Stale guard test | >50 blk | — | **0.09 s** | N/A (rejected) | — | api-server: `success: false`, sync-first message |
| E | 2026-06-22 | Sync + send #2 | ≤1 blk | 54 blk / **0.3 s** | **205.8 s** | **PASS** | `902cf006…` | Mark-spent fix verified; no warning |

**Stack constants:** `PILOT_EXPIRY_DELTA_BLOCKS = 5`, `MAX_SEND_WITNESS_LAG_BLOCKS = 50`, `WITNESS_CATCHUP_PARALLEL_BLOCKS = 10`, Zebrad mainnet via WSL.

---

## Run C — large catch-up sync + send (2026-06-22)

### Sync

| Field | Value |
|-------|--------|
| Command | `nozy sync --to-tip` |
| Range | 3381309 → 3386440 |
| Blocks | 5132 |
| Elapsed | **~32 s** |
| Balance after | 0.0064 ZEC |
| `last_scan_height` | 3386440 |

### Send readiness (post-sync)

| Field | Value |
|-------|--------|
| Chain tip | 3386422 |
| Max witness lag (unspent) | **1 block** |
| Spendable notes | 2 |
| Proving warm-up (1st call) | **~2.1 s** |
| Proving warm-up (2nd call) | **~instant** (cached) |
| Guard result | **READY** |

### Send

| Field | Value |
|-------|--------|
| Surface | CLI (`nozy -m send`) |
| Amount | 0.0001 ZEC + 0.0001 ZEC fee (ZIP-317) |
| Recipient | Self `u1qr0zfsta9…` |
| Elapsed | **198.7 s (~3.3 min)** |
| TXID | `5a03fbd19547f9499182d78c88791eeb4eaab32e5d158b69ec8ffdc6068d2612` |
| Broadcast | Success |
| On-chain | Confirmed via `getrawtransaction` |
| Side effect | Warning: `mark_wallet_notes_spent` used legacy array parser on v2 index (see [BUG-2026-012](CLI_BALANCE_NOTEINDEX.md) — CLI `balance` also showed 0 until fixed) |

**Log:** `send-mainnet-test.log` (repo root)

---

## Run D — witness stale guard (2026-06-22)

Wallet **~5110 blocks** behind tip; send blocked **before** multi-minute witness catch-up.

| Field | Value |
|-------|--------|
| Surface | api-server `POST /api/transaction/send` |
| Elapsed | **0.09 s** |
| HTTP | 200 |
| Body | `{ "success": false, "message": "…Sync to tip before sending…" }` |
| Purpose | Prove guard fires early (not after 120 s scan timeout) |

**Fix applied during test:** early `ensure_cached_witness_fresh_for_send` in `scan_notes_for_sending` (before fallback rescan).

---

## Run E — incremental sync + send with mark-spent fix (2026-06-22)

### Sync

| Field | Value |
|-------|--------|
| Range | 3386441 → 3386494 |
| Blocks | 54 |
| Elapsed | **0.3 s** |
| Balance after | 0.0015 ZEC |

### Send

| Field | Value |
|-------|--------|
| Elapsed | **205.8 s (~3.4 min)** |
| TXID | `902cf006efdeef3f15fed4312f8a15fcb1162f52495098c3bffb4acbe3cde4e5` |
| Broadcast | Success |
| On-chain | Yes |
| Mark spent locally | **OK** (no parse warning; v2 `NoteIndex` path) |
| Remaining balance | 0.0013 ZEC |

**Log:** `send-mainnet-test-2.log` (repo root)

---

## Send-readiness engineering (what we built)

| Mechanism | Purpose | Where |
|-----------|---------|--------|
| Late tip refresh before `nExpiryHeight` | Build-clock race | `src/orchard_tx.rs` |
| Prove rebuild (≤3 attempts) | Slow proving vs 5-block window | `src/orchard_tx.rs` |
| Broadcast retry on expiry `-25` | Last-chance recovery | `src/transaction_builder.rs` |
| `MAX_SEND_WITNESS_LAG_BLOCKS = 50` | Block send if sync too stale | `src/send_readiness.rs` |
| Early cached witness check | Reject in **&lt;1 s** before rescan | `src/cli_helpers.rs` |
| Parallel witness catch-up (10 RPC/batch) | Faster catch-up when lag ≤50 | `src/orchard_tx.rs` |
| `warm_orchard_proving_key()` | Skip cold Halo2 setup on first send | CLI, api-server startup/unlock |
| `NoteIndex` mark-spent on broadcast | Correct local balance / double-spend guard | `src/notes.rs`, `src/note_index.rs` |

---

## Timing model (for lecture)

```text
Total send time ≈ witness_catchup + proving_setup + halo2_prove + sign + broadcast

Stale wallet (~5000 blk):  witness_catchup DOMINATES  → 7+ min, expiry risk
Synced wallet (lag ≤50):   proving_setup + prove      → ~3–3.5 min (observed)
Guard (lag >50):           reject immediately         → ~0.1 s
```

| Component | Observed (this stack) |
|-----------|------------------------|
| Sync 5132 blocks | ~32 s |
| Sync 54 blocks | ~0.3 s |
| Proving warm-up (cold) | ~2.1 s |
| Proving warm-up (warm) | negligible |
| Full send (synced) | **~200 s** |

Orchard Halo2 prove alone is typically **tens of seconds to a few minutes** on VPS-class CPU — consistent with our ~3.3 min end-to-end.

---

## Comparison: failure modes

| Mode | User sees | Funds | Fix class |
|------|-----------|-------|-----------|
| Pre-broadcast `-25` (Gilmore) | Error at broadcast | Unchanged | BUG-2026-011: late expiry + rebuild |
| Stale witness at send | Fast “sync to tip” | Unchanged | Send-readiness guard |
| Post-broadcast expiry | Pending → Expired | Locked until expired | Pilot 5-block + speed-up (by design) |
| Zodl as recipient | N/A | N/A | No effect on sender prove time |

---

## On-chain verification commands

```powershell
$txid = "902cf006efdeef3f15fed4312f8a15fcb1162f52495098c3bffb4acbe3cde4e5"
$body = @{ jsonrpc="2.0"; method="getrawtransaction"; params=@($txid,1); id=1 } | ConvertTo-Json
Invoke-RestMethod -Uri "http://172.20.199.206:8232" -Method Post -ContentType "application/json" -Body $body
```

---

## Paper paragraph (copy-paste — send readiness + mainnet evidence)

> Beyond fixing pre-broadcast expiry races (BUG-2026-011), we instrumented send-time readiness on operator-class hardware: a fifty-block witness lag threshold blocks sends that would otherwise trigger multi-minute per-block witness catch-up over JSON-RPC; parallel block fetches amortize smaller gaps; and eager Orchard proving-key warm-up removes cold-start latency on first spend. On mainnet in June 2026, with Zebrad on WSL and the CLI on Windows, syncing five thousand blocks required about thirty-two seconds, after which witness lag fell to one block and shielded sends completed in approximately three and a half minutes end-to-end with successful broadcast and no consensus expiry errors—compared with seven-plus minutes and expiry risk when witnesses were thousands of blocks stale. A deliberate stale-wallet test rejected sends in under one tenth of a second with an explicit sync-to-tip message, confirming the guard prevents repeating Gilmore-style long builds. We retained the five-block pilot expiry throughout so mempool expire-and-replace semantics stay responsive.

---

## Lecture outline (10–15 min)

1. **Problem:** Shielded send = two clocks (build vs mempool expiry).
2. **Gilmore incident:** `-25` before mempool; why 15 blocks was rejected.
3. **Fix layer 1:** Late tip, rebuild, broadcast retry (BUG-2026-011).
4. **Fix layer 2:** Sync-first policy, witness lag guard, warm proving.
5. **Live numbers:** Table from “Test matrix” above.
6. **Demo path:** `nozy sync --to-tip` → `test_send_readiness` → `nozy send` (dust).
7. **Takeaway:** Product policy (5-block expiry) vs runtime engineering (sync + prove).

---

## Artifacts in repo

| File | Contents |
|------|----------|
| `send-mainnet-test.log` | Run C CLI transcript |
| `send-mainnet-test-2.log` | Run E CLI transcript |
| `sync-to-tip.log` | Large catch-up sync transcript |
| `src/bin/test_send_readiness.rs` | Live lag + warm-up diagnostic |

---

## References

- BUG-2026-011 — [`../issues/bugs/2026-06-send-expiry-before-broadcast.md`](../issues/bugs/2026-06-send-expiry-before-broadcast.md)
- BUG registry — [`../issues/BUG_REGISTRY.md`](../issues/BUG_REGISTRY.md)
- Paper generator — [`../../scripts/generate-nozy-paper.py`](../../scripts/generate-nozy-paper.py) §6.3.1–6.3.2
