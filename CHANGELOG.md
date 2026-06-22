# Changelog

## [Unreleased]

### Fixed

- **Bug 1 — Send rescanned ~50k blocks despite a synced wallet (Gilmore, post note-persistence fix):**
  - **Symptom:** After sync showed healthy state (`balance_zec: 0.0025`, `last_sync_height` ~25 blocks behind tip), `POST /api/transaction/send` immediately logged `Scanning blocks 3333582 → 3383606` (~50,025 blocks) instead of reusing the known spendable note.
  - **Root cause:** `scan_notes_for_sending()` always rewound **50,000 blocks** from `last_scan_height` before building a transaction — a pre-persistence safety net that ignored cached `notes.json` and persisted Orchard witnesses.
  - **Fix:** Send now loads spendable notes directly from `notes.json` via `load_spendable_notes_from_wallet()`. Witness catch-up to chain tip still happens at spend-build time (only the blocks behind tip, not a historical rescan). Fallback scan runs only when the cache is empty or notes lack witnesses, and uses incremental bounds (100-block reorg rewind) instead of 50k.
  - **Expected after fix:** Send reuses existing wallet state / spendable notes; at most a small incremental scan when cache is missing.

- **Bug 2 — Transaction history empty despite persisted balance (Gilmore, post note-persistence fix):**
  - **Symptom:** `/api/sync` and `/api/wallet/status` showed correct balance and sync height, but `GET /api/transaction/history` returned `{ "transactions": [], "total": 0 }` — no received deposit entry.
  - **Root cause:** History endpoints read only `SentTransactionStorage` (outgoing txs recorded at broadcast). Received deposits live in persisted `notes.json` and were never merged into history views.
  - **Fix:** Added `collect_wallet_transaction_views()` to merge sent records with received deposits grouped by txid from `notes.json`. `/api/transaction/history`, `/api/transaction/{txid}`, `/api/wallet/status` (`total_transactions`), and `web_read_state` now use this merged view. Responses include `transaction_type: "Received"` or `"Sent"`.
  - **Expected after fix:** At least the detected deposit appears in history (txid, amount, block height, confirmations).

- **Bug 3 — Send broadcast fails when Orchard proving outruns pilot expiry (Gilmore, VPS):**
  - **Symptom:** Orchard bundle built, proof generated, and tx signed, but `sendrawtransaction` rejected with Zebrad `-25`: chain tip past `expiry_height` (e.g. tip 3385384, expiry 3385380). Balance unchanged — tx never entered mempool.
  - **Root cause:** Expiry clock started at the beginning of `build_single_spend` (before witness fetch + Halo2 proving). The 5-block pilot window is shorter than proving latency on slow VPS/WSL hosts. History also recorded `tip + 5` while the tx encoded `(tip + 1) + 5`.
  - **Fix:** Late tip refresh before encoding expiry; auto-rebuild when proving outruns expiry (up to 3 attempts); broadcast retry on expiry `-25`; unified `build_and_broadcast_send_transaction()` across CLI, api-server, and desktop; history uses on-chain `expiry_height` from the signed tx. **Pilot expiry stays at 5 blocks** (~6 min) so users get fast expire/fail feedback for speed-up UX — slow-host reliability comes from rebuild/retry, not a longer window.
  - **Detail:** [`docs/issues/bugs/2026-06-send-expiry-before-broadcast.md`](docs/issues/bugs/2026-06-send-expiry-before-broadcast.md) (BUG-2026-011).

## [2.3.6.5] — Teriyaki Hot (CLI) (2026-06-17)

Patch on **v2.3.6.4**. Crate SemVer remains **2.3.6**; `nozy --version` reports **2.3.6.5 (Teriyaki Hot (CLI))**.

### Fixed

- **`/api/balance` zero after successful sync:** when `notes.json` is empty but `last_scan_height` already equals chain tip, sync skipped scanning ("already synced") and balance stayed 0. Empty cache now triggers a full historical rescan to tip so on-chain funds are discovered and persisted.

## [2.3.6.4] — Teriyaki Hot (CLI) (2026-06-17)

Patch on **v2.3.6.3**. Crate SemVer remains **2.3.6**; `nozy --version` reports **2.3.6.4 (Teriyaki Hot (CLI))**.

### Fixed

- **Remote VPS Zebra connect (`ZEBRA_RPC_UNREACHABLE`):** the configured `zebra_url` is now treated as a trusted operator endpoint (direct RPC, no Tor hop). Fixes sync failing at `phase: connect` when `require_privacy_network` is true and Tor is not running — even without duplicating the URL in `trusted_zebra_urls`. URL matching ignores `http` vs `https` on the same host:port. `/api/sync` errors include `connection_mode` for diagnosis. `POST /api/config/set-zebra-url` auto-adds remote URLs to `trusted_zebra_urls`.

## [2.3.6.3] — Teriyaki Hot (CLI) (2026-06-17)

Patch on **v2.3.6.2**. Crate SemVer remains **2.3.6**; `nozy --version` reports **2.3.6.3 (Teriyaki Hot (CLI))**.

### Added

- **`trusted_zebra_urls`:** operator-controlled allowlist for direct Zebra RPC when `require_privacy_network` is true (VPS/infra nodes without disabling privacy globally).
- **Structured Zebra connect errors:** `PRIVACY_POLICY_BLOCKED`, `TOR_PROXY_UNREACHABLE`, `ZEBRA_RPC_UNREACHABLE` on sync connect phase and test-zebra failures.
- **`POST /api/config/test-zebra`:** returns JSON with `connection_mode`, `block_height`, and error `code` (same RPC path as sync).

