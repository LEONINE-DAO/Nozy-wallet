# NozyWallet White Paper

_Open `docs/NozyWallet_Whitepaper.docx` in Word or LibreOffice for the formatted version. Regenerate with `python scripts/generate-nozy-whitepaper.py`._

---

**NozyWallet**  
White Paper  
Architecture, Shielded Pools, Network Privacy, Zcash Names, and Mainnet Operation

**LEONINE DAO**  
Version **2.4.1** (Ironwood era)  
July 2026  

github.com/LEONINE-DAO/Nozy-wallet · nozywallet.com

---

# Executive Summary

NozyWallet is a self-custodial, shielded-first Zcash wallet and companion stack—not a consensus node. Operators run **Zebrad** for JSON-RPC (broadcast, tip, treestate) and **lightwalletd** for compact-block sync. The wallet derives Merkle witnesses **locally**, computes **ZIP-317** fees on-device, and builds ZIP-225 / Ironwood transactions with Halo 2 proofs on the user’s machine.

**What ships today (v2.4.x).** Mainnet-validated CLI (latest line **v2.4.1 / v2.4.1.1**), localhost **api-server**, desktop **1.0.0-beta.2**, browser extension beta, and mobile companion surfaces—all sharing the `nozy` Rust core and `zeaking` compact sync. Nozy participates in the Shielded Labs **dynamic-fee pilot** (client-side conventional fee, ×4 priority, five-block expiry) and ships **Ironwood (NU6.3)** migration tooling aligned with draft **ZIP 318**.

**Why Ironwood.** In 2026, a soundness issue in Orchard’s circuit motivated sealing Orchard and opening Ironwood with a corrected circuit and a **turnstile** so circulating supply can be reasoned about again. After activation height **3,428,143** (2026-07-28), ordinary sends move to Ironwood; Orchard value migrates via Plan → Split → Migrate → Broadcast.

**Network metadata.** Shielded proofs hide note contents; they do **not** hide that an IP submitted a transaction. Nozy’s Priority-1 path is a **hybrid**: baseline hygiene (start-height obfuscation, broadcast delay, tip guard) plus optional **Nym mixnet** broadcast / dVPN sync spikes—without claiming full “Nym integration” until live remote submit evidence is complete.

**Zcash Names.** Everyday and merchant UX will resolve `name.zcash` → UA via the ZNS indexer (proxied by `nozywallet-api`), link names to a **Business** Orchard account, and ship **Sell mode**—with in-wallet claim gated until ZNS signing is clear. Nozy is not the name registry.

This paper states architecture decisions, **mathematical policy** (fees, expiry, ZIP 318 buckets and denominations), Ironwood, Nym, and ZNS plans, release history, field evidence, and lessons for operators, Shielded Labs reviewers, grant readers, and contributors.

---

# 1. Architecture Decisions

Each decision uses an ADR pattern: context → decision → consequences.

### 1.1 Zebrad-only stack (no zcashd)

**Context.** Zcash infrastructure consolidates on Zebra; pilots target Zebrad + lightwalletd. The wallet must not embed consensus.

**Decision.** Zebrad JSON-RPC for broadcast, blocks, and treestate; lightwalletd gRPC via `zeaking::lwd` for compact sync. No zcashd in this repository.

**Consequences.** No `estimatefee` on Zebrad—fees are client-side. No spend-ready Orchard/Ironwood witness RPC—witnesses are local.

### 1.2 One Rust core, multiple surfaces

**Context.** Fee, expiry, witness catch-up, and broadcast retry must match across CLI, API, desktop, extension, and mobile.

**Decision.** Centralize logic in `nozy`; thin surfaces. Share send/migrate pipelines where possible.

**Consequences.** Extension WASM uses a separate Cargo.toml (workspace-excluded). Surface drift caused field bugs (BUG-2026-001–011); parity remains an explicit gate.

### 1.3 Local witness derivation

**Context.** Shielded spends need Merkle paths and anchors consistent with chain state.

**Decision.** Persist incremental witnesses (Orchard NoteIndex v2; Ironwood witness modules); catch up via Zebra blocks; verify with `z_gettreestate`. Parallel `getblock` batches (10/round) when lag is bounded.

**Consequences.** Stale witnesses dominate send latency. Policy: reject sends if witness lag \(L > 50\) blocks; sync to tip first.

### 1.4 Client-side ZIP-317 fees (dynamic-fee pilot)

**Context.** Pilot needs deterministic standard and priority fees; Zebrad does not implement fee estimation RPCs.

**Decision.** `fee_policy.rs`: ZIP-317 conventional fee; Nozy applies priority multiplier \(m = 4\) on all send surfaces.

**Consequences.** Orchard/Ironwood logical actions use \(\max(n_{\mathrm{spends}}, n_{\mathrm{outputs}})\), not the sum (v2.3.2 fix).

### 1.5 Five-block pilot expiry (not fifteen)

**Context.** Short mempool life enables Expired → priority rebuild (“speed-up”).

**Decision.** \(\Delta_{\mathrm{exp}} = 5\); \(h_{\mathrm{expiry}} = h_{\mathrm{tip}} + 1 + \Delta_{\mathrm{exp}}\). Reliability via late tip refresh and rebuild (≤3 attempts), not \(\Delta_{\mathrm{exp}} = 15\).

**Consequences.** Fifteen-block expiry was reverted (`a72bc6e8`): it slowed speed-up UX while only masking slow proves.

### 1.6 Note index v2

**Context.** Fast load, merged history, mark-spent after broadcast.

