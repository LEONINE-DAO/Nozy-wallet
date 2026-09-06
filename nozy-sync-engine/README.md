# Nozy Sync Engine

**GitHub (this crate):** https://github.com/Lowo88/Zeaking

Operator **sync engine** for the NozyWallet stack: **ingests** from Zebra-family JSON-RPC (**Zebrad** or **Zakura**) and **serves** the lightwalletd `CompactTxStreamer` subset that [Zeaking](../zeaking/) (`zeaking::lwd`) already speaks.

**Tracking:** [#274](https://github.com/LEONINE-DAO/Nozy-wallet/issues/274)

```text
Zebrad / Zakura  --JSON-RPC-->  Nozy Sync Engine  --gRPC CompactTxStreamer-->  Zeaking / Nozy
```

Phase 1+2: compact sync, treestate, and optional submit proxy — see [case breakdown](../docs/reference/NOZY_SYNC_ENGINE_CASE_BREAKDOWN.md). Compact `ChainMetadata` tree sizes are filled on ingest (S11).

## Build

Needs `protoc` on `PATH` (same as `zeaking` lightwalletd feature).

```bash
cargo build -p nozy-sync-engine --release
```

## Run

This is a **terminal daemon** (like `zebrad` / `zainod`): it stays in the window until Ctrl+C. On an interactive TTY it prints a **Zeaking** splash of the brand icon (zebra profile + gold shield, no wordmark), version, and thank-you. Preview without a node: `--print-banner`. Skip it with `--no-banner` or `NOZY_SYNC_ENGINE_NO_BANNER=1`. Open `nozy-sync-engine/assets/banner-pixel-preview.png` to see it.

```bash
set ZEBRA_RPC_URL=http://127.0.0.1:8232
# Zakura cookie (optional if file is in the usual path):
# set ZAKURA_RPC_COOKIE=user:password

cargo run -p nozy-sync-engine --release -- --bind 127.0.0.1:9067 --db-path nozy_sync_engine_compact.sqlite
```

Point Nozy / Zeaking at the engine (drop-in for lightwalletd):

```bash
set LIGHTWALLETD_GRPC=http://127.0.0.1:9067
```

### Useful flags / env

| Flag / env | Default | Meaning |
|------------|---------|---------|
| `--rpc-url` / `ZEBRA_RPC_URL` | `http://127.0.0.1:8232` | Node JSON-RPC |
| `--indexer-rpc-url` / `NOZY_SYNC_ENGINE_RPC_URL` | (unset) | Optional RPC URL override |
| `--bind` / `NOZY_SYNC_ENGINE_BIND` | `127.0.0.1:9067` | CompactTxStreamer listen |
| `--db-path` / `NOZY_SYNC_ENGINE_DB` | `nozy_sync_engine_compact.sqlite` | Compact SQLite |
| `--network` / `NOZY_SYNC_ENGINE_NETWORK` | (any) | Require `main` / `test` / `regtest` |
| `--backfill` / `NOZY_SYNC_ENGINE_BACKFILL` | `500` | On empty DB, start this many blocks below tip |
| `--start-height` / `NOZY_SYNC_ENGINE_START_HEIGHT` | `0` | Floor when store is empty |
| `--poll-ms` / `NOZY_SYNC_ENGINE_POLL_MS` | `2000` | Tip poll interval |
| `--no-banner` / `NOZY_SYNC_ENGINE_NO_BANNER` | off | Skip the Zeaking pixel splash |
| `--print-banner` | off | Print the logo and exit (no node needed) |

Auth (same as Nozy): `ZEBRA_RPC_*` / `ZAKURA_RPC_*` user/pass, inline cookie, or `~/.cache/{zebra,zakura}/.cookie`.

## Phase 1 + 2 RPCs

| RPC | Status |
|-----|--------|
| `GetLightdInfo`, `GetLatestBlock`, `GetBlock`, `GetBlockRange`, `Ping` | Phase 1 |
| `GetTreeState`, `GetLatestTreeState`, `GetSubtreeRoots` | Phase 2 (Zebra `z_gettreestate` / `z_getsubtreesbyindex`) |
| `SendTransaction` | Phase 2 (proxies `sendrawtransaction` — prefer local node for privacy) |

Still **UNIMPLEMENTED**: transparent UTXO/balance streams, mempool streams, `GetTransaction`, nullifier-only ranges.

## Smoke (Zeaking)

```powershell
.\scripts\start-nozy-sync-engine.ps1
# other shell:
$env:LIGHTWALLETD_GRPC = "http://127.0.0.1:9067"
cargo run -p nozy-sync-engine --example live_smoke_probe
cargo run -p nozy-sync-engine --example lwd_parity_probe
# ZEBRA_RPC_URL + optional NOZY_PARITY_ENGINE_GRPC; default reference https://zec.rocks:443
.\target\release\nozy.exe lwd sync-to-tip --start-floor <tip-minus-N>
```

**Live evidence (2026-08-20):** WSL Zebrad tip ~3454786 → Sync Engine `:9067` → Zeaking wrote compact blocks; `GetLatestTreeState` returned non-empty Sapling/Orchard trees. See [case breakdown](../docs/reference/NOZY_SYNC_ENGINE_CASE_BREAKDOWN.md).## Nym IP privacy

Nym does **not** replace the sync engine. Hybrid (same as LWD):

| Topology | Nym role |
|----------|----------|
| Colocated node + sync engine + wallet | None — best case |
| Remote Nozy Sync Engine sync | **dVPN** for `GetBlockRange` |
| Small remote RPCs / submit (later) | **Mixnet** — never bulk compact sync over mixnet |

See [NYM_SEND_EGRESS_CASE_BREAKDOWN.md](../docs/reference/NYM_SEND_EGRESS_CASE_BREAKDOWN.md).

## zero-indexer

**Local operators:** skip.  
**Public VPS (later):** put a zero-indexer TEE shim in front of Nozy Sync Engine for Orchard-exit batching + attestation. Full checklist: [NOZY_SYNC_ENGINE_CASE_BREAKDOWN.md — Deferred: public VPS + zero-indexer](../docs/reference/NOZY_SYNC_ENGINE_CASE_BREAKDOWN.md#deferred-public-vps--zero-indexer).

## License

MIT
