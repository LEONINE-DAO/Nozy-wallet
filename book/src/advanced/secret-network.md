# Secret Network (ZEC + Secret from One Seed)

NozyWallet can optionally derive **Secret Network (SCRT)** keys from the **same BIP39 mnemonic** as your Zcash Orchard wallet — one backup, two ecosystems.

> **Status:** Optional CLI feature. Build with `--features secret-network`. Not enabled in default desktop release builds.


## One seed, two chains

| Chain | What you get |
|-------|----------------|
| **Zcash** | Orchard unified addresses (`u1…`), shielded ZEC |
| **Secret Network** | Native **SCRT** plus optional **generic SNIP-20** by contract address you supply |

The mnemonic is identical; derivation paths differ per chain. **One 24-word backup recovers both** when the feature is enabled.

## Build and enable

```bash
cargo build --release --features secret-network --bin nozy
```

Requires network access to Secret RPC endpoints configured in your environment (see repo research doc below).

## CLI (`nozy secret`)

After building with the feature:

```bash
nozy secret balance              # native SCRT
nozy secret balance --token <contract>   # SNIP-20 at that contract (needs viewing key work)
nozy secret receive              # secret1… deposit address
nozy secret send                 # SNIP-20 send (experimental; signing incomplete)
nozy secret history              # local send history
```

Run `nozy secret --help` for the exact subcommands in your version.

## Desktop and mobile

- **Desktop:** ZEC-first; Secret UI is not the default shipping surface.
- **Mobile / FFI:** Shared `nozy` core patterns; Secret paths follow CLI feature flags.

## Security notes

- Same rules as ZEC: **never share mnemonic**.
- Secret RPC endpoints are trust assumptions — use nodes you control or explicitly trust.
- Optional integration code may lag core ZEC paths; treat as **experimental** until listed in release notes.

## Further reading

- Repository: [`SECRET_NETWORK_RESEARCH_AND_BUILD.md`](../../../SECRET_NETWORK_RESEARCH_AND_BUILD.md) (implementation plan)
- Root README: [Unified privacy wallet](https://github.com/LEONINE-DAO/Nozy-wallet#unified-privacy-wallet-zec--secret-network)
- ZEC setup still required: [Zebra Node Setup](zebra-node.md)
