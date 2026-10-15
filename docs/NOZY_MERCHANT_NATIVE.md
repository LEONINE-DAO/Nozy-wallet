# Nozy Merchant — native + ZekuraPay

**Status:** Active — local POS in Nozy; hosted multi-merchant gateway in [ZekuraPay](https://github.com/Lowo88/ZekuraPay) (Anypay-inspired fork).  
**Infra lock (2026-09-14):** Public app on `lwd.nozywallet.org` is shielded invoices. Detect on this node. Settle to **the merchant’s** Nozy UA. **Do not** settle customer money into `api.nozywallet.org`.  
**Parent backlog:** [`BUSINESS_ZEC_ZNS_TODO.md`](BUSINESS_ZEC_ZNS_TODO.md) (Phase 3b) · [#85](https://github.com/LEONINE-DAO/Nozy-wallet/issues/85)  
**Deploy reference:** [`nozy-mobile/VPS-DEPLOY.md`](../nozy-mobile/VPS-DEPLOY.md)

---

## Goal

Vendors accept **shielded ZEC** (Ironwood settle pool post-NU6.3). Customers pay from Nozy (or any Ironwood-capable wallet). **Transparent T-address ZEC is not supported.**

| Path | Where | When to use |
|------|--------|-------------|
| **Local POS** | This repo — `merchant_invoices` + `/api/business/invoices` | Single operator, offline-friendly, no Postgres |
| **Hosted gateway** | [ZekuraPay](https://github.com/Lowo88/ZekuraPay) | Multi-merchant, webhooks, VPS Zakura |

Funds settle on-chain to the merchant’s shielded UA. ZekuraPay holds **UFVK** for detection only (non-custodial). Spending stays in Nozy.

---

## Architecture

```text
Customer (NozyWallet)  --ZIP-321 shielded-->  Merchant UA (Ironwood note)
                                                    ^
                         match / webhook             |
ZekuraPay             --UFVK watcher--  this node (Zebrad localhost RPC
                                         and/or lwd.nozywallet.org)
nozywallet-api        --local POS only--  the operator's own till
                                         (never the public settle wallet)
```

**Out of scope (primary path):** CipherPay, ZECd, PayPal, BTCPay plugins, custodial fiat, transparent ZEC.

---

## Local API (still in Nozy)

| Method | Route | Purpose |
|--------|-------|---------|
| `POST` | `/api/business/invoices` | Create sale — diversified Business UA |
| `GET` | `/api/business/invoices/:id` | Status |
| `GET` | `/api/business/invoices/:id/qr` | ZIP-321 payload |
| `POST` | `/api/business/invoices/:id/cancel` | Cancel |

After sync, `match_notes_after_sync` marks matching open invoices **confirmed**.

## Desktop

- **Send:** paste `zcash:…` ZIP-321 URI → address / amount / memo (rejects `t1`/`t3`)
- **Business panel:** optional ZekuraPay API URL → create hosted invoice QR

## Zakura

Prefer Zakura on the operator VPS (or merchant self-host). Same JSON-RPC surface as Zebrad; see [`docs/reference/ZAKURA_NOZYWALLET_CONNECTIVITY.md`](reference/ZAKURA_NOZYWALLET_CONNECTIVITY.md).
