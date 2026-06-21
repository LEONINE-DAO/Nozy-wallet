# NozyWallet Web App

**Status:** Starting — scaffold and companion integration (ZEC only).  
**Not the launchpad:** marketing lives in [`landing/`](../landing/). This folder is the **full wallet dashboard** in the browser.

---

## Role

Community-shaped **privacy super wallet** UI:

- Send, receive, sync **shielded ZEC** via **`nozywallet-api`** on localhost (or your VPS)
- Optional pairing with the **browser extension** for keys (same pattern as MetaMask extension + web dashboard, without EVM)
- **No seeds in static pages** — unlock flows go through the companion API and/or extension messaging

```text
web-app (browser tab)
    →  nozywallet-api  (http://127.0.0.1:3000)
    →  zebrad + lightwalletd
    ↔  browser-extension (optional, later)
```

---

## Stack (locked)

| Piece | Choice |
|-------|--------|
| UI | **Vite + React + TypeScript + Tailwind** (match `landing/` tokens) |
| Wallet logic | **Rust `nozy`** via **`api-server`** only — no duplicate crypto in JS |
| Deploy | GitHub Pages subpath `/Nozy-wallet/app/` or custom domain later |
| Auth | Companion password unlock; extension bridge in a later phase |

See [`docs/journal/research/2026-06-21-nozy-super-wallet-stack.md`](../docs/journal/research/2026-06-21-nozy-super-wallet-stack.md).

---

## Prerequisites

1. **Zebrad + lightwalletd** running (or remote Zebra URL in API config)
2. **`nozywallet-api`** — from repo root: `cargo run -p nozywallet-api` or [`scripts/run-nozy-api.ps1`](../scripts/run-nozy-api.ps1)
3. Wallet file created (CLI or API `POST /api/wallet/create`)

Companion routes: [`api-server/README.md`](../api-server/README.md)  
Extension companion pattern: [`browser-extension/COMPANION.md`](../browser-extension/COMPANION.md)

---

## Implementation phases

| Phase | Deliverable |
|-------|-------------|
| **W0** | Vite app boots; settings screen (API base URL); link from launchpad |
| **W1** | Unlock + balance + sync status from companion |
| **W2** | Send / receive ZEC |
| **W3** | Extension `postMessage` bridge (optional) |
| **W4** | CI build + Pages deploy under `/app/` |

Track progress in [`ENHANCEMENT_ROADMAP.md`](../ENHANCEMENT_ROADMAP.md).

---

## Local development (after W0 scaffold)

```bash
cd web-app
npm install
npm run dev
```

Default API URL: `http://127.0.0.1:3000` (override in app settings).

---

## Security

- Bind **`nozywallet-api`** to **localhost** unless you operate a hardened VPS ([`nozy-mobile/VPS-DEPLOY.md`](../nozy-mobile/VPS-DEPLOY.md)).
- Do not embed mnemonics in this repo or in client localStorage without an explicit threat-model review.
- Large behavior changes: GitHub issue + [`AGENTS.md`](../AGENTS.md).

---

## Related

- [Enhancement roadmap](../ENHANCEMENT_ROADMAP.md)
- [Multichain RFC](../docs/rfcs/MULTICHAIN_PRIVACY_CHAINS_RFC.md) (Namada / Penumbra later)
- [Landing / launchpad](../landing/README.md)
