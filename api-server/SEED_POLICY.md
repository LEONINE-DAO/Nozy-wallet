# Companion API - seed / mnemonic policy

**Product decision (localhost companion track):** create/restore may accept a mnemonic over HTTP **only** as a local sidecar. This is intentional, documented, and constrained - not "seed never on the wire."

## Rules

| Binding | Mnemonic create/restore | API key |
|---------|-------------------------|---------|
| `127.0.0.1` / `::1` (default), dev | Allowed | Optional (dev convenience) |
| `127.0.0.1` / `::1` with `NOZY_PRODUCTION` | Allowed only with auth | **Required** — process refuses to start without `NOZY_API_KEY` |
| `0.0.0.0` / `::` (LAN/hosted) | Allowed only with auth | **Required** — process refuses to start without `NOZY_API_KEY` |

## Guarantees

- Full mnemonic is **never** returned in API responses (masked via `display_mnemonic_safe`).
- Mnemonics must **never** appear in logs, URLs, or error strings.
- Extension / web clients must use loopback (`http://127.0.0.1:3000`) unless the user intentionally runs a hosted companion with API key + TLS.

## Out of scope for public claims

- Claiming “seed never touches HTTP”
- Public unauthenticated hosted wallet API
- Formal product **GA** label for companion (engineering may be ready; public GA is a later decision — see [`../GRANT_80K_PRODUCTION_CHECKLIST.md`](../GRANT_80K_PRODUCTION_CHECKLIST.md) §2)
- Third-party audit (track separately)

## See also

- [`SECURITY_CONFIG.md`](SECURITY_CONFIG.md) - bind address, CORS, rate limits, `NOZY_API_KEY`
- [`../SECURITY_REVIEW.md`](../SECURITY_REVIEW.md) section 4 - companion review checklist
- [`../browser-extension/COMPANION.md`](../browser-extension/COMPANION.md) - extension localhost companion