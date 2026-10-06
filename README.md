# NozyWallet

Orchard-first Zcash wallet for **Zebrad**. This repository is the wallet and its companion services. It is not a consensus node.

You run [Zebra](https://github.com/ZcashFoundation/zebra) or [Zakura](https://zakura.com/) for JSON-RPC and the [Nozy Sync Engine](nozy-sync-engine/README.md) (or lightwalletd) for compact blocks. Keys stay on the device. `lwd.nozywallet.org` is public compact-sync only. `api.nozywallet.org` is a single operator companion, not a multi-user hosted wallet.

**Latest release:** [v2.4.7 — Mango Habanero (CLI)](https://github.com/LEONINE-DAO/Nozy-wallet/releases/tag/v2.4.7) · Desktop [Hot Lemon Pepper Sprinkles beta.7](https://github.com/LEONINE-DAO/Nozy-wallet/releases/tag/desktop-v1.0.0-beta.7) · Extension Sweet Chili 0.1.26.

## What it does

Create or restore a shielded wallet (Orchard, now Ironwood), scan for notes, and send ZEC on the CLI, desktop app, and localhost companion API. The browser extension and mobile app are in active development.

Transparent `t1` addresses are rejected in user-facing send and receive. The product is shielded-first (Orchard / unified `u1`).

| Surface | Path | Status |
|--------|------|--------|
| CLI (Nozy Lite) | `src/` | Production download |
| Desktop | `desktop-client/` | Production download (beta tag; no formal GA label) |
| Localhost API | `api-server/` | Production for same-machine use |
| Compact sync | `zeaking/`, `nozy-sync-engine/` | Used by the wallet |
| Extension | `browser-extension/` | Production download |
| Mobile | `nozy-mobile/`, `zeaking-ffi/` | In development |
| Landing site | `landing/` | Marketing only |

**Stack:** `zebrad` or Zakura (`:8232`) + Nozy Sync Engine or lightwalletd (`:9067`) + Nozy. Operators: [`docs/reference/NOZY_LITE.md`](docs/reference/NOZY_LITE.md).

## Download

| Platform | CLI binary |
|----------|------------|
| Windows | [nozy-windows.exe](https://github.com/LEONINE-DAO/Nozy-wallet/releases/latest/download/nozy-windows.exe) |
| Linux | [nozy-linux](https://github.com/LEONINE-DAO/Nozy-wallet/releases/latest/download/nozy-linux) |
| macOS Apple Silicon | [nozy-macos-arm](https://github.com/LEONINE-DAO/Nozy-wallet/releases/latest/download/nozy-macos-arm) |
| macOS Intel | [nozy-macos-intel](https://github.com/LEONINE-DAO/Nozy-wallet/releases/latest/download/nozy-macos-intel) |
| Checksums | [HASHES.txt](https://github.com/LEONINE-DAO/Nozy-wallet/releases/latest/download/HASHES.txt) |

Build from source, desktop, and the companion API: [Installation](docs/getting-started/installation.md).

## Quick start

Rust 1.70+, a Zebra RPC endpoint, and lightwalletd on the same network.

```bash
git clone https://github.com/LEONINE-DAO/Nozy-wallet.git
cd Nozy-wallet
cargo build --release

./target/release/nozy new
./target/release/nozy sync --to-tip
./target/release/nozy receive
```

Full command flow, fees, and mainnet confirmation: [CLI usage](docs/guides/cli-usage.md).

## Documentation

Longer material lives under [`docs/`](docs/README.md), grouped the way a maintainer would look it up. The published user book is separate: [`book/`](book/README.md).

| If you need | Read |
|-------------|------|
| Install and downloads | [Installation](docs/getting-started/installation.md) |
| Create, sync, send | [CLI usage](docs/guides/cli-usage.md) |
| Config, paths, environment | [Configuration](docs/guides/configuration.md) |
| When sync can start | [Node operator FAQ](docs/operators/node-faq.md) |
| Features, NU6.2, surfaces | [Product](docs/product/features.md) |
| Crates and layout | [Architecture](docs/architecture/overview.md) |
| `cargo test` and CI | [Testing](docs/development/testing.md) |
| Recorded mainnet and pilot results | [Evidence](docs/evidence/README.md) |
| Rust wallet API | [Library API](docs/development/library-api.md) |
| Scan throughput notes | [Performance](docs/development/performance.md) |
| Crate updates | [Dependencies](docs/development/dependencies.md) |
| Security policy and audits | [SECURITY.md](SECURITY.md) |
| Everything else (papers, grants, bugs) | [Docs index](docs/README.md) |

Shielded-send rules: [`ZEBRAD_SHIELDED_SEND_LIMIT.md`](ZEBRAD_SHIELDED_SEND_LIMIT.md). Roadmap: [`ENHANCEMENT_ROADMAP.md`](ENHANCEMENT_ROADMAP.md).

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) and [`AGENTS.md`](AGENTS.md). Open an issue before large protocol, security, or sync changes. Areas that need people: [where to help](docs/development/where-to-help.md).

## Security

Report vulnerabilities in private using [`SECURITY.md`](SECURITY.md). Self-audit (December 2025): [`SELF_AUDIT_RESULTS.md`](SELF_AUDIT_RESULTS.md). Prep notes for a future third-party review: [`docs/reference/security-audit/`](docs/reference/security-audit/).

## License

MIT. See [LICENSE](LICENSE).

## Disclaimer

This software is provided as is, without warranty. Verify every transaction before broadcasting on mainnet.

## Support

- [GitHub Issues](https://github.com/LEONINE-DAO/Nozy-wallet/issues)
- [GitHub Discussions](https://github.com/LEONINE-DAO/Nozy-wallet/discussions)
- [Discord](https://discord.gg/XMmFGvcQ89)
- [Releases](https://github.com/LEONINE-DAO/Nozy-wallet/releases)

## Acknowledgments

- [Zcash Foundation](https://zfnd.org/) for the Zcash protocol
- [Zebra](https://github.com/ZcashFoundation/zebra) for the node
- [Orchard](https://github.com/zcash/orchard) for shielded transactions
- [Rust](https://www.rust-lang.org/)
