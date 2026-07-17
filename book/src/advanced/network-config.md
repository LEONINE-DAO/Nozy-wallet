# Network Configuration

Configure which Zcash network and backend nodes NozyWallet uses.

## Config file

| OS | Path |
|----|------|
| Windows | `%APPDATA%\nozy\nozy\config\config.json` |
| Linux / macOS | XDG config — typically `~/.config/nozy/config.json` |

Common fields:

```json
{
  "zebra_url": "http://127.0.0.1:8232",
  "last_scan_height": 3395000,
  "privacy_network": "mainnet"
}
```

## Environment overrides

| Variable | Overrides |
|----------|-----------|
| `ZEBRA_RPC_URL` | Primary full-node RPC URL (Zebrad or Zakura) |
| `LIGHTWALLETD_GRPC` | lightwalletd gRPC endpoint |

## CLI

```bash
nozy config --set-network mainnet
nozy config --set-network testnet
nozy config --set-zebra-url http://host:8232
nozy config --use-local
nozy config --use-remote http://vps:8232
```

## Desktop

**Settings → Network** — edit URL, **Test Connection**.

## Backend selection

Default: **Zebrad** or **Zakura** JSON-RPC (Zebra-fork compatible). Optional Crosslink backend via `nozy config` flags — see `nozy config --help` in your build.

## UTF-8 BOM note

Editing `config.json` with PowerShell can add a BOM. Current releases strip BOM on load; prefer UTF-8 without BOM when hand-editing.

## Trust

Configured `zebra_url` is trusted for broadcast and chain tip. Optional `trusted_zebra_urls` list in config for additional endpoints.

## Related

- [Zebra Node Setup](zebra-node.md)
- [Zakura Node Setup](zakura-node.md)
- [Zebrad connectivity reference](../../../docs/reference/ZEBRAD_NOZYWALLET_CONNECTIVITY.md)
- [Zakura connectivity reference](../../../docs/reference/ZAKURA_NOZYWALLET_CONNECTIVITY.md)
