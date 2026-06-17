//! Crate SemVer comes from `Cargo.toml` (MAJOR.MINOR.PATCH only).
//! Git release tags may add a fourth segment (e.g. **v2.3.6.1**); set [`RELEASE_PATCH`]
//! and [`RELEASE_CODENAME`] when cutting a patch release.

macro_rules! set_release_codename {
    ($name:literal) => {
        /// Marketing / release codename. Change when cutting a new named release.
        pub const RELEASE_CODENAME: &str = $name;
        /// Git/release patch segment; empty when the tag matches crate SemVer exactly.
        pub const RELEASE_PATCH: &str = "";
        /// Tag-style version (e.g. `2.3.6.1`).
        pub const RELEASE_VERSION: &str = env!("CARGO_PKG_VERSION");
        /// `nozy --version`, health checks, and local analytics (release version + codename).
        pub const VERSION_DISPLAY: &str = concat!(env!("CARGO_PKG_VERSION"), " (", $name, ")");
    };
    ($name:literal, patch $patch:literal) => {
        pub const RELEASE_CODENAME: &str = $name;
        pub const RELEASE_PATCH: &str = $patch;
        pub const RELEASE_VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), ".", $patch);
        pub const VERSION_DISPLAY: &str =
            concat!(env!("CARGO_PKG_VERSION"), ".", $patch, " (", $name, ")");
    };
}

set_release_codename!("Teriyaki Hot (CLI)", patch "5");
