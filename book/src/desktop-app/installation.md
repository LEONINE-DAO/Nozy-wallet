# Desktop Installation

Install the NozyWallet desktop app (Tauri + React) on your machine.

## Prerequisites (build from source)

- **Rust 1.70+**
- **Node.js 18+**
- **Platform Tauri deps** — [tauri.app/start/prerequisites](https://tauri.app/start/prerequisites/)

## From source (developers)

```bash
git clone https://github.com/LEONINE-DAO/Nozy-wallet.git
cd NozyWallet/desktop-client
npm install
cargo tauri dev
```

Launch the **NozyWallet window** from the taskbar — not a browser tab at `http://localhost:5173`.

## Production build

```bash
cd desktop-client
cargo tauri build
```

Artifacts under `desktop-client/src-tauri/target/release/`:

| OS | Output |
|----|--------|
| Windows | `.exe` / MSI installer |
| macOS | `.dmg` |
| Linux | `.AppImage` or `.deb` |

## Pre-built installers

When available, download from [GitHub Releases](https://github.com/LEONINE-DAO/Nozy-wallet/releases/latest).

## Also required for full function

- **Zebrad** with RPC — [Zebra Node Setup](../advanced/zebra-node.md)
- **Orchard proving params** — downloaded on first send or via Settings
- **lightwalletd** (optional) — compact sync; default `127.0.0.1:9067`

## Related

- [First-Time Setup](first-time-setup.md)
- [Desktop README](../../../desktop-client/README.md)
