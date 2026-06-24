# BUG-2026-011: Send fails at broadcast when Orchard proving outruns pilot expiry



**Status:** Fixed on `master` (commits `42505ff1`, `a72bc6e8`)  

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

| Approx. encode tip | ~3385374 (10 blocks before broadcast) |

| Pilot delta | **5 blocks** after mempool build height (~6 min expire feedback) |

| Environment | VPS Zebrad + api-server, WSL-style stack |

| Recipient | Any Orchard `u1` (e.g. Zodl) — **does not affect prove time** |



---



## Session context (Gilmore VPS, June 2026)



Same test session as other Gilmore bugs:



| ID | Issue | Status |

|----|-------|--------|

| BUG-2026-001 | Send rescanned ~50k blocks | Fixed (master) |

| BUG-2026-002 | History empty despite balance | Fixed (master) |

| **BUG-2026-011** | **Broadcast `-25` expiry before mempool** | **Fixed (master)** |



Gilmore was sending **from NozyWallet api-server**, not from Zodl. Zodl may have been the **recipient**; that does not change Orchard proving on the Nozy/VPS sender.



---



## Root cause



Four interacting issues:



1. **Expiry clock started too early** — chain tip was read at the beginning of `build_single_spend`, before witness fetch, bundle construction, Halo2 proving, and signing. Slow hosts (VPS/WSL) can spend many blocks in that window.



2. **Short pilot window by design** — `PILOT_EXPIRY_DELTA_BLOCKS = 5` targets the Shielded Labs dynamic-fee pilot (~6 minutes at 75 s/block). Proving latency is not bounded to that window on constrained hardware.



3. **History metadata mismatch** — api-server / CLI recorded `expiry_height = tip + 5` while the transaction encoded `(tip + 1) + 5` (Zebrad mempool build context is one block ahead of the current tip).



4. **No pre-broadcast recovery** — a tx that expired before broadcast was discarded with `-25`; speed-up requires a prior successful broadcast.



### “Six minutes” clarification



The **5-block delta is not “send duration.”** It is the **mempool validity window** after expiry is encoded. Gilmore’s build likely spanned **~10 blocks (~12+ min)** of chain time while expiry was stamped from an early tip — hence tip 3385384 vs expiry 3385380 at broadcast.



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



### 5. Pilot expiry delta unchanged (5 blocks)



`PILOT_EXPIRY_DELTA_BLOCKS` stays at **5**. We briefly tried **15** in `42505ff1` and **reverted** in `a72bc6e8` because ~19 minutes to **Expired** / speed-up hurts UX. Slow VPS/WSL reliability is handled by late tip refresh + auto-rebuild + broadcast retry.



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

| `CHANGELOG.md`, `README.md` | Unreleased notes |

| `docs/reference/PILOT_EXPIRY_PROVING_LATENCY.md` | Paper reference |

| `docs/MAINNET_SEND_EXPIRY_TEST.md` | Mainnet test guide |



---



## Expected after fix



- Sends on slow VPS/WSL stacks complete without manual retry when proving spans several blocks.

- Logs may show `Orchard proof outran pilot expiry; rebuilding (2/3)` — normal on slow hosts.

- History `expiry_height` matches the on-chain encoded value.

- Funds remain safe on failure (no mempool accept until broadcast succeeds).

- Post-broadcast: unmined txs still expire in ~6 minutes for speed-up pilot.



---



## Verification



**Full mainnet procedure:** [`docs/MAINNET_SEND_EXPIRY_TEST.md`](../../MAINNET_SEND_EXPIRY_TEST.md)



Quick checklist:



1. Build from `master` at or after `a72bc6e8`.

2. On VPS or local: `POST /api/transaction/send` or `nozy send` with **0.0001 ZEC**.

3. Confirm `success: true` + TXID; no `-25` expiry error.

4. Confirm history `expiry_height` = `(tip_at_encode + 1) + 5`.

5. Optional: `getrawtransaction` on Zebrad; optional post-broadcast expire/speed-up per [`PILOT_MAINNET_EVIDENCE.md`](../../PILOT_MAINNET_EVIDENCE.md).



---



## Paper / technical narrative (draft)



Full paper reference (two clocks, 5 vs 15, Zodl FAQ, mainnet tests): [`docs/reference/PILOT_EXPIRY_PROVING_LATENCY.md`](../../reference/PILOT_EXPIRY_PROVING_LATENCY.md).



> **Problem.** NozyWallet's dynamic-fee pilot uses a short transaction expiry window so unconfirmed sends can expire and be replaced via a priority fee speed-up. Expiry height is part of the signed transaction (ZIP-225 `nExpiryHeight`). On fast local nodes, witness fetch + Orchard Halo2 proving completes within five blocks; on operator VPS stacks, proving routinely exceeds that window, producing valid signed transactions that Zebrad rejects at mempool admission because the chain tip has already passed `nExpiryHeight`.

>

> **Observation.** Reporter Gilmore observed a fully built 0.003341 ZEC Orchard send rejected at height 3385384 with expiry 3385380 — four blocks past expiry — with no balance impact because the transaction never broadcast.

>

> **Solution.** We decouple witness anchor height (still tied to tip at spend preparation) from expiry encoding (refreshed immediately before sighash and proving), add an automatic rebuild loop when proving outruns expiry, and retry broadcast on Zebrad expiry consensus errors. Expiry metadata in wallet history now uses the same formula as on-chain encoding: `expiry_height = chain_tip + 1 + PILOT_EXPIRY_DELTA_BLOCKS`.

>

> **Why not 15 blocks?** A 15-block delta (~19 min) would reduce first-attempt broadcast failures on slow hosts but delays expire/fail feedback and the speed-up loop for **every** user. Five blocks preserves ~6-minute pilot semantics; slow-host reliability comes from late tip refresh and rebuild/retry, not a longer `nExpiryHeight`.

>

> **Trade-off.** Rebuild retries add CPU time on slow hosts but keep the pilot's **5-block** expire/replace cycle (~6 minutes) so users are not left waiting ~20 minutes to learn a send failed. Slow-host broadcast reliability comes from late tip refresh and automatic rebuild/retry, not a longer `nExpiryHeight`.



---



## References



- [`docs/MAINNET_SEND_EXPIRY_TEST.md`](../../MAINNET_SEND_EXPIRY_TEST.md) — **mainnet test guide**

- [`docs/reference/PILOT_EXPIRY_PROVING_LATENCY.md`](../../reference/PILOT_EXPIRY_PROVING_LATENCY.md) — **paper reference**

- [`docs/PILOT_MAINNET_EVIDENCE.md`](../../PILOT_MAINNET_EVIDENCE.md) — expire-after-broadcast speed-up (different failure mode)

- [`src/fee_policy.rs`](../../../src/fee_policy.rs) — pilot fee/expiry policy

- BUG-2026-002 — Gilmore history fix (same VPS test session)


