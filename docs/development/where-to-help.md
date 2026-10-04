# Where to help

Process, style, and pull-request rules: [`CONTRIBUTING.md`](../../CONTRIBUTING.md). Agent policy: [`AGENTS.md`](../../AGENTS.md). Product plan: [`ENHANCEMENT_ROADMAP.md`](../../ENHANCEMENT_ROADMAP.md).

## Active areas

| Area | Where to look | Skills |
|------|----------------|--------|
| Extension + Zeaking companion | [`browser-extension/README.md`](../../browser-extension/README.md), [`COMPANION.md`](../../browser-extension/COMPANION.md) | MV3, WASM, LWD sync via `api-server` |
| Mobile | [`nozy-mobile/README.md`](../../nozy-mobile/README.md), roadmap mobile section | React Native / Expo, Swift, Kotlin |
| Security | [`SECURITY.md`](../../SECURITY.md), [audit prep](../reference/security-audit/README.md) | Cryptography, hardware wallets, review |
| Web UI | [`web-app/README.md`](../../web-app/README.md) | Frontend, localhost API |

Also welcome: UI design, tests, documentation, bug fixes, scan and prove performance, and core Rust wallet work.

## Start

```bash
git clone https://github.com/LEONINE-DAO/Nozy-wallet.git
cd Nozy-wallet
cargo build
cargo test
RUST_LOG=debug cargo run --bin nozy -- --help
```

1. Read the roadmap and open issues labeled `good first issue` or `help wanted`.
2. Open or reference a GitHub issue before large or behavior-changing work (new pool, RPC surface, sync architecture, crypto).
3. Disclose AI assistance in the pull request. The human author remains responsible.
4. Ask in [Discussions](https://github.com/LEONINE-DAO/Nozy-wallet/discussions) or [Discord](https://discord.gg/XMmFGvcQ89).

Security reports: follow [`SECURITY.md`](../../SECURITY.md). Do not file public issues for undisclosed vulnerabilities.
