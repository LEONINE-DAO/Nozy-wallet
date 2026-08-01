# Nozy Merchant — native payment service (plan)

**Status:** Planning — saved for later; do not start large slices without a GitHub issue ([`AGENTS.md`](../AGENTS.md)).  
**Decision:** **Native only** — extend `nozywallet-api` + Nozy clients. **No** CipherPay, ZECd, or PayPal on the primary merchant path.  
**Parent backlog:** [`BUSINESS_ZEC_ZNS_TODO.md`](BUSINESS_ZEC_ZNS_TODO.md) (Phase 3b) · [#85](https://github.com/LEONINE-DAO/Nozy-wallet/issues/85)  
**Deploy reference:** [`nozy-mobile/VPS-DEPLOY.md`](../nozy-mobile/VPS-DEPLOY.md)

---

## Goal

One **Nozy-owned** merchant stack: vendors accept shielded ZEC, customers pay to the merchant’s Business wallet, and **all payment logic lives in this repo** (`nozy` core + `api-server` + mobile/desktop/web surfaces).

Merchant **sales revenue** settles on-chain to the **merchant’s Orchard account** (Business profile → account index **1**). The API does not custody funds and does not take a cut unless operators add separate hosting/subscription products later.

---

## Architecture

```text
Customer (any ZEC wallet)
        │ shielded ZEC
        ▼
Merchant Business wallet (Orchard account 1)
        ▲
        │ sync, notes, send, invoices
nozywallet-api  ←  Nozy Merchant API (this plan)
        ▲
        │ HTTPS + API key
nozy-mobile Sell / desktop / web-app checkout
        │
Zebrad + lightwalletd  (operator infra — not a third-party payment processor)
```

**In scope:** wallet companion API extended with invoices, payment detection, ledger, optional webhooks.  
**Out of scope (primary path):** CipherPay, ZECd, PayPal, BTCPay plugins, custodial fiat settlement.

External research on CipherPay remains in [`journal/research/2026-06-21-cipherpay-nozywallet-integration.md`](journal/research/2026-06-21-cipherpay-nozywallet-integration.md) — **reference only**, not implementation.

---

## Product split

| Surface | Role |
|---------|------|
| **nozy-mobile Sell mode** | In-person POS — QR, waiting for payment, paid confirmation |
| **nozywallet-api** | Invoices, address derivation, payment matching, ledger, webhooks |
| **ZNS resolve** (Phase 2) | `yourname.zcash` on QR and Send |
| **ZIP-321** (Phase 4) | Scan-to-pay URIs with amount + memo |
| **Phase 5 ledger/CSV** | Accountant export; optional UFVK disclosure |
| **web-app / landing** (later) | Optional hosted checkout page for online invoices |

---

## Planned API surface (Phase 3b+)

Illustrative routes — names may change at implementation:

| Method | Route | Purpose |
|--------|-------|---------|
| `POST` | `/api/business/invoices` | Create sale — `invoice_id`, `payment_address`, `price_zec`, `zcash_uri`, `expires_at` |
| `GET` | `/api/business/invoices/:id` | Status: `open` \| `detected` \| `confirmed` \| `expired` \| `cancelled` |
| `GET` | `/api/business/invoices/:id/qr` | QR payload or PNG for Sell / checkout |
| `POST` | `/api/business/invoices/:id/cancel` | Cancel open invoice |
| `GET` | `/api/business/ledger` | Normalized rows (Phase 5; may ship with 3b) |
| `GET` | `/api/business/export.csv` | Accountant export (Phase 5) |
| `POST` | `/api/business/webhooks` | Register fulfillment URL + secret (v1.1) |

**Invoice create body (example):**

```json
{
  "amount_zec": 0.05,
  "amount_fiat": 29.99,
  "fiat_currency": "USD",
  "product_name": "Taco plate",
  "memo": "stall-1"
}
```

**Detection:** after sync, match incoming notes to open invoices by **payment address**, **amount** (± tolerance policy TBD), and optional **memo**. Idempotent on `txid` + note nullifier.

**Storage:** SQLite (or existing wallet-adjacent store) beside operator datadir — invoice metadata only; funds stay on-chain.

---

## Dependencies on earlier phases

| Phase | Required for native merchant |
|-------|------------------------------|
| **1** Business profile + account index 1 | Yes — separate business receive path |
| **2** ZNS resolve | Recommended — branded QR |
| **3** Sell mode (balance poll) | Yes — MVP stall demo |
| **3b** Invoices + matching | **This doc** — replaces external gateways |
| **4** ZIP-321 | Recommended — customer scan-to-pay |
| **5** Ledger / CSV / UFVK | Books + accountant |

**MVP without 3b:** Phase 3 Sell mode + single receive address + balance poll (food vendor demo).  
**Full native gateway:** Phase 3b + 4 + 5.

---

## Build order (when resumed)

1. **Phase 1–3** — Business profile, ZNS resolve, Sell mode (parent TODO).  
2. **Phase 3b** — Invoice store, `POST/GET` invoices, per-invoice diversified address from Business account, payment matcher after sync.  
3. **Wire Sell mode** — Create invoice via API instead of raw balance diff only.  
4. **Phase 4** — ZIP-321 on invoice create.  
5. **Phase 3b v1.1** — Webhooks (HMAC), optional hosted checkout in `web-app/`.  
6. **Phase 5** — Ledger + CSV + UFVK export.  
7. **Later** — E-commerce plugins (WooCommerce) calling **Nozy** API only.

**Stop and review** before webhooks in production or UFVK export (same gates as parent TODO).

---

## Security & ops

- **API key** on all public routes ([`VPS-DEPLOY.md`](../nozy-mobile/VPS-DEPLOY.md)).  
- **Future:** scoped keys — `receive-only` (invoices + status) vs `spend` (send routes).  
- **No seed or mnemonic in logs**; wallet encrypted at rest (existing patterns).  
- **Watch-only terminal** (Phase 7) — receive-only device; spend on separate hardware (future).  
- **Webhooks:** HMAC signature + replay protection when implemented.

---

## Revenue model (clarification)

| Money | Recipient |
|-------|-----------|
| Customer pays for goods (ZEC) | **Merchant** Business wallet |
| ZIP-317 network fee | Miners |
| Optional future “Nozy hosting / subscription” | Operator product — **not** implemented in merchant API today |

There is **no** built-in take-rate on ZEC sales in this plan.

---

## Explicitly not doing

- PayPal / card fiat checkout as primary rail  
- CipherPay / ZECd integration as dependency  
- Custodial pooling of merchant ZEC  
- Apple Pay–style instant fiat settlement (on-chain latency remains)

Phase 8 **“Add ZEC”** (customer on-ramp partner) may return as an optional **acquire ZEC** link — separate from merchant checkout.

---

## Success criteria (native gateway v1)

- [ ] Merchant creates invoice via `nozywallet-api`; gets unique shielded address + ZIP-321 URI.  
- [ ] Customer pays from Nozy or compatible wallet; invoice moves to `confirmed` after policy confirmations.  
- [ ] Sell mode shows invoice QR and paid state without third-party services.  
- [ ] Ledger totals match on-wallet history.  
- [ ] Documented VPS deploy: Zebrad + lightwalletd + `nozywallet-api` only.

---

## When picking this up again

1. Confirm [#85](https://github.com/LEONINE-DAO/Nozy-wallet/issues/85) or file a child issue: **“Phase 3b — Native Nozy Merchant API”**.  
2. Check Phase 1–3 checkboxes in [`BUSINESS_ZEC_ZNS_TODO.md`](BUSINESS_ZEC_ZNS_TODO.md).  
3. Implement invoice store + routes in `api-server/`; reuse `nozy` address generation and note index.  
4. Update [`CHANGELOG.md`](../CHANGELOG.md) and [`docs/issues/FEATURE_REGISTRY.md`](issues/FEATURE_REGISTRY.md) on ship.
