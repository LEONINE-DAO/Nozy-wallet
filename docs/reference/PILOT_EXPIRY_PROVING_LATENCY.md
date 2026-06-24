# Pilot expiry vs Orchard proving latency (paper reference)

**Status:** Engineering note — BUG-2026-011 (2026-06-21)  
**Audience:** Technical paper, Shielded Labs pilot reporting, operator docs  
**Related:** [`docs/issues/bugs/2026-06-send-expiry-before-broadcast.md`](../issues/bugs/2026-06-send-expiry-before-broadcast.md)

---

## Executive summary (paper-ready)

NozyWallet’s dynamic-fee pilot uses a **5-block** transaction expiry after the mempool build height (`nExpiryHeight = tip + 1 + 5`, ~6 minutes at 75 s/block). That window exists so **unmined** transactions expire quickly and users can **speed up** with a ×4 priority fee rebuild.

Field testing on a VPS Zebrad stack (Gilmore, June 2026) exposed a **different failure mode**: Orchard Halo2 proving outlasted the time between **early** expiry encoding and **broadcast**, so Zebrad rejected a fully signed tx with `-25` even though it never entered the mempool.

**Key design lesson:** lengthening expiry to 15 blocks (~19 minutes) was tried briefly and **reverted** (commit `a72bc6e8`). It would reduce broadcast failures on slow hosts but **degrades product UX** — users wait far too long to learn a send failed or to use speed-up. The correct fix is to **decouple proving latency from the pilot expiry window**: refresh chain tip before encoding expiry, auto-rebuild when proving outruns the window, and retry broadcast on expiry consensus errors — while **keeping `PILOT_EXPIRY_DELTA_BLOCKS = 5`**.

---

## Two clocks (do not conflate them)

Shielded sends involve two independent timers:

| Clock | What it measures | User-visible effect |
|-------|------------------|---------------------|
| **Build clock** | Witness fetch → bundle → Halo2 prove → sign → `sendrawtransaction` | If chain tip passes `nExpiryHeight` **before broadcast**, Zebrad rejects with `-25`. Funds unchanged. |
| **Mempool expiry clock** | Blocks **after successful broadcast** until `nExpiryHeight` | If unmined, tx becomes **Expired** in wallet history → user can **Speed up** at ×4 fee. |

BUG-2026-011 was a **build-clock** bug. The **5-block pilot delta** governs the **mempool expiry clock** after broadcast. Fixing build-clock races by stretching mempool expiry to 15 blocks trades one problem for another.

```text
BUILD CLOCK (Gilmore bug — pre-fix)
──────────────────────────────────────────────────────────────────► time
│ read tip │ witness + bundle │ prove + sign │ broadcast │
│          │◄── expiry counted from HERE ───────────────►│ FAIL -25

BUILD CLOCK (post-fix — expiry still 5 blocks)
──────────────────────────────────────────────────────────────────► time
│ witness + bundle │ refresh tip │ prove + sign │ broadcast │
│                  │◄─ expiry from HERE ─ 5 blk ─►│ OK
│                  │ (rebuild if prove too slow)  │
```

---

## Incident timeline (Gilmore, mainnet)

| Field | Value |
|-------|--------|
| TXID | `daed46a019c686df0974ef482c1dbbeb88ae906b448274f3c988eb396736dd48` |
| Encoded `nExpiryHeight` | 3385380 |
| Chain tip at validation | 3385384 (+4 blocks past expiry) |
| Outcome | Broadcast rejected; balance untouched |
| Environment | VPS Zebrad + api-server (WSL-style stack) |

**Root cause (pre-fix):** `nExpiryHeight` was derived from chain tip at the **start** of `build_single_spend`. Witness catch-up, bundle construction, and Orchard proving consumed several blocks on a constrained host. By broadcast, the signed transaction was already expired under consensus rules.

This is **not** the same as [`docs/PILOT_MAINNET_EVIDENCE.md`](../PILOT_MAINNET_EVIDENCE.md), where txs **broadcast successfully** and later expired in mempool — those are candidates for speed-up.

---

## Why 5 blocks is better than 15 (product + protocol)

We explicitly **rejected** raising `PILOT_EXPIRY_DELTA_BLOCKS` from 5 → 15 after implementing rebuild/retry.

### Mempool UX (primary reason)

