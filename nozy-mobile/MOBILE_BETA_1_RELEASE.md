# Mobile v1.0.0-beta.1 — companion beta

**Tag:** `mobile-v1.0.0-beta.1`  
**Date:** 2026-08-02  
**Surface:** Expo / React Native companion (`nozy-mobile`)

---

## Status

First tagged GitHub Release for the mobile companion. **Pre-release.** Store builds use **EAS** (`eas build --platform android --profile production`); there is no GitHub Actions APK attach yet. See `nozy-mobile/STORE-CHECKLIST.md` and `ANDROID-EAS-SMOKE.md`.

## What's in this tag

- Quiet Sapling legacy via companion API (status / scan / shield) wired with extension/desktop work
- brace-expansion Dependabot bump
- Version set to **1.0.0-beta.1** for tagged companion beta

## Security note

The August 2026 AI-assisted self-review focused on **CLI + shared core + Desktop**. Mobile FFI / Expo UI were **out of scope** for that pass. Do not claim mobile was covered by that review. Audit pack: [`docs/reference/security-audit/`](https://github.com/LEONINE-DAO/Nozy-wallet/tree/master/docs/reference/security-audit).

## Requirements

- Companion **`nozywallet-api`** reachable (emulator → `10.0.2.2:3000`, or hosted URL)
- Zebrad + lightwalletd behind the API for shielded flows

## AI disclosure

Release packaging was **agent-assisted** (Cursor). Human author remains responsible for correctness and security.
