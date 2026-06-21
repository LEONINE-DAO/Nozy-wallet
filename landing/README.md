# Launch page (NozyWallet launchpad)

Public **marketing / product hub** at GitHub Pages root (`https://leonine-dao.github.io/Nozy-wallet/`). This is **not** the wallet app.

## Stack (ZEC launchpad — locked for now)

| Layer | Choice | Why |
|-------|--------|-----|
| **Launchpad** | `landing/` — Vite + React + Tailwind | Already wired in `.github/workflows/pages.yml`; fast static deploy |
| **User docs** | `book/` (mdBook) at `/book/` | Long-form guides separate from marketing |
| **Wallet core** | Rust `nozy` + `zeaking` | Orchard prove/sync; never duplicate in JS |
| **Super wallet (future)** | Extension (MV3 + WASM) + `nozywallet-api` + **web app** SPA | Community-shaped: keys in extension/companion, not in a random website |
| **Not using** | Keplr SDK, Cosmos Kit, Next.js for launchpad | Wrong chain family; Vite is enough for static hub |

**ZEC only on the launchpad today.** Namada / Penumbra appear as “planned” badges until modules ship.

## Develop

```bash
cd landing
npm install
npm run dev
```

Build (CI uses this):

```bash
npm run build   # → landing/dist/
```

## Site sections

- **Hero** — privacy super wallet positioning, ZEC-first
- **Products** (`#products`) — Keplr-style surface cards (extension, web app, CLI, desktop, mobile, API)
- **Download** (`#download`) — production CLI binaries from GitHub Releases
- **Features / FAQ / About** — existing content

## Real wallet surfaces

| Surface | Status | Entry |
|---------|--------|--------|
| **CLI** | Mainnet | [Releases](https://github.com/LEONINE-DAO/Nozy-wallet/releases/latest) |
| **Extension** | Contributor preview | [`browser-extension/`](../browser-extension/README.md) |
| **Web app** | Coming soon | Extension + companion architecture |
| **Desktop** | In development | [`desktop-client/`](../desktop-client/README.md) |
| **Mobile** | In development | [`nozy-mobile/`](../nozy-mobile/README.md) |
| **Operator API** | In development | [`api-server/`](../api-server/README.md) |

See journal: [`docs/journal/research/2026-06-21-nozy-super-wallet-stack.md`](../docs/journal/research/2026-06-21-nozy-super-wallet-stack.md).  
Product roadmap: [`ENHANCEMENT_ROADMAP.md`](../ENHANCEMENT_ROADMAP.md) · Web app: [`web-app/README.md`](../web-app/README.md).