| | **5 blocks** | **15 blocks** |
|---|-------------|---------------|
| Time to expire (75 s/block) | ~6 minutes | ~19 minutes |
| User learns send failed / can speed up | Fast | Slow |
| Aligns with Shielded Labs pilot (short expire → replace) | Yes | No |
| Speed-up loop iteration time | Minutes | ~20 minutes |

The pilot’s purpose is **expire-and-replace**: if a standard-fee tx does not confirm quickly, the wallet marks it **Expired**, releases note locks, and the user rebuilds at **×4 priority**. A 15-block window means operators and end users stare at **Pending** for almost twenty minutes before the product can offer speed-up or clear failure state — unacceptable for a payment wallet.

### Build-time reliability (secondary — solved without 15)

Slow VPS proving does **not** require a longer mempool window. It requires:

1. **Late tip refresh** — encode `nExpiryHeight` from tip **after** bundle build, immediately before sighash/prove.
2. **Prove-time rebuild** — if `tip > expiry_height` after proving, discard and rebuild (up to 3 attempts).
3. **Broadcast retry** — if Zebrad still returns expiry `-25`, rebuild and retry.

Each rebuild gets a **fresh** `tip + 1 + 5`. Proving latency is absorbed by retries, not by stretching the pilot semantics.

### What 15 would have “fixed” (and why that is misleading)

Bumping to 15 would have made Gilmore’s **single** tx more likely to broadcast on the first try — but only by embedding a longer `nExpiryHeight` into every send. Every user would then wait ~19 minutes for mempool expiry on **all** subsequent sends, including fast local machines where 5 blocks is already sufficient.

**Better framing for the paper:** expiry delta is a **product policy knob** for confirmation feedback; proving latency is an **implementation/runtime** problem solved by refresh-and-rebuild.

---

## On-chain expiry formula (canonical)

Zebrad validates against the **next-block** mempool context:

```text
tx_build_height = chain_tip + 1
nExpiryHeight   = tx_build_height + PILOT_EXPIRY_DELTA_BLOCKS
                = chain_tip + 1 + 5   (with default delta)
```

Implemented in `src/fee_policy.rs` as `pilot_expiry_height(tip, delta)`.

Wallet history must persist the **encoded** value from the signed transaction, not a pre-build estimate (`tip + 5` without the `+1` context was a metadata bug fixed in BUG-2026-011).

---

## Fix summary (commits on master)

| Commit | Change |
|--------|--------|
| `42505ff1` | BUG-2026-011: late tip refresh, prove rebuild loop, broadcast retry, unified send path |
| `a72bc6e8` | Revert `PILOT_EXPIRY_DELTA_BLOCKS` 15 → **5**; document why 15 hurts speed-up UX |

| Mechanism | Location | Purpose |
|-----------|----------|---------|
| `pilot_expiry_height`, `is_expiry_consensus_error` | `src/fee_policy.rs` | Single source of truth |
| Late tip refresh + prove rebuild loop | `src/orchard_tx.rs` | Build-clock race |
| `build_and_broadcast_send_transaction` | `src/transaction_builder.rs` | Broadcast `-25` retry |
| Unified send path | CLI, api-server, desktop | Consistent behavior |
| `PILOT_EXPIRY_DELTA_BLOCKS = 5` | unchanged | Fast expire/speed-up UX |

`PILOT_EXPIRY_MAX_REBUILD_ATTEMPTS = 3`.

---

## Design principles (for paper § discussion)

1. **Separate build latency from mempool policy.** Proving time varies by CPU; pilot expiry should not be inflated to mask slow builds.

2. **Fail fast after broadcast, not before.** Pre-broadcast expiry is a consensus rejection — user sees an error immediately. Post-broadcast expiry is a wallet state transition — user gets speed-up.

3. **Rebuild is cheaper than wrong UX.** Extra CPU on slow hosts (up to 3 rebuilds) preserves the 5-block pilot contract for all users.

4. **Operator stacks need the same code path as dev laptops.** VPS/WSL latency is normal; wallets must refresh tip late and retry, not assume sub-block prove times.

---

## FAQ — common misunderstandings

### “The send took six minutes” — no, the expiry *window* is ~6 minutes

`PILOT_EXPIRY_DELTA_BLOCKS = 5` at ~75 s/block ≈ **6 minutes of chain time** for the **validity window** after expiry is encoded — not a timer that says “every send takes 6 minutes.”

Gilmore’s failure: encoded expiry **3385380**, tip at broadcast **3385384**. Working backward, expiry was stamped around tip **3385374**; the chain advanced **~10 blocks** (~12+ minutes) during witness + prove + sign. The tx was **4 blocks past** its encoded expiry at broadcast.

