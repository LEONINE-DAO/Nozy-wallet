# nozy-zsa — isolated OrchardZSA toolkit

Separate Cargo package so **QEDIT OrchardZSA** deps do not replace Ironwood `orchard 0.15` in the root Nozy workspace.

Pins follow [QED-it/zcash_tx_tool](https://github.com/QED-it/zcash_tx_tool) (`orchard` + `zcash_*` with `zsa-issuance`).

## Build

```bash
cd nozy-zsa
cargo build --release
cargo run --release -- status
```

## Goal

Issue / transfer / burn custom assets (e.g. test `zUSD`) on `https://zsa.methyl.cc`, then expose helpers the main `nozy zsa` CLI can call.

See [`docs/reference/ZSA_TESTNET.md`](../docs/reference/ZSA_TESTNET.md).
