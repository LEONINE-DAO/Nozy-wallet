# Later: AI agents on companion API / web companion

**Status:** Idea only — not scheduled. Do not claim in grant/forum copy until designed.

## Hook

AI agents (Cursor, scripts, custom tools) can call the same HTTP surface as the Web companion:

```text
AI agent / tool
  →  http://127.0.0.1:3000  (nozywallet-api)
  →  wallet / LWD / ZNS routes
```

Web companion (`web-app/`) is one UI on that API; agents are another client class.

## Good first uses

- Health, wallet exists, config, balance, sync status, LWD tip, fee estimate, ZNS resolve
- Operator “is my stack healthy?” agents

## Hard requirements before agent *spend*

- `NOZY_PRODUCTION` + `NOZY_API_KEY` (or stricter)
- Human confirmation for unlock / send / restore
- Loopback or locked-down host only — never an open public agent with spending keys
- Explicit tool allowlist (read-only vs spend)

## Related

- [`README.md`](README.md) — operator / download
- [`SEED_POLICY.md`](SEED_POLICY.md) — mnemonic on wire policy
- [`SECURITY_CONFIG.md`](SECURITY_CONFIG.md) — auth / bind
- [`../web-app/README.md`](../web-app/README.md) — web companion preview
- Grant smoke proof screenshot: [`scripts/companion-api-smoke-proof.png`](scripts/companion-api-smoke-proof.png)
