# Dependency management

Versions below match the NU6.2 stack described in the root `Cargo.toml`. Confirm the lockfile before relying on a pin in a script.

## Key crates

| Area | Crates |
|------|--------|
| CLI / async | `clap` 4, `tokio` 1, `reqwest` 0.12 (HTTP, SOCKS/Tor), `thiserror`, `anyhow` (API server) |
| Wallet crypto | `aes-gcm` 0.10, `argon2` 0.5, `zeroize` |
| Zcash | `orchard` 0.14, `zcash_primitives` 0.28, `zcash_protocol` 0.9, `zcash_address` |

Do not hand-roll ciphers or proofs. Use librustzcash / orchard.

## Practices

- Pin security-critical crypto crates.
- Apply advisory overrides when a patch release is required (example previously used: `tracing-subscriber = "0.3.20"` for RUSTSEC-2025-0055).
- Commit `Cargo.lock`. CI checks that it is current.
- Enable only the features you need.
- Review `cargo update` diffs and re-run tests before merging.
- Large Zcash crate bumps need a build of every affected surface (CLI, API, desktop, FFI). See [`AGENTS.md`](../../AGENTS.md).

```bash
cargo install cargo-outdated
cargo outdated

cargo install cargo-audit
cargo audit

cargo update -p crate-name   # or: cargo update
cargo test
cargo build --release
```

CI runs `cargo audit` on pull requests. Prefer weekly local audits and RustSec advisories for anything you ship.
