# Scanning performance

NozyWallet can fetch blocks in parallel while scanning and can cache blocks to avoid repeat RPC calls.

## Parallel fetch

The scanner defaults to **5 blocks in parallel** (allowed range 1–50).

```rust
let mut scanner = NoteScanner::new(wallet, zebra_client);
scanner.set_parallel_blocks(10);
scanner.enable_block_cache(); // cache blocks for 1 hour
```

Earlier notes in the project README estimated roughly:

- Sequential: ~100 ms per block (~100 s for 1,000 blocks)
- 5-wide: ~20 s for 1,000 blocks
- 10-wide: ~12 s for 1,000 blocks

Treat those figures as order-of-magnitude notes from that write-up, not a current benchmark. Measured CLI and desktop numbers that were actually timed live in [`NOZY_LITE_BENCHES.md`](../reference/NOZY_LITE_BENCHES.md). Mainnet send timings: [`MAINNET_SEND_READINESS_EVIDENCE.md`](../reference/MAINNET_SEND_READINESS_EVIDENCE.md).

## Block cache

`enable_block_cache()` keeps blocks for one hour so a rescan of the same range hits the cache instead of the node.

## Incremental scan

```rust
let scanner = NoteScanner::with_index_file(wallet, client, &index_path)?;
scanner.scan_notes(Some(last_height + 1), None).await?;
```

CLI users should prefer `nozy sync --to-tip` after a receive. See [CLI usage](../guides/cli-usage.md).
