# Joaco `lwd-mixnet-proxy` — use case & case breakdown

**Status:** External operator measurement **ongoing**; **not** wired into Nozy product submit today  
**Date:** 2026-08-20  
**Author surface:** LaDale / Lowo88  
**Upstream:** [jpgonzalezra/lwd-mixnet-proxy](https://github.com/jpgonzalezra/lwd-mixnet-proxy) · [forum #57000](https://forum.zcashcommunity.com/t/lwd-mixnet-proxy-light-wallet-grpc-over-the-nym-mixnet-and-what-three-days-of-measuring-it-found/57000)  
**Nozy neighbor PRs:** [PR #3](https://github.com/jpgonzalezra/lwd-mixnet-proxy/pull/3) (Aug 15 load) · [PR #6](https://github.com/jpgonzalezra/lwd-mixnet-proxy/pull/6) (midnight heartbeat write-up)  
**Related Nozy papers:** [NYM_SEND_EGRESS_CASE_BREAKDOWN.md](NYM_SEND_EGRESS_CASE_BREAKDOWN.md) (path E4) · [NYM_OMNIBUS_CASE_BREAKDOWN.md](NYM_OMNIBUS_CASE_BREAKDOWN.md) (J1–J2) · [NYM_MIXNET_BROADCAST_CASE_BREAKDOWN.md](NYM_MIXNET_BROADCAST_CASE_BREAKDOWN.md) (native Zebrad smolmix)

This paper explains **what Joaco’s proxy is for**, **which wallet calls belong on it**, **how Nozy uses it as an operator** (measurement + forum evidence), and **how that differs from Nozy’s own mixnet submit path**. It is written for forum reviewers, Nym people, and future Nozy contributors who might otherwise conflate “mixnet enabled in config” with “this sidecar is the wallet.”

---

## For newcomers (read this first)

Joaco’s project is a **byte pipe**: your wallet still speaks ordinary lightwalletd gRPC; a local client half listens on `:9068`, opens mixnet streams to a remote serving half, which forwards bytes to real lightwalletd. Neither half parses gRPC.

Nozy’s **product** submit path today is different: **`sendrawtransaction` over Zebrad JSON-RPC** via `nym-smolmix-broadcast-spike` (path **E1**), not LWD gRPC. The Joaco sidecar is path **E4**: **measured neighbor**, optional future second URL, **never default sync**.

| Question | Short answer |
|----------|--------------|
| What problem does it solve? | Hide **IP↔small LWD call** (submit, lookups) on a **public** lightwalletd — the calls that leak the most per byte. |
| What must you **not** do? | Full compact-block **sync** over the mixnet; sync+submit through the **same hosted** LWD back-to-back. |
| What is Nozy doing with it? | Running the **client half** against Joaco’s testnet serving address, capturing **metrics + midnight heartbeats**, contributing measurements upstream — **not** shipping it as default wallet transport. |
| TCP connect to `:9068` enough? | **No.** TCP only proves the local half accepted a socket. Real wallet traffic is **gRPC** (`GetLightdInfo`, `SendTransaction`, …) and is what moves **`upstream_connections_total`** on the serving half. |
| Mixnet flag ON in Nozy + LAN Zebrad? | Still **Local** submit (Case A1). Unrelated to whether this Docker client is running. |

---

## The privacy use case

Shielded Zcash hides note contents. It does **not** hide that **your IP** contacted a light wallet server. Leakage is **inverse to bandwidth**:

| Call class | Bytes | Leak severity | Recommended transport |
|------------|-------|---------------|------------------------|
| Compact-block sync | Very large | Lower (trial decrypt locally) | Local LWD, or **dVPN / Fast mode** if remote |
| `SendTransaction` / submit | Tiny | **High** (IP↔tx) | Local node, or **mixnet** if remote |
| Transparent-address / tx lookup | Small | **High** | Mixnet or Tor; not bulk sync path |
| `GetLightdInfo` | Tiny | Moderate (health check, tip) | Fine over mixnet for **probes**; not a sync substitute |

Joaco’s design target is the **small, high-leak** row: carry gRPC for submit and lookups when you cannot run your own node, without rewriting the wallet.

Nym’s recommended hybrid for Zcash wallets matches this split: [recommended hybrid scenario](https://zcash-sdk.nym.com/scenario/recommended-hybrid/).

---

## Architecture (what it is)

```
  wallet / probe  --TCP gRPC-->  [lwd-mixnet-client :9068]
                                        |
                                   Nym mixnet
                                   (probe, open-3,
                                    watchdog)
                                        |
                                 [lwd-mixnet-server]
                                        |
                                   --TCP-->  lightwalletd
```

**Transport defect that drives the design:** a mixnet stream can open, be accepted, and **silently lose its first payload**. gRPC libraries retry on errors, not on silence. The client half therefore:

1. **Probes** streams before the wallet sends bytes (dead streams swallow the probe, not the wallet).
2. Opens **three streams at a time**, keeps the first to answer (p99 establish ~6.3 s vs ~31 s with serial retry — Joaco’s measurements).
3. Runs a **watchdog** so hung far sides become a closed socket the wallet can retry.

Nozy does **not** maintain this crate in-tree as a product dependency. We clone/run via `tools/lwd-mixnet-proxy/` (Docker) for operator evidence.

---

## Where this sits in Nozy (E4 vs E1 vs E2)

```
                    Nozy wallet surfaces
                              |
         +--------------------+--------------------+
         |                    |                    |
    Sync (bulk)          Submit (tiny)        Joaco sidecar
         |                    |                    |
    E2 dVPN spike      E1 smolmix spike       E4 measured only
    zeaking LWD        Zebrad JSON-RPC        LWD gRPC :9068
    zec.rocks etc.     sendraw                (Docker client)
         |                    |                    |
    NOT mixnet         NOT LWD gRPC today     NOT in send badge
```

| Path | Socket | Nozy status | Operator box (2026-08-19) |
|------|--------|-------------|---------------------------|
| **E0** Local Zebrad | `http://172.20…:8232` | Default submit | **Active** — badge **Local** |
| **E1** smolmix submit | Zebrad JSON-RPC | Wired; live remote open | N/A (LAN Zebrad) |
| **E2** dVPN sync | LWD gRPC via tunnel | PASS July pins | Off (`sync_via_nym_dvpn: false`) |
| **E4** Joaco proxy | LWD gRPC → `:9068` | **Measured, not product** | Docker client **healthy** |

**Policy (do not drift):** two URLs when using hosted infrastructure — one instance for sync, another for anonymous submit, separated in time. Wallet policy, not something the pipe enforces.

---

## Living scoreboard (J-track)

| ID | Item | Status | Notes |
|----|------|--------|-------|
| J-UC | Use-case clarity (small calls yes, sync no) | **Documented** | Joaco README + this paper |
| J1 | External client-half load test | **PASS** | 2026-08-15; PR #3; ADR 0012 probe-attempts 4→6 |
| J2 | Longevity / idle reclaim | **PASS** | ~50h+ RestartCount 0; midnight reclaim ~200–525 ms |
| J3 | Midnight ±10 min heartbeat (TCP) | **Captured** | Two nights: dead far end vs live far end |
| J4 | Midnight heartbeat (**GetLightdInfo**) | **Harness landed** | `scripts/lwd-mixnet-midnight-heartbeat.ps1` + `verify_lwd` |
| J5 | Cross-machine correlation with Joaco | **Partial** | Aug 15 load matched his server metrics; Aug 18 dead serving half explained 19/19 |
| J6 | Product wire optional LWD submit URL | **Open / defer** | Only after sustained grpc ok; never after 19/19 unanswered window |
| J7 | PR #6 forum measurement package | **Open** | May need update with Aug 20 live rerun + gRPC probes |
| J8 | Max (#18): probe/watchdog = SDK **reliability layer** | **Forum 2026-08-24** | Offered PR review + stream error plumbing; Joaco #19/#20; LaDale `develop` rerun — [FORUM_DRAFT_MAXNYM_57000_REPLY.md](FORUM_DRAFT_MAXNYM_57000_REPLY.md) |

---

## Use-case cases (what belongs on the pipe)

### Case UC-1 — Submit shielded transaction (primary)

**When:** Wallet submits via lightwalletd `SendTransaction` (or equivalent), remote LWD, user accepts mixnet latency.  
**Why mixnet:** Strongest IP↔tx link in the light-client protocol.  
**Expectation:** Multi-second establish; rely on probe + open-3; treat `connections_unestablished` as user-visible failure.  
**Nozy today:** Submit goes to **Zebrad E1**, not here — unless we add an explicit optional LWD submit URL later (J6).

### Case UC-2 — Transparent-address or single-tx lookup

**When:** Small query that directly reveals addresses or tx ids to the server.  
**Why mixnet:** High leak, few bytes — same class as submit.  
**Nozy today:** Not product-wired through E4.

### Case UC-3 — Health / tip probe (`GetLightdInfo`)

**When:** Operator or wallet checks chain name, tip height, backend version.  
**Why mixnet:** Acceptable for **measurement** and occasional health checks; Joaco asked external runners to use this instead of TCP-only dials.  
**Evidence:** Rehearsal 2026-08-20 — 7/7 grpc ok, p50 ~4.2 s, chain `test`, height advancing.  
**Tooling:** `verify_lwd http://127.0.0.1:9068` (exit 2 ok on testnet without donation UA).

### Case J8 — Nym SDK “reliability layer” (Max #18)

**Forum:** [post #18](https://forum.zcashcommunity.com/t/lwd-mixnet-proxy-light-wallet-grpc-over-the-nym-mixnet-and-what-three-days-of-measuring-it-found/57000/18) (@maxnym).

Max named Joaco’s probe / open-3 / watchdog (the same mechanisms our client-half evidence exercises) as **reliability-layer functionality the SDK currently lacks**, and offered to:

1. Review a PR that abstracts it for SDK modules.  
2. Thread `no node with identity … is known` up the stream API as a catchable connection error (today: WARN only; caller sees silence).

Joaco [#19](https://forum.zcashcommunity.com/t/lwd-mixnet-proxy-light-wallet-grpc-over-the-nym-mixnet-and-what-three-days-of-measuring-it-found/57000/19) accepted the layer + wire-format issue path. Joaco [#20](https://forum.zcashcommunity.com/t/lwd-mixnet-proxy-light-wallet-grpc-over-the-nym-mixnet-and-what-three-days-of-measuring-it-found/57000/20) showed `develop` (#7057) zeros first-payload residue that `1.21.5-rc.3` still shows — and asked **LaDale** for an independent `develop` rerun.

**Nozy role (honest):** We do **not** claim ownership of the SDK PR. We claim **external measurement** that made the gap visible, plus a standing offer of second-machine evidence and a wallet-caller design note. Paste reply: [FORUM_DRAFT_MAXNYM_57000_REPLY.md](FORUM_DRAFT_MAXNYM_57000_REPLY.md).

**Grant / ecosystem line:** Nym engineer validated the measurement campaign as upstream-relevant (missing SDK surface), not “wallet debug only.”

### Case UC-4 — Historical compact-block sync (reject)

**When:** Wallet birthday is far in the past; tens of GB over mixnet.  
**Why not:** Median RTT in seconds → days of sync; wrong tool. Use **local LWD** or **dVPN (E2)**.  
**Nozy policy:** Never route zeaking compact sync through Joaco client.

### Case UC-5 — Same hosted LWD for sync then anonymous submit (reject)

**When:** User syncs on `lwd.example.com`, then submits through mixnet to the **same** operator moments later.  
**Why not:** Timing correlation; negligible anonymity set with few concurrent users.  
**Wallet policy:** Separate sync and submit infrastructure.

---

## Operator measurement cases (what Nozy actually ran)

### Case M1 — Aug 15 external load (forum #7 ask)

**Setup:** Docker client only; Joaco public testnet `SERVER_ADDRESS`; TCP connect/close to `:9068` (no gRPC bytes).  
**Results:**

| Metric | Value |
|--------|-------|
| `connections_total` | 30 |
| First-round failures | 5/30 (**16.67%**) |
| Wallet-visible unestablished | 2/30 (**6.67%**) |
| Streams discarded (unanswered) | 17 |
| Bench wallet-visible (20 trials) | **0%** with open-3 |

**Outcome:** First external reproduction of Joaco’s ladder math; [PR #3](https://github.com/jpgonzalezra/lwd-mixnet-proxy/pull/3) merged; upstream raised default `--probe-attempts` 4→6 (ADR 0012).

**Evidence:** `docs/reference/evidence/lwd-mixnet-client-run-20260815.txt`

### Case M2 — Idle longevity + midnight reclaim

**Setup:** Client left up `restart: unless-stopped`; Docker Desktop kept alive.  
**Results:** RestartCount **0**; ~50h+ serving; bandwidth reclaim at **00:00 UTC** (~200–525 ms) matches Joaco’s serving-half accounting cliff — testnet quota, not gateway death.

**Evidence:** `docs/reference/evidence/lwd-mixnet-client-longevity-20260817.txt`

### Case M3 — Midnight heartbeat, **dead** serving half (2026-08-19 UTC)

**Setup:** `scripts/lwd-mixnet-midnight-heartbeat.ps1`; ±10 min around 00:00 UTC; **TCP only** (pre-gRPC harness).  
**Results:**

| Layer | Result |
|-------|--------|
| Local TCP `:9068` | **20/20 ok** (~210 ms) — misleading if read alone |
| Mixnet answered | **0/19** — `first_round_failures_total 19`, `streams_discarded unanswered 76` |
| Reclaim | ~298 ms at midnight — cliff happened but path was already silent |

**Interpretation:** Joaco [#57000/14](https://forum.zcashcommunity.com/t/lwd-mixnet-proxy-light-wallet-grpc-over-the-nym-mixnet-and-what-three-days-of-measuring-it-found/57000/14): serving half **dead since Aug 18 14:12Z** (gateway identity stale). Local port up ≠ mixnet path up.

**Evidence:** `docs/reference/evidence/lwd-mixnet-midnight-heartbeat-20260818-215629.txt`

### Case M4 — Midnight heartbeat, **live** serving half (2026-08-20 UTC)

**Setup:** Recreated client with new gateway `6PkVkJ8nq882V1C95uCHUUoBWgo1ZzWHVpDtrhSjwDVn`; same script; TCP only that night.  
**Results:**

| Layer | Result |
|-------|--------|
| Local TCP | **20/20 ok** |
| Mixnet answered | **20/20** — `connections_unestablished_total 0`, `streams_opened_total 57` (19×3) |
| Reclaim | ~353 ms at `00:00:00.602Z` |
| SDK warnings | Zero dead-gateway WARNs (contrast with 154× `no node with identity 3xLD3rp…` on dead night) |

**Evidence:** `docs/reference/evidence/lwd-mixnet-midnight-heartbeat-20260819-225456.txt`

### Case M5 — gRPC probe harness (2026-08-20 rehearsal)

**Setup:** Heartbeat script enhanced: each sample = TCP + `verify_lwd` GetLightdInfo over `:9068`.  
**Results:** grpc **7/7 ok**, p50 **~4235 ms**, tip height advanced 4286872→4286873; client metrics `connections_unestablished 0`, bytes moved on wire.

**Evidence:** `docs/reference/evidence/lwd-mixnet-midnight-heartbeat-20260820-021007.txt`

**Lesson:** Always report **both** TCP and gRPC in forum updates. TCP-only PASS on a dead far end was the Aug 18 trap.

---

## Measurement methodology

### Client half (operator)

1. Clone upstream or use `tools/lwd-mixnet-proxy/`.
2. Set `.env` `SERVER_ADDRESS` from Joaco’s serving half (gateway identity matters — dead gateway → 19/19 unanswered).
3. `docker compose up -d client` → wallet port `127.0.0.1:9068`, metrics `127.0.0.1:9070`.
4. Snapshot `/health` and `/metrics` before and after load windows.

### Midnight window (Joaco forum ask)

```powershell
# Real run: wait until ±10 min around 00:00 UTC (~7:00 PM EST same calendar evening)
powershell -ExecutionPolicy Bypass -File scripts\lwd-mixnet-midnight-heartbeat.ps1

# Rehearsal (immediate, short window)
powershell -ExecutionPolicy Bypass -File scripts\lwd-mixnet-midnight-heartbeat.ps1 `
  -SkipWait -MinutesBefore 0 -MinutesAfter 1 -IntervalSeconds 15
```

Each minute logs:

- `tcp ok=… ms=…` — local accept only  
- `grpc ok=… ms=… detail=grpc_probe ok=1 … height=…` — wallet-shaped call  

Attach BEFORE/AFTER `lwd_mixnet_client_*` counters to forum posts. Note gateway, SDK pin, UTC timestamps, RestartCount.

### Key metrics to quote

| Counter | Meaning |
|---------|---------|
| `connections_total` | Local connections accepted |
| `connections_unestablished_total` | **Wallet-visible failure rate** |
| `first_round_failures_total` | Transport failure before retry |
| `streams_opened_total` | Mixnet streams (expect ~3× answered connections with open-3) |
| `streams_discarded_total{reason="unanswered"}` | Probes lost to silent streams |
| `establishment_seconds` histogram | Tail latency users feel |

---

## ASCII: one midnight minute (live path)

```
  verify_lwd / wallet
         |
         |  gRPC GetLightdInfo (~3–6 s)
         v
  :9068 client half  ----mixnet (3 streams, 1 answers)---->  serving half  -->  LWD
         |
         |  metrics: rounds_total += 1, streams_opened += 3
         v
  :9070 /metrics
```

On a **dead** far end, TCP to `:9068` still succeeds in ~200 ms, but mixnet counters show **unanswered** streams and **`connections_unestablished`** climbs if the wallet sent real bytes.

---

## Product stance for NozyWallet

### What we **are** doing

- Contributing **reproducible external measurements** to Joaco’s project and forum thread.
- Documenting hybrid policy so contributors do not put compact sync on mixnet.
- Send egress badge (E0–E6) honestly separates **Local** vs **Mixnet** vs **Blocked** for **Zebrad submit**.
- Keeping measurement harness in-repo: `scripts/lwd-mixnet-midnight-heartbeat.ps1`, evidence under `docs/reference/evidence/`.

### What we **are not** doing (yet / maybe never)

- Defaulting wallet LWD URL to `127.0.0.1:9068` for all users.
- Claiming “Nozy has Nym integrated” because a Docker sidecar runs on an dev machine.
- Using Joaco client for compact sync (UC-4).
- Shipping after a **19/19 unanswered** night without grpc ok on a **live** serving half.

### Possible J6 (deferred)

Optional config: `lightwalletd_submit_url = http://127.0.0.1:9068` **only** for operators who explicitly run the client half **and** submit via LWD rather than Zebrad. Requires:

- grpc probe success sustained over reclaim windows  
- UI copy that this is **not** sync transport  
- Failure mapping from `connections_unestablished` to send errors  

Native Zebrad smolmix (E1) remains the intended Nozy submit integration for remote RPC.

---

## Forum record (condensed)

| When | Who | Takeaway |
|------|-----|----------|
| Aug 2026 | Joaco #57000 | Byte pipe; 2–51% silent stream rate; open-3; **two URLs** must be wallet policy |
| Aug 15 | Lowo88 run | External PASS; PR #3; probe-attempts 4→6 |
| Aug 16–17 | Joaco #5–#7 | Midnight reclaim is testnet accounting; wants gateway + SDK + raw metrics; asks **GetLightdInfo** not TCP-only |
| Aug 18 | Joaco #14 | Serving half dead; explains 19/19 unanswered |
| Aug 19–20 | Lowo88 rerun | Live gateway → 20/20 answered; gRPC harness landed |

**Paste-ready forum anchor:** dead vs live table in **Case M3 / M4** above; attach evidence filenames; link PR #3 and PR #6.

---

## Reproduction checklist (this operator box)

```powershell
# Client health
curl http://127.0.0.1:9070/health
curl http://127.0.0.1:9070/metrics

# Single gRPC probe (mixnet-shaped)
cargo build --release --bin verify_lwd
target\release\verify_lwd.exe http://127.0.0.1:9068
# exit 0 or 2 with block_height = ok on Joaco testnet

# Nozy send path (unchanged — still Local on LAN Zebrad)
nozy privacy-network send-egress
```

Keep **Docker Desktop** running for overnight longevity and midnight windows.

---

## Open items

1. **Next real midnight run** with gRPC probes enabled (not TCP-only) on live serving half.  
2. **Update PR #6** / forum post with M4 + M5 contrast (dead vs live + grpc).  
3. **Joaco server-side:** confirm `upstream_connections_total` moved when grpc probes run.  
4. **J6 decision:** whether Nozy ever exposes optional LWD submit via `:9068` vs staying Zebrad-only (E1).  
5. **D2c-live:** separate track — exit-reachable Zebrad for native smolmix submit.

---

## AI disclosure

This case breakdown and the heartbeat harness were drafted with Cursor assistance. Measurements, evidence files, and forum claims remain the human operator’s responsibility. Do not cite this paper as a product guarantee.
