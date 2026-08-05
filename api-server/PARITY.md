# CLI <-> Companion API parity

**Status:** routes exist; field sign-off open.

Use this table during QA against a local `nozywallet-api` (default `http://127.0.0.1:3000`) and matching CLI (`nozy`). Sign off a row only after comparing response fields / error behavior on the same wallet data dir.

| Capability | CLI command or N/A | API route | Sign-off | Notes |
|------------|-------------------|-----------|----------|-------|
| Wallet exists | N/A (inspect data dir / `nozy status`) | `GET /api/wallet/exists` | | |
| Wallet create | `nozy new` | `POST /api/wallet/create` | | Mnemonic on wire allowed on loopback - see [`SEED_POLICY.md`](SEED_POLICY.md) |
| Wallet unlock | N/A (CLI password-on-demand) | `POST /api/wallet/unlock` | | Session unlock is API/desktop/extension |
| Balance | `nozy balance` | `GET /api/balance` | | |
| Sync | `nozy sync` / `nozy sync --to-tip` | `POST /api/sync` | | Checkpoint / height fields may differ |
| Send | `nozy send` | `POST /api/transaction/send` | | |
| Fee estimate | N/A (fee policy inside send) | `GET /api/transaction/fee-estimate` | | Confirm ZIP-317 alignment |
| Speed-up | CLI/desktop alignment via tx lifecycle | `POST /api/transaction/speed-up` | | |
| LWD info | N/A | `GET /api/lwd/info` | | Soft-fail without lightwalletd |
| LWD chain tip | N/A (see `nozy status` tip) | `GET /api/lwd/chain-tip` | | Soft-fail without lightwalletd |
| LWD sync compact | N/A (range via zeaking / internal) | `POST /api/lwd/sync/compact` | | |
| LWD compact-to-tip | `nozy lwd sync-to-tip` | `POST /api/lwd/sync/compact-to-tip` | | |
| Sapling status | `nozy sapling status` | `GET /api/sapling/status` | | Quiet legacy balance |
| Sapling scan | `nozy lwd scan-sapling` | `POST /api/sapling/scan` | | |
| Sapling shield | `nozy sapling shield` | `POST /api/sapling/shield` | | |
| ZNS resolve | Inline in `nozy send` (name to address) | `POST /api/zns/resolve` | | Dedicated API route |

Smoke (health + soft LWD probes): [`scripts/smoke-companion.ps1`](scripts/smoke-companion.ps1) / [`scripts/smoke-companion.sh`](scripts/smoke-companion.sh).

Related: [`SECURITY_CONFIG.md`](SECURITY_CONFIG.md), [`SEED_POLICY.md`](SEED_POLICY.md), [`../browser-extension/COMPANION.md`](../browser-extension/COMPANION.md).