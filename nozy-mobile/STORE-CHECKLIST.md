# Mobile store release checklist

**Status:** In progress — production build profile wired; infra + listing still open  
**Bundle ID:** `com.leoninedao.nozywallet`  
**Store strategy (v1):** **Companion-only** — hosted HTTPS API; experimental FFI hidden in production builds

---

## Phase 0 — Before VPS payment (no monthly cost)

- [x] Production build profile + experimental UI gated ([`app.config.js`](app.config.js))
- [x] API key generator + production QA scripts ([`BEFORE-VPS-PAYMENT.md`](BEFORE-VPS-PAYMENT.md))
- [x] VPS one-shot deploy script ([`../scripts/deploy-mobile-api-vps.sh`](../scripts/deploy-mobile-api-vps.sh))
- [x] HTTPS tunnel QA path ([`HTTPS-TUNNEL-QA.md`](HTTPS-TUNNEL-QA.md))
- [ ] Run `prepare-mobile-store-credentials.ps1` → save key offline
- [ ] Optional: tunnel QA on physical device with production build
- [ ] EAS production build on physical Android
- [x] Privacy policy page ([`landing/src/pages/Privacy.tsx`](../../landing/src/pages/Privacy.tsx)) — deploy landing to GitHub Pages

---

## Phase 1 — App config (in repo)

- [x] `app.config.js` — production vs development (`EXPO_PUBLIC_APP_VARIANT` / EAS profile)
- [x] Production: `usesCleartextTraffic: false` (HTTPS only for Play review)
- [x] Production: default API URL → hosted preset
- [x] Production: hide on-device wallet + light client settings
- [x] Production: require API key when using hosted URL
- [x] iOS: `ITSAppUsesNonExemptEncryption: false`
- [x] `eas.json` production env + submit placeholders
- [ ] Replace `eas.json` submit placeholders with real Apple / Play credentials

---

## Phase 2 — Infrastructure (operator)

- [ ] Production API live at `https://nozywallet.leoninedao.org` ([`VPS-DEPLOY.md`](VPS-DEPLOY.md))
- [ ] API key enforced on hosted VPS (`NOZY_API_KEY` / middleware)
- [ ] TLS certificate valid; `/health` returns 200 from public internet
- [ ] Zebrad backend stable for hosted stack

---

## Phase 3 — Store assets & policy

- [ ] Privacy policy URL live and linked ([`src/constants/links.ts`](src/constants/links.ts)) — `https://leonine-dao.github.io/Nozy-wallet/privacy/`
- [ ] Icons and splash finalized (`assets/`)
- [ ] Screenshots: Welcome, Dashboard, Send, Receive, Settings
- [ ] Short + full description ([`STORE-LISTING.md`](STORE-LISTING.md))
- [ ] Play Data safety + Apple privacy nutrition labels filled from listing doc
- [ ] No test mnemonics or secrets in repo

---

## Phase 4 — QA before submit

- [ ] EAS production build: `eas build --platform android --profile production`
- [ ] Install AAB/APK on physical device (not emulator-only)
- [ ] End-to-end: connect hosted API + key → create/restore → sync → send (testnet or small mainnet)
- [ ] Verify experimental settings **not** visible in production build
- [ ] Verify cleartext `http://` blocked (hosted HTTPS works)

---

## Build commands

```bash
npm install -g eas-cli
eas login

# Production store candidate (companion-only UI)
eas build --platform android --profile production
eas build --platform ios --profile production

# Internal QA (dev features + cleartext for emulator)
eas build --platform android --profile preview
```

**Local production preview** (UI only, no EAS):

```powershell
$env:EXPO_PUBLIC_APP_VARIANT = "production"
cd nozy-mobile
npx expo start
```

---

## After launch

- [ ] Update [`landing/`](../landing/) mobile card from “soon” to store links
- [ ] Note release in [`CHANGELOG.md`](../CHANGELOG.md)

See also [`ENHANCEMENT_ROADMAP.md`](../ENHANCEMENT_ROADMAP.md) · [`docs/reference/NOZY_MOBILE_CASE_BREAKDOWN.md`](../docs/reference/NOZY_MOBILE_CASE_BREAKDOWN.md)
