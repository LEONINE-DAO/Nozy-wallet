# Android EAS production smoke (grant P0)

**Scope:** Android-first companion build against hosted or tunnel HTTPS API.  
**Related:** [`STORE-CHECKLIST.md`](STORE-CHECKLIST.md) · [`eas.json`](eas.json) · grant gate [`GRANT_80K_PRODUCTION_CHECKLIST.md`](../GRANT_80K_PRODUCTION_CHECKLIST.md)

## Prerequisites

- Expo account with access to project `nozy-wallet` (EAS project id in `app.json` / `app.config.js`).
- Production or preview profile API key + HTTPS base URL (`EXPO_PUBLIC_*` / EAS env).
- Physical Android device or emulator with Google Play services.

## Build

```bash
cd nozy-mobile
npm ci
npx eas build --platform android --profile production
```

Or preview (internal):

```bash
npx eas build --platform android --profile preview
```

Install the artifact from the Expo dashboard (APK/AAB) onto the device.

## Smoke script (manual, ~15 min)

| # | Step | Pass? |
|---|------|-------|
| 1 | App launches; no cleartext HTTP to public hosts in production | ☐ |
| 2 | Settings: API URL is HTTPS; API key accepted | ☐ |
| 3 | Unlock / create wallet against companion | ☐ |
| 4 | Sync completes or shows honest progress (not stuck 0%) | ☐ |
| 5 | Receive shows Orchard UA; copy works | ☐ |
| 6 | Send to a known UA (small testnet or mainnet dust) or dry-run fee estimate succeeds | ☐ |
| 7 | Dashboard **Sell mode** opens; Business switch + ZIP-321 URI copy works | ☐ |
| 8 | Optional: resolve a ZNS name on Send (`zenith` or your name) via companion | ☐ |
| 9 | App survives background → foreground without corrupt session | ☐ |
| 10 | No seed / mnemonic in logcat (`adb logcat` spot-check) | ☐ |

## Record for grant pack

- EAS build URL: ________________________________
- Device model / Android version: ________________
- Date + operator: ______________________________
- Notes / failures: _____________________________

## Out of scope for this smoke

- Play Console listing approval  
- iOS TestFlight  
- On-device Zebrad  
- Chrome Web Store (extension track)
