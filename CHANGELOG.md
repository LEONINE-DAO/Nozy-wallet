# Changelog

## [2.3.3] — Teriyaki Hot (NU6.2) (2026-06-10)

### Fixed

- **Orchard spend detection (#61):** derive and store canonical note nullifiers at discovery; detect on-chain spends during scan; mark notes spent immediately on broadcast. Fixes second-send double-spend (duplicate nullifier / `-25`) when change notes were not excluded from the spendable set.
- **Release lockfiles:** refresh `Cargo.lock` when bumping crate version (CI `cargo metadata --locked`).

## [2.3.2] — NU6.2 mainnet compatibility (2026-06-10)

### Fixed

- **NU6.2 consensus branch ID:** bump librustzcash to orchard 0.14, zcash_primitives 0.28, zcash_protocol 0.9, and related crates so `BranchId::for_height` returns NU6.2 (`0x5437f330`) on mainnet. Prevents zebrad rejection with incorrect consensus branch id (code -25).
- **Transaction expiry:** default expiry delta raised from 2 to 5 blocks for Orchard sends.
- **ZIP-317 Orchard fees:** count actions as `max(spends, outputs)` instead of `spends + outputs` (correct conventional fee for typical 1-in / 1-out sends).
- **api-server, desktop, extension wasm-core:** align `zcash_protocol` and lockfiles with the NU6.2 dependency set.

### Changed

- Extension CI runs when root `Cargo.toml` or `src/**` changes so wasm-core stays in sync with the main crate.

## [2.3.1] — Receive / sync UX (2026-06-05)

### Added

- CLI `nozy sync --to-tip` — scan from last scanned height through chain tip (recommended after receiving funds)
- CLI `nozy lwd prune` and `nozy lwd sync-to-tip` — compact cache maintenance
- `nozy status` sync section: Zebra tip, RPC scan gap, LWD tip, compact cache (with stale-cache warning)
- Integration tests: `test_sync_follows_zebra_tip`, `test_lwd_compact_sync_follows_tip` (live node, `#[ignore]`)

### Fixed

- **Shielded Labs pilot:** plain `sync` only advances ~1000 blocks; deposits in newer blocks were missed until `--to-tip` (see discussion #37)
- Stale `lwd_compact.sqlite` rows above LWD tip — auto-prune on compact-to-tip; `nozy lwd prune`
- Release dispatch for desktop/extension assets after tag publish
- `nozy status` works without interactive wallet unlock for node/sync section

### Changed

- `receive` and post-`sync` output warn when chain tip is ahead of scanned range

## [2.3.0] — Priority Lane (2026-06-03)

### Added

- **Dynamic fee pilot (Phase A1):** client-side ZIP-317 conventional fees for Orchard sends
- CLI `nozy send --priority` — opt-in fee multiplier ×4 (Shielded Labs pilot alignment)
- Short transaction expiry (~2 blocks from chain tip at build time)
- Desktop and api-server: priority flag on send and live fee estimates (replaces hardcoded slow/normal/fast ZEC amounts)

### Changed

- Send flows no longer rely on Zebrad `estimatefee` (unimplemented; previously fell back to 10_000 zats)
- `transaction_history` records `priority` and `expiry_height` on pilot sends

### Not in this release

- Extension WASM fee alignment, expired-tx polling, speed-up rebuild, Zeaking-shared `fee_policy` (see [docs/rfcs/DYNAMIC_FEE_PHASE_A_IMPLEMENTATION.md](docs/rfcs/DYNAMIC_FEE_PHASE_A_IMPLEMENTATION.md))

## [2.2.3] — Hot Lemon Pepper

Prior release. See git history and [GitHub Releases](https://github.com/LEONINE-DAO/Nozy-wallet/releases).