**Decision.** Versioned `NoteIndex` with nullifier/height maps; atomic rename writes.

**Consequences.** Legacy array parsers broke mark-spent until June 2026 fixes.

### 1.7 Ironwood-first after NU6.3

**Context.** Orchard sealed after Ironwood activation; ordinary value must use the corrected pool.

**Decision.** Post-activation routing to Ironwood; Orchard→Ironwood turnstile migration (ZIP 318 schedule, `{1,2,5}\times 10^k` denominations); desktop/landing Ironwood UX.

**Consequences.** Migration is a **privacy operation** (amounts visible on turnstile); network hygiene and optional mixnet apply especially to migrate-broadcast.

### 1.8 Network metadata privacy (Nym hybrid + hygiene)

**Context.** Clearnet submit links IP to mempool arrival time and (for turnstile) amount.

**Decision.** Baseline hygiene always available; Priority-1 modes include `NymMixnetBroadcast`; dVPN compact-sync spike tracked separately. Local Zebrad (Case A1) needs no mixnet.

**Consequences.** Do not market “Nym integrated” from attestation alone; require live remote evidence for strong claims.

### 1.9 System stack

```text
CLI · api-server · desktop · extension · mobile
        ↓
   nozy core  +  zeaking::lwd  +  ironwood/*  +  nym_mixnet_broadcast (opt-in)
        ↓                    ↓
 lightwalletd :9067     Zebrad :8232 (JSON-RPC)
```

---

# 2. Mathematical Model

This section states the formulas Nozy implements. Constants match `src/fee_policy.rs` and `src/ironwood/migration.rs`.

## 2.1 ZIP-317 conventional fee

Let \(f_m = 5000\) zatoshis be the marginal fee per logical action, and \(g = 2\) the grace action count.

Logical actions for an Orchard (or analogous shielded) send shape:

\[
a = \max\!\bigl(g,\; \max(n_{\mathrm{spends}}, n_{\mathrm{outputs}}) + a_{\mathrm{memo}}\bigr)
\]

where \(a_{\mathrm{memo}}\) counts memo chunks beyond the two free 512-byte chunks (ZIP-317).

**Conventional fee:**

\[
F_{\mathrm{conv}} = f_m \cdot a
\]

**Nozy priority fee** (always applied on send surfaces):

\[
F_{\mathrm{Nozy}} = m \cdot F_{\mathrm{conv}}, \qquad m = 4
\]

Floor when \(a = g = 2\): \(F_{\mathrm{conv}} = 10\,000\) zat, \(F_{\mathrm{Nozy}} = 40\,000\) zat.

## 2.2 Pilot expiry height

Given chain tip \(h_{\mathrm{tip}}\) and delta \(\Delta_{\mathrm{exp}} = 5\):

\[
h_{\mathrm{build}} = h_{\mathrm{tip}} + 1, \qquad
h_{\mathrm{expiry}} = h_{\mathrm{build}} + \Delta_{\mathrm{exp}} = h_{\mathrm{tip}} + 6
\]

A transaction is expired when \(h_{\mathrm{tip}} > h_{\mathrm{expiry}}\).

At mean block time \(t_b \approx 75\,\mathrm{s}\), mempool lifetime after build is approximately:

\[
T_{\mathrm{exp}} \approx \Delta_{\mathrm{exp}} \cdot t_b \approx 375\,\mathrm{s} \approx 6.25\,\mathrm{min}
\]

(Fifteen-block policy would have been \(\approx 19\,\mathrm{min}\)—rejected for pilot UX.)

## 2.3 Witness lag guard

Let \(h_w\) be the height to which spend witnesses are caught up. Lag:

\[
L = h_{\mathrm{tip}} - h_w
\]

Send policy:

\[
\text{allow send} \iff L \le L_{\max}, \qquad L_{\max} = 50
\]

If \(L > L_{\max}\), reject immediately and require sync-to-tip (avoids multi-minute mid-send catch-up).

**Latency model** (operator evidence):

\[
T_{\mathrm{send}} \approx T_{\mathrm{witness}} + T_{\mathrm{setup}} + T_{\mathrm{Halo2}} + T_{\mathrm{sign}} + T_{\mathrm{broadcast}}
\]

Synced wallets (\(L \le 50\)) observed \(T_{\mathrm{send}} \sim 200\,\mathrm{s}\) on a WSL Zebrad + Windows CLI stack (June 2026).

## 2.4 ZIP 318 anchor buckets (Ironwood migration)

Draft ZIP 318 groups migrations into shared **anchor-height buckets** of width \(B = 256\) blocks:

\[
b(h) = B \left\lfloor \frac{h}{B} \right\rfloor, \qquad
b_{\mathrm{next}}(h) = b(h) + B
\]

At \(t_b \approx 75\,\mathrm{s}\):

\[
T_{\mathrm{bucket}} \approx 256 \cdot 75\,\mathrm{s} \approx 5.33\,\mathrm{h}
\]

Transfer window / expiry alignment uses \(B\) blocks (`ZIP318_TRANSFER_EXPIRY_BLOCKS`). Default same-denomination cap per bucket: \(K_{\max} = 4\) (until ZIP finalizes).

## 2.5 Migration denominations \(\{1,2,5\}\times 10^k\)

Active Shielded Labs / Appendix A ladder (zatoshis): amounts of the form

\[
v \in \bigl\{ 1, 2, 5 \bigr\} \times 10^{k}, \quad k \in \mathbb{Z}_{\ge 0}
\]

(expressed in zatoshis with the smallest practical bucket \(0.001\,\mathrm{ZEC} = 10^5\) zat). Residuals

