# Mainnet Ironwood migration evidence — NozyWallet CLI

**Status:** First Orchard → Ironwood turnstile **confirmed on mainnet**  
**Date:** 2026-07-30  
**Surface:** CLI (`nozy ironwood` plan / split / migrate / broadcast)  
**Profile (local):** `5a5c81bda1fa6343` (do not publish wallet paths beyond this id)  
**Related:** [`IRONWOOD_WALLET_READINESS.md`](IRONWOOD_WALLET_READINESS.md) · twin-note postmortem (local paper) · [forum thread](https://forum.zcashcommunity.com/t/ironwood-is-here-updated-wallets-libraries-aug-1/56557/38)

---

## Verdict (honest)

NozyWallet **has moved real mainnet ZEC from Orchard into Ironwood** via a V6 turnstile built and broadcast by the CLI. That is operator proof, not a third-party audit and not “every community user can click Migrate.”

| Claim | Evidence |
|-------|----------|
| Mainnet Orchard → Ironwood turnstile confirmed | TXID below, height **3,430,663**, tx version **6** |
| ZIP 318–shaped prep (canonical Orchard splits before crossing) | Split TXIDs below |
| Ironwood note indexed + witnessed in wallet | `notes.json` pool=`ironwood`, value **1,960,000** zat |
| Full ZIP 318 “conformant” for all wallets / desktop GA | **Not claimed** — remaining Orchard schedule still pending; desktop beta; ladder defaults differ from plain power-of-ten |
| Sapling → Ironwood shield (separate path) | TXID `526c9243…571b` (not a ZIP 318 Orchard turnstile) |

---

## Anchor TXIDs (public)

### 1) Orchard → Ironwood turnstile (the proof)

| Field | Value |
|-------|--------|
| **TXID** | `ea2fa4e64a5ca3f588dea58f38feb2a72a8d4e30292ac012d983e23bde7048fd` |
| **Height** | **3,430,663** (after NU6.3 activation **3,428,143**) |
| **Date (UTC)** | 2026-07-30 ~18:52 |
| **Version** | **6** (Ironwood-era) |
| **Zebrad `orchard.valueBalanceZat`** | **+2,000,000** (0.02 ZEC leaves Orchard) |
| **Zebrad `ironwood.valueBalanceZat`** | **−1,960,000** (0.0196 ZEC enters Ironwood) |
| **Implied fee** | **40,000** zat (difference; not a separate `fee` field on the decoded tx) |
| **Wallet Ironwood output** | **1,960,000 zat** — unspent, `pool: ironwood` |
| **Funding shape** | Canonical **2,000,000** zat Orchard note → Ironwood **1,960,000** + fee **40,000** (`fee_from_output`, zero Ironwood change) |
| **Public check** | [Blockchair](https://blockchair.com/zcash/transaction/ea2fa4e64a5ca3f588dea58f38feb2a72a8d4e30292ac012d983e23bde7048fd) (V6 at height 3430663) |
| **RPC check** | WSL Zebrad `getrawtransaction` (verbose) @ `172.20.199.206:8232` — 2026-08-01; **2454+** confirmations at query time |
| **Operator watch log** | Local `ironwood-migrate-watch.log` — prebuild TXID `ea2fa4e6…`, broadcast confirmed (mined **3430663**; log noted confirm near **3430659**) |

**Schedule caveat:** after later `ironwood preflight` / plan refresh, `ironwood_migration_schedule.json` may only show **remaining** pending transfers (e.g. **200,000** zat). The completed turnstile is proven by **chain + `notes.json`**, not by a lasting `broadcast_txid` row in the schedule. `sent_transactions.json` also lacks this turnstile entry today.

Operator verification (local Zebrad):

```bash
# Prefer WSL Zebrad (Windows :8232 may be down)
curl -s --data '{"jsonrpc":"1.0","id":"c","method":"getrawtransaction","params":["ea2fa4e64a5ca3f588dea58f38feb2a72a8d4e30292ac012d983e23bde7048fd",1]}' \
  -H 'content-type: text/plain' http://172.20.199.206:8232/
# Expect: orchard.valueBalanceZat = 2000000, ironwood.valueBalanceZat = -1960000
```

### 2) ZIP 318 Orchard note-splits (prep, same wallet)

| Role | TXID | Height |
|------|------|--------|
| Split A | `9b3b2f819a3fef8a41aa67418be383a43b3171424e6e7a4c2bfa3a7bdde9df54` | 3,428,651 | orchard **+60,000** zat (fee only; no `ironwood`) |
| Split B | `e0e233e8119da732ea959d4e35e074804214dd2be3ec4f9ee31e69167c2c0123` | 3,428,662 | orchard **+60,000** zat (fee only; no `ironwood`) |

These are Orchard→Orchard rearrangements into canonical denominations before the turnstile. They are **not** pool crossings (RPC shows no `ironwood` valueBalance).

### 3) Sapling → Ironwood shield (related, not ZIP 318)

| Field | Value |
|-------|--------|
| **TXID** | `526c92439887e715aab0c52d78eaddddec51d54f724796bbf80b83f008da571b` |
| **Height** | 3,431,668 |
| **Wallet Ironwood note** | **40,000** zat |

Useful for “Nozy can land value in Ironwood,” but **do not** cite this as ZIP 318 Orchard migration.

---

## Flow that produced the turnstile

```text
nozy ironwood plan --save
nozy ironwood split          # canonical Orchard notes
# wait for ZIP 318 anchor bucket
nozy ironwood migrate        # prebuild V6 Orchard spend → Ironwood output
nozy ironwood broadcast      # submit inside window
nozy sync                    # index Ironwood note + witnesses
```

Post-turnstile wallet snapshot (example from operator notes / local index):

| Pool | Approx unspent |
|------|----------------|
| Ironwood | **1,960,000** zat from turnstile (+ later shield note) |
| Orchard | residual including pending schedule (~**270,000** zat still scheduled) |

Current schedule still lists a **pending** transfer of **200,000** zat (next bucket) — migration of this profile is **in progress**, not finished.

---

## What this does / does not prove

**Does prove**

- CLI can build and broadcast a **mainnet V6** Orchard→Ironwood turnstile.
- Wallet can **decrypt and witness** the resulting Ironwood note.
- Prep used ZIP 318–style **canonical splits** before crossing.

**Does not prove**

- Emersonian (or anyone else) has independently reproduced the flow.
- Desktop Migrate button is community-ready / GA.
- Byte-for-byte ZIP 318 normative ladder + cadence alignment with every other wallet (Nozy also ships Shielded Labs `{1,2,5}×10^k` amount selection).
- Safer-migration Priority 1–3 (Nym/Tor defaults, shared cover) for this specific broadcast.

---

## Forum / grant paste (short)

> NozyWallet CLI completed a mainnet Orchard→Ironwood turnstile on 2026-07-30:  
> `ea2fa4e64a5ca3f588dea58f38feb2a72a8d4e30292ac012d983e23bde7048fd` at height **3430663** (V6).  
> Wallet holds the resulting Ironwood note (**0.0196 ZEC**). Prep splits: `9b3b2f81…` / `e0e233e8…`.  
> This is operator CLI evidence for ZIP 318–shaped migration; desktop remains beta and remaining Orchard notes on this profile are still scheduled.

---

## Update log

| Date | Change |
|------|--------|
| 2026-08-01 | Evidence pack created from confirmed mainnet TXID + local Ironwood note index |
| 2026-08-01 | Zebrad RPC verbose decode: orchard +2e6 / ironwood −1.96e6 zat on turnstile; splits fee-only |
| 2026-08-01 | Noted watch log + schedule refresh gap (completed turnstile not retained in schedule JSON) |
