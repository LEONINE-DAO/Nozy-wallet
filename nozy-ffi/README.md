# nozy-ffi

UniFFI bindings for quiet **Sapling legacy** status / scan / shield-to-self — the same core path as CLI `nozy sapling`, Tauri, and `api-server` `/api/sapling/*`.

**Issue:** [#208](https://github.com/LEONINE-DAO/Nozy-wallet/issues/208) (follow-up to #200).

## Network model

On-device proving still needs:

- **Zebrad JSON-RPC** — treestate / anchors / broadcast (`zebra_url`)
- **lightwalletd** — compact cache for witnesses (`lightwalletd_url` + `compact_db_path`)

This crate does **not** implement LWD-only treestate.

## Exported API

| Function | Purpose |
|----------|---------|
| `sapling_status(wallet_data_dir)` | Quiet legacy balance from persisted notes |
| `sapling_scan(mnemonic, wallet_data_dir, compact_db_path, start_floor?, full)` | Scan compact SQLite for Sapling notes |
| `sapling_shield(mnemonic, wallet_data_dir, compact_db_path, zebra_url, lightwalletd_url, dry_run, no_broadcast)` | Shield-to-self (Groth16 + Halo2) |

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

```bash
cargo install uniffi_bindgen --locked --version 0.28.0
uniffi-bindgen generate --library target/release/libnozy_ffi.so \
  --language kotlin \
  --out-dir nozy-mobile/modules/nozy-wallet/android/src/main/java/uniffi/nozy_ffi
```

(On Windows host builds use `target/release/nozy_ffi.dll`.)

## Out of scope

- Keystone Sapling
- Outbound Sapling send (`zs1`)
- Full on-device Orchard send (separate from this shield path)
