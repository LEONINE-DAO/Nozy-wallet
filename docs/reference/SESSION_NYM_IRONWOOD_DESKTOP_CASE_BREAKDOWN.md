# Session case breakdown — Nym egress + Desktop Ironwood MVP (2026-07-11)

**Status:** Landed in tree (engineering); not a product “done” claim for mainnet Ironwood or marketed Nym support  
**Date:** 2026-07-11  
**Related docs:** [NYM_IP_PRIVACY_CASE_BREAKDOWN.md](NYM_IP_PRIVACY_CASE_BREAKDOWN.md) · [IRONWOOD_WALLET_READINESS.md](IRONWOOD_WALLET_READINESS.md) · [SAFE_MIGRATION_NETWORK_PRIVACY_FORUM_POST.md](SAFE_MIGRATION_NETWORK_PRIVACY_FORUM_POST.md)  
**Tracking:** [issue #147](https://github.com/LEONINE-DAO/Nozy-wallet/issues/147) (smolmix broadcast) · [issue #146](https://github.com/LEONINE-DAO/Nozy-wallet/issues/146) (smol-dvpn sync, secondary)

Case IDs in **this document only** summarize work from this engineering pass. Deep threat-model detail stays in the Nym / Ironwood readiness docs.

---

## What we set out to do

1. Prove and wire **Nym mixnet (smolmix) egress** for remote `sendrawtransaction` (biggest IP↔tx win).
2. Bring **desktop Ironwood migration** up toward CLI parity (plan → migrate → broadcast).
3. Keep the **home page clean** while preserving Ironwood status elsewhere.

---

## Living scoreboard

| Area | Outcome | Honest limit |
|------|---------|--------------|
| Nym IP relocate (D2a) | **PASS** | Exit IP ≠ host |
| Nym JSON-RPC probe (D2b) | **Not proven** | Needs exit-reachable Zebrad (LAN refused) |
| Nym remote broadcast (D2c/D2d) | **Wired via subprocess** | No live remote submit PASS yet; sqlite blocks in-process smolmix |
| Desktop Ironwood plan/migrate/broadcast | **MVP wired** | Note-split still CLI; no mainnet “ship day” claim |
| Desktop UX (priorities / nav) | **Done** | Priorities removed from card; Ironwood tab hosts card |

---

## Case family N — Nym / network metadata

### Case N1 — Why mixnet on submit (not sync alone)

**Problem:** Shielded notes hide amounts/addresses on-chain; they do **not** hide that your IP talked to a remote submit endpoint when a tx hits the mempool.

**Decision:** Biggest win = all **outgoing tx submits** over Nym smolmix (Mark / Nym guidance). Sync-over-dVPN (#146) is secondary and does not close IP↔tx alone.

**Status:** Documented in Nym case breakdown; product priority ordered broadcast-first.

### Case N2 — IP relocate proof (D2a)

**Flow:** Clearnet Cloudflare `/cdn-cgi/trace` vs same path through smolmix tunnel.

**Evidence (2026-07-11):**

| Path | IP | Latency |
|------|-----|---------|
| Clearnet | `2607:fb92:183:6d6:81da:b7cc:e247:4e88` | ~126 ms |
| Mixnet exit | `154.26.153.210` | ~12 s |

**Conclusion:** **PASS** — mixnet egress works. Binary: [`tools/nym-smolmix-broadcast-spike`](../../tools/nym-smolmix-broadcast-spike/).

### Case N3 — JSON-RPC probe / LAN trap (D2b)

**Flow:** `getblockcount` over smolmix TCP to Zebrad JSON-RPC.

**Trap:** Wallet config `http://172.20.199.206:18232` is fine for **local/WSL** sync (Case A1). A public Nym exit **cannot** route to RFC1918. Spike now **refuses** loopback/private targets before tunnel waste.

**Conclusion:** D2b remains **blocked on operator reachability** until an exit-reachable RPC exists. Do not treat LAN refusal as “smolmix broken.”

### Case N4 — Product wire-up without linking smolmix into `nozy` (D2c)

**Blocker:** Path-depending smolmix into the root crate hits `libsqlite3-sys` **links** clash (zeaking `rusqlite` vs Nym `sqlx`).

**Decision:** Subprocess helper instead of in-process:

```text
ZebraClient::broadcast_transaction
  → nym_mixnet_broadcast::maybe_broadcast_via_nym_mixnet
       if (env|config) && remote URL:
         spawn nym-smolmix-broadcast-spike --sendraw-stdin --zebra <url>
       else:
         normal make_request (local/LAN stays direct)
```

**Enable:**

- `NOZY_BROADCAST_VIA_NYM_MIXNET=1` and/or `privacy_network.broadcast_via_nym_mixnet`
- `NOZY_NYM_SMOLMIX_BIN` → release helper binary
- Optional `NOZY_NYM_IPR`

**Files:** [`src/nym_mixnet_broadcast.rs`](../../src/nym_mixnet_broadcast.rs) · spike `--sendraw` / `--sendraw-stdin` · [`src/zebra_integration.rs`](../../src/zebra_integration.rs) (`NymMixnet` connection mode)

### Case N5 — Ironwood broadcast shares egress (D2d)

**Flow:** Ironwood `execute_orchard_migration_broadcast` → same `ZebraClient::broadcast_transaction`.

**Safer-migration Priority 1:** mode `NymMixnetBroadcast` when mixnet broadcast is enabled and helper resolves ([`src/ironwood/network_privacy.rs`](../../src/ironwood/network_privacy.rs)).

**Conclusion:** Wired in policy + submit path; still needs D2b/D2c exercise on a **remote** URL to claim.

### Case N6 — Desktop Nym UX (D2e partial)

**Done:** Network privacy settings document helper + env; connection mode can show `nym_mixnet`.

**Not done:** Live in-app mixnet session UI replacing attestation as the primary story.

---

## Case family I — Ironwood desktop vs CLI

### Case I1 — CLI ready, desktop was read-only

**Before:** CLI `nozy ironwood {plan,migrate,broadcast,split,…}` validated on testnet (v2.4.0). Desktop only `get_ironwood_status` with `migration_enabled: false`; Plan/Start buttons disabled.

**Decision:** Thin Tauri wrappers around existing `nozy::ironwood` APIs — no new ZIP 318 protocol.

### Case I2 — Desktop migration MVP (landed)

| Action | Tauri command | CLI equivalent |
|--------|---------------|----------------|
| Plan + save | `ironwood_plan_save` | `ironwood plan --save` |
| Migrate / prebuild | `ironwood_migrate` | `ironwood migrate` |
| Broadcast | `ironwood_broadcast` | `ironwood broadcast` |

**UI:** [`IronwoodReadinessCard.tsx`](../../desktop-client/src/components/IronwoodReadinessCard.tsx) gates on `migration_enabled`, `ready_to_prebuild`, `ready_to_broadcast`, network privacy.

**Backend:** [`desktop-client/src-tauri/src/commands/ironwood.rs`](../../desktop-client/src-tauri/src/commands/ironwood.rs) · status fields in [`status.rs`](../../desktop-client/src-tauri/src/commands/status.rs)

**Verify:** `cargo check` for `nozywallet-desktop` **PASS** (2026-07-11).

### Case I3 — Note split still CLI

**Why:** Split is out of MVP scope. If status reports `zip318_note_split_required`, run `nozy ironwood split`, then return to desktop Migrate.

### Case I4 — Not “mainnet Ironwood-only” yet

Nozy remains **Orchard-first** with Ironwood migration/send path. Desktop MVP ≠ mainnet ship-day claim (activation height, mainnet Zebrad, full UX polish, D2b still open).

---

## Case family U — UX cleanup

### Case U1 — Priority rows overcrowded Home

**Problem:** Priority 1/2/3 “Needs attention” (esp. shared cover) dominated Home.

**Decision:** Remove Priority rows from the card; keep activation/balances/plan/safety summary.

### Case U2 — Ironwood nav tab

**Decision:** Home stays clean; **Ironwood** button next to History opens a page that hosts the readiness card ([`pages/Ironwood.tsx`](../../desktop-client/src/pages/Ironwood.tsx) · [`Header.tsx`](../../desktop-client/src/components/Header.tsx)).

---

## Evidence log (this pass)

| Date | Item | Result |
|------|------|--------|
| 2026-07-11 | smolmix `--ip-relocate` | **PASS** (exit ≠ host) |
| 2026-07-11 | `--rpc-probe` to `172.20.199.206` | **Refused** (private LAN) by design |
| 2026-07-11 | Subprocess mixnet broadcast gate | **Landed** |
| 2026-07-11 | Desktop Tauri ironwood cmds + UI | **Landed**; `cargo check` OK |
| 2026-07-11 | Home priorities removed; Ironwood tab | **Landed** |

---

## Operator checklist (what’s left)

1. **Nym D2b:** Obtain exit-reachable testnet Zebrad RPC → `--rpc-probe` / `--both` → log PASS/FAIL in Nym case breakdown.
2. **Nym D2c live:** Build spike release, set `NOZY_NYM_SMOLMIX_BIN` + mixnet env, remote send once — confirm submit IP ≠ host.
3. **Desktop Ironwood testnet:** Plan → Migrate → Broadcast on Ironwood testnet profile + WSL Zebrad; CLI split if required.
4. **Do not market** “Nym integrated” or “mainnet Ironwood complete” until the above PASS rows exist.

---

## Key paths (quick index)

| Concern | Path |
|---------|------|
| Nym threat cases + checklist | [`NYM_IP_PRIVACY_CASE_BREAKDOWN.md`](NYM_IP_PRIVACY_CASE_BREAKDOWN.md) |
| Smolmix spike + `--sendraw-stdin` | [`tools/nym-smolmix-broadcast-spike/`](../../tools/nym-smolmix-broadcast-spike/) |
| Wallet mixnet gate | [`src/nym_mixnet_broadcast.rs`](../../src/nym_mixnet_broadcast.rs) |
| Safer migration / NymMixnetBroadcast | [`src/ironwood/network_privacy.rs`](../../src/ironwood/network_privacy.rs) |
| Desktop Ironwood cmds | [`desktop-client/src-tauri/src/commands/ironwood.rs`](../../desktop-client/src-tauri/src/commands/ironwood.rs) |
| Desktop Ironwood UI | [`desktop-client/src/components/IronwoodReadinessCard.tsx`](../../desktop-client/src/components/IronwoodReadinessCard.tsx) |
| Desktop README (testnet notes) | [`desktop-client/README.md`](../../desktop-client/README.md) |

---

## Local-only follow-ons since this session (indexed 2026-07-14)

| Item | Where it lives now |
|------|--------------------|
| Smolmix / dvpn spike trees + vendor | Documented under [`NYM_IP_PRIVACY_CASE_BREAKDOWN.md`](NYM_IP_PRIVACY_CASE_BREAKDOWN.md) |
| Cover estimator testnet results | [`IRONWOOD_WALLET_READINESS.md`](IRONWOOD_WALLET_READINESS.md) + [`COVER_ESTIMATOR_TESTNET_RESULTS.md`](COVER_ESTIMATOR_TESTNET_RESULTS.md) |
| Untracked `ironwood_handlers` / profile handlers | Ironwood readiness local index |
| Desktop sync incremental options | [`CLI_SHIELDED_SEND_CASE_BREAKDOWN.md`](CLI_SHIELDED_SEND_CASE_BREAKDOWN.md) local engine follow-ons |

---

## AI disclosure

Session engineering and this case breakdown assisted by Cursor Agent. Human review required before forum claims or release notes that assert mainnet/Nym product readiness.
