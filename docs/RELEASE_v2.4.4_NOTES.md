# NozyWallet v2.4.4 — Teriyaki Hot (CLI)

**Tag:** `v2.4.4`  
**Date:** 2026-08-02  
**Surface:** CLI (`nozy`) — production download path

---

## What's new since v2.4.3

### Security — AI-assisted self-review remediations (#223–#225)

We ran a Least Authority–**style** AI-assisted self-review of CLI + shared core + Desktop, then triaged and fixed findings in-tree. **This is not a Least Authority certificate.** Pack: [`docs/reference/security-audit/`](https://github.com/LEONINE-DAO/Nozy-wallet/tree/master/docs/reference/security-audit).

Highlights:

- Migration broadcast binds submit client to assessed privacy (proxy / Nym) — no clearnet submit after Tor gate
- `notes.json` encrypted at rest (**NZN1** AES-GCM + Argon2id); vault KDF upgraded to **NZK2** Argon2id
- Migration schedule encryption (**NZS1**); wipe presigned hex after broadcast/confirm
- “Local” node for safer migration = **loopback only**
- api-server default bind `127.0.0.1`; refuse `0.0.0.0` without API key
- Path allowlist, exact zatoshis amounts, schedule rebuild preserves Broadcast/Confirmed
- HDWallet no longer clones secrets; Drop zeroizes

### Also landed on master

- Zcash Names (ZNS) resolve across Send surfaces; business / merchant dashboard work
- Sapling quiet-legacy companion API foundations (status / scan / shield)
- Note-index CI fixes for NZN1 encryption (#224 / #225)

## Downloads

| Platform | File |
|----------|------|
| Windows | `nozy-windows.exe` |
| Linux | `nozy-linux` |
| macOS Intel | `nozy-macos-intel` |
| macOS Apple Silicon | `nozy-macos-arm` |

Verify with `HASHES.txt` or `.sha256` sidecars.

## Prerequisites

Zebrad JSON-RPC + lightwalletd gRPC on the same network. Prefer local loopback for migration broadcast.

## Honest boundaries

- CLI remains the production-ready surface for operators.
- Desktop / extension / mobile ship under their own tags (same food-name families).
- Audit scope did **not** include extension WASM, mobile FFI, or upstream circuit soundness.
