//! SemVer comes from `Cargo.toml`; the **release codename** is a human-facing label for marketing and support.
//! When you tag a release, update [`RELEASE_CODENAME`] (and bump the crate version as usual).

macro_rules! set_release_codename {
    ($name:literal) => {
        /// Marketing / release codename. Change when cutting a new named release.
        pub const RELEASE_CODENAME: &str = $name;
        /// `nozy --version`, health checks, and local analytics (SemVer + codename).
        pub const VERSION_DISPLAY: &str = concat!(env!("CARGO_PKG_VERSION"), " (", $name, ")");
    };
}

set_release_codename!("Hot Lemon Pepper");
