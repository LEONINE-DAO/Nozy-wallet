# Advanced Commands

## `nozy test-zebra`

Verify JSON-RPC to Zebrad, **Zakura**, or Crosslink (if configured). Detects node kind from `getnetworkinfo` and probes `z_gettreestate` at tip.

```bash
nozy test-zebra
nozy test-zebra --zebra-url http://172.x.x.x:8232
```

First diagnostic step for any sync/send issue.

## `nozy lwd`

lightwalletd compact-block cache via **Zeaking** (shared with desktop and api-server).

```bash
nozy lwd --help
# Subcommands vary: info, sync, sync-to-tip, prune, etc.
```

Default gRPC: `http://127.0.0.1:9067` or `LIGHTWALLETD_GRPC`.

Requires **protoc** at build time. See `zeaking/README.md`.

## `nozy zeaking`

Local blockchain indexer utilities.

```bash
nozy zeaking --help
```

## `nozy analytics`

Wallet statistics and usage summary.

```bash
nozy analytics
```

## `nozy privacy-network`

Tor / I2P connection testing (experimental).

```bash
nozy privacy-network --help
```

Book: [Privacy Networks](../privacy-networks/overview.md) (experimental).

## `nozy nu61`

Display NU 6.1 / protocol information.

```bash
nozy nu61
```

## `nozy shade` (feature `secret-network`)

Secret Network / Shade Protocol — build with:

```bash
cargo build --release --features secret-network --bin nozy
nozy shade balance
```

See [Secret Network](../advanced/secret-network.md).

## `nozy swap` / `nozy monero`

Cross-chain and Monero integrations — experimental; may not compile on default feature set.

## Diagnostics

| Script | Platform |
|--------|----------|
| `scripts/test-zebrad-nozywallet.ps1` | Windows Zebrad + config smoke test |
| `test_zebra_node.sh` | Unix RPC curl test |
| `scripts/zebra-wsl-rpc.ps1` | Set `ZEBRA_RPC_URL` from WSL IP |

Reference: [Zebrad connectivity](../../../docs/reference/ZEBRAD_NOZYWALLET_CONNECTIVITY.md).

## api-server mirror

Many CLI operations have HTTP equivalents on `localhost:3000` — [API Server Setup](../advanced/api-server.md).
