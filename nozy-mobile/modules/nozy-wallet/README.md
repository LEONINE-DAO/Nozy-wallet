/**
 * Thin Expo module for `libnozy_ffi` (UniFFI).
 *
 * Android: generate Kotlin bindings after a host or NDK build:
 *   `.\scripts\build-nozy-ffi.ps1 -Target host -Bindgen kotlin`
 *   `.\scripts\build-nozy-ffi.ps1 -Target android`
 *
 * iOS staticlib is not linked yet (needs macOS). `nativeLibReady` is false there.
 */

