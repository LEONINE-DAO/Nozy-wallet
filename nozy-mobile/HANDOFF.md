# NozyWallet Mobile — contributor handoff

Read this before changing `nozy-mobile/` or the hosted API story.

---

## What it is

Expo / React Native **companion** — UI on phone, wallet logic on **`nozywallet-api`** (PC or VPS). Not a standalone on-device full node wallet yet.

---

## Read order

1. [`README.md`](README.md) — quick start, emulator API URL (`http://10.0.2.2:3000`)
2. [`VPS-DEPLOY.md`](VPS-DEPLOY.md) — public HTTPS API for mobile data
3. [`STORE-CHECKLIST.md`](STORE-CHECKLIST.md) — App Store / Play when ready
4. [`../ENHANCEMENT_ROADMAP.md`](../ENHANCEMENT_ROADMAP.md) — mobile vs web-app priority
5. [`../api-server/README.md`](../api-server/README.md) — HTTP routes the app calls

---

## Key files

| File | Role |
|------|------|
| `src/services/api.ts` | All companion HTTP calls |
| `src/context/WalletSessionContext.tsx` | API URL, API key, password session |
| `src/screens/` | UI flows (Welcome, Dashboard, Send, …) |
| `App.tsx` | Entry + navigation |

---

## Local dev checklist

- [ ] `nozywallet-api` listening (e.g. port 3000)
- [ ] Zebrad + lightwalletd reachable from API config
- [ ] Emulator uses `10.0.2.2` for host localhost
- [ ] `npm run typecheck` clean before PR

---

## Out of scope (mobile v1)

- In-app Zebrad / lightwalletd
- Namada / Penumbra native SDK in the bundle (see multichain RFC — companion-only)

---

*Update when mobile milestones shift on the enhancement roadmap.*
