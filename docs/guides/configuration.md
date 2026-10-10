# Configuration

## Network

```bash
nozy config --set-network mainnet   # default
nozy config --set-network testnet
nozy config --show
```

- **Mainnet** sends require typing `SEND`.
- **Testnet** sends require typing `yes`.
- Confirm the network before every send.

## Zebra

```bash
cargo install zebrad
zebrad start --rpc.bind-addr 127.0.0.1:8232
nozy test-zebra
```

On Windows, run Zebrad in WSL. See [`scripts/README.md`](../../scripts/README.md) and the [node FAQ](../operators/node-faq.md).

## Proving parameters

Orchard Halo 2 proving is built into the library. No external parameter files are required.

```bash
nozy proving --status
```

`proving --download` is kept for compatibility and performs no download.

## Config file

| OS | Path |
|----|------|
| Linux | `~/.config/nozy/config.json` or `$XDG_CONFIG_HOME/nozy/config.json` |
| macOS | `~/Library/Application Support/com.nozy.nozy/config/config.json` |
| Windows | `%APPDATA%\nozy\config\config.json` |

Fallback if XDG directories are unavailable: `~/.nozy/config` (Linux/macOS) or `%USERPROFILE%\.nozy\config` (Windows).

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `zebra_url` | string | `http://127.0.0.1:8232` | Zebra RPC URL |
| `crosslink_url` | string | `""` | Crosslink backend URL (falls back to `zebra_url` when empty) |
| `network` | string | `mainnet` | `mainnet` or `testnet` |
| `last_scan_height` | number or null | `null` | Last scanned height |
| `theme` | string | `dark` | `dark` or `light` |
| `backend` | string | `zebra` | `zebra` or `crosslink` |

```json
{
  "zebra_url": "http://127.0.0.1:8232",
  "crosslink_url": "",
  "network": "mainnet",
  "last_scan_height": 2000000,
  "theme": "dark",
  "backend": "zebra"
}
```

```bash
nozy config --set-zebra-url http://127.0.0.1:8232
nozy config --set-network mainnet
nozy config --show
```

## Data directories

Wallet files (`wallet.dat`, notes cache, history):

| OS | Path |
|----|------|
| Linux | `~/.local/share/nozy/data` or `$XDG_DATA_HOME/nozy/data` |
| macOS | `~/Library/Application Support/com.nozy.nozy/data` |
| Windows | `%APPDATA%\nozy\data` |

Fallback: `~/.nozy/data` or `%USERPROFILE%\.nozy\data`.

Important files:

- `wallet.dat` — encrypted wallet (data directory)
- `config.json` — configuration (config directory)
- `notes.json` — scanned notes cache (data directory)

## CLI environment variables

| Variable | Default | Description |
|----------|---------|-------------|
| `ZEBRA_RPC_URL` | `http://127.0.0.1:8232` | Override Zebra RPC URL |
| `LIGHTWALLETD_GRPC` | unset | lightwalletd gRPC URL (for example `http://<WSL-IP>:9067` on Windows) |
| `NOZY_PLAIN_OUTPUT` | unset | Set to `1` for script-friendly output |
| `RUST_LOG` | unset | `debug`, `info`, `warn`, `error` |
| `HOME` / `USERPROFILE` | system | Fallback data paths |
| `XDG_CONFIG_HOME` | `~/.config` | Linux config root |
| `XDG_DATA_HOME` | `~/.local/share` | Linux data root |

```bash
export RUST_LOG=debug
export ZEBRA_RPC_URL=http://192.168.1.100:8232
nozy sync
```

## API server environment variables

The companion API (`api-server`) is localhost-only. Do not publish it as a multi-user hosted wallet.

| Variable | Default | Description |
|----------|---------|-------------|
| `NOZY_API_KEY` | unset | API key (required in production mode) |
| `NOZY_RATE_LIMIT_REQUESTS` | `100` | Max requests per window |
| `NOZY_RATE_LIMIT_WINDOW` | `60` | Window in seconds |
| `NOZY_PRODUCTION` | unset | Stricter security when set |
| `NOZY_HTTP_PORT` | `3000` | HTTP port |
| `NOZY_HTTPS_ENABLED` | unset | Enable HTTPS |
| `NOZY_HTTPS_PORT` | `443` | HTTPS port |
| `NOZY_CORS_ORIGINS` | unset | Comma-separated CORS origins |
| `NOZY_SSL_CERT_PATH` | unset | Certificate path |
| `NOZY_SSL_KEY_PATH` | unset | Private key path |

```bash
export NOZY_API_KEY=your-secret-api-key
export NOZY_PRODUCTION=1
export NOZY_HTTP_PORT=8080
cd api-server
cargo run
```

More: [api-server README](../../api-server/README.md) and [SECURITY_CONFIG.md](../../api-server/SECURITY_CONFIG.md).
