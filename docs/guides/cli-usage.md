# CLI usage

Command list: [book CLI overview](../../book/src/cli/overview.md). Sending walkthrough in the book: [Sending ZEC](../../book/src/user-guide/sending-zec.md). Install: [installation](../getting-started/installation.md).

Sample terminal text below is illustrative. Your mnemonic, addresses, heights, and txids will differ. Treat a real mnemonic like cash: write it on paper, do not store it in a screenshot or a cloud note.

## Create a wallet

```bash
nozy new
# or, from a source checkout:
cargo run --bin nozy new
```

This generates an HD wallet with a BIP39 mnemonic, optionally sets a password, creates an Orchard address, and saves the wallet under the platform data directory (see [configuration](configuration.md)).

## Restore

```bash
nozy restore
```

Enter the 24-word mnemonic and the wallet password if one was set.

## Addresses

```bash
nozy receive
nozy addresses --count 5
```

Transparent `t1` addresses are rejected in user-facing send and receive flows. Use unified `u1` receivers.

## Sync

```bash
# After receiving funds, or after upgrading to v2.3.3+
nozy sync --to-tip

# Bounded range (advanced)
nozy scan --start-height 1000000 --end-height 1000100
```

Plain `sync` advances about 1,000 blocks per run on mainnet. Operator details: [node FAQ](../operators/node-faq.md).

## Balance, status, history

```bash
nozy balance
nozy status
nozy history
```

## Proving

Orchard uses Halo 2. Proving is built into the library. No Sapling-style parameter download is required.

```bash
nozy proving --status
```

`proving --download` remains for API compatibility and does not fetch parameter files.

## Send

```bash
nozy send --recipient "u1..." --amount 0.1

# Opt-in priority fee (ZIP-317 standard × 4, about a 2-block expiry window)
nozy send --recipient "u1..." --amount 0.1 --priority
```

Fees are computed client-side (ZIP-317): 5,000 zats per logical action, minimum 2 grace actions. A typical 1-in / 1-out send is about 0.0001 ZEC. Zebrad does not implement `estimatefee`. Transactions expire 5 blocks after the mempool build height by default (about 6 minutes at 75 seconds per block).

### Mainnet confirmation

- **Testnet:** type `yes` to confirm.
- **Mainnet:** type `SEND` (all caps) to confirm. This spends real ZEC.

```bash
nozy config --set-network testnet   # recommended while learning
nozy config --set-network mainnet
nozy config --show
```

## End-to-end flow

```bash
nozy new
nozy receive
# share the unified address, then after the payment is mined:
nozy sync --to-tip
nozy balance
nozy send --recipient "u1..." --amount 0.5
nozy history
```

## Library example

Programmatic send uses the Rust crate, not the CLI. See [library API](../development/library-api.md).

## Demos

Walkthroughs are posted in [GitHub Discussions](https://github.com/LEONINE-DAO/Nozy-wallet/discussions) and [Discord](https://discord.gg/XMmFGvcQ89).
