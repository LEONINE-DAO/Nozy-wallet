# Error Messages

Desktop maps backend errors to short codes in `desktop-client/src/utils/errors.ts`. CLI often prints the raw Rust message.

## Wallet & auth

| Code | Message | Typical cause |
|------|---------|---------------|
| AUTH_001 | Incorrect password | Wrong unlock password |
| AUTH_002 | Please enter your password | Empty password field |
| WALLET_001 | Wallet is locked | Unlock before send/sync |
| WALLET_002 | No wallet found | Create or restore first |
| WALLET_003 | Invalid recovery phrase | Bad mnemonic word or order |
| WALLET_005 | Wallet already exists | Create on machine that already has one |

## Send

| Code | Message | Typical cause |
|------|---------|---------------|
| SEND_001 | Insufficient balance | Amount + fee > spendable |
| SEND_002 | Invalid recipient address | Not a valid unified Orchard address |
| SEND_003 | Invalid amount | Non-positive or unparsable amount |
| SEND_004 | Transaction failed | RPC, proving, or expiry error |

## Network

| Code | Message | Typical cause |
|------|---------|---------------|
| NET_001 | Cannot connect to node | Zebrad down, wrong URL, RPC disabled |
| NET_002 | Request timed out | Slow or overloaded node |
| NET_003 | Network error | Generic fetch failure |
| NET_004 | Node unavailable | RPC error from Zebrad |

## Sync & proving

| Code | Message | Typical cause |
|------|---------|---------------|
| SYNC_001 | Sync failed | Node unreachable or scan error |
| PROVE_001 | Proving failed | Missing or corrupt proving params |

## Backend

| Code | Message | Typical cause |
|------|---------|---------------|
| BACKEND_001 | Feature not available | Tauri command not registered; rebuild app |
| RUNTIME_001 | Use desktop window | Browser preview without Tauri IPC |

## CLI-specific messages

| Message | Meaning |
|---------|---------|
| `Failed to parse config.json` | Invalid JSON or UTF-8 BOM — fix `config.json` or upgrade (BOM stripped in current `load_config`) |
| `Orchard witness is N blocks behind` | Sync/witness catch-up required |
| `nExpiryHeight` / `-25` | Transaction expired before broadcast — sync and retry |
| `No spendable notes found` | Scan not run or wrong profile |

## Zebra RPC codes

`nozy` maps connect failures to structured codes via `zebra_connect_api_code()` for API clients. Use `nozy test-zebra` for a human-readable diagnosis first.

---

Fix guides: [Common Issues](common-issues.md) | [Zebra Node Setup](../advanced/zebra-node.md)
