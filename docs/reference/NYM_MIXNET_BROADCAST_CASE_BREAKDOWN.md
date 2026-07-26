# Nym mixnet broadcast — case breakdown (track B / issue #147)

**Status:** Engineering hardened 2026-07-25 — D2a re-proveable; D2b/D2c live PASS still needs exit-reachable Zebrad  
**Date:** 2026-07-25  
**Tracking:** [issue #147](https://github.com/LEONINE-DAO/Nozy-wallet/issues/147)  
**Nym guidance:** [zcash-sdk.nym.com/guidance](https://zcash-sdk.nym.com/guidance/) · [recommended architecture](https://zcash-sdk.nym.com/recommended/)  
**Related:** [NYM_IP_PRIVACY_CASE_BREAKDOWN.md](NYM_IP_PRIVACY_CASE_BREAKDOWN.md) · [NYM_IRONWOOD_BASELINE_HYGIENE_CASE_BREAKDOWN.md](NYM_IRONWOOD_BASELINE_HYGIENE_CASE_BREAKDOWN.md) · [NYM_MIXNET_BROADCAST_FORUM_ARTICLE.md](NYM_MIXNET_BROADCAST_FORUM_ARTICLE.md)  
**Code:** [`tools/nym-smolmix-broadcast-spike/`](../../tools/nym-smolmix-broadcast-spike/) · [`src/nym_mixnet_broadcast.rs`](../../src/nym_mixnet_broadcast.rs) · `nozy privacy-network nym-mixnet`

Case IDs here deepen **family D2** (mixnet submit). Hygiene (H*) and dVPN sync (D1) live in their own docs.

---

## Living scoreboard

| ID | Item | Status | Evidence |
|----|------|--------|----------|
| D2a | IP relocate (clearnet ≠ mixnet exit) | **PASS** (2026-07-11; **re-PASS 2026-07-26**) | `docs/reference/evidence/nym-d2a-rerun.json` |
| D2b-reachability | Refuse LAN / classify candidate URLs | **PASS** | Live WSL node `172.20.199.206:8232` correctly **N/A** — `nym-d2b-wsl-lan-refuse.json` |
| D2b | `getblockcount` over mixnet to exit-reachable Zebrad | **N/A on this machine** | Operator Zebrad is WSL LAN only (Case A1). Public RPC still needed for remote-submit proof. |
| D2c-wire | Wallet subprocess gate for remote `sendraw` | **Landed** | `nym_mixnet_broadcast` + env/config |
| D2c-local-skip | Local/LAN never uses mixnet helper | **PASS** (unit) | Case A1 preserved — **this is the correct path for the running WSL node** |
| D2c-live | Real remote `sendrawtransaction` over mixnet | **Open / not required for local-node ops** | Only needed when zebra_url is public |
| D2d | Ironwood migrate-broadcast shares gate | **Wired** | Priority 1 `NymMixnetBroadcast` |
| D2e | Desktop live session UI | **Partial** | CLI `privacy-network nym-mixnet` landed; full UI later |
| D2-evidence | Structured JSON evidence writer | **Landed** | `--evidence-json path` |

---

## Why track B exists

Nym’s recommended native hybrid puts **broadcasts on the mixnet**. Baseline hygiene (track A) decorrelates timing and start heights. Track B is the **transport** half for submits:

> Biggest win: remote lightwalletd / Zebrad must not see the **host IP** when the wallet submits a tx (send, migrate-broadcast, split).

Local Zebrad (Case A1) already wins without Nym. Track B is for **remote** submit stacks.

---

## Case family D2 — Mixnet submit

### Case D2a — IP relocate

**Goal:** Prove smolmix egress changes the public IP observers see.

**How:** Cloudflare `/cdn-cgi/trace` clearnet vs through tunnel.

**Historical PASS (2026-07-11):** clearnet `2607:fb92:…` vs mixnet `154.26.153.210` (~12s).

**Re-run:**

```powershell
cd tools/nym-smolmix-broadcast-spike
cargo run --release -- --ip-relocate --evidence-json ..\..\docs\reference\evidence\nym-d2a.json
```

### Case D2b-reachability — Don’t confuse LAN refusal with broken mixnet

**Trap:** Wallet config `http://172.20.x.x:18232` works for WSL Zebrad. A **public Nym exit cannot route RFC1918**.

**Decision:** Spike refuses private/loopback **before** opening a tunnel. Result is **N/A**, not FAIL.

```powershell
cargo run --release -- --dry-reachability --zebra http://127.0.0.1:8232
cargo run --release -- --dry-reachability --zebra http://172.20.199.206:18232
cargo run --release -- --dry-reachability --zebra https://example.com:18232
```

### Case D2b — JSON-RPC probe over mixnet

**Goal:** `getblockcount` through smolmix to an **exit-reachable** Zebrad JSON-RPC.

**Blocker:** No committed public Zebrad RPC in-repo (operators must supply one). Until then status stays **BLOCKED**, not FAIL.

```powershell
$env:ZEBRA_URL = "https://EXIT_REACHABLE_HOST:18232"
cargo run --release -- --rpc-probe --zebra $env:ZEBRA_URL --evidence-json ..\..\docs\reference\evidence\nym-d2b.json
```

A JSON-RPC **error** body that still arrives over mixnet may be logged as `D2b-path` PASS (proves TCP/HTTP path; fix auth separately).

### Case D2c-wire — Wallet subprocess gate

**Why subprocess:** In-process smolmix hits `libsqlite3-sys` **links** clash with zeaking `rusqlite`.

**Flow:**

```text
ZebraClient::broadcast_transaction
  → maybe_broadcast_via_nym_mixnet
       if enabled && remote URL:
         spawn helper --sendraw-stdin --zebra <url>
       else:
         direct / Tor path
```

**Enable:**

- `privacy_network.broadcast_via_nym_mixnet = true` and/or `NOZY_BROADCAST_VIA_NYM_MIXNET=1`
- `NOZY_NYM_SMOLMIX_BIN` → release helper path
- Optional `NOZY_NYM_IPR`

**CLI readiness (no tunnel):**

```powershell
nozy privacy-network nym-mixnet
```

### Case D2c-local-skip

Even when mixnet is enabled, **local/LAN URLs stay direct**. Unit test: `local_url_skips_helper_even_when_enabled`.

### Case D2c-live — Real remote sendraw

**Needs:** D2b-capable URL + a valid raw tx hex (testnet preferred).

```powershell
$env:NOZY_NYM_SMOLMIX_BIN = (Resolve-Path .\target\release\nym-smolmix-broadcast-spike.exe).Path
$env:NOZY_BROADCAST_VIA_NYM_MIXNET = "1"
# point wallet zebra_url at exit-reachable host, then send / ironwood broadcast
```

Do **not** claim product “Nym support” until this row is **PASS** with evidence JSON.

### Case D2d — Ironwood shares egress

`execute_orchard_migration_broadcast` → same `ZebraClient::broadcast_transaction`. Safer-migration mode `NymMixnetBroadcast` when helper resolves.

### Case D2e — Operator / UI surface

| Surface | Status |
|---------|--------|
| `nozy privacy-network nym-mixnet` | **Landed** |
| Spike `--evidence-json` | **Landed** |
| Desktop live mixnet session panel | Later |

---

## Operator runbook (scripts)

```powershell
# From repo root
powershell -ExecutionPolicy Bypass -File scripts\nym-smolmix-d2-evidence.ps1
# Optional live probe:
powershell -ExecutionPolicy Bypass -File scripts\nym-smolmix-d2-evidence.ps1 -ZebraUrl https://EXIT_REACHABLE:18232 -IpRelocate
```

Writes under `docs/reference/evidence/` when paths are writable.

---

## Evidence log

| Date (UTC) | Step | Result | Detail |
|------------|------|--------|--------|
| 2026-07-11 | D2a | **PASS** | clearnet ≠ mixnet exit IP |
| 2026-07-11 | D2b to LAN | **N/A** | refused RFC1918 |
| 2026-07-11 | D2c-wire | **Landed** | subprocess gate |
| 2026-07-25 | D2b-reachability dry evidence | **PASS** | `docs/reference/evidence/nym-d2-dry-local.json`, `nym-d2-lan-refuse.json`, `nym-d2-candidate.json` |
| 2026-07-25 | D2c-local-skip | **PASS** | unit test |
| 2026-07-25 | Evidence harness | **Landed** | `--evidence-json`, `--dry-reachability`, CLI readiness |
| 2026-07-25 | D2b live / D2c-live | **BLOCKED** | awaiting operator exit-reachable Zebrad |
| 2026-07-26 | Live WSL Zebrad `172.20.199.206:8232` | **UP** | `getblockcount` → tip **3425397** (Windows→WSL). Mixnet dry-reachability correctly **N/A** (RFC1918). |
| 2026-07-26 | D2a IP relocate re-run | **PASS** | clearnet `2607:fb90:…:e819`; mixnet `82.221.101.117` (~9.6s) — `evidence/nym-d2a-rerun.json` |
| 2026-07-26 | D2b vs live node | **N/A** | Same LAN refuse; Case A1 is the intended submit path for this host |

---

## Honest limits

1. Mixnet hides **IP from the destination**; destination still sees tx bytes + arrival time → hygiene (track A) still required.
2. No NYM bandwidth credentials ⇒ tunnel build fails — acquire via [swap.nym.com](https://swap.nym.com/) before migration day.
3. Crowding / shared exits matter for anonymity-set size (Nym ops note).
4. Extension remains **mixFetch** later; this track is native CLI/desktop/api-server.

---

## AI disclosure

Track B engineering and this case breakdown assisted by Cursor Agent. Human review required before forum claims of live remote submit PASS.
