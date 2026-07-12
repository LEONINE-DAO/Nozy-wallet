# Desktop v1.0.0-beta.2 — Windows (Ironwood WIP)

Copy into the GitHub Release body when tagging **`desktop-v1.0.0-beta.2`**.

---

## Summary

**NozyWallet Desktop v1.0.0-beta.2** is the next **Windows beta** after Hot Lemon Pepper Sprinkles. It stays **pre-release** until **Ironwood (NU6.3) is official**.

The **CLI (Teriyaki Hot / CLI Lite, v2.4.1.1+)** remains the production surface for operators. Desktop is for interactive Ironwood migration and day-to-day GUI testing.

## Requirements

- **OS:** Windows 10/11 (x86_64)
- **Node stack:** Zebrad + lightwalletd — configure in **Settings → Network**
- Same wallet data directory as the CLI (`%APPDATA%\nozy\…`)

## Install

1. Download **`nozy-desktop-windows-x86_64-installer.exe`** (NSIS) from the assets.
2. Run the installer; launch from Start menu / desktop shortcut.
3. Create or restore a wallet, set RPC URLs, **Sync**, then use Send / Ironwood as needed.

## What's new since beta.1

- **Ironwood tab:** Split → Plan → Migrate → Broadcast (same core path as CLI)
- Network privacy attestation for remote Zebrad (Settings); optional Nym smolmix via env/config (same as CLI)
- UI polish: sync/send/settings, wallet switcher, readiness card
- Aligns with CLI Lite / Ironwood 2.4.x core

## Known limits / beta disclaimer

- **Beta** until Ironwood is officially released — not a third-party security audit
- **Windows only** for this release
- Requires **Zebrad + lightwalletd** (not bundled)
- Browser / dApp tab disabled by default
- Nym smolmix helper is **opt-in env/config**, not a marketed “Nym integrated” product claim
- Prefer **local Zebrad** for migration broadcast

## Downloads

| Asset | Description |
|-------|-------------|
| `nozy-desktop-windows-x86_64-installer.exe` | NSIS installer |
| `nozy-desktop-windows-x86_64-installer.exe.sha256` | SHA256 checksum |

## Smoke

Run `.\scripts\desktop-smoke.ps1` when Zebrad/LWD are up. Manual send + Ironwood flow: maintainer sign-off before promoting beyond beta.
