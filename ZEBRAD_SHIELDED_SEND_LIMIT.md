# Zebrad (Zebra) and shielded sends — wallet + node roadblock

This document explains **why NozyWallet can sync and receive against Zebrad + lightwalletd** but **shielded ZEC sends** still hit a **capability gap** between what our wallet expects and what Zebrad exposes over JSON-RPC. It records how we reached this roadblock so users and contributors share the same mental model.

## What we shipped in NozyWallet

We consolidated **lightwalletd compact-block sync** in Rust as **`zeaking`** (`zeaking::lwd`): gRPC to lightwalletd, SQLite cache of compact blocks, and wiring to:

- **Desktop** (Tauri `lwd_*` commands)
- **`api-server`** (`/api/lwd/*` for localhost JSON)
- **`zeaking-ffi`** (mobile)
- **Chrome / Microsoft Edge** (MV3 extension → **companion** `nozywallet-api` on `127.0.0.1`, not gRPC in the browser)

That path is designed so **indexing / scanning / receiving** does **not** depend on zcashd-only RPCs like `z_findnoteposition` or `z_getauthpath` for *fetching chain data*.

## What Zebrad (Zebra) does not provide

**[Zebra](https://github.com/ZcashFoundation/zebra)** (what people run as **Zebrad**) is a **full node**, not a zcashd-compatible wallet server. It **does not implement**:

- `z_findnoteposition`
- `z_getauthpath`

Those methods exist on **zcashd** and are used by many shielded wallet stacks to obtain **Merkle witness data** when **proving Orchard (or Sapling) spends** in flows that still delegate witness lookup to the node.

## Where the roadblock appears

NozyWallet’s **send / prove** path for **shielded** transactions has historically been built around **JSON-RPC to the node** (our `ZebraClient` and related Orchard transaction building). Parts of that stack still assume **witness-related RPCs** that **Zebrad will never answer** in the zcashd sense.

So in practice:

| Stage | Zebrad + lightwalletd + zeaking / companion |
|--------|---------------------------------------------|
| Node sync, chain tip, `sendrawtransaction` | Works when RPC is configured correctly |
| Compact sync, receiving shielded notes (light-client style) | **Improved** with zeaking + lightwalletd |
| **Building and signing a shielded send** if the code path still calls **`z_getauthpath` / `z_findnoteposition`** | **Blocked** on Zebrad-only — those methods are missing |

This is **not** “lightwalletd isn’t running” or “the extension can’t reach port 3000.” It is a **deliberate architectural difference** between Zebra and zcashd, plus **wallet code that still expects zcashd-style witness RPCs** for spends.

## Ways forward (project direction)

1. **Near-term / operational:** Point the wallet at **zcashd** (or any node that exposes the needed witness RPCs) for the **prove / spend** step while continuing to use Zebrad + lightwalletd for **sync** if desired.
2. **Proper fix in NozyWallet:** Evolve the Orchard spend path to **derive witnesses entirely in the client** from **compact blocks** and local wallet state (same direction as `zcash_client_backend` / `zcash_client_sqlite`), then use Zebrad only for **`sendrawtransaction`** (and whatever read RPCs we need). **zeaking** is a foundation for sync storage, not a substitute for that prove refactor until it is wired end-to-end.

## Related docs in this repo

- **`zeaking/README.md`** — lightwalletd feature, `protoc`, integration surfaces  
- **`browser-extension/COMPANION.md`** — extension + localhost API (sync); does not remove the send RPC gap  
- **`api-server/README.md`** — HTTP API prerequisites  
- **`scripts/README.md`** — running `nozywallet-api` with `LIGHTWALLETD_GRPC`

External reference (node-focused detail): community Zebrad / Zebra setup guides may duplicate this table; the **source of truth for Nozy’s product stance** is this file.
