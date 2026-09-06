# Nym send egress — case breakdown (each wallet path)

**Status:** Badge **landed** 2026-08-18 (CLI + desktop). Not a claim that mixnet send is live on remote Zebrad.  
**Date:** 2026-08-18  
**Author surface:** LaDale / Lowo88  
**Code:** [`src/send_egress.rs`](../../src/send_egress.rs) · CLI `nozy privacy-network send-egress` · desktop Send + Settings  
**Related:** [NYM_OMNIBUS_CASE_BREAKDOWN.md](NYM_OMNIBUS_CASE_BREAKDOWN.md) · [NYM_MIXNET_BROADCAST_CASE_BREAKDOWN.md](NYM_MIXNET_BROADCAST_CASE_BREAKDOWN.md) · [NYM_DVPN_SYNC_CASE_BREAKDOWN.md](NYM_DVPN_SYNC_CASE_BREAKDOWN.md) · [NYM_IP_PRIVACY_CASE_BREAKDOWN.md](NYM_IP_PRIVACY_CASE_BREAKDOWN.md) · [NYM_LWD_MIXNET_PROXY_CASE_BREAKDOWN.md](NYM_LWD_MIXNET_PROXY_CASE_BREAKDOWN.md)

Shielded notes hide amounts. They do **not** hide that this host IP talked to a submit endpoint. This file is the map of **every submit/sync path** Nozy can take with the Nym kit, what the user sees, and what is still open.

---

## For Nym people and newcomers (read this first)

Nozy is **not** “in mixnet mode” or “in dVPN mode” as one global switch. Nym’s recommended hybrid is **two different hops**, plus a third path that uses **neither**:

```
  Compact sync (lots of bytes)  →  local LWD, or dVPN / Fast mode if the LWD is public
  Submit / sendraw (tiny, leaks IP↔tx)  →  local Zebrad, or mixnet (smolmix) if Zebrad is public
  Local/LAN Zebrad (loopback or RFC1918)  →  mixnet is skipped on purpose even if the mixnet flag is ON
```

