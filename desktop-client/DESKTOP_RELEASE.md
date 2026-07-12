# Desktop v1.0.0 — Windows GA

> **Deferred.** GA `desktop-v1.0.0` waits until **Ironwood is officially released**.  
> Ship **[`DESKTOP_BETA_2_RELEASE.md`](DESKTOP_BETA_2_RELEASE.md)** / tag **`desktop-v1.0.0-beta.2`** until then.

Copy the sections below into the GitHub Release body when tagging **`desktop-v1.0.0`** (after Ironwood official).

---

## Summary

**NozyWallet Desktop v1.0.0** is the first **Windows GA** build for interactive users. It is a **Tauri** app that shares the same shielded-wallet core as the CLI and uses your local **Zebrad** + **lightwalletd** stack for sync and send.

Desktop is production for the Windows UI path. The **CLI remains recommended** for operators, scripting, and VPS workflows.

This release was **internally reviewed** (in-house production + security checklist). It is **not** a third-party cryptocurrency wallet audit.

## Requirements

- **OS:** Windows 10/11 (x86_64)
- **Node stack:** [Zebrad](https://github.com/ZcashFoundation/zebra) + [lightwalletd](https://github.com/zcash/lightwalletd) — configure RPC URLs in **Settings → Network**
- Same wallet data directory conventions as the CLI (`%APPDATA%\nozy\…`)

## Install

1. Download **`NozyWallet_1.0.0_x64-setup.exe`** (NSIS) from the assets below.
2. Run the installer.
3. Launch **NozyWallet** from the Start menu or desktop shortcut (native window — not `http://localhost:5173`).
4. Create or restore a wallet, configure the node, then **Sync** before send.

## What's included

- Wallet lifecycle: create / restore / unlock / lock
- Home: balance, assets with ZEC spot + fiat holdings, Ironwood readiness, chain sync %
- History: full list, filters, detail modal, CSV export
- Send: shielded Orchard send with on-page / backend progress stages
- Sync: percentage feedback in header, banner, and Chain sync panel
- Settings: network, security, display (fiat), Keystone, contacts
- Dark-only UI
- CSP enabled; browser/dApp tab disabled by default

## What's new since beta.1

- Sync progress percentage and clearer catching-up status
- Send progress panel (backend stages + proving wait UX)
- Dark-only theme; Display settings contrast improvements
- Home: removed Recent Activity; Your Assets shows live ZEC price + fiat value
- Production checklists + in-house security review gates
- Feature freeze: no dead QR-scan primary control; multi-account clearly deferred

## Known limits / deferred

- **Windows only** — macOS/Linux installers not published yet
- **Requires Zebrad + lightwalletd** — node not bundled
- **Multi-account / multi-device / multi-sig** — not in v1.0.0
- **Browser / Nym tab** — disabled by default
- **QR address scanning** — not in v1.0.0
- Mainnet use is at your own risk; prefer testnet for rehearsals

## Smoke sign-off

Automated desktop smoke (**A1–A11**): **11/11 passed** via `.\scripts\desktop-smoke.ps1` (2026-07-08).

Manual checklist sections **B–G** and end-to-end send: maintainer responsibility before public tag — see [`PRODUCTION_READY_CHECKLIST.md`](PRODUCTION_READY_CHECKLIST.md) and [`RELEASE_SMOKE_CHECKLIST.md`](RELEASE_SMOKE_CHECKLIST.md).

## Downloads

| Asset | Description |
|-------|-------------|
| `NozyWallet_1.0.0_x64-setup.exe` | NSIS installer |
| `NozyWallet_1.0.0_x64-setup.exe.sha256` | SHA256 checksum |

Build: from `desktop-client`, `npm run tauri build -- --bundles nsis` → `src-tauri/target/release/bundle/nsis/` (or your `CARGO_TARGET_DIR` equivalent).

**SHA256:** `f0556b89b27d68db48b1cd2d6b488206fb016a383afb9859a8f54cbc727485af`
