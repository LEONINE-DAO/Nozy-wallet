# Desktop v1.0.0-beta.3 — Hot Lemon Pepper Sprinkles (Windows)

Copy into the GitHub Release body when tagging **`desktop-v1.0.0-beta.3`**.

---

## Summary

**NozyWallet Desktop v1.0.0-beta.3 — Hot Lemon Pepper Sprinkles** is the next **Windows beta** after Hot Lemon (beta.2). Food names stay — that's the product style. It remains **pre-release** until GA.

The **CLI (Teriyaki Hot / Nozy Lite, v2.4.2+)** remains the production surface for operators. Desktop is for interactive Ironwood migration and day-to-day GUI testing.

## Requirements

- **OS:** Windows 10/11 (x86_64)
- **Node stack:** Zebrad + lightwalletd — configure in **Settings → Network**
- Same wallet data directory as the CLI (`%APPDATA%\nozy\…`)

## Install

1. Download **`nozy-desktop-windows-x86_64-installer.exe`** (NSIS) from the assets.
2. Run the installer; launch from Start menu / desktop shortcut.
3. Create or restore a wallet, set RPC URLs, **Sync**, then use Send / Ironwood as needed.

## What's new since beta.2

- Preserve ZIP 318 equal-value twin notes on sync (nullifier-keyed merge) so split balances no longer look halved
- ZIP 318 turnstile funding aligned with ECC-style canonical crossings (oldest zero-change note; fee-from-output when needed)
- `ironwood preflight` surfaces a dry-run ZIP 318 crossing proposal
- Ironwood-aware mainnet builders for migrate / Ironwood sends
- Desktop: Network Privacy blank-screen fix, theme/toast polish, refreshed app icons + brand logo

## Known limits / beta disclaimer

- **Beta** — not a third-party security audit
- **Windows** installer is the primary published asset (CI may also build other targets)
- Requires **Zebrad + lightwalletd** (not bundled)
- Browser / dApp tab disabled by default
- Prefer **local Zebrad** for migration broadcast

## Downloads

| Asset | Description |
|-------|-------------|
| `nozy-desktop-windows-x86_64-installer.exe` | NSIS installer |
| `nozy-desktop-windows-x86_64-installer.exe.sha256` | SHA256 checksum |

## Smoke

Run `.\scripts\desktop-smoke.ps1` when Zebrad/LWD are up. Manual send + Ironwood flow: maintainer sign-off before promoting beyond beta.
