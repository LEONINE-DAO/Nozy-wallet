# Business & POS (ZEC-first) + ZNS — implementation TODO

**Status:** Planning — ordered backlog; do not start large slices without a GitHub issue ([`AGENTS.md`](../AGENTS.md)).  
**Goal:** One wallet for **personal use** and **business owners** (solo vendor → company), shielded Orchard ZEC, optional **Zcash Names (ZNS)**, mobile **Sell mode** for QR/tap-style pay.  
**Out of scope (primary path):** Secret / SILK / Fina stack — see [`PRIVACY_DEFI_IMPLEMENTATION_TODO.md`](PRIVACY_DEFI_IMPLEMENTATION_TODO.md) for optional Secret work only.

**Related:** [ZcashNames docs](https://www.zcashnames.com/docs), [`zcashname-sdk`](https://www.npmjs.com/package/zcashname-sdk), [`nozy-mobile/`](../nozy-mobile/), [`api-server/`](../api-server/).  
**Native merchant service (canonical):** [`NOZY_MERCHANT_NATIVE.md`](NOZY_MERCHANT_NATIVE.md) — extend `nozywallet-api` only; no CipherPay / ZECd / PayPal on the primary path.  
**External research (reference only):** [`journal/research/2026-06-21-cipherpay-nozywallet-integration.md`](journal/research/2026-06-21-cipherpay-nozywallet-integration.md).

---

## How to use this doc

- Work **top to bottom** within each phase; later phases depend on earlier gates.
- Check boxes when done; link PRs in parentheses if helpful.
- **MVP for food vendor demo:** Phase 0 + Phase 1 + Phase 2 (resolve + Sell QR + stable sync/send).

---

## Phase 0 — Preconditions

- [x] **GitHub issue** — [#85](https://github.com/LEONINE-DAO/Nozy-wallet/issues/85) filed; draft: [`GITHUB_ISSUE_DRAFT_BUSINESS_ZNS.md`](GITHUB_ISSUE_DRAFT_BUSINESS_ZNS.md).
- [x] **Product decisions** — Locked in [`BUSINESS_ZEC_ZNS_PHASE0_DECISIONS.md`](BUSINESS_ZEC_ZNS_PHASE0_DECISIONS.md).
  - [x] Personal vs Business at **create wallet** (same seed; Business → Orchard account index **1**, Personal → **0**).
  - [x] **Hosted API** path — canonical: [`nozy-mobile/VPS-DEPLOY.md`](../nozy-mobile/VPS-DEPLOY.md).
  - [x] ZNS indexer URLs — mainnet beta `https://light.zcash.me/zns-mainnet-test`, testnet `https://light.zcash.me/zns-testnet`; env `ZNS_MAINNET_URL` / `ZNS_TESTNET_URL`.
- [x] **Success criteria** — Locked in Phase 0 decisions §5 (MVP / v1 / v2).

---

## Phase 1 — Wallet foundation (Personal / Business profiles)

No ZNS registration yet; stable ZEC core + profile flag.

### Config & setup

- [ ] **`WalletProfile` enum** — `Personal` | `Business` in config / wallet metadata (persisted).
- [ ] **Create-wallet flow** — Ask profile; Business: optional display name, default Orchard account index (e.g. 1).
- [ ] **Settings** — Switch profile later; show active receive account / UA.
- [ ] **Copy** — One seed; Business gets ledger/export/disclosure defaults; Personal keeps simple UX.

### Surfaces

- [ ] **Desktop (Tauri)** — Create wallet + Settings profile.
- [ ] **Mobile** — Create wallet + Settings ([`nozy-mobile/`](../nozy-mobile/)).
- [ ] **api-server** — Expose profile in config/status if needed for mobile.

### Acceptance

- [ ] Personal and Business can send/receive/sync on same build.
- [ ] Business account index does not break existing single-account wallets (migration: default Personal, account 0).

---

## Phase 2 — ZNS resolve (read-only) — highest leverage

Identity layer for pay UX; no on-chain claim from wallet yet.

### api-server

- [x] **Config** — defaults + env `ZNS_MAINNET_URL` / `ZNS_TESTNET_URL`.
- [x] **`POST /api/zns/resolve`** — Returns `{ name, found, registration? }`; proxy JSON-RPC `resolve`.
- [x] **Verification** — Indexer URL allowlist (`nozy::zns::verify_indexer_url`); reject unknown hosts unless `ZNS_ALLOW_UNTRUSTED_INDEXER=1`.
- [x] **Cache** — Short TTL (60s) in-memory cache on `POST /api/zns/resolve`.

### Send path (all clients)

- [x] **Recipient parsing** — Accept `u1…`, `name`, or `name.zcash` / `name.zec`; resolve name before build/send.
- [x] **Desktop + mobile Send** — Wire to resolve (desktop direct indexer + CSP; mobile/extension via companion).
- [x] **CLI** — `nozy send --to <name>` resolves via indexer before validation.
### Receive / reverse lookup (optional in Phase 2)

- [x] **`GET /api/zns/reverse?address=`** — Show linked `.zcash` name on Dashboard if registered.

### Acceptance

- [ ] Send to a known testnet/mainnet beta name succeeds end-to-end.
- [ ] Invalid name fails with user-readable message (no opaque 500).

---

## Phase 3 — Mobile Sell mode (POS)

Target: food stall, market vendor — phone as register.

### UI (`nozy-mobile`)

- [x] **Mode toggle** — Dashboard: **Use** | **Sell** (`Sell mode` button → Sell screen).
- [x] **Sell screen** — Large amount field (optional fixed ZEC / fiat display only); **Receive** identity.
- [x] **QR display** — ZIP-321 `zcash:` URI payload (copy); on-screen QR widget still open.
- [x] **Copy address / name** — One tap for customers without QR scanner.
- [x] **“Waiting for payment…”** — Poll `wallet/status` + balance or incremental sync after sale.
- [ ] **Paid confirmation** — Haptic + “Received X ZEC” when note detected (sync). (text confirmation shipped; haptic open)

### api-server / ops

- [x] **Hosted vendor guide** — [`VENDOR_VPS_SELL_GUIDE.md`](VENDOR_VPS_SELL_GUIDE.md).
- [ ] **Background sync** — Document or automate post-receive sync trigger from mobile (optional endpoint).

### Acceptance

- [ ] Vendor can run Sell mode on emulator/device against local or hosted API.
- [ ] Customer pays to QR; vendor sees updated balance within documented sync time.

---

## Phase 3b — Native Nozy Merchant API (invoices & detection)

**Status:** Planned — saved for later. Full spec: [`NOZY_MERCHANT_NATIVE.md`](NOZY_MERCHANT_NATIVE.md).  
**Goal:** Replace external gateways (CipherPay, ZECd, PayPal) by extending **`nozywallet-api`** with invoice + payment-matching — funds still go directly to the merchant Business wallet.

**Depends on:** Phase 1 (Business account 1) + Phase 3 Sell mode (or parallel after Phase 1).

### api-server

- [x] **Invoice store** — JSON beside wallet datadir (`merchant_invoices.json`).
- [x] **`POST /api/business/invoices`** — Amount (ZEC and/or fiat display), optional product name/memo; returns `invoice_id`, `payment_address`, `price_zec`, `zcash_uri`, `expires_at`.
- [x] **Per-invoice address** — Diversified Orchard UA from Business account index **1**; persist diversifier ↔ invoice mapping.
- [x] **`GET /api/business/invoices/:id`** — Status: `open` | `detected` | `confirmed` | `expired` | `cancelled`.
- [x] **`POST /api/business/invoices/:id/cancel`** — Cancel open invoice.
- [ ] **Payment matcher** — After sync, match notes to open invoices (address + amount ± policy; optional memo); idempotent on txid/nullifier. (helper `match_incoming_payment` exists; wire to sync path open)
- [x] **`GET /api/business/invoices/:id/qr`** — QR / URI payload for Sell mode and checkout (ZIP-321).
- [ ] **Trigger sync helper** — Optional endpoint or documented poll pattern for post-sale sync (extends Phase 3 ops item).

### Clients

- [ ] **Sell mode** — Create invoice via API (not only raw balance diff); show invoice QR and paid state from invoice status.
- [ ] **Desktop / web-app** (later) — Invoice list + optional hosted checkout page.

### v1.1 (after core invoices)

- [ ] **Webhooks** — HMAC-signed POST to merchant URL on `confirmed`.
- [ ] **Scoped API keys** — Receive-only vs spend (security hardening).

### Acceptance

- [ ] End-to-end: create invoice → customer pays shielded ZEC → invoice `confirmed` without third-party payment service.
- [ ] Sell mode uses native invoice API on local or VPS-hosted `nozywallet-api`.
- [ ] No CipherPay / ZECd / PayPal dependency in the primary flow.

---

## Phase 4 — ZIP-321 payment URIs

Standard deep links for scan-to-pay (customer and vendor).

- [x] **Generate URI** — Amount + address + memo helpers in `nozy::zip321`; Sell mode builds URI; invoice `zcash_uri`.
- [ ] **Scan to pay** — Mobile camera / QR scanner on Send; parse URI → resolve name if present → send flow.
- [x] **Memo field** — Business: optional invoice id in shielded memo (within size limits) via invoice create.

### Acceptance

- [ ] Vendor QR scanned in Nozy (or compatible wallet) pre-fills send amount and recipient.

---

## Phase 5 — Business ledger & accountant (viewing keys)

Forum “boring middle layer” — books, not payment identity.

### Ledger API

- [ ] **`GET /api/business/ledger`** — Normalized rows from merged history (sent/received); filter by date, profile account.
- [ ] **`GET /api/business/export.csv`** — Accountant-friendly columns (date, type, amount ZEC, txid, memo, block height).
- [ ] **Business-only gates** — Export enabled for Business profile (or explicit opt-in for Personal).

### Selective disclosure (ZEC UFVK)

- [ ] **`nozy disclosure export`** / API — Orchard UFVK or scoped IVK + height range; **never** spending key.
- [ ] **Disclosure grant log** — Append-only: grant id, scope, created, expiry, access events.
- [ ] **Settings UX** — “Share with accountant” (export file or read-only instructions).

### Acceptance

- [ ] CSV opens in Excel/Sheets; matches on-wallet history totals.
- [ ] Exported UFVK package documented for third-party scan tools (no spend).

---

## Phase 6 — ZNS claim & update (in-wallet)

After resolve is stable; optional for MVP.

- [ ] **Integrate `zcashname-sdk`** — `prepareClaim`, `prepareUpdate` in api-server or desktop helper.
- [ ] **ZNS Ed25519 key** — Derive/store per protocol (SLIP-0010 path from seed); document backup; separate from spend key.
- [ ] **Claim flow** — Business onboarding: link to zcashnames.com or in-app claim → Orchard tx with ZNS memo via existing send pipeline.
- [ ] **Update flow** — Point name at new UA when business rotates receive account.
- [ ] **180-day reminder** — Notify user to keep name active (protocol requirement).

### Acceptance

- [ ] Claim/update on testnet (then mainnet beta) with maintainer sign-off.

---

## Phase 7 — Tap & scale (optional)

- [ ] **NFC (Android)** — NDEF payload with ZIP-321 or `.zcash` name (opens wallet / pre-fills pay).
- [ ] **Receive-only terminal profile** — Big business: stall phone read-only, spend on separate device (future).
- [ ] **Multi-label ledger** — Stalls / cost centers as memo prefixes or tags.

---

## Phase 8 — Bridge-in & growth

- [ ] **“Add ZEC” entry** — Dashboard CTA to bridge/swap **into shielded Orchard** (partner TBD).
- [ ] **Cross-pay** — If ZcashNames cross-pay fits product, link from Receive (names only; no Secret-native stack in Nozy).

---

## Phase 9 — Docs, release, security

- [ ] **User guide** — Personal vs Business; Sell mode; ZNS names; hosted API for vendors.
- [ ] **Book / README** — Update unified wallet section: ZEC-first business path (not Secret DeFi).
- [ ] **Security review** — ZNS indexer trust, disclosure exports, API key on hosted VPS; no seed in logs.
- [ ] **CHANGELOG** — Feature flags and build notes.

---

## Explicitly deferred (track separately)

- [ ] Secret / SILK / Fina / Shade Quick Actions as **primary** business story ([`PRIVACY_DEFI_IMPLEMENTATION_TODO.md`](PRIVACY_DEFI_IMPLEMENTATION_TODO.md)).
- [ ] Apple Pay–style instant fiat settlement (on-chain ZEC remains block-time bound).
- [ ] Full multi-user org custody (shared spend without shared seed) — needs own RFC.

---

## Suggested first sprint (when approved)

1. Phase 0 issue + Phase 1 profile flag (config + create flow, one surface).
2. Phase 2 `GET /api/zns/resolve` + Send accepts names.
3. Phase 3 mobile Sell mode + QR (name or UA).

**Stop and review** before Phase 5 disclosure or Phase 6 in-wallet ZNS claim.

---

## Quick reference — what blocks what?

| Work | Blocks |
|------|--------|
| Phase 1 profiles | Business defaults for ledger |
| Phase 2 ZNS resolve | Sell QR with `.zcash` name; Send by name |
| Phase 3 Sell mode | Food vendor demo |
| Phase 3b Native Merchant API | Invoices + detection; no external gateway |
| Phase 4 ZIP-321 | Scan-to-pay from customer wallet |
| Phase 5 ledger / UFVK | Accountant / enterprise books |
| Phase 6 ZNS claim | In-app name registration |
| Phase 4+ send | Stable sync/send (recent fixes) assumed |
