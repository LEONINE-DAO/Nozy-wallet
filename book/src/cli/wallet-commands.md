# Wallet Commands

## `nozy new`

Create a new wallet and display a **24-word mnemonic**. Store it offline immediately.

```bash
nozy new
```

## `nozy restore`

Restore from mnemonic (interactive).

```bash
nozy restore
```

## `nozy receive`

Generate a shielded Orchard unified address.

```bash
nozy receive
```

## `nozy info`

Wallet addresses, network, and metadata.

```bash
nozy info
```

## `nozy balance`

Current shielded balance (spendable notes).

```bash
nozy balance
```

If balance shows 0 incorrectly on v2 note index, see [CLI balance reference](../../../docs/reference/CLI_BALANCE_NOTEINDEX.md) and try `list-notes`.

## `nozy list-notes`

List Orchard notes with amounts and heights.

```bash
nozy list-notes
```

## `nozy status`

Sync checkpoint, chain tip, witness lag summary.

```bash
nozy status
```

Use before send to confirm catch-up.

## `nozy config`

Configure network and node URL.

```bash
nozy config --set-zebra-url http://127.0.0.1:8232
nozy config --set-network mainnet
nozy config --use-local
nozy config --use-remote http://vps:8232
```

Environment override: `ZEBRA_RPC_URL=http://host:8232`.

## `nozy proving`

Orchard Halo2 proving parameters (~large download, once per machine).

```bash
nozy proving --download
nozy proving --status
```

## `nozy address-book`

Manage saved addresses from CLI.

```bash
nozy address-book list
nozy address-book add --name Alice --address u1…
nozy address-book remove --name Alice
```

## Multi-wallet (desktop)

CLI historically used a single data dir; desktop supports **profiles** under `%APPDATA%\nozy\nozy\data\profiles\`. CLI profile switching follows `wallet_profiles` in core — check your build’s `nozy --help` for profile flags if available.

See [Wallet Management](../user-guide/wallet-management.md).
