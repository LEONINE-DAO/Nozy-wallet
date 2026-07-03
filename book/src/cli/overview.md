# Command Overview

The `nozy` CLI is the primary production surface for Orchard shielded ZEC on mainnet.

```bash
cargo build --release --bin nozy
./target/release/nozy --help
```

Global flags (common):

| Flag | Purpose |
|------|---------|
| `-m`, `--mainnet` | Mainnet (default in many builds) |
| `--testnet` | Testnet |
| `--zebra-url <URL>` | Override Zebrad RPC for this invocation |

Config persists in platform config dir — see [Zebra Node Setup](../advanced/zebra-node.md).

## Core commands

| Command | Purpose |
|---------|---------|
| `new` | Create wallet + mnemonic |
| `restore` | Restore from 24-word phrase |
| `receive` | Generate Orchard receiving address |
| `sync` | Scan blocks (`--to-tip` for full catch-up) |
| `send` | Shielded send (`-r`, `-a`, optional `--memo`) |
| `balance` | Shielded balance |
| `list-notes` | Spendable notes detail |
| `history` | Transaction history |
| `status` | Sync and witness summary |
| `info` | Wallet metadata |
| `config` | Network, `zebra_url`, backend |
| `test-zebra` | Verify node RPC |
| `proving` | Download / status for Orchard params |

## Extended commands

| Command | Purpose |
|---------|---------|
| `lwd` | lightwalletd compact cache (Zeaking) |
| `zeaking` | Local indexer utilities |
| `address-book` | Saved contacts (CLI) |
| `analytics` | Wallet statistics |
| `check-confirmations` | TX confirmation lookup |
| `privacy-network` | Tor / I2P helpers (experimental) |
| `nu61` | NU 6.1 info |

Optional feature builds:

```bash
cargo build --features secret-network   # adds `shade`
```

## Typical workflow

```bash
nozy test-zebra
nozy sync --to-tip
nozy balance
nozy send -r u1… -a 0.0001
```

## Chapters

- [Wallet Commands](wallet-commands.md)
- [Transaction Commands](transaction-commands.md)
- [Advanced Commands](advanced-commands.md)

More: root [`COMMAND_HELP.md`](../../../COMMAND_HELP.md) if present in your checkout.
