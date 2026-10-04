# Architecture overview

NozyWallet is a wallet and companion services, not a Zcash consensus node. Shielded sends use local witness derivation against Zebrad treestate. Do not add node-side witness lookups; read [`ZEBRAD_SHIELDED_SEND_LIMIT.md`](../../ZEBRAD_SHIELDED_SEND_LIMIT.md) and the current `ZebraClient` / witness providers.

## Stack

| Layer | Technology |
|-------|------------|
| Language | Rust (2021). Workspace crates: `nozy`, `zeaking`, `nozywallet-api`, `zeaking-ffi` |
| Zcash / Orchard | `orchard 0.14`, `zcash_primitives 0.28`, `zcash_protocol 0.9`, `zcash_address`, `zip32`, `bip39` (see root `Cargo.toml`) |
| Desktop UI | Tauri 2 + React + Vite (`desktop-client/`) |
| Extension | Rust → WASM (`browser-extension/wasm-core/`) + MV3 service worker |
| Compact sync | gRPC to lightwalletd, SQLite cache in `zeaking::lwd` |
| Node (you run it) | [Zebra](https://github.com/ZcashFoundation/zebra) JSON-RPC + [lightwalletd](https://github.com/zcash/lightwalletd), or Zakura. Not `zcashd` |

Optional Secret Network support (`--features secret-network`) shares the wallet seed for SCRT. See [features](../product/features.md).

Root `[workspace]` members: `zeaking`, `api-server`, `zeaking-ffi`. The desktop app and `browser-extension/wasm-core` have their own manifests and are excluded from the root workspace. See [`AGENTS.md`](../../AGENTS.md).

## Core library layout

```
src/
├── main.rs              # CLI
├── lib.rs               # Library exports
├── error.rs             # NozyError / NozyResult
├── hd_wallet.rs         # HD wallet
├── notes.rs             # Note scanning
├── storage.rs           # Wallet persistence
├── zebra_integration.rs # Zebra RPC client
├── orchard_tx.rs        # Orchard transaction building
├── proving.rs           # Orchard proving
├── transaction_builder.rs
└── tests.rs
```

Companion crates sit beside `src/`: `zeaking/`, `api-server/`, `zeaking-ffi/`, `nozy-ffi/`, `nozy-sync-engine/`, `desktop-client/`, `browser-extension/`, `nozy-mobile/`.

## Scanning performance

Note scanning can fetch blocks in parallel and cache them. Defaults, knobs, and what was actually measured: [performance](../development/performance.md).

## Further reading

- White paper (Markdown): [`NozyWallet_Whitepaper.md`](../reference/NozyWallet_Whitepaper.md)
- Outline: [`WHITEPAPER_OUTLINE.md`](../reference/WHITEPAPER_OUTLINE.md)
- User book: [`book/README.md`](../../book/README.md)
- Documentation index: [`docs/README.md`](../README.md)
