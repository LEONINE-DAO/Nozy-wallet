# Desktop v1.0.0-beta.4 — Hot Lemon Pepper Sprinkles (Windows)

Copy into the GitHub Release body when tagging **`desktop-v1.0.0-beta.4`**.

---

## Summary

**NozyWallet Desktop v1.0.0-beta.4 — Hot Lemon Pepper Sprinkles** is the next **Windows beta** after beta.3 (same product name; new build). Food names stay — that's the product style. It remains **pre-release** until GA.

This build stacks **Sapling quiet legacy** client wiring from master (#205+) on top of the prior Hot Lemon Pepper Sprinkles desktop base.

The **CLI (Teriyaki Hot / Nozy Lite)** remains the production surface for operators. Desktop is for interactive Ironwood migration, Sapling legacy quiet flows, and day-to-day GUI testing.

## Requirements

- **OS:** Windows 10/11 (x86_64)
- **Node stack:** Zebrad + lightwalletd — configure in **Settings → Network**
- Same wallet data directory as the CLI (`%APPDATA%\nozy\…`)

## Install

1. Download **`nozy-desktop-windows-x86_64-installer.exe`** (NSIS) from the assets (attached by CI after this release is published).
2. Run the installer; launch from Start menu / desktop shortcut.
3. Create or restore a wallet, set RPC URLs, **Sync**, then use Send / Ironwood / Sapling quiet flows as needed.

## What's new since beta.3

- **Sapling quiet legacy UI (#205):** desktop status, scan, and shield for legacy Sapling funds without noisy banners
- Client wiring stacked on master for Sapling companion flows (status / scan / shield)
- Includes Sapling phases and related master work landed since beta.3 (keys, LWD scan, UA receive, Phase 4 spend foundations)

## Known limits / beta disclaimer

- **Beta** — not a third-party security audit
- **Windows** installer is the primary published asset (CI builds on release publish)
- Requires **Zebrad + lightwalletd** (not bundled)
- Prefer **local Zebrad** for migration broadcast

## Downloads

| Asset | Description |
|-------|-------------|
| `nozy-desktop-windows-x86_64-installer.exe` | NSIS installer (CI attaches after publish) |
| `nozy-desktop-windows-x86_64-installer.exe.sha256` | SHA256 checksum |

## Smoke

Run `.\scripts\desktop-smoke.ps1` when Zebrad/LWD are up. Manual send + Ironwood + Sapling quiet flow: maintainer sign-off before promoting beyond beta.

## AI disclosure

Release packaging (version bump, notes, tag, GitHub Release) was **agent-assisted** (Cursor). Human author remains responsible for correctness and security.
