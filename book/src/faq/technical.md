# Technical Questions

## Why Zebrad instead of zcashd?

JSON-RPC alignment with Zebra Foundation direction; wallet derives Orchard witnesses locally. See [`ZEBRAD_SHIELDED_SEND_LIMIT.md`](../../../ZEBRAD_SHIELDED_SEND_LIMIT.md).

## Why does send take minutes?

Orchard Halo2 proving + witness freshness. Timings: [mainnet evidence](../../../docs/reference/MAINNET_SEND_READINESS_EVIDENCE.md).

## What is witness lag?

Orchard spends need up-to-date witnesses. Scan height at tip ≠ witnesses fresh — check `nozy status` or desktop sync status.

## What is lightwalletd for?

Compact block sync for extension/mobile paths via **Zeaking** — optional alongside Zebrad RPC scan.

## Workspace crates?

`nozy`, `zeaking`, `api-server`, `zeaking-ffi`; desktop and WASM excluded from root workspace. [Development Setup](../contributing/development-setup.md).

## Feature flags?

`secret-network` enables `nozy shade`. Monero/swap/bridge may require additional flags — experimental.

## NU 6.1 / 6.2 support?

See [NU 6.1 Support](../features/nu61-support.md) and release CHANGELOG.

## Build from source?

```bash
cargo build --release --bin nozy
cd desktop-client && cargo tauri build
```

## Verify Zebrad connection?

```bash
nozy test-zebra
```

Guide: [Zebrad connectivity](../../../docs/reference/ZEBRAD_NOZYWALLET_CONNECTIVITY.md).

## How does Keystone PCZT signing work?

NozyWallet builds a proved **PCZT** (partial transaction), encodes it as `zcash-pczt` **UR QR frames**, Keystone signs offline, Nozy extracts and broadcasts. Mainnet Orchard only. Details: [Keystone Hardware Wallet](../security/keystone-hardware-wallet.md).
