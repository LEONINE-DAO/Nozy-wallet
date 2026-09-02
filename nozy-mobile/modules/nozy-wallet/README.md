/**
 * Thin React Native package for `libnozy_ffi`.
 *
 * Build the `.so` and Kotlin bindings from the repo root:
 *   `.\scripts\build-nozy-ffi.ps1 -Target android -Bindgen kotlin`
 *
 * Place `libnozy_ffi.so` under `android/src/main/jniLibs/{abi}/` and expose a
 * `NativeModules.NozyFfi` TurboModule / Expo module that forwards to UniFFI.
 *
 * Issue: https://github.com/LEONINE-DAO/Nozy-wallet/issues/208
 * NU7 vote export/sign: https://github.com/LEONINE-DAO/Nozy-wallet/issues/273
 *
 * After rebuilding nozy-ffi, expose `voteCalendarInfo`, `voteExportNotes`,
 * and `voteSignDelegation` on NativeModules.NozyFfi (same UniFFI surface).
 */
