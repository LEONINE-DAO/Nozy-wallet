# BUG-2026-011: Send fails at broadcast when Orchard proving outruns pilot expiry

**Status:** Fixed on `master` (unreleased tag at time of writing)  
**Severity:** P1  
**Surface:** core (`orchard_tx`), api-server, CLI, desktop  
**Reporter:** Gilmore (VPS testing, 2026-06-21)  
**GitHub issue:** _(file if/when created)_

---

## Summary

Shielded send could build, prove, and sign successfully but fail at `sendrawtransaction` with Zebrad `-25`:

```text
transaction must not be mined at a block Height(3385384) greater than its expiry Height(3385380)
```

Balance was unchanged (tx never entered mempool). This is distinct from the **speed-up** path, which only applies after a successful broadcast.

| Field | Value |
|-------|--------|
| Example TXID | `daed46a019c686df0974ef482c1dbbeb88ae906b448274f3c988eb396736dd48` |
| Encoded expiry | 3385380 |
| Chain tip at reject | 3385384 (+4 blocks past expiry) |
| Pilot delta (before fix) | 5 blocks after mempool build height |
| Pilot delta (after fix) | **15 blocks** after mempool build height |
| Environment | VPS Zebrad + api-server, WSL-style stack |

---

## Root cause

Three interacting issues:

1. **Expiry clock started too early** — chain tip was read at the beginning of `build_single_spend`, before witness fetch, bundle construction, Halo2 proving, and signing. Slow hosts (VPS/WSL) can spend many blocks in that window.

2. **Short pilot window by design** — `PILOT_EXPIRY_DELTA_BLOCKS = 5` targets the Shielded Labs dynamic-fee pilot (~6 minutes at 75 s/block). Proving latency is not bounded to that window on constrained hardware.

3. **History metadata mismatch** — api-server / CLI recorded `expiry_height = tip + 5` while the transaction encoded `(tip + 1) + 5` (Zebrad mempool build context is one block ahead of the current tip).

4. **No pre-broadcast recovery** — a tx that expired before broadcast was discarded with `-25`; speed-up requires a prior successful broadcast.

---

## Fix (2026-06-21)

### 1. Centralized expiry helpers (`src/fee_policy.rs`)

- `pilot_expiry_height(tip, delta)` → `(tip + 1) + delta`
- `pilot_transaction_expired(tip, expiry)` → `tip > expiry`
- `is_expiry_consensus_error(message)` → detect Zebrad `-25` expiry rejections
- `PILOT_EXPIRY_MAX_REBUILD_ATTEMPTS = 3`

### 2. Late tip refresh + rebuild loop (`src/orchard_tx.rs`)

After the Orchard bundle is built (before sighash / prove / sign):

- Re-fetch chain tip and encode expiry from the fresh tip.
- After proving, re-fetch tip; if `tip > expiry_height`, rebuild (up to 3 attempts).

### 3. Broadcast retry (`src/transaction_builder.rs`)

- `SignedTransaction` now carries `expiry_height` from the built tx.
- New `build_and_broadcast_send_transaction()` rebuilds on expiry `-25` (up to 3 attempts).
- CLI, api-server, desktop, and speed-up use this unified path.

### 4. Correct history expiry

All surfaces persist `transaction.expiry_height` from the signed transaction instead of a pre-build estimate.

### 5. Raised default pilot expiry delta

`PILOT_EXPIRY_DELTA_BLOCKS` increased from **5 → 15** (~19 minutes at 75 s/block after mempool `+1` context) so slow VPS/WSL proving stacks have headroom beyond auto-rebuild alone.

---

## Files changed

| Path | Change |
|------|--------|
| `src/fee_policy.rs` | Expiry helpers, rebuild constant, unit tests |
| `src/orchard_tx.rs` | Late tip refresh, prove-time rebuild loop |
| `src/transaction_builder.rs` | `expiry_height` on `SignedTransaction`, broadcast retry |
| `src/cli_helpers.rs` | Use `build_and_broadcast_send_transaction` |
| `src/tx_lifecycle.rs` | Speed-up uses unified broadcast path |
| `api-server/src/handlers.rs` | Send handler uses unified path |
| `desktop-client/src-tauri/` | Send commands use unified path |
| `src/lib.rs` | Re-export new helpers |

---

## Expected after fix

- Sends on slow VPS/WSL stacks complete without manual retry when proving spans several blocks.
- Logs may show `Orchard proof outran pilot expiry; rebuilding (2/3)` — normal on slow hosts.
- History `expiry_height` matches the on-chain encoded value.
- Funds remain safe on failure (no mempool accept until broadcast succeeds).

---

## Verification

1. On a slow host (or artificially delayed prove), trigger `POST /api/transaction/send` or `nozy send`.
2. Confirm broadcast succeeds despite multi-block proving.
3. Confirm history entry `expiry_height` equals `(tip_at_encode + 1) + 5`.
4. Optional regression: reproduce Gilmore's error on pre-fix build; confirm `-25` no longer occurs for the same workload.

---

## Paper / technical narrative (draft)

> **Problem.** NozyWallet's dynamic-fee pilot uses a short transaction expiry window so unconfirmed sends can expire and be replaced via a priority fee speed-up. Expiry height is part of the signed transaction (ZIP-225 `nExpiryHeight`). On fast local nodes, witness fetch + Orchard Halo2 proving completes within five blocks; on operator VPS stacks, proving routinely exceeds that window, producing valid signed transactions that Zebrad rejects at mempool admission because the chain tip has already passed `nExpiryHeight`.
>
> **Observation.** Reporter Gilmore observed a fully built 0.003341 ZEC Orchard send rejected at height 3385384 with expiry 3385380 — four blocks past expiry — with no balance impact because the transaction never broadcast.
>
> **Solution.** We decouple witness anchor height (still tied to tip at spend preparation) from expiry encoding (refreshed immediately before sighash and proving), add an automatic rebuild loop when proving outruns expiry, and retry broadcast on Zebrad expiry consensus errors. Expiry metadata in wallet history now uses the same formula as on-chain encoding: `expiry_height = chain_tip + 1 + PILOT_EXPIRY_DELTA_BLOCKS`.
>
> **Trade-off.** Rebuild retries increase CPU time on slow hosts. Default `PILOT_EXPIRY_DELTA_BLOCKS` was raised from 5 to 15 so operator VPS stacks get ~19 minutes of validity after late tip refresh, while speed-up / expire-replace pilot flows remain available for txs that still miss the window.

---

## References

- [`docs/PILOT_MAINNET_EVIDENCE.md`](../../PILOT_MAINNET_EVIDENCE.md) — expire-after-broadcast speed-up (different failure mode)
- [`src/fee_policy.rs`](../../../src/fee_policy.rs) — pilot fee/expiry policy
- BUG-2026-002 — Gilmore history fix (same VPS test session)