### Zodl (recipient) does not affect Nozy prove time

[Zodl](https://zodl.com/) (formerly Zashi) is the ZODL team’s mobile wallet. If Gilmore sent **to** a Zodl `u1` address, that does not change Orchard proving — all proving runs on the **sender** (Nozy api-server on VPS).

| | Zodl send (typical) | Gilmore’s Nozy VPS send |
|---|---------------------|-------------------------|
| Runtime | Native mobile | VPS / WSL + JSON-RPC |
| Witness | Mobile / lightwalletd optimized | Block-by-block Zebra RPC if witness lags |
| Tx expiry policy | Wallet default (longer) | 5-block pilot |
| First prove | Warm on device | Cold ProvingKey + slow RPC |

Zodl feeling “fast” does not mean Nozy on a VPS should match it without the rebuild fix.

### Why we rejected 15 blocks (June 2026)

| Approach | Build-time broadcast | Post-broadcast speed-up UX |
|----------|----------------------|----------------------------|
| **5 blocks + rebuild/retry** | Fixed via late tip + retries | ~6 min to Expired |
| **15 blocks, no rebuild** | More headroom on slow hosts | ~19 min to Expired — **rejected** |

Product decision: keep **5**; fix slow hosts in software.

---

## Mainnet verification

Step-by-step test plan: [`docs/MAINNET_SEND_EXPIRY_TEST.md`](../MAINNET_SEND_EXPIRY_TEST.md).

**Recorded field evidence (June 2026):** [`MAINNET_SEND_READINESS_EVIDENCE.md`](MAINNET_SEND_READINESS_EVIDENCE.md) — sync/send timings, TXIDs, witness guard, lecture outline.

---

## Send-readiness layer (June 2026)

After BUG-2026-011, operator testing showed **stale witnesses** (not expiry encoding) dominated send time when `last_scan_height` lagged thousands of blocks behind tip.

| Mechanism | Threshold / behavior |
|-----------|----------------------|
| `ensure_cached_witness_fresh_for_send` | Reject if unspent note witness lag **> 50 blocks** |
| `warm_orchard_proving_key` | ~2 s cold; cached thereafter |
| Parallel witness catch-up | 10 `getblock` RPCs per batch when lag ≤50 |

**Observed mainnet (WSL Zebrad, June 2026):**

| State | Send latency |
|-------|----------------|
| ~5000 blocks stale | ~7+ min (witness catch-up) or **0.09 s** reject with guard |
| Synced (lag 1 block) | **~200 s (~3.3 min)** broadcast success |

See [`MAINNET_SEND_READINESS_EVIDENCE.md`](MAINNET_SEND_READINESS_EVIDENCE.md) for full matrix and paper paragraph.

---

## Paper paragraph (copy-paste draft)

> NozyWallet participates in the Shielded Labs dynamic-fee pilot with client-side ZIP-317 fees, an optional four-fold priority multiplier, and a five-block transaction expiry after the mempool build height—approximately six minutes at mainnet block times. This short expiry enables an expire-and-replace workflow: unconfirmed sends transition to an expired state quickly, releasing spent-note locks and allowing rebuild at priority fee. VPS operator testing revealed that Orchard Halo2 proving on constrained hardware can span multiple blocks between transaction construction and broadcast. When expiry height was encoded from an early chain-tip sample, Zebrad correctly rejected otherwise valid transactions whose expiry had passed—a distinct failure mode from post-broadcast mempool expiry. We address proving latency through late chain-tip refresh before encoding expiry, automatic rebuild when proving outruns the window, and broadcast retry on consensus expiry errors, deliberately preserving the five-block delta rather than extending it to fifteen blocks, which would defer expire-and-replace feedback by roughly nineteen minutes and degrade payment UX.

---

## References

- Mainnet test guide — [`docs/MAINNET_SEND_EXPIRY_TEST.md`](../MAINNET_SEND_EXPIRY_TEST.md)
- BUG-2026-011 — [`docs/issues/bugs/2026-06-send-expiry-before-broadcast.md`](../issues/bugs/2026-06-send-expiry-before-broadcast.md)
- Pilot evidence (post-broadcast expiry) — [`docs/PILOT_MAINNET_EVIDENCE.md`](../PILOT_MAINNET_EVIDENCE.md)
- Implementation — `src/fee_policy.rs`, `src/orchard_tx.rs`, `src/transaction_builder.rs`
