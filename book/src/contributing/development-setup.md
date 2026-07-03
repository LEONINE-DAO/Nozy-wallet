# Development Setup

Set up a local environment to build and test NozyWallet.

## Prerequisites

- **Rust 1.70+** — [rustup.rs](https://rustup.rs/)
- **Git**
- **Zebrad** (for integration tests and manual sync) — [Zebra Node Setup](../advanced/zebra-node.md)
- **protoc** — required for `zeaking` / lightwalletd gRPC builds (`sudo apt install protobuf-compiler` on Debian/Ubuntu)

Optional for desktop: **Node.js 18+**, **Tauri** prerequisites per [Tauri docs](https://tauri.app/start/prerequisites/).

## Clone and build

```bash
git clone https://github.com/LEONINE-DAO/Nozy-wallet.git
cd NozyWallet
cargo build
cargo test
cargo fmt --all -- --check
```

## Workspace layout

| Path | Crate / app |
|------|-------------|
| `src/` | `nozy` — core wallet + CLI |
| `zeaking/` | Compact sync / LWD |
| `api-server/` | HTTP companion |
| `zeaking-ffi/` | Mobile bindings |
| `desktop-client/` | Tauri app (separate `Cargo.toml`) |
| `browser-extension/wasm-core/` | WASM (excluded from root workspace) |

See [`AGENTS.md`](../../../AGENTS.md) for where code belongs.

## Desktop development

```bash
cd desktop-client
npm install
cargo tauri dev
```

Use the **desktop window**, not the browser tab on port 5173.

## api-server

```bash
cd api-server
cargo run
# http://127.0.0.1:3000
```

## Environment variables

| Variable | Effect |
|----------|--------|
| `ZEBRA_RPC_URL` | Override Zebrad URL for this process |
| `LIGHTWALLETD_GRPC` | lightwalletd gRPC URL |
| `NOZY_LWD_DB` | Compact block SQLite path |

## Zebrad on Windows + WSL

```powershell
. .\scripts\zebra-wsl-rpc.ps1   # sets ZEBRA_RPC_URL to WSL IP
cargo run --bin nozy -- test-zebra
```

## Next steps

- [Code Guidelines](code-guidelines.md)
- [Contributing Guide](guide.md)
- [Roadmap](roadmap.md)

Full detail: [`CONTRIBUTING.md`](../../../CONTRIBUTING.md).
