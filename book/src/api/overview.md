# API Overview

NozyWallet HTTP API is served by **`api-server`** on port **3000** by default.

## Base URL

```
http://127.0.0.1:3000
```

Bind to localhost only in production companion setups.

## Authentication

Wallet operations require unlock with password in POST body where the wallet is encrypted. No OAuth — local trust model.

See [Authentication](authentication.md).

## Response shape

JSON with `success`, `message`, and optional `data` / error `code`. Mirrors Tauri error patterns where possible.

## Endpoint groups

| Chapter | Routes |
|---------|--------|
| [Wallet](wallet-endpoints.md) | `/api/wallet/*` |
| [Transactions](transaction-endpoints.md) | `/api/transaction/*` |
| [Addresses](address-endpoints.md) | `/api/address/*` |
| [Balance & Sync](balance-sync-endpoints.md) | `/api/balance`, `/api/sync`, `/api/lwd/*` |
| [Configuration](config-endpoints.md) | `/api/config/*` |
| [Proving](proving-endpoints.md) | proving status / download |
| [Error Codes](error-codes.md) | Desktop + API codes |

## Companion pattern

Chromium MV3 extensions should **not** run gRPC/SQLite in the service worker. Call this server from the extension with `host_permissions` for `127.0.0.1:3000`.

## Setup

[API Server Setup](../advanced/api-server.md)

Source: [`api-server/README.md`](../../../api-server/README.md)
