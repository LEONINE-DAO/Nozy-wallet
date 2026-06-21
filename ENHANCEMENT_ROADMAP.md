# NozyWallet enhancement roadmap

**Status:** Living document — ZEC-first, privacy super wallet.  
**Launchpad:** [`landing/`](landing/) · **Latest CLI:** [Releases](https://github.com/LEONINE-DAO/Nozy-wallet/releases/latest)

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

---

## Shipped / stable

- NU6.2 mainnet CLI (v2.3.x)
- ZIP-317 fees, spend detection, sync-to-tip
- Zeaking compact sync (`zeaking::lwd`)
- Landing product hub (ZEC-only)

---

## Backlog (ordered)

1. Web app W0–W2 (ZEC dashboard)
2. Extension production path + Chrome listing prep
3. Business profile + mobile Sell mode (issue #85)
4. Desktop production release
5. Multichain sidecars (Penumbra smoke → Namada spike per RFC)
6. Mobile App Store / Play (STORE-CHECKLIST)

---

## References

- [`AGENTS.md`](AGENTS.md) — contribution gate
- [`docs/journal/research/2026-06-21-nozy-super-wallet-stack.md`](docs/journal/research/2026-06-21-nozy-super-wallet-stack.md)
- [`browser-extension/COMPANION.md`](browser-extension/COMPANION.md)
- [`CHANGELOG.md`](CHANGELOG.md)

---

*Update this file when a surface moves from “in development” to production or when web-app phases complete.*
