# Vendored dependency note

`libcrux-psq` 0.0.8 is vendored here solely to apply a one-line rustc 1.88
lifetime fix (`E0716` in `registration.rs`). Upstream crate is Apache-2.0
(Cryspen / libcrux). Remove this vendor directory and the `[patch.crates-io]`
entry in the parent `Cargo.toml` once Nym depends on a fixed release.
