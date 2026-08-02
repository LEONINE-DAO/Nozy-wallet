# Desktop v1.0.0-beta.5 — Hot Lemon Pepper Sprinkles (Windows)

Copy into the GitHub Release body when tagging **`desktop-v1.0.0-beta.5`**.

---

## Summary

**NozyWallet Desktop v1.0.0-beta.5 — Hot Lemon Pepper Sprinkles** is the next **Windows beta** after beta.4 (same product name; new build). Food names stay. It remains **pre-release** until GA.

This build stacks **AI-assisted security remediations** (#223) and ZNS / merchant Send work on the Hot Lemon base. The **CLI (Teriyaki Hot v2.4.4)** remains the production surface for operators.

## Security (self-review — not a Least Authority cert)

Desktop inherits shared-core fixes and desktop-specific hardening from [`docs/reference/security-audit/`](https://github.com/LEONINE-DAO/Nozy-wallet/tree/master/docs/reference/security-audit):

- Migrate/broadcast step-up password; force_clearnet / skip_hygiene ignored on IPC
- Notes + migration schedule encrypted at rest (NZN1 / NZS1)
- Path allowlist for backups / LWD DB; restore over existing wallet requires password
- Exact zatoshis for send / Keystone / cosign; session scrub
- Loopback-only “local” node for safer migration privacy

**Out of this audit pass:** browser extension WASM, mobile FFI, upstream circuit soundness.

## What's new since beta.4

- Security remediations above (#223–#225)
- Zcash Names resolve on Send; business profile / merchant dashboard phases
- Vite Dependabot path-traversal bump

## Requirements

- **OS:** Windows 10/11 (x86_64)
- **Node stack:** Zebrad + lightwalletd — Settings → Network
- Same wallet data directory as the CLI (`%APPDATA%\nozy\…`)

## Install

1. Download **`nozy-desktop-windows-x86_64-installer.exe`** (NSIS) from Assets (CI attaches after publish).
2. Install, create/restore wallet, set RPC URLs, Sync, then Send / Ironwood / Sapling quiet flows.

## Known limits

- **Beta** — not third-party audited
- Windows installer is the primary published asset
- Zebrad + lightwalletd not bundled; prefer local Zebrad for migration broadcast

## AI disclosure

Release packaging was **agent-assisted** (Cursor). Human author remains responsible for correctness and security.
