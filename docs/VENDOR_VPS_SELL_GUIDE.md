# Vendor VPS guide — Sell mode (companion API)

**Audience:** Solo vendor running Nozy mobile Sell mode against a hosted `nozywallet-api`.  
**Canonical deploy:** [`../nozy-mobile/VPS-DEPLOY.md`](../nozy-mobile/VPS-DEPLOY.md)

## What you need

1. A VPS with HTTPS reverse proxy to `nozywallet-api` (TLS required for Play / production mobile).
2. `NOZY_API_KEY` set on the server; the same key entered in the mobile app.
3. Zebrad + lightwalletd reachable **from the VPS** (not from the phone).
4. Business profile in the wallet (Orchard account **1**); optional ZNS name claimed on [zcashnames.com](https://www.zcashnames.com) pointing at that Business UA, then linked in-app.

## Phone does not run Zebrad

The Expo app is companion-only in production: it talks to your HTTPS API. Sync and send proofs run on the VPS wallet process.

## Sell flow (MVP)

1. Dashboard → **Sell mode**.
2. Switch to Business; link your name if claimed.
3. Enter amount → copy **ZIP-321 URI** (or generate invoice via `POST /api/business/invoices`).
4. Show URI/QR to customer; tap **Waiting for payment…** to poll balance after sync.
5. Optional: create invoices for per-sale diversified addresses (Phase 3b API).

## Security minimums

- Never expose the API without an API key on the public internet.
- Prefer localhost-only bind behind the reverse proxy (`NOZY_BIND_ADDR=127.0.0.1`).
- Do not put seed phrases in chat logs or screenshots for support.
