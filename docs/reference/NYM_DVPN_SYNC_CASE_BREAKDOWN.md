# Nym dVPN compact sync — case breakdown (track C / issue #146)

**Status:** Spike hardened; **C2/C3 live PASS** on `smoldvpn`/`develop` (2026-07-26)  
**Date:** 2026-07-26  
**Tracking:** [issue #146](https://github.com/LEONINE-DAO/Nozy-wallet/issues/146)  
**Nym guidance:** [recommended architecture](https://zcash-sdk.nym.com/recommended/) — bulk sync over **2-hop dVPN**; broadcasts stay on mixnet (track B)  
**Related:** [NYM_MIXNET_BROADCAST_CASE_BREAKDOWN.md](NYM_MIXNET_BROADCAST_CASE_BREAKDOWN.md) · [NYM_IRONWOOD_BASELINE_HYGIENE_CASE_BREAKDOWN.md](NYM_IRONWOOD_BASELINE_HYGIENE_CASE_BREAKDOWN.md) · [NYM_IP_PRIVACY_CASE_BREAKDOWN.md](NYM_IP_PRIVACY_CASE_BREAKDOWN.md)  
**Forum draft:** [NYM_DVPN_SYNC_FORUM_ARTICLE.md](NYM_DVPN_SYNC_FORUM_ARTICLE.md)  
**Code:** [`tools/nym-dvpn-lwd-spike/`](../../tools/nym-dvpn-lwd-spike/)  
**Evidence:** [`docs/reference/evidence/nym-c3-*.txt`](evidence/)

This is **track C**: prove and eventually product-wire **compact-block sync** through Nym smol-dvpn. It does **not** replace mixnet broadcast for IP↔tx.

---

## Living scoreboard

| ID | Item | Status | Notes |
|----|------|--------|-------|
| C0 | Why dVPN for sync (not mixnet) | **Documented** | Bulk transfer on mixnet is impractically slow (Nym guidance) |
| C1 | Isolated smol-dvpn + LWD gRPC spike | **Spike landed** | `tools/nym-dvpn-lwd-spike` (#146); defaults to **mainnet** when `NETWORK_NAME`/`NYM_API` unset |
| C2 | Direct vs tunnel blocks/s compare | **PASS** (2026-07-26) | 526 vs 67 blocks/s (~7.9×) — `evidence/nym-c3-dvpn-run-smoldvpn-develop.txt` |
| C3 | Sync against public LWD (`zec.rocks`) over dVPN | **PASS** (2026-07-26) | After Mark dep bump (`smoldvpn`/`develop`); tunnel exit Seoul KR ≠ Atlanta clearnet |
| C4 | Sync against local LWD `:9067` | **N/A for dVPN** | Same LAN trap as D2b — exit cannot reach `127.0.0.1` / WSL |
| C5 | `zeaking` / tonic `connect_with_connector` product path | **Landed** (2026-07-26) | `connect_lightwalletd_with_connector`; default direct unchanged; in-process Nym+zeaking blocked by sqlite `links` (C6 = companion/subprocess) |
| C6 | Desktop / API toggle for dVPN sync | **Landed + smoke PASS** (2026-07-26) | Settings → Network privacy; `nym_dvpn_sync` subprocess; live 100-block probe `evidence/nym-c6-desktop-dvpn-smoke.txt` (444 vs 55 blk/s, ~8.1×) |
| C7 | Destination split: dVPN sync LWD ≠ mixnet submit Zebrad | **Policy** | This machine: sync may use private LWD/local; remote submit stays Case A1 or smolmix |
| C8 | Forum / article draft | **Updated** | Honest FAIL/partial narrative for paper — [NYM_DVPN_SYNC_FORUM_ARTICLE.md](NYM_DVPN_SYNC_FORUM_ARTICLE.md) |

---

## Operator context (this machine, 2026-07-26)

| Service | Where | Reachability |
|---------|--------|--------------|
| Zebrad JSON-RPC | WSL `172.20.199.206:8232` | **LAN** — tip ~`3425401`; Case A1 for submit |
| lightwalletd | `::1:9067` | **Loopback** |
| Mixnet broadcast to Zebrad | — | **N/A** (RFC1918 refuse; Case A1 correct) |
| Mixnet IP relocate (D2a) | — | **PASS** re-run — exit `82.221.101.117` ≠ host |
| NymVPN consumer app | `nym-vpnd` installed | **Separate** from SDK spike credentials; may change host egress IP |

Recommended hybrid **on this host**:

1. **Submit / Ironwood broadcast** → local WSL Zebrad (Case A1) — already private enough for IP↔tx.
2. **Optional remote compact sync** → dVPN → public LWD (`zec.rocks` or equivalent) when not using local LWD.
3. **Hygiene** (track A) stays on for start-height + broadcast delay even on Case A1.

Do **not** expose unauthenticated Zebrad JSON-RPC to the public internet just to force a D2b PASS.

---

## Case family C — dVPN sync

### Case C0 — Mixnet is wrong for bulk sync

Nym: compact-block sync over the mixnet is not recommended. Use **2-hop dVPN** (WireGuard userspace) for line-rate bulk; keep mixnet for small sensitive submits.

### Case C1 — Spike crate

`nym-dvpn-lwd-spike` streams `GetBlockRange` through a smol-dvpn tunnel. Excluded from root workspace (large git deps + vendor patch for `libcrux-psq`).

Defaults:

- `NymNetworkDetails::new_mainnet()` when `NETWORK_NAME` + `NYM_API` are unset (mainnet Keplr `$NYM`).
- `DVPN_DIRECTORY_URL` default = `https://nymvpn.com/api/public/v1/directory/gateways?show_vpn_only=true`.
- Source Nym `envs/sandbox.env` only if the mnemonic/funds are **sandbox**.

### Case C2 / C3 — Live throughput proof (2026-07-26 campaign)

```powershell
cd tools/nym-dvpn-lwd-spike
$env:MNEMONIC = "<funded mainnet Nyx mnemonic — set only in your shell>"
# Do NOT source sandbox.env for mainnet Keplr NYM
cargo run --release -- --blocks 100 --lwd https://zec.rocks:443
```

#### What worked

| Step | Result | Evidence |
|------|--------|----------|
| Clearnet compact sync → `zec.rocks` | **PASS** — 100 blocks in ~1.5–1.6s (~61–67 blocks/s) | all `nym-c3-*.txt` |
| Mainnet session without sandbox.env | **PASS** after spike fix (`new_mainnet` default) | panic `nym api not set` fixed |
| Ticketbook recovery after failed issuance | **PASS** (implicit) — retry got past `ensure_ticketbooks` | attempt 1 FAIL → attempt 2 past issuance |

#### What failed

| Step | Error | Evidence |
|------|--------|----------|
| First ticketbook issuance | `Could not gather enough signature shares. Try again using the recovery command` (~15 min) | [`nym-c3-dvpn-run.txt`](evidence/nym-c3-dvpn-run.txt) |
| Gateway register (random) | timeout receive from `185.246.188.219:41264` | [`nym-c3-dvpn-run-retry.txt`](evidence/nym-c3-dvpn-run-retry.txt) |
| Gateway register (`--entry DE --exit CH --quic`) | timeout receive from `185.191.239.212:41264` (~27s) | [`nym-c3-dvpn-run-de-ch-quic.txt`](evidence/nym-c3-dvpn-run-de-ch-quic.txt) |

#### Network diagnostics (follow-up)

- TCP to both gateway `:41264` endpoints from this host: **Reachable** (`Test-NetConnection` = True). So this is **not** a simple firewall block on the registration port.
- During some runs, spike-reported clearnet egress was **Sydney / HostHatch** while a later host check showed **Atlanta / T-Mobile** — `nym-vpnd` was still installed/running. Consumer NymVPN and the SDK dVPN session should not be assumed independent; disconnect the app before claiming a clean C3 PASS.

#### Paper takeaway

Track C is **not** a green “Nym sync supported” claim. It is an honest engineering record:

1. Public LWD sync over clearnet is fine (baseline throughput).
2. Mainnet `$NYM` → ticketbooks is **operationally fragile** (issuance can fail; recovery may succeed on retry).
3. Even with ticketbooks, **smol-dvpn gateway registration** can time out while TCP to the same host:port succeeds — protocol/session layer, not “port closed.”
4. Local Zebrad (Case A1) remains the correct default for Ironwood **submit** IP privacy on this operator machine; dVPN is the optional remote-sync path.

### Case C4 — Local LWD

Listening `::1:9067` is fine for **direct** companion sync. Routing it through a Nym exit is the same E5 trap as LAN Zebrad — refuse / don't attempt.

### Case C5 — Product wire

**Landed (2026-07-26):**

1. `zeaking::lwd::connect_lightwalletd_with_connector` + `normalize_lwd_uri` — optional tonic connector; **default** `connect_lightwalletd` stays direct.
2. `LightwalletdBlockSource::connect_with_connector` mirrors the same hook.
3. Nym / smoldvpn stay **out** of `zeaking` deps (caller owns the tunnel). Spike proves the same `Endpoint::connect_with_connector` pattern; it does **not** path-dep zeaking because Nym `sqlx` and zeaking `rusqlite` both `links = "sqlite3"` (same class of clash as in-process smolmix).
4. Destination split still policy: never share the dVPN session with mixnet sendraw.

**Not yet:** auto-routing every companion LWD sync through dVPN when the flag is on (probe is opt-in today). Default compact sync remains direct local LWD.

---

## Evidence log

| Date (UTC) | Step | Result | Detail |
|------------|------|--------|--------|
| 2026-07-26 | Context: WSL Zebrad + local LWD | **Noted** | tip ~3425401; LWD ::1:9067 |
| 2026-07-26 | Track C case breakdown opened | **Landed** | this document |
| 2026-07-26 | Spike: mainnet default when env unset | **Landed** | avoids `nym api not set` panic |
| 2026-07-26 | C3 direct LWD sync | **PASS** | 100 blocks / ~1.5–1.6s → `zec.rocks` |
| 2026-07-26 | C3 ticketbooks (attempt 1) | **FAIL** | not enough signature shares ~15m — `nym-c3-dvpn-run.txt` |
| 2026-07-26 | C3 ticketbooks (retry) | **PASS** (implicit) | past issuance; failed later at gateway register |
| 2026-07-26 | C3 tunnel register (random) | **FAIL** | timeout `185.246.188.219:41264` — `nym-c3-dvpn-run-retry.txt` |
| 2026-07-26 | C3 tunnel register (DE/CH QUIC) | **FAIL** | timeout `185.191.239.212:41264` — `nym-c3-dvpn-run-de-ch-quic.txt` |
| 2026-07-26 | TCP probe gateways `:41264` | **PASS** | both register hosts reachable from Windows |
| 2026-07-26 | Dep bump per Mark | **Landed** | smolmix **1.21.2→1.21.4**; dVPN `nym-smol-dvpn`/`feature/nym-sdk-dvpn` → `smoldvpn` + session on **`develop`** (`1.21.5-rc.1`); both spikes `cargo check --release` OK |
| 2026-07-26 | C3 smoldvpn/develop live | **PASS** | clearnet Atlanta T-Mobile → tunnel Seoul KR; 100 blocks direct 0.19s / tunnel 1.50s (~7.9×) — `nym-c3-dvpn-run-smoldvpn-develop.txt` |
| 2026-07-26 | C5 zeaking connector API | **Landed** | `connect_lightwalletd_with_connector`; URI normalize tests; spike cannot link zeaking+Nym (sqlite) |
| 2026-07-26 | C6 desktop dVPN toggle | **Landed** | `nym_dvpn_sync` + Settings Network privacy opt-in/probe; subprocess helper |
| 2026-07-26 | C6 desktop smoke (same funded Nyx wallet) | **PASS** | `evidence/nym-c6-desktop-dvpn-smoke.txt` — 100 blocks; direct 444 blk/s vs tunnel 55 blk/s (~8.1×); exit FI |

---

## AI disclosure

Track C scaffolding, live runs, and this case breakdown assisted by Cursor Agent. Human review before product claims or forum posting.
