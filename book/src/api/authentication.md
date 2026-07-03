# Authentication

## Model

- **Localhost companion** — api-server trusts whoever can reach the port.
- **Encrypted wallet** — sensitive routes require `password` in JSON body to unlock in-memory session for the request or session lifetime.

## Unlock

```http
POST /api/wallet/unlock
Content-Type: application/json

{ "password": "your-wallet-password" }
```

## Create / restore

```http
POST /api/wallet/create
POST /api/wallet/restore
```

Body includes password and optional mnemonic on restore.

## Security guidance

- Bind to **127.0.0.1** only.
- Do not expose port 3000 to the internet without reverse proxy, TLS, and strong auth redesign.
- Extension: restrict to packaged extension origins via companion design.

## Desktop contrast

Tauri uses OS process isolation + invoke IPC — no HTTP password on wire for normal desktop use.

## Related

- [Wallet Endpoints](wallet-endpoints.md)
- [API Server Setup](../advanced/api-server.md)
