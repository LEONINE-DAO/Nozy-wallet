# Mobile store release checklist

**Status:** Android listing copy matches the 1.0.0 binary (keys on phone, receive + sync). Still open: Play credentials, production AAB on a physical phone, and iOS `libnozy_ffi` (Mac only).  
**Bundle ID:** `com.leoninedao.nozywallet`  
**Store strategy (v1):** Light wallet like Zodl — keys on the phone, compact sync to `https://lwd.nozywallet.org:443`. UniFFI Expo module is wired; rebuild `libnozy_ffi` + Kotlin bindings before a store binary.

**Do not** ship `https://api.nozywallet.org` or `NOZY_API_KEY` as a public default. That box is one wallet (LEONINE operator). Mass users must not share it.

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
- [x] Release AndroidManifest: targetSdk 36, no cleartext, `allowBackup=false`, no storage / overlay perms
- [x] Production: default LWD → `https://lwd.nozywallet.org:443` (keys on phone)
- [x] Seed / session password / API key in `expo-secure-store` (migrates leftovers out of AsyncStorage)
- [x] Store version `1.0.0` (app.json / package.json / APP_VERSION)
- [x] Emulator smoke (NozyPixel): create wallet → compact sync to lwd.nozywallet.org → Receive UA (`scripts/smoke-ui.ps1`)
- [ ] Production: on-device create/restore/sync smoke on a physical device
- [x] iOS: `ITSAppUsesNonExemptEncryption: false`
- [x] `eas.json` production env + submit placeholders
- [ ] Replace `eas.json` submit placeholders with real Apple / Play credentials
- [x] On-device Send screen tells the truth: this version receives; Orchard spend proving is the next release (no lightwalletd-only prover in `nozy-ffi`)

---

## Phase 2 — Operator VPS (not a store blocker)

The store build does not use `api.nozywallet.org` or `nozywallet.leoninedao.org`. Those hosts are one operator wallet. Leave them unchecked until a separate companion product exists.

- [ ] Production API live at `https://nozywallet.leoninedao.org` ([`VPS-DEPLOY.md`](VPS-DEPLOY.md))
- [ ] API key enforced on hosted VPS (`NOZY_API_KEY` / middleware)
- [ ] TLS certificate valid; `/health` returns 200 from public internet
- [ ] Zebrad backend stable for hosted stack

---

## Phase 3 — Store assets & policy

- [x] Privacy policy URL live and linked ([`src/constants/links.ts`](src/constants/links.ts)) — `https://leonine-dao.github.io/Nozy-wallet/privacy/`
- [x] Icons and splash are 1024×1024 (`assets/icon.png`, `adaptive-icon.png`, `splash-icon.png`)
- [x] Screenshots captured at 1080×2160 in `store-assets/android/` (welcome, home, receive with address covered, send, settings, about). Upload these in Play Console.
- [x] Short + full description ([`STORE-LISTING.md`](STORE-LISTING.md)) — keys on phone, receive + sync, spend is the next release
- [ ] Play Data safety + Apple privacy nutrition labels filled from listing doc
- [ ] No test mnemonics or secrets in repo

---

## Phase 4 — QA before submit

- [ ] EAS production build: `eas build --platform android --profile production`
- [ ] Install AAB/APK on physical device (not emulator-only)
- [ ] End-to-end: create/restore on device → compact sync via lwd.nozywallet.org → receive address (send proving is next slice)
- [ ] Verify experimental settings **not** visible in production build
- [ ] Verify cleartext `http://` blocked (hosted HTTPS works)

---

## Build commands

```bash
npm install -g eas-cli
eas login

# Production store candidate (Android). iOS needs a Mac build of libnozy_ffi first.
eas build --platform android --profile production

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