### Changed

- **Unified Zebra client:** test-zebra, sync, send, confirmations, and wallet status all use `ZebraClient::from_config_with_url()` — privacy/Tor policy matches real operations (no more false-green test endpoint).

### Fixed

- **Wallet history confirmations:** Zebra `getrawtransaction` expects numeric verbosity `1` (not boolean `true`) and returns block height as `height` (not `blockheight`). Confirmed txs no longer stay Pending/Expired in history.
- **Wrong expiry on mined txs:** reconcile wrongly-expired broadcasts during check-confirmations; skip expiry when RPC errors (don't treat errors as "not on chain").

## [2.3.6.2] — Teriyaki Hot (CLI) (2026-06-17)

Patch on **v2.3.6.1**. Crate SemVer remains **2.3.6**; `nozy --version` reports **2.3.6.2 (Teriyaki Hot (CLI))**.

### Added

- **`nozy::wallet_sync`:** unified note scan → merge → persist orchestrator; api-server and background sync use shared path.
- **Desktop History:** Speed up button on expired transactions (rebuild at priority ×4 fee via Tauri `speed_up_transaction`).
- **Extension:** Pilot speed-up rebuilds a new priority transaction in WASM (replaces wrong rebroadcast retry for expired txs); optional companion API path when `companionPassword` is supplied.
- **Dynamic-fee pilot speed-up:** `POST /api/transaction/speed-up` rebuilds a new priority transaction after expiry (not a rebroadcast). Core logic in `nozy::tx_lifecycle`.
- **Expired transaction detection:** pending pilot txs past `expiry_height` are marked `Expired` and release locally locked notes via `/api/transaction/check-confirmations`.

### Changed

- **Extension:** `wallet_retry_broadcast` kept for failed pre-broadcast retries only; expired txs use `wallet_speed_up`.
- **`/api/sync` errors:** structured JSON (`phase`, `block_height`, `scan_start`/`scan_end`, `code`) instead of opaque HTTP 500; Zebra outages return **503** `ZEBRA_UNAVAILABLE`.
- **Orchard scan logging:** per-action decrypt chatter is quiet by default; set `NOZY_VERBOSE_SCAN=1` or `RUST_LOG=nozy::notes=debug` for detail. API/non-TTY builds skip indicatif progress spam.

### Fixed

- **Zebra RPC during scan:** retry transient `getblock` transport failures (`error decoding response body`, truncated reads) instead of failing the whole `/api/sync` on first glitch.
- **`/api/sync` repeat calls:** when already caught up to chain tip, return cached balance without rescanning; serialize concurrent syncs; richer sync response (`balance_zatoshis`, `already_synced`, `total_notes`).
- **api-server `/api/sync` + `/api/balance` mismatch:** sync merges scan results into cached `notes.json` (instead of overwriting), fails if save fails, updates `last_scan_height` to the scanned range, and returns the same unspent balance that `/api/balance` reads.
- **api-server send:** mark notes spent in `notes.json` after broadcast (match CLI).
- **Transaction history note marking:** use v2 `notes.json` index format when marking spent notes.

## [2.3.6.1] — Teriyaki Hot (CLI) (2026-06-15)

Patch on **v2.3.6**. Crate SemVer remains **2.3.6** (Cargo); `nozy --version` reports **2.3.6.1 (Teriyaki Hot (CLI))**.

### Fixed

- **api-server `/api/transaction/send`:** check cached `notes.json` balance before Zebra rescan; return `success: false` with **Insufficient funds** instead of HTTP 500 when balance is too low. Zebra connection failures during send rescan return **503** with `code: ZEBRA_UNAVAILABLE`.
- **`/api/balance`:** exclude spent notes from balance sum (match CLI `balance` behavior).

## [2.3.6] — Teriyaki Hot (CLI) (2026-06-14)

### Changed

- **Release policy:** GitHub releases attach **CLI binaries only** (`nozy` for Windows, Linux, macOS Intel/ARM). Desktop, extension, and API server remain in the repo for contributors but are not promoted on release pages until production-ready.
- **Documentation:** README and landing page download sections point to CLI only.

### Fixed

- **CI:** sync desktop Tauri `Cargo.lock` after version bumps; patch `esbuild` npm audit on desktop-client.
- **Repo:** remove legacy broken `temp-electron-client` submodule entry (CI cleanup noise).

## [2.3.5] — Teriyaki Hot (2026-06-14)

### Fixed

- **api-server `/api/transaction/send` address validation:** rejected valid mainnet unified addresses (`u1...`) longer than 100 characters (Nozy-generated UAs are often ~106 chars). Validation now uses shared `validate_zcash_address` (up to 256 chars), matching CLI and core wallet behavior.

## [2.3.4] — Send Select (2026-06-10)

### Fixed

- **Orchard single-note coin selection:** `build_single_spend` only creates one Orchard spend, but change was computed from the **sum of all** spendable notes. Wallets with multiple notes built txs whose outputs exceeded the spent note; zebrad rejected broadcast with `could not calculate the transaction fee` (code `-25`). Now selects the smallest note that covers amount + fee and derives change from that note only (`select_single_spend_note`).

### Changed

- **Desktop + extension stack alignment:** desktop Tauri app version `2.3.4`; extension `0.1.5` uses shared `fee_policy` (ZIP-317 fees, 5-block expiry constant) via wasm-core; tx build height `tip + 1` matches CLI/desktop; wasm `Cargo.lock` tracks core `2.3.4`.

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
