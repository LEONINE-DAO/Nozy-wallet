# zeaking-ffi

UniFFI bindings for **[`zeaking::lwd`](../zeaking)** — the same lightwalletd sync surface as **Tauri** (`lwd_*` commands) and **`api-server`** (`/api/lwd/*`), for **iOS** and **Android** native apps.

## Exported API

| Function | Purpose |
|----------|---------|
| `lwd_get_info(url)` | `GetLightdInfo` |
| `lwd_chain_tip(url)` | Tip height (`GetLatestBlock`) |
| `lwd_sync_compact(url, db_path, start, end?)` | Stream compact blocks into SQLite |

Errors are returned as `ZeakingFfiError` with a message string (maps from `ZeakingError`).

## Build the native library

From the Nozy-wallet repo root:

```bash
cargo build -p zeaking-ffi --release
```

Artifacts (platform-dependent):

- Linux: `target/release/libzeaking_ffi.so`
- macOS: `target/release/libzeaking_ffi.dylib`
- Windows: `target/release/zeaking_ffi.dll`

iOS: use `cargo build --release --target aarch64-apple-ios` (and `x86_64-apple-ios` for simulators if needed) with your Rust iOS toolchain.

Android: use the NDK targets, e.g. `aarch64-linux-android`, `armv7-linux-androideabi`, `i686-linux-android`, `x86_64-linux-android`.

## Generate Kotlin (Android)

Install the matching UniFFI bindgen (same minor version as the `uniffi` dependency in `Cargo.toml`):

```bash
cargo install uniffi_bindgen --locked --version 0.28.0
```

Then:

```bash
uniffi-bindgen generate --library target/release/libzeaking_ffi.so \
  --language kotlin \
  --out-dir ../android/app/src/main/java/uniffi/zeaking_ffi
```

(On macOS use `libzeaking_ffi.dylib`; on Windows use `target/release/zeaking_ffi.dll`.)

Wire the generated Kotlin and load the `.so` from your app’s `jniLibs` as usual for UniFFI.

## Generate Swift (iOS)

```bash
uniffi-bindgen generate --library target/release/libzeaking_ffi.dylib \
  --language swift \
  --out-dir ./bindings/swift
```

Add the generated Swift to your Xcode target and link `libzeaking_ffi.a` / framework per your Rust-iOS setup (e.g. `cargo-xcode` or `apple-crate` workflows).

## Configuration

Pass the same gRPC base URL you use for desktop (`LIGHTWALLETD_GRPC`), e.g. `http://127.0.0.1:9067`. Store `db_path` under the app sandbox (e.g. `filesDir/lwd_compact.sqlite` on Android, `Application Support` on iOS).
