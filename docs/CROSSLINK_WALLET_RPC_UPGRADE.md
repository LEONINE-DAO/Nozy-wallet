# Crosslink node upgrade — `get_wallet_sync_status`

Nozy shows **Available to stake** (unstaked node wallet cTAZ) when your Crosslink monolith exposes the `get_wallet_sync_status` JSON-RPC.

Older v13 builds return **Method not found** for that call. The RPC was added in [Shielded Labs crosslink_monolith PR #51](https://github.com/ShieldedLabs/crosslink_monolith/pull/51) (branch `USCMig:add-wallet-sync-status-rpc` against `s1_dev`).

## Quick probe (Windows)

From the repo root:

```powershell
.\scripts\Invoke-CrosslinkWalletRpcProbe.ps1
```

Optional RPC URL:

```powershell
.\scripts\Invoke-CrosslinkWalletRpcProbe.ps1 -RpcUrl http://127.0.0.1:18232
```

Or with Nozy CLI (after pointing config at your node):

```bash
nozy crosslink wallet
nozy crosslink wallet --json
```

If the RPC is missing, Nozy prints upgrade instructions and the desktop Crosslink tab shows an amber notice.

## What you get after upgrade

| Field | Meaning |
|-------|---------|
| `user_shielded_spendable_zats` | Shielded cTAZ ready to stake |
| `user_unshielded_zats` | Transparent balance (if any) |
| `user_shielded_pending_zats` | Incoming shielded (not yet spendable) |
| `staked_zats` | Node view of bonded principal |
| `withdrawable_zats` | Finished unbonds awaiting withdraw |
| `sync_height` / `tip_height` | Wallet scan progress |

**Available to stake** in Nozy = `user_shielded_spendable_zats + user_unshielded_zats` (same as the monolith GUI spendable line).

This is **not** bond rewards (`latest_val − initial_val`). Rewards stay on the bond until you unbond and withdraw.

## Upgrade options

1. **Wait for merge** — When PR #51 lands on `s1_dev` / a tagged release, install that build from [Shielded Labs releases](https://github.com/ShieldedLabs/crosslink_monolith/releases).

2. **Build from PR branch** (testers) — Clone `crosslink_monolith`, check out the PR branch, build per upstream README, enable RPC in `config.toml`:

   ```toml
   [rpc]
   listen_addr = "127.0.0.1:18232"
   enable_cookie_auth = false
   ```

3. **Keep your wallet.dat** — Stop the old node, replace the binary, start the new one with the same data directory. Re-sync may take a while; balances can read low until `sync_height` catches `tip_height`.

## Nozy surfaces

| Surface | Command / path |
|---------|----------------|
| CLI | `nozy crosslink wallet` · `nozy crosslink status` (includes `wallet`) |
| API | `GET /api/crosslink/wallet-status` · `GET /api/crosslink/status` |
| Desktop | Crosslink tab → **Available to stake** hero |
| Extension | XL tab → status card |

Related: `get_wallet_ufvk` for Season 1 ZEC payout — `nozy crosslink ufvk` (works on current v13 builds).
