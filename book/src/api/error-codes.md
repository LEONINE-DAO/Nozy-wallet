# Error Codes

API and desktop share user-facing codes where mapped.

## Network

| Code | Meaning |
|------|---------|
| NET_001 | Cannot connect to Zebrad |
| NET_002 | Timeout |
| NET_003 | Generic network error |
| NET_004 | Node unavailable |

## Wallet / auth

| Code | Meaning |
|------|---------|
| AUTH_001 | Wrong password |
| WALLET_001 | Locked |
| WALLET_002 | Not found |
| WALLET_003 | Invalid mnemonic |

## Send / sync

| Code | Meaning |
|------|---------|
| SEND_001 | Insufficient balance |
| SEND_002 | Invalid address |
| SYNC_001 | Sync failed |
| PROVE_001 | Proving failed |

## Backend

| Code | Meaning |
|------|---------|
| BACKEND_001 | Command not registered (desktop) |
| RUNTIME_001 | Non-Tauri host |

Full desktop map: `desktop-client/src/utils/errors.ts`.

Human guide: [Error Messages](../troubleshooting/error-messages.md).

Zebra connect codes: `nozy::zebra_connect_api_code()` in Rust backend.
