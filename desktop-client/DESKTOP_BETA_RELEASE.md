# Desktop v1.0.0-beta.1 — Hot Lemon Pepper Sprinkles (Windows)

> **Superseded for GA.** Use [`DESKTOP_RELEASE.md`](DESKTOP_RELEASE.md) for **`desktop-v1.0.0`**. This file remains the beta.1 release notes archive.

Copy the sections below into the GitHub Release body when tagging **`desktop-v1.0.0-beta.1`**.

---

## Summary

**Hot Lemon Pepper Sprinkles** is the first public **NozyWallet desktop beta** for **Windows**. It is a **Tauri** app that shares the same shielded-wallet core as the CLI and uses your local **Zebrad** + **lightwalletd** stack for sync and send.

The **CLI remains the production surface** for operators and mainnet workflows. Treat this desktop build as a beta preview.

## Requirements

- **OS:** Windows 10/11 (x86_64)
- **Node stack:** [Zebrad](https://github.com/ZcashFoundation/zebra) + [lightwalletd](https://github.com/zcash/lightwalletd) — configure RPC URLs in desktop **Settings → Network** (same expectations as CLI)
- **Recommended CLI baseline:** [NozyWallet v2.3.6.7 — Teriyaki Hot (CLI)](https://github.com/LEONINE-DAO/Nozy-wallet/releases/tag/v2.3.6.7) for witness-sync behavior parity

## Install

1. Download **`nozy-desktop-windows-x86_64-installer.exe`** (NSIS) from the assets below.
2. Run the installer on a clean or existing Windows profile.
3. After install you get a **Start menu** shortcut under **NozyWallet** and a **desktop icon** with the NozyWallet logo.
4. Launch **NozyWallet** from the desktop or Start menu (use the desktop window — not a browser tab at `http://localhost:5173`).
5. Create or restore a wallet, set network endpoints, then **Sync** before send.

**MSI:** not shipped for beta.1 — Windows Installer cannot use semver pre-release versions like `1.0.0-beta.1`. Use the NSIS `.exe`.

## What's included

- **Tauri desktop shell** with Home, History, Send, Settings, and wallet lifecycle (create / restore / unlock / lock)
- **Witness-sync parity with CLI v2.3.6.7+** — shared sync path with witness catch-up and lag checks before send
- **Real NozyWallet logo icons** in the app and installer
- **Content Security Policy (CSP)** enabled
- **Browser tab disabled by default** for this beta (subscription-gated web wallet deferred)

## Known limits / beta disclaimer

- **Beta software** — not audited for third-party security review; use at your own risk on mainnet.
- **Windows only** for this release; macOS/Linux builds are not published yet.
- **Requires Zebrad + lightwalletd** — the desktop app does not bundle a full node.
- **Multi-account, multi-device sync, multi-sig** — not in this beta.
- **Browser / Nym subscription tab** — disabled by default; enablement planned for a later release.
- **CLI remains production** — operators should continue to rely on `nozy` for scripted and VPS workflows.

## Smoke sign-off

Automated desktop smoke (**A1–A11**): **11/11 passed** via `.\scripts\desktop-smoke.ps1` before tag. Manual checklist sections **B–G** remain operator responsibility for your environment.

## Downloads

| Asset | Description |
|-------|-------------|
| `nozy-desktop-windows-x86_64-installer.exe` | NSIS installer (recommended) |
| `nozy-desktop-windows-x86_64-installer.exe.sha256` | SHA256 checksum |

Attach the NSIS installer built from `desktop-client` (`npm run tauri build -- --bundles nsis`) under `src-tauri/target/release/bundle/nsis/`.