\[
r < r_{\min} = 10^5 \,\mathrm{zat} = 0.001\,\mathrm{ZEC}
\]

are **abandoned** rather than emitted as one-off turnstile sizes (`ZOOKO_RESIDUAL_ABANDON_ZAT`).

ZIP 318 power-of-ten ladders remain as a **compatibility** path; the wallet’s active default is the \(\{1,2,5\}\times 10^k\) schedule.

## 2.6 Baseline network hygiene (parameters)

Implemented defaults (`baseline_hygiene.rs`), complementary to mixnet:

| Parameter | Value | Role |
|-----------|-------|------|
| Checkpoint spacing | 256 blocks | Aligns with ZIP 318 bucket geometry |
| Max overlap rewind | 128 blocks | Bounds start-height choice |
| Broadcast delay | uniform in \([30, 300]\) s | Breaks “sync-then-immediate-submit” timing |
| Tip-sync guard | refuse migrate-broadcast if tip sync within 120 s | Reduces tip-correlated submit |

---

# 3. Ironwood (NU6.3) and Migration

## 3.1 Motivation

Orchard hides amounts. A circuit soundness bug (research by Taylor Hornby / Shielded Labs) created a **counterfeiting risk inside the shielded pool**. Patching stops new abuse of that path; it does not by itself restore the transparent-world property “anyone can verify circulating supply.” Ironwood:

1. **Seals** ordinary Orchard activity after activation.  
2. Opens a **new** shielded pool with the corrected circuit.  
3. Moves value through a **turnstile** so exits are accountable.

Primary narrative sources: Shielded Labs Ironwood overview; forum thread on verifying circulating supply; Nozy book `book/src/features/ironwood.md`.

## 3.2 Activation

| Network | Height | Calendar target |
|---------|--------|-----------------|
| Mainnet NU6.3 | \(H_{\mathrm{act}} = 3{,}428{,}143\) | 2026-07-28 |

After \(h \ge H_{\mathrm{act}}\): normal Orchard sends blocked; new sends route to Ironwood; Orchard balances migrate.

## 3.3 Wallet flow

Privacy-preserving operator flow:

1. **Sync** to tip (witnesses + notes).  
2. **Plan** ZIP 318 schedule (splits, denominations, buckets).  
3. **Split** Orchard→Orchard into schedule-sized notes if needed.  
4. **Migrate** (prebuild locked turnstile txs).  
5. **Broadcast** inside the scheduled window, with network-privacy mode selected.

Surfaces: CLI Ironwood commands, desktop Ironwood UI, mobile Ironwood screens, landing dashboard (`/ironwood`) with live pool stats, api-server handlers.

## 3.4 Privacy expectations

Turnstile crossings **reveal amounts** on-chain by design. Clearnet broadcast can link that amount to network identity. Nozy treats migration as an explicit privacy warning + safer egress path—not a silent balance upgrade. See `IRONWOOD_PRIVACY_EXPECTATIONS_ARTICLE.md` and `SAFE_MIGRATION_NETWORK_PRIVACY_FORUM_POST.md`.

---

# 4. Network Privacy and Nym

Shielded cryptography and mixnets solve **different** problems. Orchard / Ironwood hide **note contents**. Nym (and Tor) hide **who talked to which endpoint**. Ironwood migration makes the gap acute: turnstile amounts are **public by design**, so an observer who also sees **IP ↔ submit time** can join network identity to a pool-crossing event even when ZIP 318 denominations are perfect.

Nozy’s network-privacy work follows three priorities after ZIP 318 mechanics themselves (forum draft `SAFE_MIGRATION_NETWORK_PRIVACY_FORUM_POST.md`):

1. **Protect the broadcasting IP** (local node / Tor / Nym).  
2. **Share cover traffic** (real ZIP 318 cohorts, not lonely turnstiles).  
3. **Amount + timing selection** (`{1,2,5}\times10^k`, bucket discipline).

Nym engineering sits primarily in **Priority 1**, with hygiene that transport alone cannot provide.

## 4.1 Threat model

### Attacker capabilities (assumed)

1. Observes ISP / Wi‑Fi / VPS egress: connections to Zebrad RPC, lightwalletd gRPC, or explorers.  
2. Operates or logs a **remote** lightwalletd or submit API: sees **client IP + timestamp + raw tx** (or gRPC `SendTransaction`).  
3. Correlates **mempool appearance time** of a known txid with that IP session.  
4. For Ironwood: joins **IP ↔ turnstile amount** even when ZIP 318 buckets and denoms are correct.

### Attacker non-capabilities (do not assume broken)

- Cannot decrypt Orchard / Ironwood note plaintexts from the wire alone.  
- Cannot forge spends without keys.  
- Localhost / LAN Zebrad with no remote submit is out of scope for “public LWD saw your IP.”

### Defender goal

Prevent: **“this IP submitted this txid / this migration turnstile.”**  
Secondary: reduce sync-volume metadata and censorship resistance (dVPN / QUIC bridge).

**Rule of thumb.** Biggest remote win is **IP ↔ transaction submit**—send, split, and especially **migrate-broadcast**. Sync-over-Nym is valuable but secondary; sync-over-Nym with **clearnet submit** is a false sense of security.

## 4.2 Where the IP leak actually is (case family A)

