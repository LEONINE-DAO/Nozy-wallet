# Contributing Guide

Thank you for contributing to NozyWallet — an Orchard-first, shielded-only Zcash wallet.

## Where to start

1. Read [`AGENTS.md`](../../../AGENTS.md) (machine-readable policy for coding agents).
2. Read [`CONTRIBUTING.md`](../../../CONTRIBUTING.md) (human contributor guide).
3. Set up the repo: [Development Setup](development-setup.md).
4. Pick work from [Roadmap](roadmap.md) or [GitHub Issues](https://github.com/LEONINE-DAO/Nozy-wallet/issues).

## Contribution gate (important changes)

Before large or behavior-changing work (new pool, RPC surface, sync architecture, crypto):

1. **Open or reference a GitHub issue** describing intent.
2. **Disclose AI assistance** in the PR if tools helped write or review code.
3. Be ready to explain interaction with **Zebrad JSON-RPC**, **lightwalletd**, and **witness/anchor** flows.

See [`ZEBRAD_SHIELDED_SEND_LIMIT.md`](../../../ZEBRAD_SHIELDED_SEND_LIMIT.md) for node vs wallet responsibilities.

## Code of conduct

- Respectful, technical discussion
- Security and privacy first
- Responsible disclosure for vulnerabilities (no public exploit issues)

## What we need help with

- Desktop UI polish and release hardening
- Browser extension milestones
- Mobile via `zeaking-ffi`
- Documentation (this book!)
- Zebrad operator docs and test coverage

## Pull request checklist

- [ ] `cargo fmt --all` and `cargo test` pass
- [ ] Scope matches issue / description
- [ ] No secrets in logs or commits
- [ ] AI disclosure in PR body if applicable

## Repository map (quick)

| Area | Path |
|------|------|
| Core + CLI | `src/`, root `Cargo.toml` |
| Compact sync | `zeaking/` |
| HTTP API | `api-server/` |
| Desktop | `desktop-client/` |
| Extension WASM | `browser-extension/wasm-core/` |

## Questions

[GitHub Discussions](https://github.com/LEONINE-DAO/Nozy-wallet/discussions) or issue comments.
