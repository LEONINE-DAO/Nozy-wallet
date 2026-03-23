# Zebrad (Zebra) and shielded sends — wallet + node roadblock

**Update:** The wallet now implements **client-side Orchard witnesses** using `z_gettreestate` checkpoints, **`incrementalmerkletree` `IncrementalWitness`**, and chain-ordered Orchard `cmx` values from **full blocks during scan** (and optional **compact-block replay** via `zeaking::lwd` for root checks). The prove path **no longer calls** `z_findnoteposition` / `z_getauthpath` on `ZebraClient`. You still need **JSON-RPC** to Zebra for `z_gettreestate` during scan/spend; **gRPC-only** Zebra transport does not populate real treestates yet.

### Clarification: what actually broke sends

The practical failure mode was **wrong witness and anchor data** in Nozy (placeholder `ZebraClient` logic feeding the prover), **not** “Zebra is missing two RPCs that appear in old zcashd-oriented docs.” Zebra exposes **`z_gettreestate`** / **`z_getsubtreesbyindex`** for tree state; the fix is to use those for checkpoints and roots and to **derive Merkle paths in the wallet** (same direction as light-wallet stacks and the newer [zcash/wallet](https://github.com/zcash/wallet) work on **Zallet**—not a 1:1 copy of every historical zcashd wallet RPC). Framing the issue only around `z_findnoteposition` / `z_getauthpath` was misleading for contributors comparing against repos that never centered on those methods.

---

This document originally explained **why** NozyWallet could sync against Zebrad + lightwalletd while shielded sends assumed **zcashd-only witness RPCs**. The historical analysis below is kept for context; the **blocked** table row is **obsolete** once you run a **JSON-RPC scan** that stores incremental witnesses on notes.

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
| **Building and signing a shielded send** with notes scanned via **JSON-RPC** + stored witnesses | Uses **`ZebraJsonRpcOrchardWitnessProvider`** + **`z_gettreestate`** for anchor checks — **no** `z_getauthpath` / `z_findnoteposition` |

## Ways forward (project direction)

1. **Near-term / operational:** If witness or anchor checks fail, **rescan** with JSON-RPC Zebra from a height where `z_gettreestate` returns Orchard `finalState`, or replay from **`LwdCompactStore`** using `nozy::orchard_chain_tree` helpers when compact blocks cover the range.
2. **Ongoing:** Optional **gRPC treestate** for Zebra (if exposed later), subtree verification polish, and faster witness catch-up when the wallet is far behind the chain tip.

## Related docs in this repo

- **`zeaking/README.md`** — lightwalletd feature, `protoc`, integration surfaces  
- **`browser-extension/COMPANION.md`** — extension + localhost API (sync); does not remove the send RPC gap  
- **`api-server/README.md`** — HTTP API prerequisites  
- **`scripts/README.md`** — running `nozywallet-api` with `LIGHTWALLETD_GRPC`

External reference (node-focused detail): community Zebrad / Zebra setup guides may duplicate this table; the **source of truth for Nozy’s product stance** is this file.
