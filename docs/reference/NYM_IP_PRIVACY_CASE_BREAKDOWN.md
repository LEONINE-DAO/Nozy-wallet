# Nym IP privacy — case breakdown (NozyWallet)

**Status:** Engineering in progress (Priority 1 network metadata)  
**Tracking:** [issue #146](https://github.com/LEONINE-DAO/Nozy-wallet/issues/146) (smol-dvpn compact sync spike) · [issue #147](https://github.com/LEONINE-DAO/Nozy-wallet/issues/147) (broadcast / submit over Nym — biggest win)  
**Related:** [SAFE_MIGRATION_NETWORK_PRIVACY_FORUM_POST.md](SAFE_MIGRATION_NETWORK_PRIVACY_FORUM_POST.md) · [`src/ironwood/network_privacy.rs`](../../src/ironwood/network_privacy.rs) · [`src/ironwood/baseline_hygiene.rs`](../../src/ironwood/baseline_hygiene.rs) · [`tools/nym-dvpn-lwd-spike/`](../../tools/nym-dvpn-lwd-spike/) · [`tools/nym-smolmix-broadcast-spike/`](../../tools/nym-smolmix-broadcast-spike/) · [`src/nym_mixnet_broadcast.rs`](../../src/nym_mixnet_broadcast.rs) · [SESSION_NYM_IRONWOOD_DESKTOP_CASE_BREAKDOWN.md](SESSION_NYM_IRONWOOD_DESKTOP_CASE_BREAKDOWN.md) (2026-07-11 omnibus) · [NYM_IRONWOOD_BASELINE_HYGIENE_CASE_BREAKDOWN.md](NYM_IRONWOOD_BASELINE_HYGIENE_CASE_BREAKDOWN.md) (2026-07-25) · Nym docs: https://zcash-sdk.nym.com/  
**Guidance (Nym / Mark / Harry):** Biggest remote win is **IP ↔ tx submit**. Recommended hybrid: **dVPN sync + mixnet broadcast**, plus **baseline hygiene** the transport cannot provide.

---

## Living checklist (keep this current)

| ID | Item | Status | Evidence / notes |
|----|------|--------|------------------|
| H1–H4 | Baseline hygiene (start obfuscation, broadcast delay, tip guard) | **Landed** (2026-07-25) | [NYM_IRONWOOD_BASELINE_HYGIENE_CASE_BREAKDOWN.md](NYM_IRONWOOD_BASELINE_HYGIENE_CASE_BREAKDOWN.md) · forum draft [NYM_ZCASH_HYBRID_FORUM_ARTICLE.md](NYM_ZCASH_HYBRID_FORUM_ARTICLE.md) · Nym [guidance](https://zcash-sdk.nym.com/guidance/) |
| D2a | smolmix **IP relocate** (exit IP ≠ host) | **PASS** (2026-07-11; **re-PASS 2026-07-26**) | `evidence/nym-d2a-rerun.json` — mixnet `82.221.101.117` ≠ clearnet |
| D2b-reachability | LAN refuse / URL classify (no tunnel) | **PASS** (2026-07-25/26) | Live WSL `172.20.199.206:8232` correctly N/A |
| D2b | **JSON-RPC probe** (`getblockcount`) over smolmix | **N/A on this host** | Operator Zebrad is LAN (Case A1). See [NYM_MIXNET_BROADCAST_CASE_BREAKDOWN.md](NYM_MIXNET_BROADCAST_CASE_BREAKDOWN.md) |
| D2c | Opt-in mixnet remote `broadcast_transaction` | **Wired (subprocess)** | Env/config + helper; local skip **PASS** — use Case A1 with WSL node |
| D2c-live | Live remote sendraw over mixnet | **Open / not required for local-node ops** | Only when zebra_url is public |
| D2d | Ironwood `broadcast` shares same egress | **Wired** | Same `ZebraClient::broadcast_transaction` + Priority 1 mode `NymMixnetBroadcast` when helper present. |
| D2e | Desktop Advanced: live mixnet session (not attest-only) | **Partial** | CLI readiness landed; full live session UI later. |
| D1 / C* | smol-dvpn compact sync | Spike + case breakdown (2026-07-26) | [NYM_DVPN_SYNC_CASE_BREAKDOWN.md](NYM_DVPN_SYNC_CASE_BREAKDOWN.md) — live C2/C3 pending NYM credentials |

**Operator do-list (next concrete runs):**

1. Keep using local/WSL Zebrad for sync/send (Case A1) — no mixnet required.
2. Evidence harness: `powershell -File scripts\nym-smolmix-d2-evidence.ps1` (dry + LAN refuse).
3. Optional: `-IpRelocate` to re-prove D2a; `-ZebraUrl <public> -RpcProbe` for D2b.
4. For remote submit (D2c-live): set `NOZY_NYM_SMOLMIX_BIN` + mixnet env; zebra URL must be exit-reachable.
5. Record PASS/FAIL under `docs/reference/evidence/`; do not claim product “Nym support” until D2b+D2c-live.

---

## Evidence log

| Date (UTC) | Step | Result | Detail |
|------------|------|--------|--------|
| 2026-07-11 | D2a IP relocate | **PASS** | clearnet IP `2607:fb92:183:6d6:81da:b7cc:e247:4e88` (126ms); mixnet IP `154.26.153.210` (12.0s); exit differs — mixnet egress works |
| 2026-07-11 | D2b rpc-probe to `172.20.199.206:18232` | **N/A (refused)** | Private LAN; spike refuses before tunnel waste. Wallet config URL remains valid for **local** Case A1 only. |
| 2026-07-11 | D2c opt-in hook | **Subprocess wired** | `nym_mixnet_broadcast` → spike `--sendraw-stdin`; config + env; Ironwood Priority 1 mode `NymMixnetBroadcast` |
| 2026-07-25 | D2b-reachability + evidence harness | **PASS** | `--dry-reachability`, `--evidence-json`, CLI `privacy-network nym-mixnet`; see [NYM_MIXNET_BROADCAST_CASE_BREAKDOWN.md](NYM_MIXNET_BROADCAST_CASE_BREAKDOWN.md) |
| 2026-07-26 | WSL Zebrad live + D2a re-run | **PASS / N/A** | tip 3425397 at `172.20.199.206:8232`; mixnet refuse LAN; IP relocate re-PASS (`evidence/nym-d2a-rerun.json`); track C opened [NYM_DVPN_SYNC_CASE_BREAKDOWN.md](NYM_DVPN_SYNC_CASE_BREAKDOWN.md) |

---

## Local-only / untracked spike trees (belong here)

These are **not** shipped as workspace members; kept local/untracked (large vendor) so the root crate stays clean.

| Path | Role | Case |
|------|------|------|
| `tools/nym-smolmix-broadcast-spike/` (+ `vendor/`) | Smolmix egress helper; `--ip-relocate`, `--rpc-probe`, `--sendraw-stdin` | D2a–D2d |
| `tools/nym-dvpn-lwd-spike/` (+ `vendor/`) | Compact sync over smol-dvpn | D1 / #146 (secondary) |
| `src/nym_mixnet_broadcast.rs` | Wallet subprocess gate | D2c |
| `src/ironwood/network_privacy.rs` | Safer migration Priority 1 modes incl. NymMixnetBroadcast | D2d |

**Ignored root md (related ops):** `PRIVACY_NETWORK_*.md`, `PRIVACY_IMPLEMENTATION_COMPLETE.md`, `PRIVACY_POSITIONING.md` — early Tor/I2P notes; current product priority is **mixnet broadcast**.

---

## Reader’s guide

Case numbers in **this document only** are not global across Ironwood / KYC / dynamic-fee docs.

| Document | Topic |
|----------|--------|
| **This document** | Network metadata: IP, lightwalletd/Zebrad egress, Nym modes (mixnet / dVPN / mix-fetch) |
| **[SAFE_MIGRATION_NETWORK_PRIVACY_FORUM_POST.md](SAFE_MIGRATION_NETWORK_PRIVACY_FORUM_POST.md)** | Forum-facing Priority 1–3 narrative |
| **[KYC_INBOUND_PRIVACY_CASE_BREAKDOWN.md](KYC_INBOUND_PRIVACY_CASE_BREAKDOWN.md)** | On-chain / KYC amount trails (different layer) |
| **[IRONWOOD_WALLET_READINESS.md](IRONWOOD_WALLET_READINESS.md)** | Orchard → Ironwood migration protocol |

### Core rule of thumb

> **Shielded crypto hides note contents. It does not hide that your IP talked to a submit endpoint when a turnstile or send hit the mempool.**  
> Biggest win: **every outgoing transaction** (send, migrate-broadcast, split) leaves the device **over Nym** (or never leaves to a remote party — local Zebrad).

### What Nozy does today vs what we are building

| Layer | Today (v2.4.x) | In progress / next |
|-------|----------------|--------------------|
| Local Zebrad preferred | Yes — config + safer-migration gate | Keep as default |
| Tor / I2P SOCKS | Detect + optional `require_privacy_network` | Keep |
| Nym product | Settings links + **attestation** checkbox | Replace attestation with **in-app transport** |
| Compact sync over dVPN | Spike crate [`tools/nym-dvpn-lwd-spike`](../../tools/nym-dvpn-lwd-spike/) (#146) | Secondary to broadcast |
| **Tx broadcast over Nym** | Subprocess helper + env/config (#147) | D2b public RPC proof; polish desktop live session (D2e) |
| Extension mix-fetch | Not wired | Later |

---

## Threat model (network metadata)

### Attacker capabilities (assume true)

1. Observes **ISP / Wi‑Fi / VPS** egress: sees connections to Zebrad RPC, lightwalletd gRPC, or explorers.
2. Operates or logs a **remote lightwalletd / submit API**: sees **client IP + timestamp + submitted raw tx** (or gRPC `SendTransaction`).
3. Correlates **mempool appearance time** of a known txid with that IP session.
4. For Ironwood migration: joins **IP ↔ pool-crossing turnstile** even when ZIP 318 denominations are perfect.

### Attacker non-capabilities (do not assume broken)

- Cannot decrypt Orchard/Ironwood note plaintexts from the wire alone.
- Cannot forge spends without keys.
- Localhost Zebrad with no remote submit is out of scope for “remote LWD saw your IP.”

### Defender goal

Prevent: **“this IP submitted this txid / this migration turnstile.”**

Secondary: reduce sync-volume metadata and censorship (dVPN / QUIC bridge).

---

## Case family A — Where the IP leak actually is

### Case A1 — Local Zebrad only (best desktop default)

**Flow:** Wallet → `http://127.0.0.1` / LAN Zebrad → `sendrawtransaction`.

Evidence / properties:

- No public LWD learns the client IP.
- ISP may still see Zcash P2P from **the node**, not “wallet app → LWD submit.”
- Safer-migration Priority 1 already treats this as **allowed** (`MigrationNetworkPrivacyMode::LocalNode`).

Conclusion: **Keep as preferred path.** Nym is for when the wallet must talk to a **remote** submit endpoint.

### Case A2 — Remote Zebrad JSON-RPC clearnet (high risk)

**Flow:** Wallet → remote `:8232` / `:18232` → `sendrawtransaction`.

Leak: remote operator and path observers see **IP ↔ broadcast**.

Today: gated by attestation / Tor SOCKS / `force_clearnet` (discouraged). Opt-in mixnet: `NOZY_BROADCAST_VIA_NYM_MIXNET=1` fail-closes remote clearnet until in-process smolmix links (D2c); prove path with the spike (D2a/D2b).

Conclusion: **Must gain Nym (or Tor) transport on broadcast**, not only a checkbox.

### Case A3 — Remote lightwalletd submit (Mark’s primary case)

**Flow:** Wallet builds tx locally → gRPC/HTTP **SendTransaction** (or equivalent) to public/private LWD.

Leak: **exact** IP ↔ tx bytes at the LWD. Compact sync over the same LWD is noisy; **submit is surgical**.

Conclusion: **Biggest win = all outgoing txs over Nym to that LWD (or any submit proxy).** Sync-over-Nym is valuable but secondary. (D3 after D2.)

### Case A4 — Compact sync only over Nym, submit clearnet (false sense of security)

**Flow:** GetBlockRange via smol-dvpn; `sendrawtransaction` / SendTransaction on clearnet.

Leak: still **IP ↔ tx** on submit.

Conclusion: **Reject as “Nym done.”** dVPN sync spike (#146) does **not** close Priority 1 alone.

---

## Case family B — Nym product modes (what to use when)

### Case B1 — smolmix (mixnet) for submit / small RPC

**What it buys:** Timing obfuscation + hide client IP from RPC/LWD; exit can rotate / multi-exit.

**Fit:** `sendrawtransaction`, Ironwood migrate-broadcast, fee/status RPCs, small unary gRPC.

**Cost:** Higher latency than WireGuard; fine for rare broadcasts.

Conclusion: **Primary transport for Case A2/A3 (biggest win).** Spike + opt-in hook (#147); D2a PASS, D2b pending public RPC.

### Case B2 — smol-dvpn (2-hop WireGuard) for bulk sync

**What it buys:** Faster throughput; user-space tunnel = natural killswitch; optional QUIC bridge.

**Fit:** Compact `GetBlockRange` (Nym’s `zcash-sync` example; our #146 spike).

Conclusion: **Keep for sync / censorship.** Do not substitute for mixnet on submit unless mixnet is blocked and dVPN is explicit fallback.

### Case B3 — mix-fetch / mix-websocket (browser / extension)

**What it buys:** Drop-in `fetch` over mixnet in WASM/WebView.

**Fit:** Browser extension companion API, explorer HTTP — later.

Conclusion: **Phase after desktop/CLI broadcast path.**

### Case B4 — System NymVPN + attestation only

**What it buys:** Possible OS-wide cover if the user actually runs it.

**Risk:** Wallet cannot prove traffic entered the mixnet; easy to forget; wrong app bypass.

Conclusion: **Bridge only** until in-app smolmix/dVPN land. Do not claim “Nym integrated” from the checkbox alone.

---

## Case family C — Nozy surfaces and required routing

### Case C1 — CLI `nozy send` / `nozy ironwood broadcast`

**Submit path today:** [`ZebraClient::broadcast_transaction`](../../src/zebra_integration.rs) → JSON-RPC `sendrawtransaction`, with optional [`nym_mixnet_broadcast`](../../src/nym_mixnet_broadcast.rs) when env+feature set.

**Required win:** When zebra URL is remote, force **smolmix (or attested Tor)** under broadcast; refuse clearnet unless `--force-clearnet`.

### Case C2 — Desktop Tauri send / future migrate UX

Same core library path as C1. UI must show **egress mode** (local / Tor / Nym mixnet / dVPN / blocked).

### Case C3 — API server / extension companion

If the companion accepts “broadcast hex,” the **machine running api-server** is the IP that LWD/Zebrad sees. Nym must wrap that process’s submit, or broadcast must happen in-process in the extension via mix-fetch.

### Case C4 — Mobile

Follow desktop once Rust FFI can own a smolmix/dVPN session; do not ship attestation-only as “Nym.”

---

## Case family D — What we are implementing now (roadmap)

Ordered by **linkage severity** (updated after Mark’s “biggest win” guidance):

| Step | Work | Status |
|------|------|--------|
| D0 | Safer-migration gate: local \|\| Tor/I2P \|\| attest \|\| force clearnet | **Shipped** (`network_privacy.rs`) |
| D1 | Isolated **smol-dvpn** compact-sync spike | **Spike crate** [#146](https://github.com/LEONINE-DAO/Nozy-wallet/issues/146) — not productized |
| D2 | **Broadcast / submit over Nym (smolmix first)** | **In progress** — see living checklist D2a–D2e |
| D2a | IP relocate | **PASS** 2026-07-11 |
| D2b | rpc-probe over mixnet | Waiting on exit-reachable Zebrad |
| D2c | `NOZY_BROADCAST_VIA_NYM_MIXNET` / config + subprocess helper | **Wired** |
| D3 | `zeaking` / tonic `connect_with_connector` for LWD Submit + optional sync | After D2 path proven |
| D4 / D2e | Desktop Advanced: real session status (not only attest) | After D2b+D2c |
| D5 | mix-fetch for extension | Later |
| D6 | Credential-proxy / gifted zk-nyms for end users | Coordinate with Nym |

### Success criteria for “Nym IP done enough to claim”

1. Remote submit of a testnet send **does not** expose the host public IP to the submit endpoint (verified via exit IP / operator log agreement).
2. Product UI distinguishes **local node** vs **Nym mixnet submit** vs **clearnet exception**.
3. Ironwood `broadcast` uses the same egress policy as normal send.
4. Docs never claim sync-over-dVPN alone protects tx linkage.

---

## Case family E — Failure modes / honest limits

### Case E1 — Nym on sync, clearnet on send

Still loses. See A4.

### Case E2 — Local node but explorer / price HTTP clearnet

Lower severity than submit; still timing. Out of scope for D2; optional later.

### Case E3 — Compromised exit / malicious LWD

Mixnet hides IP from LWD; malicious LWD still sees **tx bytes**. Amount/timing (Priority 3) and cohorts (Priority 2) remain necessary.

### Case E4 — KYC amount fingerprint

Nym does not fix subset-sum on revealed migration amounts. See KYC / Ironwood amount-timing docs.

### Case E5 — Mixnet rpc-probe to private LAN

`172.20.x.x` / `192.168.x.x` / loopback are reachable from the wallet host but **not** from a public Nym exit. Treating a timeout as “smolmix broken” is wrong — fix the **target reachability**, not the tunnel (D2a already proved egress).

---

## Implementation sketch (D2 — broadcast over smolmix)

```text
build_and_broadcast_transaction / ironwood broadcast
  → ZebraClient::broadcast_transaction
       → nym_mixnet_broadcast::maybe_broadcast_via_nym_mixnet
            if (env|config) && remote:
              spawn nym-smolmix-broadcast-spike --sendraw-stdin --zebra <url>
                → sendrawtransaction_over_smolmix
            else → make_request(sendrawtransaction)  # local / Tor / direct
```

Scaffold / probe: [`tools/nym-smolmix-broadcast-spike/README.md`](../../tools/nym-smolmix-broadcast-spike/README.md).

---

## Asks / coordination

- Nym: confirm recommended default for **submit** = smolmix, dVPN as sync/censorship path (aligned with Mark).
- Maintainers: treat #147 as the Priority-1 product spike; #146 remains infrastructure learning.
- Do not market “Nym support” until D2b+D2c land on a remote node or local-node-only claims are explicit.

---

## AI disclosure

Draft assisted by Cursor Agent. Human review required before forum paste or product claims.
