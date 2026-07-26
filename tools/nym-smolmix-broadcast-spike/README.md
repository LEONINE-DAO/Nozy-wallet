# nym-smolmix-broadcast-spike

Isolated binary + library for **Nozy Priority 1 biggest win**: prove **Nym mixnet (`smolmix`) egress** and submit `sendrawtransaction` so a remote node cannot link **host IP ↔ submitted tx**.

- Case breakdown: [`docs/reference/NYM_MIXNET_BROADCAST_CASE_BREAKDOWN.md`](../../docs/reference/NYM_MIXNET_BROADCAST_CASE_BREAKDOWN.md)
- Living checklist: [`docs/reference/NYM_IP_PRIVACY_CASE_BREAKDOWN.md`](../../docs/reference/NYM_IP_PRIVACY_CASE_BREAKDOWN.md)
- Forum draft: [`docs/reference/NYM_MIXNET_BROADCAST_FORUM_ARTICLE.md`](../../docs/reference/NYM_MIXNET_BROADCAST_FORUM_ARTICLE.md)
- Tracking: [issue #147](https://github.com/LEONINE-DAO/Nozy-wallet/issues/147)
- Wallet hook: [`src/nym_mixnet_broadcast.rs`](../../src/nym_mixnet_broadcast.rs)
- Evidence script: [`scripts/nym-smolmix-d2-evidence.ps1`](../../scripts/nym-smolmix-d2-evidence.ps1)
- Excluded from the root Cargo workspace (invoked as a **subprocess** to avoid sqlite `links` clash with zeaking)

## Modes

| Mode | Purpose | Status |
|------|---------|--------|
| `--dry-reachability` | Classify LAN refuse vs exit-reachable (no tunnel) | **PASS** (tests + CLI) |
| `--ip-relocate` | Cloudflare trace clearnet vs smolmix | **PASS** 2026-07-11 |
| `--rpc-probe` | `getblockcount` over mixnet | Needs exit-reachable Zebrad |
| `--both` | ip-relocate then rpc-probe | Same |
| `--sendraw <hex>` / `--sendraw-stdin` | `sendrawtransaction`; prints **txid** on stdout | Wallet helper |
| `--evidence-json <path>` | Write structured evidence for case breakdowns | Landed |

## Wallet wiring (D2c)

```powershell
cd tools/nym-smolmix-broadcast-spike
cargo build --release

$env:NOZY_NYM_SMOLMIX_BIN = (Resolve-Path .\target\release\nym-smolmix-broadcast-spike.exe).Path
$env:NOZY_BROADCAST_VIA_NYM_MIXNET = "1"
# Optional: $env:NOZY_NYM_IPR = "<Recipient>"
# Config alternative: privacy_network.broadcast_via_nym_mixnet = true

nozy privacy-network nym-mixnet
```

Remote `zebra_url` only — local/LAN stays direct (Case A1). Remote URL must be reachable from a Nym exit.

## Probe / relocate / evidence

```powershell
cargo run --release -- --dry-reachability --zebra http://127.0.0.1:8232
cargo run --release -- --ip-relocate --evidence-json ..\..\docs\reference\evidence\nym-d2a.json
cargo run --release -- --rpc-probe --zebra https://EXIT_REACHABLE_HOST:18232 --evidence-json ..\..\docs\reference\evidence\nym-d2b.json
```

Or from repo root:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\nym-smolmix-d2-evidence.ps1 -IpRelocate
```

## AI disclosure

Spike assisted by Cursor Agent. Link #147 on any PR.
