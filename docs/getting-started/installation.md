# Installation

NozyWallet is a wallet. It does not ship a consensus node. Run **[Zebra](https://github.com/ZcashFoundation/zebra)** or **[Zakura](https://zakura.com/)** for JSON-RPC and the **[Nozy Sync Engine](../../nozy-sync-engine/README.md)** (or lightwalletd) for compact blocks, then point the wallet at those endpoints.

**Prerequisites:** Rust 1.70+ ([rustup.rs](https://rustup.rs/)), Zebra RPC (default `http://127.0.0.1:8232`), and lightwalletd (default `http://127.0.0.1:9067`) on the same network.

Windows node helpers: [`scripts/README.md`](../../scripts/README.md). Shielded-send limits: [`ZEBRAD_SHIELDED_SEND_LIMIT.md`](../../ZEBRAD_SHIELDED_SEND_LIMIT.md).

## Downloads

**Production downloads today:** the `nozy` CLI, the desktop app, and the localhost companion API. Extension and mobile are in active development.

| What | Direct link |
|------|-------------|
| CLI — Windows | [nozy-windows.exe](https://github.com/LEONINE-DAO/Nozy-wallet/releases/latest/download/nozy-windows.exe) |
| CLI — Linux | [nozy-linux](https://github.com/LEONINE-DAO/Nozy-wallet/releases/latest/download/nozy-linux) |
| CLI — macOS Apple Silicon | [nozy-macos-arm](https://github.com/LEONINE-DAO/Nozy-wallet/releases/latest/download/nozy-macos-arm) |
| CLI — macOS Intel | [nozy-macos-intel](https://github.com/LEONINE-DAO/Nozy-wallet/releases/latest/download/nozy-macos-intel) |
| Checksums | [HASHES.txt](https://github.com/LEONINE-DAO/Nozy-wallet/releases/latest/download/HASHES.txt) |

Release page: [latest release](https://github.com/LEONINE-DAO/Nozy-wallet/releases/latest).

Current tags called out from the project page:

- CLI [v2.4.7 — Mango Habanero](https://github.com/LEONINE-DAO/Nozy-wallet/releases/tag/v2.4.7)
- Desktop [Hot Lemon Pepper Sprinkles beta.7](https://github.com/LEONINE-DAO/Nozy-wallet/releases/tag/desktop-v1.0.0-beta.7)
- Extension Sweet Chili 0.1.26 (source; not a store listing)

Release CI attaches CLI (`nozy-*`) and localhost companion API (`nozywallet-api-*`). Desktop ships under `desktop-v*` tags. iOS and Android are not on the App Store or Google Play. See the [enhancement roadmap](../../ENHANCEMENT_ROADMAP.md).

## CLI from source

```bash
git clone https://github.com/LEONINE-DAO/Nozy-wallet.git
cd Nozy-wallet
cargo build --release
```

The binary is `target/release/nozy`. First commands are in [CLI usage](../guides/cli-usage.md).

## Desktop

The Tauri app lives in `desktop-client/`. Build notes: [desktop README](../../desktop-client/README.md) (`cargo tauri dev`). Balance, sync, and broadcast depend on a reachable node. See [`ZEBRAD_SHIELDED_SEND_LIMIT.md`](../../ZEBRAD_SHIELDED_SEND_LIMIT.md).

## Companion API

`api-server/` is the localhost HTTP companion (`nozywallet-api`, default `127.0.0.1:3000`) for same-machine use. It is not a hosted multi-user wallet. See [api-server README](../../api-server/README.md).

## Extension and mobile

Build from source under `browser-extension/` and `nozy-mobile/`. Companion wiring: [COMPANION.md](../../browser-extension/COMPANION.md).

## Operator stack

Recommended: `zebrad` or Zakura (RPC `:8232`) + Nozy Sync Engine or lightwalletd (gRPC `:9067`) + Nozy.

When wallet sync can start, and what “fully synced” means, is in the [node operator FAQ](../operators/node-faq.md).
