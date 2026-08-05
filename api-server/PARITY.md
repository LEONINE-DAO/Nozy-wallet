# CLI <-> Companion API parity

**Status:** routes exist; non-chain local helper available; **chain-backed field sign-off open** (run against your Zebrad/LWD). Public GA claim for companion is deferred — see GRANT §2 internal note.

## How to sign off

1. Use the **same wallet data dir** for CLI and API (default Nozy XDG/profile dir; stop the other surface before switching if both write the same files).
2. Start companion: `nozywallet-api` on `http://127.0.0.1:3000` (dev) or with `NOZY_PRODUCTION=true` + `NOZY_API_KEY` for production-mode checks.
3. Run non-chain probes first:
   - [`scripts/smoke-companion.sh`](scripts/smoke-companion.sh) / [`.ps1`](scripts/smoke-companion.ps1)
   - [`scripts/parity-local.sh`](scripts/parity-local.sh) — exists / status / config / fee-estimate shape
4. For each chain-backed row below, compare CLI output to the HTTP JSON (fields and errors). Mark **Sign-off** with date/initials only after a successful side-by-side check.
5. Do not mark GRANT §2 parity Done until sync, balance, send, and LWD compact-to-tip are signed.

## Capability table

| Capability | CLI command or N/A | API route | Sign-off | Notes |
|------------|-------------------|-----------|----------|-------|
| Wallet exists | N/A (inspect data dir / `nozy status`) | `GET /api/wallet/exists` | | Covered by `parity-local.sh` |
| Wallet create | `nozy new` | `POST /api/wallet/create` | | Mnemonic on wire allowed on loopback - see [`SEED_POLICY.md`](SEED_POLICY.md) |
| Wallet unlock | N/A (CLI password-on-demand) | `POST /api/wallet/unlock` | | Session unlock is API/desktop/extension |
| Balance | `nozy balance` | `GET /api/balance` | | **Chain / wallet state** |
| Sync | `nozy sync` / `nozy sync --to-tip` | `POST /api/sync` | | **Chain** — checkpoint / height fields may differ |
| Send | `nozy send` | `POST /api/transaction/send` | | **Chain** |
| Fee estimate | N/A (fee policy inside send) | `GET /api/transaction/fee-estimate` | | Shape probe in `parity-local.sh`; confirm ZIP-317 vs CLI send |
| Speed-up | CLI/desktop alignment via tx lifecycle | `POST /api/transaction/speed-up` | | |
| LWD info | N/A | `GET /api/lwd/info` | | Soft without lightwalletd |
| LWD chain tip | N/A (see `nozy status` tip) | `GET /api/lwd/chain-tip` | | Soft without lightwalletd |
| LWD sync compact | N/A (range via zeaking / internal) | `POST /api/lwd/sync/compact` | | **Chain / LWD** |
| LWD compact-to-tip | `nozy lwd sync-to-tip` | `POST /api/lwd/sync/compact-to-tip` | | **Chain / LWD** |
| Sapling status | `nozy sapling status` | `GET /api/sapling/status` | | Quiet legacy balance |
| Sapling scan | `nozy lwd scan-sapling` | `POST /api/sapling/scan` | | **LWD** |
| Sapling shield | `nozy sapling shield` | `POST /api/sapling/shield` | | **Chain** |
| ZNS resolve | Inline in `nozy send` (name to address) | `POST /api/zns/resolve` | | Needs network indexer |

Related: [`SECURITY_CONFIG.md`](SECURITY_CONFIG.md), [`SEED_POLICY.md`](SEED_POLICY.md), [`../browser-extension/COMPANION.md`](../browser-extension/COMPANION.md).
