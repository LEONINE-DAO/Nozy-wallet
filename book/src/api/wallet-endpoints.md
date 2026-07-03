# Wallet Endpoints

Base: `http://127.0.0.1:3000`

## `GET /api/wallet/exists`

Check if wallet file exists.

## `POST /api/wallet/create`

Create wallet. Body: `{ "password": "…" }`. Returns mnemonic in response — handle securely.

## `POST /api/wallet/restore`

Body: `{ "mnemonic": "…", "password": "…" }`.

## `POST /api/wallet/unlock`

Body: `{ "password": "…" }`.

## Status

Some builds expose wallet lock state via additional routes — check OpenAPI or `api-server/src` handlers in repo.

CLI equivalents: `nozy new`, `nozy restore`.

See [`api-server/README.md`](../../../api-server/README.md).
