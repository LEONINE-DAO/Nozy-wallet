# Product surfaces, consensus, and features

## Surfaces

| Surface | Path | Role |
|--------|------|------|
| CLI + core library (Nozy Lite) | `nozy` (`src/`, root `Cargo.toml`) | Wallet logic, ops health/TUI, `ZebraClient`, transaction building |
| Zeaking | `zeaking/` | Compact sync client → SQLite (`zeaking::lwd`) |
| Nozy Sync Engine | `nozy-sync-engine/` | Zebra/Zakura ingest + CompactTxStreamer for Zeaking ([#274](https://github.com/LEONINE-DAO/Nozy-wallet/issues/274)) |
| API server | `api-server/` | Localhost HTTP companion (`nozywallet-api`) for same-machine use |
| Desktop | `desktop-client/` | Tauri app |
| Browser extension | `browser-extension/` | MV3 + WASM; compact sync via the companion API |
| Mobile | `nozy-mobile/` + `zeaking-ffi/` | Expo shell and UniFFI LWD bindings (in progress) |
| Landing site | `landing/` | Marketing site, not the wallet |

**Production-ready today:** the `nozy` CLI ([Nozy Lite](../reference/NOZY_LITE.md)), the desktop app, and the localhost companion API. Extension and mobile stay in active development. Hosted/public companion is not a product claim. There is no formal “GA” label for desktop or the API.

## Privacy stance

Shielded-first: Orchard / unified shielded receivers. Transparent `t1` is blocked for sends (`src/privacy.rs`). Orchard proofs use Halo 2 inside the Orchard stack (no legacy Sapling parameter download).

Operational privacy still depends on how you run the node, the network path, and the device. The wallet does not claim network-wide anonymity.

## NU6.2 mainnet (v2.3.2+)

NozyWallet tracks mainnet NU6.2 (consensus branch ID `0x5437f330`) via the librustzcash set pinned in the root `Cargo.toml`:

- `orchard 0.14`, `zcash_primitives 0.28`, `zcash_protocol 0.9`
- NU 6.1 activated at block **3,146,400** (November 23, 2025). NU6.2 is active on mainnet. The node must be NU6.2-aware (for example zebrad 5.x).

Wallet and node must agree on branch ID. A stale dependency stack can emit an NU6.1 branch ID; zebrad then rejects broadcast with code `-25` (“incorrect consensus branch id”).

```bash
nozy nu61      # historical helper for NU 6.1 activation / protocol info
nozy status    # node and wallet sync summary
```

Ironwood (NU6.3) readiness lives in [`IRONWOOD_WALLET_READINESS.md`](../reference/IRONWOOD_WALLET_READINESS.md).

## Shipped wallet behavior

- NU6.2 mainnet alignment (v2.3.2+)
- ZIP-317 client-side fees; `nozy send --priority` for the pilot priority lane (×4)
- Spend detection: canonical nullifiers at discovery; on-chain spend marking during scan and on broadcast (v2.3.3, [#61](https://github.com/LEONINE-DAO/Nozy-wallet/issues/61))
- `nozy sync --to-tip`
- BIP39 HD wallet, Argon2 password, Orchard address generation
- Zebra RPC, note scanning, transaction build and broadcast
- Encrypted backup and recovery
- Unified-address parsing, Orchard Merkle paths, bundle authorization

## Secret Network (optional)

One BIP39 seed can derive ZEC (Orchard) and Secret Network (native SCRT; SNIP-20 by contract address) when the CLI is built with `--features secret-network`.

```bash
cargo build --release --features secret-network
nozy secret balance
nozy secret receive
```

Book chapter: [Secret Network](../../book/src/advanced/secret-network.md).

## Roadmap

Full list: [ENHANCEMENT_ROADMAP.md](../../ENHANCEMENT_ROADMAP.md).

In development (build from source):

- Desktop — [desktop README](../../desktop-client/README.md)
- Extension + companion API — [COMPANION.md](../../browser-extension/COMPANION.md)
- Mobile — `zeaking-ffi` / `nozy-mobile`, [roadmap](../../ENHANCEMENT_ROADMAP.md)

Contributor focus areas: [where to help](../development/where-to-help.md).