| Case | Flow | Leak | Product stance |
|------|------|------|----------------|
| **A1** Local Zebrad | Wallet → `127.0.0.1` / LAN `:8232` → `sendrawtransaction` | No public LWD sees wallet IP; ISP may see **node** P2P, not “app → LWD submit” | **Preferred default.** `MigrationNetworkPrivacyMode::LocalNode` allowed |
| **A2** Remote Zebrad clearnet | Wallet → public `:8232` → `sendraw` | Remote operator + path: **IP ↔ broadcast** | Must gain Nym/Tor on broadcast; refuse clearnet unless explicit force |
| **A3** Remote LWD submit | Build local → gRPC/HTTP SendTransaction | Surgical **IP ↔ tx bytes** at LWD | **Biggest Mark/Nym win:** all outgoing txs over mixnet to that endpoint |
| **A4** dVPN sync + clearnet submit | GetBlockRange via smol-dvpn; submit clearnet | Still **IP ↔ tx** on submit | **Reject as “Nym done.”** Issue #146 alone does not close Priority 1 |

## 4.3 Nym product modes (case family B)

Nym’s Zcash guidance ([zcash-sdk.nym.com](https://zcash-sdk.nym.com/)) maps to three transports Nozy tracks:

### B1 — smolmix (mixnet) for submit / small RPC

**Buys:** Client IP hidden from RPC/LWD; timing obfuscation; exit can rotate.  
**Fit:** `sendrawtransaction`, Ironwood migrate-broadcast, fee/status RPCs, small unary calls.  
**Cost:** Higher latency than WireGuard—acceptable for rare broadcasts.  
**Role:** **Primary** transport for Cases A2/A3. Wired via `src/nym_mixnet_broadcast.rs` + spike `tools/nym-smolmix-broadcast-spike/` (issue #147).

### B2 — smol-dvpn (2-hop WireGuard) for bulk sync

**Buys:** Throughput; user-space tunnel ≈ natural killswitch; optional QUIC bridge.  
**Fit:** Compact `GetBlockRange` (Nym `zcash-sync` pattern; our #146 spike).  
**Role:** Sync / censorship path. **Does not** replace mixnet on submit unless mixnet is blocked and dVPN is an explicit fallback.

### B3 — mix-fetch / mix-websocket (browser)

**Buys:** `fetch` over mixnet in WASM / WebView.  
**Fit:** Extension companion API, explorer HTTP—**later** (after desktop/CLI broadcast path).

### B4 — System NymVPN + attestation only

**Buys:** Possible OS-wide cover **if** the user actually runs it.  
**Risk:** Wallet cannot prove traffic entered the mixnet; easy to forget; wrong-app bypass.  
**Role:** **Bridge only.** Do not claim “Nym integrated” from a checkbox.

## 4.4 Recommended hybrid architecture

\[
\begin{align}
\text{remote sync} &\xrightarrow{\text{smol-dvpn (B2)}} \text{lightwalletd}\\
\text{remote submit} &\xrightarrow{\text{smolmix (B1)}} \text{Zebrad / LWD SendTransaction}\\
\text{always} &\xrightarrow{\text{baseline hygiene}} \text{start height, delay, tip guard}\\
\text{local Zebrad (A1)} &\xrightarrow{\text{direct JSON-RPC}} \text{(mixnet optional)}
\end{align}
\]

In words: **dVPN for bulk sync when LWD is remote; mixnet for every outgoing transaction when submit is remote; hygiene on every migrate path; prefer local Zebrad when the operator can run one.**

## 4.5 Baseline hygiene (transport cannot invent this)

Landed 2026-07 (`src/ironwood/baseline_hygiene.rs`). These break “sync-then-immediate-submit” and tip-correlated migrate patterns that mixnets alone do not fix.

| ID | Mechanism | Default | Intent |
|----|-----------|---------|--------|
| H1 | Checkpoint spacing for scan start | \(S = 256\) blocks | Align start choices with ZIP 318 bucket geometry |
| H2 | Max overlap rewind | \(R_{\max} = 128\) blocks | Bound how far start may rewind |
| H3 | Randomized broadcast delay | \(D \sim U[30, 300]\) s | Decorrelate tip-sync / UI open from mempool arrival |
| H4 | Tip-sync guard | refuse migrate-broadcast if tip sync within \(T_{\mathrm{guard}} = 120\) s | Stop “just caught tip → instantly turnstile” |

**Obfuscated start (sketch).** Given a naive resume height \(h_0\), draw a rewind \(r \in [0, R_{\max}]\) and snap to a checkpoint:

\[
h^\star = S \left\lfloor \frac{\max(0,\, h_0 - r)}{S} \right\rfloor
\]

with \(h^\star \le h_0\) (never raise the start). Exact policy is in `obfuscate_scan_start`.

**Broadcast delay.** Before migrate-broadcast (when enabled):

\[
D \sim \mathrm{Uniform}(D_{\min}, D_{\max}), \quad D_{\min}=30,\; D_{\max}=300
\]

**Tip-sync guard.** Let \(t_{\mathrm{sync}}\) be the Unix time of the last tip sync and \(t_{\mathrm{now}}\) now. Proceed only if:

\[
t_{\mathrm{now}} - t_{\mathrm{sync}} \ge T_{\mathrm{guard}}
\]

(or operator explicitly skips the guard in advanced flows).

## 4.6 Safer migration Priority 1 modes

`MigrationNetworkPrivacyMode` (`network_privacy.rs`) gates migrate-broadcast:

| Mode | Meaning |
|------|---------|
| `LocalNode` | Healthy local/LAN Zebrad—preferred |
| `TorSocks` / I2P (detect) | SOCKS privacy network detected |
| `NymMixnetBroadcast` | Opt-in smolmix helper on the broadcast path |
| Attestation / clearnet force | Discouraged exception; never pre-selected as “OK” |

Policy sketch: **refuse clearnet broadcast to a remote node** unless the user takes an explicit, discouraged exception. ZIP 318 says no network-privacy option may be pre-selected; Nozy agrees and may go further on high-risk flows.

## 4.7 Surfaces (case family C)

| Surface | Submit path today | Required win |
|---------|-------------------|--------------|
| CLI `nozy send` / `ironwood broadcast` | `ZebraClient::broadcast_transaction` ± `nym_mixnet_broadcast` | Remote URL → force smolmix or attested Tor; refuse clearnet unless `--force-clearnet` |
| Desktop Tauri | Same core library | Show **egress mode** in UI (local / Tor / Nym / dVPN / blocked) |
| api-server / extension companion | Machine running API is the IP Zebrad/LWD sees | Nym must wrap that process’s submit, or extension mix-fetch later |
| Mobile | Companion → user API | Follow desktop once FFI owns a session; no attestation-only “Nym” |

Env gates (operators): `NOZY_BROADCAST_VIA_NYM_MIXNET`, `NOZY_NYM_SMOLMIX_BIN`—fail-closed for remote clearnet when mixnet mode is selected until the helper is present.

## 4.8 Evidence and honesty bar

| ID | Item | Status (as of 2026-07) |
|----|------|-------------------------|
| H1–H4 | Baseline hygiene | **Landed** |
| D2a | smolmix IP relocate (exit ≠ host) | **PASS** (2026-07-11; re-PASS 2026-07-26) |
| D2b-reachability | LAN refuse / URL classify | **PASS** |
| D2b | JSON-RPC probe over smolmix to **public** Zebrad | **N/A** on operator LAN Case A1; still needed for remote claims |
| D2c | Opt-in mixnet `broadcast_transaction` | **Wired** (subprocess) |
| D2c-live | Live remote `sendraw` over mixnet | **Open** (only when `zebra_url` is public / exit-reachable) |
| D2d | Ironwood broadcast shares same egress | **Wired** (`NymMixnetBroadcast`) |
| D2e | Desktop live mixnet session UI | **Partial** |
| D1 / C* | smol-dvpn compact sync | Spike + case breakdown; live C2/C3 pending credentials |

**Marketing rule.** Do not claim product “Nym support” or “Nym integrated” until **D2b + D2c-live** (or equivalent) are PASS for the remote-submit story users care about. Local-node operators (A1) are already in the strong default without mixnet.

Living checklist and evidence JSON: `NYM_IP_PRIVACY_CASE_BREAKDOWN.md`, `docs/reference/evidence/nym-*`, issues [#146](https://github.com/LEONINE-DAO/Nozy-wallet/issues/146), [#147](https://github.com/LEONINE-DAO/Nozy-wallet/issues/147).

## 4.9 What Nym does *not* claim

Nym / Tor reduce **IP ↔ session** linkage. They do **not**:

- erase KYC or CEX deposit history;  
- defeat subset-sum on **revealed** turnstile amounts;  
- protect a compromised device or leaked seed;  
- replace ZIP 318 cohort privacy (Priority 2) or denomination algorithms (Priority 3).

Orchard/Ironwood hide note plaintexts. Mixnets hide **who talked to the network**. Both are required for a credible Ironwood migration story on remote infrastructure.

---

# 5. Zcash Names (ZNS)

Human-readable names are the missing UX layer between **shielded addresses** and everyday payments. **Zcash Names (ZNS)** maps names such as `tacostand.zcash` to an Orchard (and, over time, Ironwood) **unified address (UA)** so users and merchants can send and receive without pasting long `u1…` strings.

Nozy does **not** operate the ZNS registry. We integrate as a **wallet + companion**: resolve names for send/receive, link a claimed name to a Business profile, and—later—optional in-wallet claim/update once protocol signing is clear. Protocol home: [zcashme/ZNS](https://github.com/zcashme/ZNS), product site [zcashnames.com](https://www.zcashnames.com), SDK [`zcashname-sdk`](https://www.npmjs.com/package/zcashname-sdk). Product backlog: issue [#85](https://github.com/LEONINE-DAO/Nozy-wallet/issues/85), `docs/BUSINESS_ZEC_ZNS_TODO.md`, Phase 0 decisions locked in `docs/BUSINESS_ZEC_ZNS_PHASE0_DECISIONS.md`.

## 5.1 How ZNS works with Zcash

At a high level:

1. **Registration / update** happens as a **shielded Zcash transaction** whose **memo** carries a ZNS protocol payload (claim / update semantics documented by Zcash Names). Value may be sent to a **registry UA** at a protocol-defined cost.  
2. An **indexer** watches the chain, verifies authorized updates, and exposes a **JSON-RPC** API (`resolve`, availability, status, …).  
3. Wallets **resolve** `name` / `name.zcash` → UA before building a normal shielded send. Payment itself remains a standard Orchard/Ironwood transfer to that UA—ZNS is identity, not a new pool.

\[
\mathrm{name.zcash} \;\xrightarrow{\;\mathrm{indexer\;resolve}\;} \mathrm{UA}
\;\xrightarrow{\;\mathrm{ZIP\text{-}225\;/\;Ironwood\;send}\;} \mathrm{notes}
\]

**Trust model for Nozy.** Clients should not treat a public indexer as an unauthenticated oracle. Phase 0 decisions require:

- api-server **proxies** resolve (`GET /api/zns/resolve?name=`);  
- call protocol **`verify()`** / Merkle identity checks before treating production responses as authoritative;  
- short **TTL cache** (e.g. 60 s) for name → UA;  
- indexer URLs configurable (`ZNS_MAINNET_URL` / `ZNS_TESTNET_URL`), defaults currently point at Zcash Names beta endpoints (`light.zcash.me/zns-*`).

**Recipient parsing.** Accept bare `u1…`, bare `name`, or `name.zcash`; normalize names before resolve.

**Signing gate (important).** As of mid-2026 review against ZNS HEAD, indexer authorization centers on **admin pubkey** verification. README/SDK text still describes optional **sovereign** user Ed25519 claims in places, but that path was not reliable on public indexer builds. Therefore Nozy’s MVP is **external claim/buy** (zcashnames.com) + **link** inside the wallet—not holding registry admin keys, and not shipping in-wallet CLAIM until signing mode is clear.

## 5.2 How ZNS fits NozyWallet (Personal / Business)

| Concept | Decision |
|---------|----------|
| Seed | **One BIP39 mnemonic** for Personal and Business |
| On-chain separation | Orchard **account index** Personal **0**, Business **1** (ZIP-32 coin type 133) |
| Display name | Optional local stall/brand label (not on-chain) |
| Migration | Existing wallets default Personal / account 0 |
| Merchant identity | Linked `.zcash` name should resolve to **Business UA** (account 1) |

**Product story.** A vendor claims `tacostand.zcash` (externally at first), links it to Business in Nozy, opens **Sell mode**, and shows a QR for the name or UA. A customer sends shielded ZEC to that name from any compatible wallet. Books and optional UFVK disclosure come later—names are for **pay UX**, not for replacing shielding.

```text
Customer:  resolve(tacostand.zcash) → UA₁  →  shielded pay
Vendor:    Business account 1 + linked name  →  Sell QR / invoice
Indexer:   ZNS JSON-RPC (proxied by nozywallet-api)
```

## 5.3 Implementation roadmap (future plans)

Ordered phases from the locked backlog (do not skip gates):

| Phase | Scope | Status intent |
|-------|--------|----------------|
| **0** | Product decisions, indexer URLs, MVP/v1/v2 criteria | **Locked** |
| **1** | Personal / Business profiles (create + Settings) | Planned |
| **2** | ZNS **resolve** + **link** name to Business UA; Send accepts `name.zcash`; CLI optional | **Highest leverage next** |
| **3** | Mobile **Sell mode** (QR POS, prefer linked name) | Planned |
| **3b** | Native merchant invoices on `nozywallet-api` (no third-party gateway) | Spec’d (`NOZY_MERCHANT_NATIVE.md`) |
| **4** | **ZIP-321** payment URIs (amount + address/name + memo) | Planned |
| **5** | Business CSV ledger + **UFVK** accountant disclosure (never spending key) | Planned |
| **6** | In-wallet ZNS **claim/update** (gated on sovereign/cosign clarity) | Deferred until signing clear |
| **7+** | Optional NFC, deeper POS | Later |

**MVP success (food-vendor demo).** Business profile + Sell QR showing linked `.zcash` name or UA; customer pays; vendor sees balance after documented sync.

**v1.** Send-by-name everywhere + Business CSV export.  
**v2.** In-wallet claim/update + UFVK grant with audit log.

**Explicitly deferred.** Secret/SILK/Fina as primary business story; multi-user org custody without shared seed; fiat instant settlement; in-wallet claim before resolve + Sell are stable.

## 5.4 Privacy and security notes for names

- Resolving a name reveals **which UA** the name points at (to the wallet and, if careless, to the indexer operator). Prefer api-server proxy + verified indexer; avoid logging resolved names next to customer IPs.  
- Paying to a name is still a **shielded** transfer; ZNS does not make amounts public (unlike Ironwood turnstile).  
- A linked public `.zcash` name is a **merchant identity** choice—advertising a name intentionally discloses “this UA is my shop.”  
- Never store ZNS **admin** private keys in Nozy; Nozy is not the registry.

## 5.5 Relation to Ironwood and Nym

After NU6.3, Business UAs and ZNS targets should follow the **active shielded pool** (Ironwood for new value). Name → UA resolution stays an indexer concern; Nozy’s send path must resolve then build against the correct pool. Merchant **migrate-broadcast** and remote submit still follow §4 (local Zebrad preferred; Nym/Tor for remote IP↔submit). ZNS does not replace network-privacy gates on migration.

---

# 6. Phased Development and Releases

## 6.1 Phase table

| Phase | Name | Shipped | Gate |
|-------|------|---------|------|
| 0 | Foundation | HD wallet, Orchard scan, CLI | Mainnet scan |
| 1 | Zebrad + LWD | `zeaking::lwd`, compact SQLite | Sync to tip |
| 2 | Mainnet send | Witnesses, ZIP-225, broadcast | Mainnet TXID |
| 3 | NU6.2 + pilot A1 | librustzcash bump, 5-block expiry, ZIP-317 shape | Branch ID + fees |
| 4 | Surfaces | api-server, extension companion, desktop WIP | Send/sync parity |
| 5 | Reliability (2026-06) | BUG-2026-001–011, send-readiness | Evidence doc PASS |
| 6 | Ironwood | NU6.3 pool, ZIP 318 plan/split/migrate, landing beta.2 | Testnet→mainnet migrate |
| 7 | Network privacy | Hygiene + Nym mixnet path | Evidence before marketing claims |
| 8 | Business + ZNS | Profiles, resolve/link, Sell mode | MVP vendor demo |
| 9+ | Observatory / merchant native | Pilot metrics, invoices, claim | Per feature |

## 6.2 Release matrix (representative)

| Surface | Version line | Notes |
|---------|--------------|-------|
| CLI / `nozy` | **v2.4.0** Ironwood → **v2.4.1 / v2.4.1.1** | Mainnet CLI; Ironwood commands |
| Prior CLI | v2.3.0–v2.3.6.7 | Dynamic fee, NU6.2, witness lag guard |
| Desktop | **1.0.0-beta.2** | Aligns with CLI v2.4.1.x; Ironwood UX |
| Extension | **0.1.x** beta | Companion + WASM; store beta tags |
| Mobile | **1.0.0** companion | API + Zebrad you control; Sell/ZNS planned |
| api-server / zeaking | 0.1.x crates | Local companion / sync; ZNS proxy planned |

Exact Git tags live on GitHub Releases; this matrix is the product story for the paper.

---

# 7. Challenges and Responses

## 7.1 Zebrad integration

Missing fee and witness RPCs are **wallet** problems on a Zebrad stack. Response: client ZIP-317 + local witnesses + treestate verification (`ZEBRAD_SHIELDED_SEND_LIMIT.md`).

## 7.2 Two clocks on shielded sends

**Build clock:** witness → prove → sign → broadcast.  
**Mempool clock:** starts after successful broadcast; pilot expiry \(\Delta_{\mathrm{exp}}=5\).

Conflating them caused pre-broadcast −25 errors on slow VPS; fixed with late tip refresh and rebuild—not longer \(\Delta_{\mathrm{exp}}\).

## 7.3 Ironwood migration UX vs privacy

Schedule math (buckets, denominations) is necessary but not sufficient. Users must understand turnstile amount disclosure and choose egress (local node vs mixnet).

## 7.4 Surface parity

One fee/expiry/Ironwood policy across CLI, API, desktop, extension, mobile remains the expensive ongoing gate.

## 7.5 ZNS indexer trust and claim gating

Beta indexers and admin-signed claims mean wallets must **verify** indexer identity, keep URLs configurable, and avoid shipping in-wallet CLAIM until sovereign/cosign paths are real. External claim + link is the honest MVP.

---

# 8. Trade-offs

| Choice | Alternative rejected | Why |
|--------|----------------------|-----|
| Zebrad + LWD | Embed zcashd / consensus | Wallet ≠ node; operator alignment |
| Local witnesses | Hope for node witness RPC | Not available; documented limit |
| \(\Delta_{\mathrm{exp}}=5\) | 15 | Speed-up UX; rebuild instead |
| \(F = 4 F_{\mathrm{conv}}\) | User-tunable fee UI | Pilot simplicity; all surfaces identical |
| ZIP 318 schedule | One-shot migrate-all | Amount/timing privacy; draft ZIP intent |
| Hygiene + optional Nym | Claim “anonymous by default” | Honest threat model; evidence-gated claims |
| Ironwood after NU6.3 | Stay Orchard-only | Pool sealed; product would brick |
| ZNS resolve + external claim | Hold registry admin key in Nozy | Wallet ≠ name registry |
| Business = account 1 same seed | Second mnemonic for merchants | One backup; ZIP-32 accounts |

---

# 9. Security and Privacy Considerations

- High-impact code: keys, seeds, addresses, RPC URLs—no careless logging.  
- Shielded-first product policy (no accidental transparent sends).  
- Ironwood migration warnings for amount disclosure.  
- Nym/mixnet is **opt-in / evidence-gated**; attestation checkboxes are not proof of egress.  
- ZNS: proxy + verify indexer; do not log name↔IP casually; never store registry admin keys.  
- No third-party audit claimed in this document; responsible disclosure per `CONTRIBUTING.md` / `SECURITY.md`.

Privacy properties of notes follow Orchard/Ironwood ZK; network metadata is a **separate** layer addressed in §4; **names** are an intentional identity layer addressed in §5.

---

# 10. Mainnet Field Evidence (June 2026)

Operator stack: Windows host, Zebrad in WSL (JSON-RPC), nozy CLI release build. Dust amounts (0.0001 ZEC) for regression.

Successful send TXIDs:

- `5a03fbd19547f9499182d78c88791eeb4eaab32e5d158b69ec8ffdc6068d2612`
- `902cf006efdeef3f15fed4312f8a15fcb1162f52495098c3bffb4acbe3cde4e5`

Observed \(T_{\mathrm{send}} \sim 200\,\mathrm{s}\) when \(L \le 50\). Proving warm-up \(\sim 2.1\,\mathrm{s}\) cold.

Ironwood / Nym evidence continues under `docs/reference/evidence/` and case-breakdown docs (2026-07); see §4.8 for what is PASS vs open.

---

# 11. Dynamic-Fee Pilot Alignment

| Pilot feature | Implementation | Lesson |
|---------------|----------------|--------|
| Standard fee | ZIP-317 in `fee_policy.rs` | Zebrad has no fee RPC |
| Priority ×4 | All Nozy send surfaces | Speed-up after Expired |
| Short expiry | \(h_{\mathrm{expiry}} = h_{\mathrm{tip}}+1+5\) | Keep 5; rebuild for slow proves |
| Speed-up | New tx at priority fee | Not rebroadcast of expired bytes |

---

# 12. Lessons Learned

1. Two clocks: build-time expiry vs mempool expiry—fix the first without lengthening the second.  
2. Wallet ≠ node: witnesses and fees are wallet duties on Zebrad.  
3. Sync-before-send: \(L_{\max}=50\) prevents pathological catch-up.  
4. Keep pilot knobs stable; improve runtime (rebuild, warm prove) first.  
5. Operator stacks (WSL Zebrad + Windows CLI) are first-class test targets.  
6. Cache migrations (NoteIndex v2) bite if any path still uses legacy parsers.  
7. Surface parity is expensive and mandatory.  
8. Ironwood migration is cryptography **plus** schedule math **plus** network metadata.  
9. Do not overclaim Nym: ship hygiene, wire mixnet, publish evidence.  
10. TXIDs and measured timings outperform narrative alone.  
11. ZNS is identity on top of shielding—resolve/link first; claim later; never become the registry.

---

# 13. Conclusion

NozyWallet shows a shielded-first wallet can run on Zebrad + lightwalletd with client-side ZIP-317 fees, short pilot expiry, and local witnesses—without zcashd and without stretching mempool expiry to hide slow proves. **Ironwood** extends that story to NU6.3 supply soundness and ZIP 318 migration. **Nym-oriented** broadcast and hygiene address the IP↔submit gap that ZK notes never covered. **Zcash Names** are the planned human layer for merchant and everyday send/receive—resolve and link on a pure ZEC Business profile, with in-wallet claim gated on protocol reality. Releases through **v2.4.1.x** and desktop **beta.2** carry the core into operator hands.

Continued work: Ironwood migrate hardening, honest Nym evidence for remote nodes, surface parity, ZNS Phase 1–3 (profiles, resolve/link, Sell), native merchant invoices, and pilot observatory metrics.

---

# References

- LEONINE-DAO/Nozy-wallet — github.com/LEONINE-DAO/Nozy-wallet  
- nozywallet.com (marketing / Ironwood dashboard)  
- `docs/reference/MAINNET_SEND_READINESS_EVIDENCE.md`  
- `docs/reference/PILOT_EXPIRY_PROVING_LATENCY.md`  
- `docs/reference/IRONWOOD_WALLET_READINESS.md`  
- `docs/reference/IRONWOOD_PRIVACY_EXPECTATIONS_ARTICLE.md`  
- `docs/reference/NYM_IP_PRIVACY_CASE_BREAKDOWN.md`  
- `docs/reference/SAFE_MIGRATION_NETWORK_PRIVACY_FORUM_POST.md`  
- `docs/BUSINESS_ZEC_ZNS_TODO.md` · `docs/BUSINESS_ZEC_ZNS_PHASE0_DECISIONS.md`  
- `ZEBRAD_SHIELDED_SEND_LIMIT.md`  
- `book/src/features/ironwood.md`  
- Zebra — github.com/ZcashFoundation/zebra  
- lightwalletd — github.com/zcash/lightwalletd  
- ZIP-316, ZIP-317, ZIP-225, ZIP-321; draft ZIP 318 (zips PR #1317)  
- Shielded Labs Ironwood materials — shieldedlabs.net/ironwood/  
- Nym Zcash guidance — zcash-sdk.nym.com  
- Zcash Names — zcashnames.com · github.com/zcashme/ZNS · npm `zcashname-sdk`  

---

# Appendix A: Bug registry summary (2026-06)

| ID | Summary | Status |
|----|---------|--------|
| BUG-2026-001 | Send rescanned ~50k blocks | Fixed |
| BUG-2026-002 | History empty despite balance | Fixed |
| BUG-2026-011 | Pre-broadcast expiry −25 on slow VPS | Fixed |
| — | Witness lag guard, warm prove | Fixed |
| — | NoteIndex v2 mark-spent | Fixed |

---

# Appendix B: Glossary

| Term | Definition |
|------|------------|
| Anchor | Commitment-tree root at a block height |
| Compact block | lightwalletd compressed block for sync |
| Ironwood | NU6.3 shielded pool (corrected circuit) |
| \(n\)ExpiryHeight | ZIP-225 last mineable height |
| Pilot expiry | \(\Delta_{\mathrm{exp}}=5\) after mempool build height |
| Turnstile | Accounted Orchard→Ironwood value exit |
| Witness | Merkle inclusion path for a note commitment |
| ZIP 318 | Draft migration scheduling (buckets, \(K_{\max}\)) |
| UFVK | Unified full viewing key (ZIP-316) |
| Mixnet broadcast | Submit `sendraw` via Nym smolmix egress (opt-in) |
| smol-dvpn | Nym 2-hop WireGuard path for bulk compact sync |
| Baseline hygiene | Start obfuscation, broadcast delay \(D\sim U[30,300]\), tip-sync guard |
| ZNS | Zcash Names — `name.zcash` → UA via memo + indexer |
| Business profile | Same seed; Orchard account index 1; Sell / ledger defaults |

---

# Appendix C: Constant cheat-sheet

| Symbol / name | Value | Source |
|---------------|-------|--------|
| \(f_m\) | 5 000 zat | ZIP-317 / `MARGINAL_FEE_ZATOSHIS` |
| \(g\) | 2 | `GRACE_ACTIONS` |
| \(m\) | 4 | `PRIORITY_MULTIPLIER` |
| \(\Delta_{\mathrm{exp}}\) | 5 | `PILOT_EXPIRY_DELTA_BLOCKS` |
| \(L_{\max}\) | 50 blocks | Send-readiness |
| \(B\) | 256 blocks | `ZIP318_ANCHOR_BUCKET_INTERVAL_BLOCKS` |
| \(K_{\max}\) | 4 | `ZIP318_DEFAULT_K_MAX` |
| \(r_{\min}\) | \(10^5\) zat | `ZOOKO_RESIDUAL_ABANDON_ZAT` |
| \(H_{\mathrm{act}}\) | 3 428 143 | NU6.3 mainnet |

---

— End of White Paper —
