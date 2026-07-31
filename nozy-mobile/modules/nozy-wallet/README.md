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
 */
