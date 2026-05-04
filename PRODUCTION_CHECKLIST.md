# NozyWallet — production readiness checklist

Single cross-repo tracker. Check items off as you complete them; link PRs or issues in the right column when useful.

| Area | Priority | Status | Notes / owner |
|------|----------|--------|----------------|
| **Security & privacy** | P0 | Partial | Execute [`SECURITY_REVIEW.md`](SECURITY_REVIEW.md) before major releases. No telemetry without opt-in (`AGENTS.md`). Responsible disclosure: `CONTRIBUTING.md`. |
| **Secrets & logging** | P0 | Ongoing | Grep for accidental logging of seeds, RPC bodies, or full addresses in extension + native. |
| **Core library (`nozy`)** | P0 | Partial | `cargo fmt --all`, `cargo test` on CI. **Clippy:** full `-D warnings` for `nozy` is not CI-enforced yet (large cleanup); CI enforces strict clippy on `zeaking` + `nozywallet-api --no-deps`. Orchard-only scan flag (`NOTE_SCAN_INCLUDE_SAPLING`) — document product stance. |
| **Zebra / RPC parity** | P0 | Improved | CLI uses `getblockhash` → `getblock`; extension now matches (`rpc-utils.js`). Re-test full scan on Zebrad after each release. |
| **Browser extension (MV3)** | P0 | Partial | WASM `wasm-pack --release` in release pipeline. Service worker: scan resume, alarms, update handler (stale “scanning”). **Scan errors:** consecutive failures abort scan + `lastRpcError` shown in Receive while scanning. Optional: **import** Desktop / `notes.json` into extension state. |
| **Extension — store / distribution** | P1 | Open | Real screenshots (`browser-extension/docs/screenshots/`). Privacy policy + store listing. `STORE_SUBMISSION_CHECKLIST.md` if present. CRX signing not in CI — Web Store / Edge Add-ons path. |
| **`api-server` (companion)** | P1 | Open | Bind to localhost by default; auth / TLS if exposed beyond loopback. CORS and rate limits documented. |
| **Desktop (Tauri)** | P1 | Open | Code signing, updater channel, IPC surface review. Align balance/sync with core Orchard-only product rules. |
| **`zeaking` / LWD** | P1 | Open | Document supported `lightwalletd` + Zebra versions; CI that builds workspace members. |
| **`zeaking-ffi` (mobile)** | P2 | Open | Same sync/send invariants as desktop; release + consumer app integration. |
| **Testing** | P1 | Partial | WASM: `browser-extension/wasm-core` unit tests + `tests/fixtures/*.json` for Zebrad-shaped `getblock` Orchard scan (`orchard_scan.rs`). Integration tests against Zebra (ignored in CI unless runner). |
| **Documentation** | P1 | Open | One “start here” path: **Desktop vs extension** (`browser-extension/README.md`). Runbook: RPC unreachable, wrong network, scan range vs birthday, witness mismatch. |
| **Dependencies** | P2 | Open | `cargo audit` (see open `RUSTSEC` for `rustls-webpki` / transitive crates); Dependabot; extension `npm audit`. No casual Zcash crate bumps without full matrix build. |
| **Release engineering** | P1 | Open | `.github/workflows/*`: `ci.yml`, `extension-ci.yml`, `extension-release.yml`, `desktop-release.yml`. Changelog + version bump discipline. |

## CI (`.github/workflows/ci.yml`)

- **Rustfmt:** `cargo fmt --all -- --check` (fail PR if misformatted).
- **Clippy (strict):** `cargo clippy -p zeaking --all-targets --all-features -- -D warnings` and `cargo clippy -p nozywallet-api --all-targets --all-features --no-deps -- -D warnings`.
- **Tests:** `cargo test --lib --bins` (fail on test failure).
- **Release builds:** `cargo build --release` for `nozy` CLI and `nozywallet-api` (fail on compile error).
- **`cargo audit`:** still **soft-fail** until transitive `rustls-webpki` (and related) advisories are cleared from the lockfile.

## Recently completed (keep short; move to git history over time)

- Extension: verbose block fetch via `getblockhash` + `getblock` (Zebrad-aligned with `ZebraClient`).
- Extension: removed localhost debug ingest calls from `service-worker.js`.
- Extension: scan aborts after many consecutive block/RPC errors; popup shows `lastRpcError` while scanning.
- `zeaking` + `api-server`: clippy clean under `-D warnings`; strict clippy wired in CI for those packages.
- WASM core: Zebrad-shaped block JSON fixtures + round-trip decrypt/witness test for `orchard_scan_tracker_apply_block_json`.

## How to use this file

1. Pick a **P0** row, open an issue if the work is large or ambiguous (`AGENTS.md` contribution gate).
2. When a row is done, set **Status** to **Done** and add the PR link in **Notes**.
3. Before a major release, skim the whole table and run the **Build, format, and test** block from `AGENTS.md`.
