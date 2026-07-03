# Code Guidelines

NozyWallet follows Rust ecosystem conventions and [librustzcash](https://github.com/zcash/librustzcash)-style patterns.

## Formatting and lint

```bash
cargo fmt --all
cargo fmt --all -- --check   # CI gate
cargo clippy -- -D warnings
```

All PRs must pass **`cargo fmt --all -- --check`**.

## Style

- **Types:** `PascalCase`
- **Functions / modules:** `snake_case`
- **Errors:** `NozyResult` / `NozyError` — no panics on recoverable paths in library code
- **Crypto:** Use `librustzcash` / `orchard` — no custom ciphers or hand-rolled proofs
- **Secrets:** `zeroize`; never log mnemonics, seeds, or private keys

## Diff discipline

- Minimal scope — no drive-by refactors in bugfix PRs.
- Match surrounding module layout and imports.
- Link GitHub issues for behavior-changing work.

## Testing

```bash
cargo test
```

Add tests when they cover real behavior; avoid trivial assertions.

## Security-sensitive areas

Treat as high impact:

- Key derivation and storage
- Transaction building and signing
- RPC URL handling and broadcast
- Witness derivation

For wallet/protocol/security changes, align with maintainers before large PRs ([`AGENTS.md`](../../../AGENTS.md)).

## AI-assisted contributions

Disclose AI tooling in PR descriptions. Human author remains responsible for correctness. Agents should read [`AGENTS.md`](../../../AGENTS.md) first.

## Pull requests

1. Fork / branch from `master`
2. Run fmt, build, test
3. PR description: what changed, how tested, AI disclosure if any
4. Link related issues

Full process: [`CONTRIBUTING.md`](../../../CONTRIBUTING.md).
