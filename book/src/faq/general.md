# General Questions

## What is NozyWallet?

A shielded-first Zcash wallet built around Orchard and Zebrad. See [What is Nozy?](../nozy/what-is-nozy.md).

## Does it support transparent ZEC?

No. Orchard shielded only.

## CLI vs desktop vs extension?

| Surface | Status |
|---------|--------|
| CLI (`nozy`) | Primary production |
| Desktop (Tauri) | Pre-release, active development |
| Browser extension | Companion + api-server |
| Mobile | FFI in progress |

## Which network?

Mainnet and testnet — configure via `nozy config` or Settings.

## Do I need my own node?

Strongly recommended. Wallet needs a Zebra-family JSON-RPC node ([Zebrad](https://github.com/ZcashFoundation/zebra) or [Zakura](https://zakura.com/)); see [Zebra Node Setup](../advanced/zebra-node.md) or [Zakura Node Setup](../advanced/zakura-node.md).

## Where is data stored?

Platform app data dir — [Backup & Recovery](../user-guide/backup-recovery.md).

## How do I get help?

[Getting Help](../troubleshooting/getting-help.md)

## Roadmap?

[Contributing → Roadmap](../contributing/roadmap.md) and [ENHANCEMENT_ROADMAP.md](https://github.com/LEONINE-DAO/Nozy-wallet/blob/master/ENHANCEMENT_ROADMAP.md).

## How do I use a Keystone hardware wallet?

1. Open **Settings → Keystone** in the desktop app (or **Keystone** on mobile with the API companion).
2. Export your mainnet UFVK and import it on Keystone (device set to Zcash mainnet).
3. Enable Keystone — sends use **Prepare → sign on device → broadcast**.
4. Step-by-step: [Keystone Hardware Wallet](../security/keystone-hardware-wallet.md).
