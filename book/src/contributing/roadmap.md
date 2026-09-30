# Roadmap

NozyWallet is an **Orchard + Ironwood, shielded-only** Zcash wallet with multiple surfaces (CLI, extension, web app, mobile, desktop). This page summarizes direction; the canonical list is in the repository:

**[ENHANCEMENT_ROADMAP.md](https://github.com/LEONINE-DAO/Nozy-wallet/blob/master/ENHANCEMENT_ROADMAP.md)**

---

## Production today

- **`nozy` CLI** on mainnet (Orchard + Ironwood NU6.3 migrate / send, ZIP-317 fees)
- **Zeaking** compact sync via lightwalletd
- **Launchpad** at [GitHub Pages](https://leonine-dao.github.io/Nozy-wallet/)

---

## In active development

| Area | Path | Goal |
|------|------|------|
| **Ironwood** | CLI / desktop / API | NU6.3 pool scan, safer migration, post-activation sends |
| **Web app** | `web-app/` | Browser dashboard via `nozywallet-api` |
| **Extension** | `browser-extension/` | MV3 privacy wallet + companion sync |
| **Mobile** | `nozy-mobile/` | Expo companion + optional VPS API |
| **Desktop** | `desktop-client/` | Tauri GUI and Ironwood migrate UX |
| **API server** | `api-server/` | HTTP companion for extension, web, mobile |

Web app starter doc: [web-app/README.md](https://github.com/LEONINE-DAO/Nozy-wallet/blob/master/web-app/README.md)

---

## Planned (aligned issues / RFCs)

- **Business / POS + ZNS** — [Issue #85](https://github.com/LEONINE-DAO/Nozy-wallet/issues/85)
- **Nozy Agent** — self-hosted Rust agent (local LLM, Nym-mandatory egress); see [ENHANCEMENT_ROADMAP.md](https://github.com/LEONINE-DAO/Nozy-wallet/blob/master/ENHANCEMENT_ROADMAP.md) and [Medium draft](https://github.com/LEONINE-DAO/Nozy-wallet/blob/master/docs/medium/BUILDING_NOZY_AGENT_INFRASTRUCTURE.md)
- **Bounded trading** — separate capped trading wallet; prefer shielded/private flows; **NEAR Intents is paper/canary-only** (demoted after [forum #57497](https://forum.zcashcommunity.com/t/my-experience-exiting-shielded-zec-from-zodl-to-near-intents-589k-usdt-still-held-50-days-despite-a-written-compliance-clearance/57497)); private→transparent exits are an explicit trust demotion, not “still private”
- **Multichain privacy** (Namada, Penumbra) — [Multichain RFC](https://github.com/LEONINE-DAO/Nozy-wallet/blob/master/docs/rfcs/MULTICHAIN_PRIVACY_CHAINS_RFC.md)

---

## How to contribute

1. Read [`AGENTS.md`](https://github.com/LEONINE-DAO/Nozy-wallet/blob/master/AGENTS.md) and [Contributing Guide](guide.md).
2. Pick an item from the enhancement roadmap or open an issue for alignment.
3. See [Development Setup](development-setup.md) for build and test commands.
