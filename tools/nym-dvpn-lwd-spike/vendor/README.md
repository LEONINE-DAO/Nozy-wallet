# Vendored dependency note

## `libcrux-psq`

`libcrux-psq` 0.0.8 is vendored here solely to apply a one-line rustc 1.88
lifetime fix (`E0716` in `registration.rs`) and to allow
`libcrux-chacha20poly1305 = "=0.0.8"` (GHSA-hc3c-63hc-2r9f). Upstream crate is
Apache-2.0 (Cryspen / libcrux). Remove this vendor directory and the
`[patch.crates-io]` entry in the parent `Cargo.toml` once Nym depends on a
fixed release.

## `nym-kkt` (+ ciphersuite / context)

Upstream Nym `develop` still pins `libcrux-chacha20poly1305 = "0.0.7"` in its
workspace. For Cargo 0.0.x semver, `^0.0.7` / `"0.0.7"` cannot select `0.0.8`,
so Dependabot alert #151 cannot be cleared with `cargo update` alone.

These three crates are vendored from nymtech/nym `@78281fb8` (same rev as the
spike lock) with only the chacha pin changed to `=0.0.8`, and are applied via
`[patch."https://github.com/nymtech/nym.git"]`. Drop the patch when upstream
Nym bumps past the vulnerable chacha crate.
