# Browser Extension Changelog

All notable changes to the Nozy browser extension are tracked here.

## 0.1.4 — 2026-03-20

### Added
- Popup **Companion** tab: connect to local **Nozy API** (`nozywallet-api`), health check, lightwalletd info/chain tip, and **compact sync** trigger (same HTTP surface as Tauri `lwd_*` commands).
- `chrome.storage` prefs for companion base URL and optional lightwalletd override.
- **`browser-extension/README.md`**: step-by-step install, architecture (Desktop vs extension), screenshot placeholders under `docs/screenshots/`.
- **GitHub Actions**: `extension-release` workflow builds **WASM** with **wasm-pack** (fixes missing `nozy_wasm_bg.wasm` in zips), corrects bundle layout to match **`wasm-core/popup/dist`** in `manifest.json`, and **attaches zips to every published GitHub Release** as well as manual `extension-v*` workflow runs.

### Changed
- **`minimum_chrome_version`**: 114; manifest description clarifies Desktop as primary full wallet, extension as WASM + optional API/lightwalletd path.

## 0.1.3 — 2026-03-23

### Added
- **Google Chrome** and **Microsoft Edge** (Chromium, MV3): manifest description and `host_permissions` for localhost **Nozy API** (`http://127.0.0.1:3000/*`) plus companion fetch patterns.
- **Companion API** (`companion-api.js`): background handlers for `companion_status`, `companion_lwd_*` calling desktop **`nozywallet-api`** / zeaking LWD routes.
- Docs: **`COMPANION.md`**, **`LOCAL_RPC.md`** (load unpacked, companion URL, Zebrad/WSL notes).

### Changed
- Popup / extension API wiring for companion base URL and LWD sync UX.

## Unreleased

### Added
- Mobile sync protocol state migration and replay protection for one-time pairing sessions.
- Device management actions for paired mobile devices (rename and revoke) with trust metadata.
- Dedicated mobile sync helper tests in `background/mobile-sync.test.mjs`.
- Transaction lifecycle harness tests in `background/tx-lifecycle.test.mjs` to validate approval/broadcast state transitions.

### Changed
- Pairing schema bumped to `nozy.mobile_sync.pairing.v2` with replay-protection metadata.
- Extension smoke and CI worker test commands now run both `tx-utils` and `mobile-sync` test suites.
