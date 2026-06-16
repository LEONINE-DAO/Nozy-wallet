//! Scan/decrypt logging: quiet by default for API server and plain/CI output.
//!
//! Enable per-action decrypt chatter with `NOZY_VERBOSE_SCAN=1`, or use
//! `RUST_LOG=nozy::notes=debug` for `tracing::debug` output without println spam.

use std::io::IsTerminal;

/// Verbose per-action decrypt logging (`NOZY_VERBOSE_SCAN=1`).
pub fn verbose_scan_logging() -> bool {
    std::env::var("NOZY_VERBOSE_SCAN").as_deref() == Ok("1")
}

/// Interactive indicatif progress bar (TTY and not plain output).
pub fn scan_progress_enabled() -> bool {
    if std::env::var("NOZY_PLAIN_OUTPUT").is_ok() {
        return false;
    }
    std::io::stdout().is_terminal()
}

#[macro_export]
macro_rules! scan_verbose {
    ($($arg:tt)*) => {{
        if $crate::scan_log::verbose_scan_logging() {
            println!($($arg)*);
        } else {
            tracing::debug!($($arg)*);
        }
    }};
}