| Question | Answer |
|----------|--------|
| Are mixnet and dVPN interchangeable? | **No.** Mixnet = small **send**. dVPN = bulk **sync**. Never compact-sync over the mixnet. Same rule when `LIGHTWALLETD_GRPC` points at [Nozy Sync Engine](../../nozy-sync-engine/README.md): local/LAN needs no tunnel; remote sync uses dVPN. |
| If mixnet is enabled in config, am I sending over Nym? | **Only if** `zebra_url` is a **public** (exit-reachable) RPC. LAN/WSL URLs stay **direct**. |
| What does the wallet badge mean? | `nozy privacy-network send-egress` / desktop **Next send:** — that is the hop the **next submit** will use. It does not describe sync. |
| Is consumer NymVPN the same as in-app mixnet/dVPN? | **No.** [zcash.nym.com](https://zcash.nym.com) is an **OS VPN stopgap**. In-app helpers are subprocesses (`nym-smolmix-broadcast-spike`, `nym-dvpn-lwd-spike`). |
| Joaco `lwd-mixnet-proxy` `:9068`? | Neighbor **LWD gRPC** pipe. Not how Nozy `sendraw` works today (that is Zebrad JSON-RPC). |

**Operator snapshot (this box, 2026-08-19):** `broadcast_via_nym_mixnet` is **true**, `sync_via_nym_dvpn` is **false**, Zebrad is **LAN/WSL RFC1918**. Next send is **Local** (Case A1). Mixnet is armed for a future *remote* URL; it is not wrapping LAN submit. That is correct, not a bug.

Matches Nym: [recommended hybrid](https://zcash-sdk.nym.com/scenario/recommended-hybrid/). Deep scoreboard: [NYM_OMNIBUS_CASE_BREAKDOWN.md](NYM_OMNIBUS_CASE_BREAKDOWN.md).

---

## Living scoreboard (E-UI)

| ID | Item | Status | Notes |
|----|------|--------|-------|
| E-UI | Send egress badge (Local / Mixnet / Tor / I2P / Clearnet / Blocked + stopgap) | **Landed** | CLI send summary, `privacy-network send-egress` / `nym-mixnet`, desktop Send + Network privacy |
| E0 | Local/LAN Zebrad submit (Case A1) | **Default** | Mixnet skipped on purpose |
| E1 | Remote `sendraw` via smolmix subprocess | **Wired**; live remote **open** | #147 / D2c — needs exit-reachable Zebrad |
| E2 | Remote compact sync via dVPN | **PASS** July pins | #146 / C3 — not a send path |
| E3 | Consumer NymVPN stopgap | **Copy landed** | [zcash.nym.com](https://zcash.nym.com) — not in-app Nym |
| E4 | Joaco `lwd-mixnet-proxy` LWD gRPC sidecar | **Measured, not product-wired** | Heartbeat 2026-08-20: TCP 20/20, mixnet 20/20 answered (live serving half). Prior night 19/19 unanswered (dead far end). gRPC probe harness landed. |
| E5 | Tor / I2P SOCKS submit | **Existing** | Shown on the same badge |
| E6 | Remote clearnet / blocked | **Honest** | Badge + stopgap URL; do not market as Nym |

---

## Path E0 — Local node (Case A1)

**When:** `zebra_url` is loopback / RFC1918 / link-local.  
**Badge:** **Local**  
**Code:** `ZebraClient::url_is_local` → `DirectLocal`; `nym_mixnet_broadcast` returns `None` even if mixnet env is on.  
**User:** Normal send. No NymVPN paragraph unless Ironwood orchard-block.  
**Outcome:** Best IP↔tx story. Nym kit stays idle. This is a valid product end state.

---

## Path E1 — Mixnet submit (smolmix / issue #147)

**When:** Remote Zebrad + `privacy_network.broadcast_via_nym_mixnet` or `NOZY_BROADCAST_VIA_NYM_MIXNET=1` + helper binary present.  
**Badge:** **Mixnet**  
**Code:** `src/nym_mixnet_broadcast.rs` → `tools/nym-smolmix-broadcast-spike`.  
**If helper missing:** badge **Blocked** (connection_mode may still say `nym_mixnet`).  
**Open:** D2c-live — real tx to an **exit-reachable** testnet RPC. LAN `172.20…` is N/A, not FAIL.  
**Do not:** use this path for compact sync.

---

## Path E2 — dVPN compact sync (issue #146)

**When:** Remote LWD + `sync_via_nym_dvpn`. Local `:9067` refused (C4).  
**Badge:** Does **not** change send egress. Sync is a second URL.  
**Code:** `src/nym_dvpn_sync.rs`, desktop Settings probe.  
**Evidence:** ~8× slower than clearnet vs `zec.rocks` (July `smoldvpn`/`develop`).  
**Policy:** Never sync and submit through the same hosted lightwalletd a minute later.

---

## Path E3 — NymVPN stopgap (no local node / Ironwood week)

**When:** Badge is **Blocked**, **Clearnet remote**, or **Trusted** on a non-local URL.  
**What the user does:** Prove shielded ZEC at zcash.nym.com → Fast mode for sync, Mixnet + new exit for send.  
**In-app Nym?** No. OS VPN. Wallet copy only (`nymvpn_ironwood_stopgap_hint`).  
**Kill the hint** only after E1 live + E2 pins refreshed.

---

## Path E4 — Joaco LWD gRPC over mixnet (neighbor sidecar)

**When:** Operator runs `lwd-mixnet-client` on `:9068`. **Nozy send does not point here today.**  
**Native Nozy send is Zebrad JSON-RPC (E1).** Joaco is lightwalletd gRPC. Same privacy *goal*, different socket.

| Run | What we learned |
|-----|-----------------|
| 2026-08-15 load | first-round 5/30 (16.67%), wallet-visible 2/30 (6.67%); PR #3 merged; ADR 0012 4→6 attempts |
| 2026-08-16/17 idle | Midnight reclaim ~200–525 ms; client re-claims; RestartCount 0 |
| **2026-08-19 heartbeat (dead far end)** | TCP 20/20 ~210 ms including `00:00:03Z`; mixnet **19/19 unanswered**; reclaim ~298 ms. Serving half dead since Aug 18 14:12Z. Evidence: `lwd-mixnet-midnight-heartbeat-20260818-215629.txt`. [PR #6](https://github.com/jpgonzalezra/lwd-mixnet-proxy/pull/6) |
| **2026-08-20 heartbeat (live far end)** | TCP 20/20; mixnet **20/20 answered**, unestablished 0, streams 57 (19×3), reclaim ~353 ms at `00:00:00.602Z`. New gateway `6PkVk…`. Evidence: `lwd-mixnet-midnight-heartbeat-20260819-225456.txt`. Harness now runs **GetLightdInfo** each minute (not TCP-only). Rehearsal 2026-08-20: grpc 7/7 ok, ~3–6.5 s latency. |

**Possible later:** optional submit URL `127.0.0.1:9068` **only** if Nozy submits via LWD. Treat unestablished streams as send failure. Do **not** default on a 19/19 night. Do **not** sync through the proxy. Full use-case paper: [NYM_LWD_MIXNET_PROXY_CASE_BREAKDOWN.md](NYM_LWD_MIXNET_PROXY_CASE_BREAKDOWN.md).

---

## Path E5 — Tor / I2P

**When:** Remote RPC + SOCKS/HTTP proxy in privacy config.  
**Badge:** **Tor** or **I2P**  
**Mixnet helper is not this hop.** Stopgap is not required.

---

## Path E6 — Clearnet remote / blocked

**Clearnet remote:** remote Zebrad, no mixnet/Tor, privacy not required. Host IP can be linked to the tx.  
**Blocked:** `require_privacy_network` (default on) and no local/Tor/mixnet helper.  
**Badge + stopgap URL.** Honest copy. Not “Nym integrated.”

---

## How to read the badge

```text
nozy privacy-network send-egress
nozy privacy-network nym-mixnet   # same snapshot, plus helper notes
```

Desktop: Send form (compose + review), Settings → Network privacy, Ironwood readiness card.

CLI `nozy send` prints `Submit: Local — …` (or Mixnet / Blocked / …) on the transaction summary before confirm.

Companion API / Tauri Ironwood status includes a `send_egress` snapshot (classification only — no Nym SDK in `api-server`).

---

## Work notes per path (2026-08-18 / 19)

What we actually did on this operator box, so later work does not re-litigate it.

### E0 Local — work done

- Default `WalletConfig` classifies **Local**; mixnet env does not wrap RFC1918 (`nym_mixnet_broadcast` returns `None`).
- Badge + CLI `send` / `privacy-network send-egress` / desktop Send + Settings + Ironwood card.
- Tests in `src/send_egress.rs`: default config is Local, stopgap hidden on loopback.

### E1 Mixnet smolmix — work done / still open

- **Done:** subprocess gate, D2a IP relocate PASS, LAN refuse PASS, D2d Ironwood shares `broadcast_transaction`.
- **Open:** D2c-live (public exit-reachable Zebrad). Do not expose unauthenticated mainnet RPC to force a PASS.
- Heartbeat 19/19 unanswered is **Joaco streams (E4)**, not a FAIL of this JSON-RPC helper.

### E2 dVPN sync — work done / still open

- **Done:** C2/C3 vs `zec.rocks` (~8×), C6 desktop probe, Settings toggle.
- **Open:** refresh crates with Max/Harry current “fast mode.”
- Send badge never claims dVPN. Two URLs remain policy.

### E3 NymVPN stopgap — work done

- Copy on CLI blocked Ironwood, API, desktop, extension, mobile, book (`nymvpn_ironwood_stopgap_hint`).
- Badge shows zcash.nym.com when kind is Blocked / Clearnet remote / remote Trusted.

### E4 Joaco LWD sidecar — work done (Nozy operator)

- 15 Aug load + PR #3 merged (Joaco raised `--probe-attempts` 4→6).
- 16–17 Aug idle reclaim; RestartCount 0.
- **19 Aug heartbeat (asked in forum #7):** 20 TCP/min ±10 min around 00:00 UTC. Local port stayed up; mixnet 19/19 unanswered; reclaim ~298 ms in the middle of an already-silent path. Raw: `docs/reference/evidence/lwd-mixnet-midnight-heartbeat-20260818-215629.txt`. Measurement write-up: [lwd-mixnet-proxy PR #6](https://github.com/jpgonzalezra/lwd-mixnet-proxy/pull/6).
- **Not product-wired.** Next if we ever submit via LWD: real gRPC on `:9068`, then optional second URL — not default after 19/19.

### E5 Tor / I2P — existing

- Same badge. Mixnet helper is not this hop.

### E6 Clearnet / blocked — work done

- Honest label + stopgap. `require_privacy_network` default still blocks remote clearnet send/migrate unless attested/forced.

---

## How to check (no tunnel)

```powershell
nozy privacy-network send-egress
nozy privacy-network nym-mixnet
```

Desktop: Send, Settings → Network privacy, Ironwood page. Look for **Next send:** Local / Mixnet / Tor / Blocked / …

---

## What we will not build from this kit

- Mixnet for full / compact historical sync  
- Nym SDK inside `api-server` or zeaking (sqlite `links`)  
- NymVPN Mixnet Tuning GUI  
- Shipping Joaco’s client as default submit after the 19/19 unanswered window  

---

## AI disclosure

Implementation and this case breakdown assisted by Cursor. Human author remains responsible for numbers, forum quotes, and any product claim.
