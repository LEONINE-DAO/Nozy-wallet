# NozyWallet enhancement roadmap

**Status:** Living document — ZEC-first, privacy super wallet.  
**Tagline:** Your gateway to shielded value. *Private by default.*  
**Launchpad:** [`landing/`](landing/) · **Latest CLI:** [Releases](https://github.com/LEONINE-DAO/Nozy-wallet/releases/latest)

Full problem / solution / impact narrative: [`docs/NOZYWALLET_PROPOSAL.md`](docs/NOZYWALLET_PROPOSAL.md) (local working doc).

---

## Product surfaces

| Surface | Path | Status | Notes |
|---------|------|--------|--------|
| **CLI + core** | `src/`, `nozy` | **Mainnet** | Orchard shielded ZEC; Zebrad + lightwalletd |
| **Launchpad** | `landing/` | **Active** | GitHub Pages product hub |
| **Web app** | [`web-app/`](web-app/) | **Starting** | Full dashboard; extension + `nozywallet-api` |
| **Browser extension** | `browser-extension/` | Contributor preview | MV3 + WASM |
| **Operator API** | `api-server/` | In development | Localhost companion |
| **Desktop** | `desktop-client/` | In development | Tauri |
| **Mobile** | [`nozy-mobile/`](nozy-mobile/) | In development | Expo; see [VPS-DEPLOY](nozy-mobile/VPS-DEPLOY.md) |

**Privacy chains (later):** Namada, Penumbra — [`docs/rfcs/MULTICHAIN_PRIVACY_CHAINS_RFC.md`](docs/rfcs/MULTICHAIN_PRIVACY_CHAINS_RFC.md)

**Business / POS:** ZNS + Sell mode — [#85](https://github.com/LEONINE-DAO/Nozy-wallet/issues/85), [`docs/BUSINESS_ZEC_ZNS_TODO.md`](docs/BUSINESS_ZEC_ZNS_TODO.md)

---

## Shipped milestones

Releases and features already on mainline or tagged CLI releases. See [`CHANGELOG.md`](CHANGELOG.md) and [`docs/issues/FEATURE_REGISTRY.md`](docs/issues/FEATURE_REGISTRY.md).

### Core wallet & consensus

| Milestone | Version | Impact |
|-----------|---------|--------|
| **Dynamic fee pilot — Phase A1** | v2.3.0+ | Client-side ZIP-317; `--priority` ×4; short expiry; live fee estimates (CLI, API, desktop) — [RFC](docs/rfcs/DYNAMIC_FEE_PHASE_A_IMPLEMENTATION.md) |
| **NU6.2 mainnet compatibility** | v2.3.2 | Correct consensus branch ID; librustzcash alignment |
| **ZIP-317 fee shape fix** | v2.3.2 | Actions = `max(spends, outputs)` (not spends + outputs) |
| **Orchard spend detection (#61)** | v2.3.3 | Nullifiers at discovery; fixes second-send double-spend |
| **Single-note coin selection** | v2.3.4 | Reliable broadcast when wallet holds multiple notes |
| **Unified `wallet_sync`** | v2.3.6.2 | Shared scan → merge → persist for API + CLI |

### Dynamic fee pilot — follow-on (A1 completion)

| Milestone | Version | Impact |
|-----------|---------|--------|
| **`Expired` status + balance release** | v2.3.6.2 | Pending notes unlocked when tx passes expiry unmined |
| **Speed-up rebuild (not rebroadcast)** | v2.3.6.2 | `POST /api/transaction/speed-up`; desktop + extension WASM |
| **Extension `fee_policy` alignment** | v2.3.4+ | WASM uses shared ZIP-317 policy with core |

### Sync & operator stack

| Milestone | Version | Impact |
|-----------|---------|--------|
| **Compact sync (`zeaking::lwd`)** | Shipped | lightwalletd → SQLite; extension/API LWD path |
| **`nozy sync --to-tip` + `lwd prune`** | v2.3.1 | Find deposits without multi-run sync; stale cache cleanup |
| **`nozy status` sync diagnostics** | v2.3.1 | Zebra tip, LWD tip, scan gap, stale-cache warning |
| **Remote VPS Zebra + `trusted_zebra_urls`** | v2.3.6.3–6.4 | Hosted infra without disabling privacy policy globally |
| **Structured sync errors** | v2.3.6.x | JSON phases, 503 `ZEBRA_UNAVAILABLE`, connect error codes |

### API & product (recent)

| Milestone | Status | Impact |
|-----------|--------|--------|
| **Send reuses note cache (no 50k rescan)** | Branch / unreleased | Fast send after sync; see [CHANGELOG Unreleased](CHANGELOG.md) |
| **History merges received deposits** | Branch / unreleased | `Received` + `Sent` in `/api/transaction/history` |
| **Launchpad + product roadmap** | Merging | Product hub, web-app/mobile plans |

---

## Dynamic fee pilot — remaining (Phase A′ & A2)

Parent plan: [`docs/rfcs/DYNAMIC_FEE_PILOT_PLAN.md`](docs/rfcs/DYNAMIC_FEE_PILOT_PLAN.md) · Checklist: [`docs/rfcs/DYNAMIC_FEE_PHASE_A_IMPLEMENTATION.md`](docs/rfcs/DYNAMIC_FEE_PHASE_A_IMPLEMENTATION.md)

| Track | Status | Next |
|-------|--------|------|
| **A′1 Extension lifecycle parity** | In progress | Full speed-up/expiry parity with core per RFC §8 |
| **A′2 Testnet soak + pilot metrics** | Planned | Opt-in counters: priority sends, speed-ups, expired-unmined |
| **A2 Zeaking shared `fee_policy`** | **Blocked** | Shielded Labs go-ahead — move policy to `zeaking/`, compact expiry detection, observatory |

**Not in scope for A2 until SL approval:** changing Zebra/lightwalletd; Phase B standard-fee swap from Shielded Labs.

---

## Current focus: web app (2026)

The next major surface is **`web-app/`** — a browser dashboard (community-shaped super wallet), not the static launchpad.

| Phase | Goal |
|-------|------|
| **W0** | Scaffold Vite + React; shared theme with `landing/`; README + env template |
| **W1** | Connect to `nozywallet-api` (health, unlock, balance) on localhost |
| **W2** | Send / receive ZEC flows via companion API |
| **W3** | Extension messaging bridge (optional unlock path) |
| **W4** | Deploy to `/Nozy-wallet/app/` or custom domain |

Details: [`web-app/README.md`](web-app/README.md)

---

## Mobile (parallel track)

| Milestone | Doc |
|-----------|-----|
| Local dev + emulator | [`nozy-mobile/README.md`](nozy-mobile/README.md) |
| Public API on VPS | [`nozy-mobile/VPS-DEPLOY.md`](nozy-mobile/VPS-DEPLOY.md) |
| Store release | [`nozy-mobile/STORE-CHECKLIST.md`](nozy-mobile/STORE-CHECKLIST.md) |

Business profile + Sell mode: Phase 0 decisions locked — [`docs/BUSINESS_ZEC_ZNS_PHASE0_DECISIONS.md`](docs/BUSINESS_ZEC_ZNS_PHASE0_DECISIONS.md)

---

## Backlog (ordered)

1. Web app W0–W2 (ZEC dashboard)
2. Dynamic fee A′1–A′2 (extension parity + pilot metrics)
3. Extension production path + Chrome listing prep
4. Business profile + mobile Sell mode (issue #85)
5. Desktop production release
6. Dynamic fee A2 in Zeaking (when Shielded Labs approves)
7. Multichain sidecars (Penumbra smoke → Namada spike per RFC)
8. Mobile App Store / Play (STORE-CHECKLIST)

---

## References

- [`AGENTS.md`](AGENTS.md) — contribution gate
- [`docs/NOZYWALLET_PROPOSAL.md`](docs/NOZYWALLET_PROPOSAL.md) — problem, solution, milestones, impact
- [`docs/issues/FEATURE_REGISTRY.md`](docs/issues/FEATURE_REGISTRY.md) — feature IDs and status
- [`browser-extension/COMPANION.md`](browser-extension/COMPANION.md)
- [`CHANGELOG.md`](CHANGELOG.md)

---

*Update this file when a surface ships, a release tags, or web-app phases complete.*
