# Balance & Sync Endpoints

## `GET /api/balance`

Returns shielded balance after wallet unlock / scan.

## `POST /api/sync`

Scan blockchain for notes.

| Field | Description |
|-------|-------------|
| `password` | Unlock if needed |
| `start_height` | Optional range start |
| `end_height` | Optional range end |
| `zebra_url` | Optional RPC override |

Default: incremental from `last_scan_height + 1`, max ~1000 blocks per request. Loop until caught up.

## `GET /api/config`

Returns `zebra_url`, `last_scan_height`, network settings.

## lightwalletd (Zeaking)

| Method | Path |
|--------|------|
| GET | `/api/lwd/info` |
| GET | `/api/lwd/chain-tip` |
| POST | `/api/lwd/sync/compact` |

Query/body: optional `lightwalletd_url`, `db_path`.

Shared implementation with Tauri `lwd_*` commands and `zeaking-ffi`.

## CLI equivalents

```bash
nozy balance
nozy sync --to-tip
nozy lwd …
```

Doc: [`docs/issues/api-sync-scan-height-response.md`](../../../docs/issues/api-sync-scan-height-response.md)
