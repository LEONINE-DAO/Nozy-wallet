# Node operator FAQ

Wallet sync is **incremental** and **node-tip-relative**. Nozy does not wait for `zebrad` to reach network tip before scanning or downloading compact blocks. It syncs from your chosen start height (or last scanned height) up to **whatever height your node stack reports now**, then you run sync again as `zebrad` catches up.

Nozy Lite (`nozy health`, `nozy status --watch`, `nozy tui`) is the operator surface. See [`NOZY_LITE.md`](../reference/NOZY_LITE.md) and [`NOZY_LITE_BENCHES.md`](../reference/NOZY_LITE_BENCHES.md).

## Does Nozy require a fully synced Zebra node before wallet sync can begin?

No.

| Activity | Needs full network sync? |
|----------|---------------------------|
| Start scanning / compact download | No — only up to **node tip** |
| See incoming notes in recent blocks | Node must have validated those blocks |
| Shielded send (witness + anchor) | Node must have blocks through your **anchor height** (`z_gettreestate`) |
| Reliable broadcast / mempool | Practically **near network tip** |
| “Balance at chain tip” | Really **balance at node tip** until the node catches up |

## Two sync paths

| Path | Tip source | Used by |
|------|------------|---------|
| Direct RPC scan | `zebrad` `getblockcount` | CLI `nozy sync`, API `/api/sync` |
| Compact / LWD | lightwalletd `GetLatestBlock` | `zeaking::lwd`, API `/api/lwd/sync/compact-to-tip`, extension companion |

Nozy does not check “fully synced”, `verificationprogress`, or lightwalletd `estimatedHeight`. It uses the current served tip.

## Zebrad below wallet birthday

Example: node at ~12k, birthday ~3.07M. That is expected. Nozy scans from birthday (or `last_scan_height+1`) **up to zebrad tip only**. If tip is below birthday, sync errors — Orchard scan cannot run on blocks the node does not have yet. Wallet create and `receive` still work. Balance and notes appear after `zebrad` passes birthday height.

For mainnet pilot testing, use a snapshot near chain tip or use testnet. Do not expect mainnet receive at tip while `zebrad` is at block 12k.

## Minimum to start wallet sync

- `zebrad` JSON-RPC reachable (default `:8232`); `getblockcount` returns a sensible height.
- Recommended: `zebrad` + lightwalletd (gRPC, default `:9067`) on the same network (mainnet or testnet).
- Optional: `nozywallet-api` on `:3000` for desktop and extension LWD routes.

## Practical setup

1. Start `zebrad` (snapshot or from genesis). On Windows, run Zebrad in WSL only — [`scripts/README.md`](../../scripts/README.md).
2. Start lightwalletd once RPC is up (`scripts/start-lightwalletd-wsl.ps1` on Windows). It does not need 100% sync.
3. Create or restore a wallet. Set `--start-height` (wallet birthday) on mainnet so the first scan does not start from the default ~3M height.
4. Run `nozy sync --to-tip` (or LWD compact-to-tip) while `zebrad` keeps indexing. Plain `sync` only advances about 1,000 blocks per run on mainnet.
5. For sends, wait until the node is close to network tip and you have scanned through spendable notes.

Incremental CLI sync without `--start-height` scans about 1,000 blocks per run (testnet default start: height `1`). Shielded-send architecture: [`ZEBRAD_SHIELDED_SEND_LIMIT.md`](../../ZEBRAD_SHIELDED_SEND_LIMIT.md).

## Stale compact cache

If `nozy status` shows compact heights **above** the LWD tip, run `nozy lwd prune` (or `nozy lwd sync-to-tip`, which prunes automatically). Then re-download with `nozy lwd sync-to-tip --start-floor <birthday>`.

## After receiving funds

Run `nozy sync --to-tip`. Plain `sync` only advances about 1,000 blocks per run on mainnet.

## After upgrading to v2.3.3 or later

Run `nozy sync --to-tip` once to repair note nullifiers from older compact discovery. That fixes the second-send double-spend tracked in [issue #61](https://github.com/LEONINE-DAO/Nozy-wallet/issues/61).

## Verify

`nozy status` shows Zebra tip, RPC last scan, LWD tip, and compact-cache height.

Integration tests that need a live node:

```bash
cargo test --test integration_tests -- --ignored test_sync_follows_zebra_tip
cargo test --test integration_tests -- --ignored test_lwd_compact_sync_follows_tip
```

See [`tests/integration_tests.rs`](../../tests/integration_tests.rs). How to run the rest of the suite: [testing](../development/testing.md). Recorded mainnet and pilot results: [evidence index](../evidence/README.md).

## Surfaces

Extension and desktop source are in the repo for contributors. See [`browser-extension/README.md`](../../browser-extension/README.md) and [`desktop-client/README.md`](../../desktop-client/README.md).
