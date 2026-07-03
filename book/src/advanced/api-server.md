# API Server Setup

The **api-server** exposes NozyWallet over HTTP for browser extensions and local dashboards.

## Build and run

```bash
cd api-server
cargo build --release
cargo run
# listens on http://0.0.0.0:3000
```

**Requires:** `protoc` for zeaking gRPC codegen.

## Use cases

- Chrome/Edge extension companion ([`browser-extension/COMPANION.md`](../../../browser-extension/COMPANION.md))
- Local React dashboard without Tauri
- Integration tests against HTTP

**Not** intended as a public internet wallet API without additional auth and TLS hardening.

## Quick endpoint list

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/health` | Health check |
| GET | `/api/wallet/exists` | Wallet present |
| POST | `/api/wallet/create` | Create |
| POST | `/api/wallet/restore` | Restore mnemonic |
| POST | `/api/wallet/unlock` | Unlock |
| POST | `/api/address/generate` | New address |
| GET | `/api/balance` | Balance |
| POST | `/api/sync` | Scan blocks |
| POST | `/api/transaction/send` | Send |
| GET | `/api/config` | Config + `last_scan_height` |
| POST | `/api/config/zebra-url` | Set node URL |
| GET | `/api/lwd/info` | lightwalletd info |
| POST | `/api/lwd/sync/compact` | Compact block sync |

Full list: [`api-server/README.md`](../../../api-server/README.md).

## Sync semantics

Default sync: incremental from `last_scan_height + 1`, up to 1000 blocks per call. Repeat until `already_synced: true`.

Rescan: pass `start_height` / `end_height` in POST body.

## Shielded sends

Same Orchard path as CLI — local witness derivation. See [`ZEBRAD_SHIELDED_SEND_LIMIT.md`](../../../ZEBRAD_SHIELDED_SEND_LIMIT.md).

## Chapters

- [API Overview](../api/overview.md)
- [Integrate with Frontend](../examples/frontend-integration.md)
