# Browser Extension Changelog

All notable changes to the Nozy browser extension are tracked here.

## 0.1.26 — 2026-09-18

### Changed
- Header no longer shows the logo (actions only). Large Nozy logo on **Send** (same treatment as Receive/Home).

## 0.1.25 — 2026-09-18

### Changed
- **Receive:** release-style layout with large Nozy logo, bigger QR, readable address block, and primary **Copy address** CTA (current platinum styling).

## 0.1.24 — 2026-09-18

### Fixed
- **Logo size / Home brand:** header mark enlarged for readability; Nozy logo restored on the Home screen (it was hidden when `showBrand` was off for dashboard).

## 0.1.23 — 2026-09-18

### Fixed
- **Settings → Reveal recovery phrase** stuck on “Checking…” — after unlock, reveal reuses the session password check instead of running a second Argon2 decrypt that froze the service worker.

## 0.1.22 — 2026-09-18

### Added
- **Create backup flow:** after create, reveal or copy the 24-word phrase, then confirm by picking the correct word for random positions (Word #N).
- **Settings → Recovery phrase:** re-view / copy the seed anytime after unlock (password step-up).

## 0.1.21 — 2026-09-18

### Added
- **Public sync** (zec.rocks-style): when no local Zebrad is found, Welcome offers **Yes — use public sync** → `https://lwd.nozywallet.org:443`. Keys stay in the extension; not `api.nozywallet.org`.
- **Public sync** mode chip on Connect your node; create/restore allowed after public LWD opt-in (birthday from `nozywallet.org/status.json` when RPC is offline).

### Changed
- Removed incorrect Zebrad JSON-RPC probes against `zec.rocks` (that host is lightwalletd gRPC).
- Companion LWD prefs may keep an explicit public LWD URL after opt-in.

## 0.1.12 — 2026-09-03

### Added
- **Network privacy (Nym)** panel: mixnet helper readiness, dVPN probe, and operator opt-in for **LWD over mixnet** (`lwd-mixnet-client` on `:9068` / metrics `:9070`) with Probe.
- Companion tab preset **Mixnet proxy :9068** (not a store default; Chrome still does not embed the Nym SDK).

### Notes
- Honest scope: extension drives local companion + optional dialling-half URL. Do not claim “Nym integrated” in store copy.

## 0.1.10 — 2026-08-03

### Fixed
- **Mandatory ZIP-317 ×4 fee** on extension WASM / service-worker / wallet-worker paths (was defaulting to conventional **10 000** zat when `priority` was false or WASM fee export was missing). Aligns with CLI / api-server / desktop (`NOZY_WALLET_PRIORITY_FEE`).
- `build_orchard_v5_tx_from_note` now clamps fee up to the policy minimum so legacy callers cannot underpay.
- Added WASM exports: `estimate_orchard_send_fee_zats`, `pilot_expiry_delta_blocks`, `nozy_version_display`.

## 0.1.9 — 2026-08-02

### Added
- Zcash Names (ZNS) resolve on Send surfaces (with desktop/API phases).

### Notes
- Ships alongside CLI **v2.4.4** / Desktop **1.0.0-beta.5**. The August 2026 AI-assisted security self-review covered CLI + core + Desktop; **extension WASM was out of scope** for that pass — see `docs/reference/security-audit/`.

## 0.1.8 — 2026-07-31

### Added
- Quiet Sapling legacy funds via companion API: companionSaplingStatus / companionSaplingScan / companionSaplingShield, service-worker handlers, Companion popup UX, and COMPANION.md notes (Refs #200 / #207).

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
