# Companion API - seed / mnemonic policy

**Product decision (localhost companion track):** create/restore may accept a mnemonic over HTTP **only** as a local sidecar. This is intentional, documented, and constrained — not "seed never on the wire."

## Rules

| Binding | Mnemonic create/restore | API key |
|---------|-------------------------|---------|
| `127.0.0.1` / `::1` (default) | Allowed with **non-empty password**; restore needs `confirm_overwrite` if wallet exists | **Required** — from `NOZY_API_KEY` or auto file `{wallet_data}/companion_api_key` |
| `0.0.0.0` / `::` (LAN/hosted) | Same | **Required** (same key resolution; refuse start without key unless `NOZY_ALLOW_UNAUTHENTICATED`) |
| Escape hatch | — | `NOZY_ALLOW_UNAUTHENTICATED=1` disables auth (**not for real funds**) |

Public without API key: `/health` and `/api/lwd/*` only (compact sync companion).

## Guarantees

- Full mnemonic is **never** returned in API responses (masked via `display_mnemonic_safe`).
- Mnemonics must **never** appear in logs, URLs, or error strings.
- Extension / web clients must use loopback (`http://127.0.0.1:3000`) unless the user intentionally runs a hosted companion with API key + TLS.
- Extension Companion settings must paste the companion API key for send/shield/vote/crosslink mutators.

## Out of scope for public claims

- Claiming “seed never touches HTTP”
- Public unauthenticated hosted wallet API
- Formal product **GA** label for companion (engineering may be ready; public GA is a later decision — chain parity sign-off in [`PARITY.md`](PARITY.md); see [`../PRODUCTION_CHECKLIST.md`](../PRODUCTION_CHECKLIST.md))
- Third-party audit (track separately)

## See also

- [`SECURITY_CONFIG.md`](SECURITY_CONFIG.md) - bind address, CORS, rate limits, `NOZY_API_KEY`
- [`../SECURITY_REVIEW.md`](../SECURITY_REVIEW.md) section 4 - companion review checklist
- [`../browser-extension/COMPANION.md`](../browser-extension/COMPANION.md) - extension localhost companion
