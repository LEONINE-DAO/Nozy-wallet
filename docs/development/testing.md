# Testing

Unit tests do not need a node. Integration tests that talk to Zebra are ignored by default.

## Commands

```bash
# Unit tests
cargo test

# Integration tests (Zebra required)
cargo test --test integration_tests -- --ignored

# One test
cargo test test_wallet_creation

# Show println output
cargo test -- --nocapture
```

Format and lint (CI):

```bash
cargo fmt --all -- --check
cargo clippy -- -D warnings
```

From the repository root. Desktop and `browser-extension/wasm-core` use their own manifests.

## Integration setup

1. Run Zebra on testnet:

```bash
cargo install zebrad
zebrad start --network testnet --rpc-listen-addr 127.0.0.1:8232
```

2. Optional environment:

```bash
export ZEBRA_RPC_URL=http://127.0.0.1:8232
export NOZY_TEST_NETWORK=testnet
```

3. Run ignored tests:

```bash
cargo test --test integration_tests -- --ignored
```

Tip-following checks used by operators:

```bash
cargo test --test integration_tests -- --ignored test_sync_follows_zebra_tip
cargo test --test integration_tests -- --ignored test_lwd_compact_sync_follows_tip
```

## What the suite covers

- Unit: wallet creation, addresses, password protection, error paths
- Integration: Zebra connection, note scanning, transaction building
- End-to-end: create, scan, and send workflows where a node is available

## CI

Pushes and pull requests run format check (`cargo fmt --all -- --check`), clippy, `cargo audit`, unit tests, and release build. `cargo chec` can run check, clippy, fmt, and test together; see [`CARGO_CHEC_SETUP.md`](../../CARGO_CHEC_SETUP.md).

Recorded mainnet, pilot, and network-privacy runs are not part of `cargo test`. They are indexed in [evidence](../evidence/README.md).
