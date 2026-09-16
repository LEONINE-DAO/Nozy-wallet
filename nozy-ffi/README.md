# nozy-ffi

UniFFI bindings for on-device mobile (Zodl-style light wallet): mnemonic, unified address, compact sync, LWD submit, plus quiet **Sapling legacy** status / scan / shield-to-self.

**Issue:** [#208](https://github.com/LEONINE-DAO/Nozy-wallet/issues/208) (follow-up to #200).

## Network model

Mass-user path (no public JSON-RPC):

- **lightwalletd** — compact cache, `GetTreeState`, `SendTransaction` (`https://lwd.nozywallet.org:443`)

Sapling shield-to-self still needs Zebrad JSON-RPC when that path is used.

## Exported API

| Function | Purpose |
|----------|---------|
| `generate_mnemonic()` | New 24-word BIP-39 phrase |
| `validate_mnemonic(mnemonic)` | Reject invalid phrases |
| `orchard_unified_address(mnemonic, account)` | On-device unified address |
| `lwd_get_info(lwd_url)` | Ping lightwalletd |
| `lwd_sync_compact_to_tip(lwd_url, compact_db, start_floor?)` | Compact sync |
| `lwd_get_latest_tree_state(lwd_url)` | Tip treestate (no Zebrad RPC) |
| `lwd_send_transaction(lwd_url, raw_tx_hex)` | Broadcast via `SendTransaction` |
| `sapling_status(wallet_data_dir)` | Quiet legacy balance from persisted notes |
| `sapling_scan(mnemonic, wallet_data_dir, compact_db_path, start_floor?, full)` | Scan compact SQLite for Sapling notes |
| `sapling_shield(mnemonic, wallet_data_dir, compact_db_path, zebra_url, lightwalletd_url, dry_run, no_broadcast)` | Shield-to-self (Groth16 + Halo2) |
| `vote_calendar_info()` | Static NU7 snapshot / vote window |
| `vote_export_notes(mnemonic, wallet_data_dir, network)` | Ironwood notes JSON for `nozy-vote` / desktop |
| `vote_sign_delegation(mnemonic, request_json)` | Sign Valar delegation PCZT request |

**Vote scope:** export + sign only (seed on device). Prepare / PIR / cast need `zcash_voting` and cannot link in this crate beside `zeaking` (sqlite). Finish those steps on **Desktop Vote** or `tools/nozy-vote`.

Errors are `NozyFfiError` with a message string. Never log mnemonics or seeds.

## Build (host)

```bash
cargo build -p nozy-ffi --release
cargo test -p nozy-ffi
```

## Build (Android)

Requires NDK + [cargo-ndk](https://github.com/bbqsrc/cargo-ndk):

```powershell
# From repo root (Windows)
.\scripts\build-nozy-ffi.ps1 -Target android
```

Or:

```bash
cargo ndk -t arm64-v8a -t x86_64 build -p nozy-ffi --release
```

Copy `libnozy_ffi.so` into `nozy-mobile/modules/nozy-wallet/android/src/main/jniLibs/{arm64-v8a,x86_64}/`.

## Generate Kotlin bindings

```powershell
.\scripts\build-nozy-ffi.ps1 -Target host -Bindgen kotlin
```

Or:

```bash
cargo run -p nozy-ffi --features bindgen --bin uniffi-bindgen -- generate --library target/release/nozy_ffi.dll --language kotlin --out-dir nozy-mobile/modules/nozy-wallet/android/src/main/java
```

## Out of scope

- Keystone Sapling
- Outbound Sapling send (`zs1`)
- Full on-device Orchard send proving (compact sync + receive land first; broadcast RPC is ready)
- In-process vote prepare / cast (`zcash_voting` — use desktop or `nozy-vote`)
