# Security Audits

NozyWallet handles keys, mnemonics, and shielded transaction data. Security review is ongoing; this page summarizes **current status** and how to verify dependencies locally.

## Dependency auditing

Run from the repository root:

```bash
cargo audit
cargo fmt --all -- --check
cargo clippy -- -D warnings
```

CI enforces formatting on pushes and PRs. Use `cargo audit` after dependency changes.

## Documented dependency fixes

Internal tracking ([`AUDIT_STATUS.md`](../../../AUDIT_STATUS.md) in the repo) records dependency updates including:

- `curve25519-dalek`, `ed25519-dalek`, `tracing-subscriber` — updated to patched versions (Dec 2025 cycle).

Verify with `cargo audit`; expect **zero known vulnerabilities** on supported builds or only unmaintained-crate warnings.

## Scope of review

| Area | Status |
|------|--------|
| Core Orchard send path (`nozy` crate) | Primary focus; production CLI |
| Desktop Tauri shell | Pre-release; operator testing |
| api-server | Localhost companion; not internet-exposed by default |
| Extension WASM | Separate build; excluded from root workspace |
| Optional features (`secret-network`, Monero, swap) | Experimental; may not build on all feature sets |

## Responsible disclosure

Do **not** file public GitHub issues for undisclosed exploits. Follow [`CONTRIBUTING.md`](../../../CONTRIBUTING.md) and contact maintainers privately for suspected vulnerabilities.

## Self-audit guide

Contributors can use [`SELF_SECURITY_AUDIT_GUIDE.md`](../../../SELF_SECURITY_AUDIT_GUIDE.md) for checklist-style review (address validation, RPC trust, storage encryption, etc.).

## Third-party audits

No independent third-party audit certificate is claimed in this book. For grant or production submissions, attach:

- `cargo audit` output
- Relevant BUG registry entries ([`docs/issues/BUG_REGISTRY.md`](../../../docs/issues/BUG_REGISTRY.md))
- Mainnet evidence docs ([`docs/reference/MAINNET_SEND_READINESS_EVIDENCE.md`](../../../docs/reference/MAINNET_SEND_READINESS_EVIDENCE.md))

## Roadmap

Formal external audit before broad retail release is tracked on [Enhancement Roadmap](https://github.com/LEONINE-DAO/Nozy-wallet/blob/master/ENHANCEMENT_ROADMAP.md#8-enhanced-security-features).
