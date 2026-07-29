# Launch page (NozyWallet launchpad)

Public **marketing / product hub** at GitHub Pages root (`https://leonine-dao.github.io/Nozy-wallet/`). This is **not** the wallet app.

## Stack (ZEC launchpad — locked for now)

| Layer | Choice | Why |
|-------|--------|-----|
| **Launchpad** | `landing/` — Vite + React + Tailwind | Already wired in `.github/workflows/pages.yml`; fast static deploy |
| **User docs** | `book/` (mdBook) at `/book/` | Long-form guides separate from marketing |
| **Wallet core** | Rust `nozy` + `zeaking` | Orchard + Ironwood prove/sync; never duplicate in JS |
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

Vite `base` defaults to `/Nozy-wallet/` for GitHub Pages. On Vercel, `VERCEL=1` forces `base: "/"`. Override with `VITE_BASE=/` if needed.

## Vercel (Ironwood dashboard + live stats)

Deploy this folder as a Vercel project (Root Directory: `landing`).

1. Import the repo in Vercel → set **Root Directory** to `landing`.
2. Add env var **`ZEBRA_RPC_URL`** (server-only) to a reachable mainnet Zebrad JSON-RPC URL, e.g. `http://host:8232`.
3. Deploy. Open `/ironwood` for the Nozy Ironwood migration dashboard.
4. `/api/ironwood-stats` proxies `getblockcount` + `getblockchaininfo` (`valuePools` orchard/ironwood) with a short cache TTL. No wallet secrets.

Without `ZEBRA_RPC_URL`, the page still renders activation constants, ZIP 318 education, and Nozy migrate steps; live pool % shows offline with links to ZODL / CipherScan.

Local Vite: set `ZEBRA_RPC_URL` and run `npm run dev` — `/api/ironwood-stats` is served by a Vite middleware (same shape as the Vercel function). Without it, the dashboard still loads with offline pool copy.

## Site sections

- **Hero** — Orchard + Ironwood shielded ZEC positioning
- **Ironwood** (`/ironwood`) — network migration dashboard + Nozy ZIP 318 operator path
- **Products** (`#products`) — surface cards (extension, web app, CLI, desktop, mobile, API)
- **Download** (`#download`) — production CLI binaries from GitHub Releases
- **Features / FAQ / About** — privacy, Ironwood FAQ, CTA

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
