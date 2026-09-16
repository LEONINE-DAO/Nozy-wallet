# ZSA testnet — Nozy wiring

**Branch / worktree:** `feat/zsa-testnet` (`NozyWallet-zsa-testnet`)  
**Public net:** dedicated OrchardZSA network (NU 6.2), **not** Zcash mainnet / regular testnet / Ironwood.

| Endpoint | URL |
|----------|-----|
| lightwalletd | `https://zsa.methyl.cc` |
| Faucet (tZEC) | `https://faucet.zsa.methyl.cc` |
| Address HRP | `uregtest1` |

Forum: [ZSA Testnet](https://forum.zcashcommunity.com/t/zsa-testnet/56884) · Deployed Zebra tip often on `zcash-shielded-assets/zebra` `merge` (e.g. `6a91831…`).

## CLI (Sprint 0 — landed)

```bash
cargo run --release -- zsa status
cargo run --release -- zsa addresses
cargo run --release -- zsa balance
# Stubs until OrchardZSA crate:
cargo run --release -- zsa issue --name zUSD --amount 100
cargo run --release -- zsa transfer -r uregtest1… -a 10
```

Override LWD: `NOZY_ZSA_LWD` (does **not** inherit `LIGHTWALLETD_GRPC` — prevents mixing chains).

## Isolation rule

Mainline Nozy pins **Ironwood** `orchard 0.15-pre` + librustzcash rev for ZEC.  
OrchardZSA needs **QEDIT** `orchard` + `librustzcash` with `zsa-issuance` (see `QED-it/zcash_tx_tool` patches).

Those pins **must not** replace Ironwood deps in the root workspace. Use the excluded package **`nozy-zsa/`** (own `Cargo.toml` / lock / `[patch]`).

## Sprint roadmap

0. **Done:** status / addresses / stub issue+transfer against public LWD  
1. **Next:** `nozy-zsa` crate — scan tZEC, issue `zUSD`, transfer, burn if supported  
2. Zone B wrap banners + Base lockbox (zUSD product plan)  
3. ZEC ↔ zUSD inventory swap in Nozy  

## Shielded Loop Charter

Wrap/unwrap USDC is Zone B (counterparty). In-pool zUSD send / ZEC↔zUSD swap is Zone A. See plan: Private zUSD + Shielded Loop Charter.
