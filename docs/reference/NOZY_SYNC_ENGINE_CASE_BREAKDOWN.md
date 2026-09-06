# Nozy Sync Engine — case breakdown

**Status:** Phase 1 + Phase 2 **landed**; **live smoke PASS** against WSL Zebrad (2026-08-20).  
**Date:** 2026-08-20  
**Author surface:** LaDale / Lowo88  
**Tracking:** [#274](https://github.com/LEONINE-DAO/Nozy-wallet/issues/274) · FEAT-2026-009  
**Code:** [`nozy-sync-engine/`](../../nozy-sync-engine/) · start [`scripts/start-nozy-sync-engine.ps1`](../../scripts/start-nozy-sync-engine.ps1)  
**Related:** [`nozy-sync-engine/README.md`](../../nozy-sync-engine/README.md) · [ZEBRAD_NOZYWALLET_CONNECTIVITY.md](ZEBRAD_NOZYWALLET_CONNECTIVITY.md) · [ZAKURA_NOZYWALLET_CONNECTIVITY.md](ZAKURA_NOZYWALLET_CONNECTIVITY.md) · [NYM_SEND_EGRESS_CASE_BREAKDOWN.md](NYM_SEND_EGRESS_CASE_BREAKDOWN.md) · [ZEBRAD_SHIELDED_SEND_LIMIT.md](../../ZEBRAD_SHIELDED_SEND_LIMIT.md)

This file maps **what the Nozy Sync Engine is**, **how it sits relative to Zeaking / lightwalletd / peer projects**, and **every operator path** we just shipped — without the product-naming history.

---

## For newcomers (read this first)

NozyWallet is a **wallet**, not a consensus node. Compact sync used to require a third process — **lightwalletd** — between Zebrad/Zakura and the wallet’s Zeaking client.

**Nozy Sync Engine** is that middle box, owned in-repo:

```text
Zebrad or Zakura (JSON-RPC :8232)
        │  getblock / tip / treestate / sendraw
        ▼
Nozy Sync Engine (CompactTxStreamer gRPC :9067)
        │  LIGHTWALLETD_GRPC drop-in
        ▼
zeaking::lwd → SQLite compact cache → CLI / desktop / api-server / FFI
```

| Question | Answer |
|----------|--------|
| Is this Zeaking? | **No.** Zeaking is the **wallet-side** compact client + SQLite cache. The Sync Engine is the **operator process** that *feeds* Zeaking. |
| Does it replace Zebrad? | **No.** It reads from Zebrad or Zakura. |
| Does it replace lightwalletd? | **For compact sync (and now treestate/submit proxy) — yes, as preferred path.** lightwalletd remains a documented fallback. |
| Is this Zinder / Zaino / Zallet? | **No.** Different jobs — see [Related ecosystem](#related-ecosystem-not-substitutes). |
| Do I rewrite the wallet? | **No.** Point `LIGHTWALLETD_GRPC` at the Sync Engine. |

---

## Living scoreboard (S-*)

| ID | Item | Status | Notes |
|----|------|--------|-------|
| S0 | Workspace crate + binary `nozy-sync-engine` | **Landed** | Issue #274 |
| S1 | Zebra-family RPC client + Zakura/Zebrad cookie auth | **Landed** | Same resolution order as Nozy `ZebraClient` |
| S2 | Raw `getblock` → CompactBlock (Sapling / Orchard / Ironwood) | **Landed** | Unit tests; live parity vs zec.rocks **PASS 2026-09-05** (shielded txs + S11 tree sizes) |
| S3 | SQLite compact store + tip ingest + reorg prune | **Landed** | Backfill window configurable |
| S4 | Phase 1 CompactTxStreamer serve | **Landed** | Tip / range / info / ping |
| S5 | Phase 2 treestate + subtree roots | **PASS live** | `live_smoke_probe` after string-height `z_gettreestate` fix |
| S6 | Phase 2 `SendTransaction` | **Landed** | Proxies `sendrawtransaction` — privacy caveats below |
| S7 | Operator start script | **Landed** | `scripts/start-nozy-sync-engine.ps1` |
| S8 | Zeaking drop-in (`LIGHTWALLETD_GRPC`) | **PASS live** | 2026-08-20 WSL Zebrad → engine → `nozy lwd sync-to-tip` |
| S9 | Nym hybrid policy vs Sync Engine | **Documented** | Local = no tunnel; remote sync = dVPN; never mixnet sync |
| S10 | zero-indexer as front shim | **Deferred** | For **public VPS** only — see [Deferred: public VPS + zero-indexer](#deferred-public-vps--zero-indexer) |
| S11 | ChainMetadata tree sizes on ingest | **Landed** | Count notes in-block + seed from prev compact or `z_gettreestate` / getblock `trees` |
| S12 | Transparent / mempool CompactTxStreamer RPCs | **Open** | Explicit UNIMPLEMENTED |

---

## Architecture cases

### Case A — Roles (do not conflate)

| Layer | Owns | Does not own |
|-------|------|--------------|
| **Zebrad / Zakura** | Consensus, JSON-RPC, treestate truth | Compact gRPC for light wallets |
| **Nozy Sync Engine** | Ingest → compact SQLite → CompactTxStreamer | Wallet keys, Orchard proves, UI |
| **Zeaking (`zeaking::lwd`)** | Client dial, compact cache, sync-to-tip APIs | Chain ingest from the node |
| **Nozy surfaces** | UX, send, witnesses from local cache + node treestate | Indexer storage |

### Case B — Preferred operator stack

**When:** Operator runs their own node (Zebrad or Zakura) on LAN/WSL.  
**How:**

```powershell
.\scripts\start-nozy-sync-engine.ps1
$env:LIGHTWALLETD_GRPC = "http://127.0.0.1:9067"
```

**Outcome:** No lightwalletd. Cookie-auth Zakura works (engine sends the cookie). Best network-privacy story for sync (no remote indexer IP).

### Case C — lightwalletd fallback

**When:** Sync Engine not running or operator prefers upstream LWD.  
**How:** Existing `start-lightwalletd-wsl.ps1` / Zakura LWD guides.  
**Outcome:** Zeaking unchanged — same `LIGHTWALLETD_GRPC` env. Zakura often needs `enable_cookie_auth = false` for stock LWD.

### Case D — Zakura vs Zebrad ingest

**When:** Either node on `:8232`.  
**How:** One RPC client; `getnetworkinfo` subversion labels **Zakura** vs **Zebrad** in logs / `GetLightdInfo`.  
**Auth:** `ZAKURA_RPC_*` / `ZEBRA_RPC_*` user/pass, inline cookie, or `~/.cache/{zakura,zebra}/.cookie`.  
**Not supported:** Zakura **zcashd-compat** RPC mode.

---

## Protocol cases (what Zeaking can call)

### Case P1 — Compact sync (Phase 1)

| RPC | Behavior |
|-----|----------|
| `GetLightdInfo` | Vendor `NozyWallet/nozy-sync-engine`; chain + tip from node + indexed height |
| `GetLatestBlock` | Max height in Sync Engine SQLite |
| `GetBlock` | By height (hash lookup not yet) |
| `GetBlockRange` | Stream of stored compact blocks |
| `Ping` | OK |

**Ingest:** Poll tip → `getblock` verbosity 0 → parse header/txs with librustzcash → protobuf CompactBlock → SQLite. Empty DB backfills `NOZY_SYNC_ENGINE_BACKFILL` (default 500) below tip. Heights above tip pruned on reorg.

### Case P2 — Treestate (Phase 2)

| RPC | Upstream |
|-----|----------|
| `GetTreeState` | `z_gettreestate` at height (or indexed tip) |
| `GetLatestTreeState` | Indexed tip, else node tip |
| `GetSubtreeRoots` | `z_getsubtreesbyindex` + optional completing block hash |

Maps Sapling/Orchard `finalState` hex into lightwalletd `TreeState` fields. Lets wallets optionally stop splitting tip sources for tree reads.

### Case P3 — Submit proxy (Phase 2)

| RPC | Upstream |
|-----|----------|
| `SendTransaction` | `sendrawtransaction` |

**Privacy:** This is a **convenience proxy**, not an IP-privacy product. Prefer:

1. Local Zebrad/Zakura submit from Nozy core (Case A1 / send-egress **Local**), or  
2. Mixnet/Tor for **remote** submit, or  
3. [zero-indexer](https://forum.zcashcommunity.com/t/zero-indexer-protecting-network-privacy-for-zcash-users/57113) in front for Orchard-exit batching.

Do **not** market Sync Engine submit as “Nym-private.”

### Case P4 — Still unimplemented

Transparent balances/txids/UTXOs, mempool streams, `GetTransaction`, nullifier-only block ranges — return gRPC **UNIMPLEMENTED** (not silent wrong answers).

---

## Privacy / Nym cases

| ID | Topology | Nym role |
|----|----------|----------|
| N0 | Colocated node + Sync Engine + wallet | **None** — best case |
| N1 | Wallet → **public** Sync Engine compact sync | **dVPN / Fast** for `GetBlockRange` (reuse Zeaking connector) |
| N2 | Small remote RPCs / remote submit | **Mixnet** (or Tor); never bulk sync over mixnet |
| N3 | Sync Engine ↔ node | Prefer **same host**; do not put ingest over mixnet |

Matches Nym’s [recommended hybrid](https://zcash-sdk.nym.com/scenario/recommended-hybrid/). Deep scoreboard: [NYM_SEND_EGRESS_CASE_BREAKDOWN.md](NYM_SEND_EGRESS_CASE_BREAKDOWN.md).

---

## Related ecosystem (not substitutes)

| Project | Job vs Nozy Sync Engine |
|---------|-------------------------|
| **lightwalletd** | Reference CompactTxStreamer; Sync Engine is the preferred in-repo replacement for our stack |
| **Zinder** (ZF) | Full ecosystem indexer + own protocol + LWD-compat — peer *category*, not adopted |
| **Zaino** | General indexer (Zallet / explorers); different product |
| **Zallet** | RPC wallet replacing `zcashd` wallet — not a sync engine |
| **zero-indexer** | TEE shim + batching hub **in front of** an indexer for Orchard-exit IP privacy |

### Case Z — zero-indexer composition

```text
Wallet  →  zero-indexer shim  →  Nozy Sync Engine  →  Zebrad/Zakura
                 │
                 └─ Orchard-exit submits → hub batch (~20 blocks)
```

Compact sync through the shim needs P1 RPCs (**done**). Orchard-exit privacy needs `SendTransaction` on the backend (**done** as proxy). Attestation / registry / production shim deploy = **out of this case file**.

---

## Evidence & tests

| Claim | Evidence |
|-------|----------|
| Crate builds | `cargo check -p nozy-sync-engine` |
| Unit tests | `cargo test -p nozy-sync-engine` — compact size, encode roundtrip, RPC helpers, treestate JSON map |
| Live Sync Engine + Zebrad (WSL) | **PASS 2026-09-05** — RPC `http://172.20.199.206:8232` (Zebrad 6.2.3, `main`), engine `0.0.0.0:9067`, tip **3472542** (also PASS 2026-08-20) |
| Live Zeaking compact sync | **PASS 2026-08-20** — `nozy lwd sync-to-tip --start-floor <tip-15>` → tip **3454786**. **Not re-run 2026-09-05:** Windows `WSAENOBUFS` blocked local tonic clients |
| Live gRPC probe | **PASS 2026-09-05** — WSL `grpcurl` `GetLightdInfo` + `GetLatestBlock` (vendor `NozyWallet/nozy-sync-engine`, height 3472542). `GetLatestTreeState` **WARN** this session (Windows→WSL `z_gettreestate` transport). Prior PASS 2026-08-20 |
| Compact parity vs lightwalletd | **PASS 2026-09-05** — heights 3472538–3472542 vs `zec.rocks:443`; shielded vtx + Sapling/Orchard/Ironwood tree sizes match. Evidence: [`evidence/lwd-parity-engine-20260905.txt`](evidence/lwd-parity-engine-20260905.txt) |
| `z_gettreestate` params | Zebrad requires height as **string**; fixed in `rpc::z_gettreestate` after first probe failure |

---

## Open next (honest backlog)

1. Transparent / mempool CompactTxStreamer RPCs only if a real client needs them.  
2. Re-run `nozy lwd sync-to-tip` against the engine when the Windows socket table is healthy (2026-09-05 blocked by `WSAENOBUFS`).  
3. **Deferred — VPS + zero-indexer (when ready):** see [Deferred: public VPS + zero-indexer](#deferred-public-vps--zero-indexer).

---

## Deferred: public VPS + zero-indexer

**Status:** **Not now.** Local Sync Engine is enough for operator boxes. Revisit when running a **public / VPS** CompactTxStreamer that wallets submit through.

**Why:** [zero-indexer](https://forum.zcashcommunity.com/t/zero-indexer-protecting-network-privacy-for-zcash-users/57113) (Shielded Labs / Caution / Zec.rocks) is a TEE **shim + Orchard-exit batching hub** in front of an existing indexer. It improves IP↔tx privacy on shared infrastructure; it does not replace Nozy Sync Engine.

**Target topology (later):**

```text
User wallet
    →  zero-indexer shim (attested; VPS / Nitro)
         ├─ ordinary sync / tip  →  Nozy Sync Engine  →  Zebrad (same host or private net)
         └─ Orchard-exit SendTransaction → hub (~20-block batch) → network
```

**Prerequisites before setup:**
- Sync Engine stable on the VPS (`SendTransaction` already proxies `sendrawtransaction`).
- Node + Sync Engine **not** exposed clearnet as the only hop (prefer shim as the public URL).
- Caution / operator registry + attestation verify path (CLI or wallet-side when SL ships it).
- Honest UX: Orchard-exit may delay ~up to ~25 minutes (batch window).
- Keep wallet Nym/Tor as defense-in-depth on the path *to* the VPS; never compact-sync over mixnet.

**Do not:** put zero-indexer in front of a LAN-only operator stack “for completeness.”

**Tracking:** fold under [#274](https://github.com/LEONINE-DAO/Nozy-wallet/issues/274) or open a follow-up issue titled e.g. “VPS: zero-indexer in front of Nozy Sync Engine” when starting that work.

---

## Quick operator checklist

```powershell
# 1) Node RPC up (Zebrad or Zakura)
# 2) Sync Engine
.\scripts\start-nozy-sync-engine.ps1

# 3) Wallet / Zeaking
$env:LIGHTWALLETD_GRPC = "http://127.0.0.1:9067"
cargo run -p nozy -- lwd sync-to-tip   # or scripts\zeaking-lwd-smoke.ps1 -LiveSync
```

AI assistance used for implementation and this write-up; human author remains responsible for correctness and security ([`AGENTS.md`](../../AGENTS.md)).
