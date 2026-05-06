# Zebrad (Zebra) shielded-send architecture

NozyWallet targets a Zebrad-first stack with local witness derivation.

## Current model

- Chain data and compact sync come from `lightwalletd` via `zeaking::lwd`.
- Wallet scan/replay keeps Orchard witness state locally.
- Anchor checks use Zebra JSON-RPC treestate (`z_gettreestate`).
- Transaction broadcast uses standard node RPC (`sendrawtransaction`).

## Supported runtime stack

- **Node:** Zebrad
- **Compact service:** lightwalletd
- **Wallet surfaces:** CLI, desktop (`api-server`), extension companion, mobile (`zeaking-ffi`)

## Send path requirements

For reliable shielded sends:

1. Scan from a valid treestate checkpoint.
2. Persist local incremental witnesses per discovered spendable note.
3. Advance witnesses to target height in chain order.
4. Verify witness root against anchor from `z_gettreestate`.
5. Build proof and broadcast raw transaction.

## Troubleshooting

- If witness/anchor mismatch occurs, rescan from a trusted checkpoint height.
- If compact history is already cached, replay cached compact blocks to rebuild tree state.
- Ensure JSON-RPC and lightwalletd are both reachable from the wallet surface you are testing.

## Related docs

- `zeaking/README.md` - compact sync and storage surfaces
- `browser-extension/COMPANION.md` - extension localhost companion flow
- `api-server/README.md` - local HTTP API and prerequisites
- `scripts/README.md` - helper scripts for local companion startup
