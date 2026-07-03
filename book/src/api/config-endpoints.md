# Configuration Endpoints

## `GET /api/config`

Read wallet configuration including:

- `zebra_url`
- `last_scan_height`
- Theme / network fields (build-dependent)

## `POST /api/config/zebra-url`

Body: `{ "url": "http://host:8232" }`

## `POST /api/config/theme`

UI theme preference for companion frontends.

## CLI equivalent

```bash
nozy config --set-zebra-url http://127.0.0.1:8232
```

## File location

Same as [Network Configuration](../advanced/network-config.md) — `%APPDATA%\nozy\nozy\config\config.json` on Windows.

No API to set `last_scan_height` while server running — use sync ranges or stop server and edit config for advanced rescans.
