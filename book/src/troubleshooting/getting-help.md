# Getting Help

## Before you ask

1. [Common Issues](common-issues.md) — NET_001, sync, send stalls, Keystone pairing.
2. [Keystone Hardware Wallet](../security/keystone-hardware-wallet.md) — PCZT signing with Keystone (mainnet).
3. [Zebra Node Setup](../advanced/zebra-node.md) — verify `nozy test-zebra`.
3. [Error Messages](error-messages.md) — match desktop codes.
4. Run diagnostics (Windows): `.\scripts\test-zebrad-nozywallet.ps1` from repo root.

## Gather this information

- OS (Windows / Linux / macOS) and Nozy surface (CLI, desktop, api-server).
- `nozy test-zebra` output (redact URLs if public).
- Whether Zebrad runs locally, WSL, or remote.
- Error code from UI (e.g. NET_001) — not your mnemonic or password.

**Never share:** 24-word seed, private keys, wallet password, or full `wallet.dat`.

## Community & issues

- [GitHub Issues](https://github.com/LEONINE-DAO/Nozy-wallet/issues) — bugs and feature requests.
- [GitHub Discussions](https://github.com/LEONINE-DAO/Nozy-wallet/discussions) — questions and setup help.

Search existing issues for NET_001, sync, witness lag, balance zero.

## Maintainer docs (advanced)

| Doc | Use when |
|-----|----------|
| [`docs/issues/BUG_REGISTRY.md`](../../../docs/issues/BUG_REGISTRY.md) | Known bugs and fix versions |
| [`docs/reference/ZEBRAD_NOZYWALLET_CONNECTIVITY.md`](../../../docs/reference/ZEBRAD_NOZYWALLET_CONNECTIVITY.md) | Paper-grade Zebrad checklist |
| [`docs/issues/bugs/2026-06-desktop-pre-release-debug-session.md`](../../../docs/issues/bugs/2026-06-desktop-pre-release-debug-session.md) | Windows desktop RCA |

## Contributing fixes

See [Contributing Guide](../contributing/guide.md) and [`AGENTS.md`](../../../AGENTS.md) for agent/human contributors.
