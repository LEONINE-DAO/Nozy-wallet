//! Terminal splash — zcashd-style pixel art in the TTY (not a GUI window).
//!
//! zcashd converted a PNG with `img2txt` and printed it at start. We do the same
//! with half-block (`▀`/`▄`) truecolor pixels of the Nozy Wallet logo.

use std::io::{self, IsTerminal, Write};

use crate::banner_ansi::{BANNER_ANSI, BANNER_ASCII};

/// Operator-facing brand for this process (crate remains `nozy-sync-engine`).
pub const PRODUCT: &str = "Zeaking";

pub fn splash_text(version: &str, rpc_url: &str, bind: &str, color: bool) -> String {
    let art = if color { BANNER_ANSI } else { BANNER_ASCII };
    format!(
        "\n{art}\n\n  {PRODUCT}  compact sync engine  v{version}\n  Thank you for using Zeaking.\n  Keeping nosy people out of Nozy.\n\n  node RPC : {rpc_url}\n  gRPC     : {bind}   (point LIGHTWALLETD_GRPC here)\n  stop     : Ctrl+C\n\n"
    )
}

/// Print the splash. `force` is for `--print-banner` (show even when piped).
pub fn print_startup_banner(
    no_banner: bool,
    force: bool,
    version: &str,
    rpc_url: &str,
    bind: &str,
) {
    if no_banner && !force {
        return;
    }
    if !force && !io::stdout().is_terminal() {
        return;
    }

    enable_windows_console();

    let _ = write!(io::stdout(), "\x1b]0;{PRODUCT} v{version}\x07");

    let color = use_color() && (force || io::stdout().is_terminal());
    let body = splash_text(version, rpc_url, bind, color);
    let _ = write!(io::stdout(), "{body}");
    let _ = io::stdout().flush();
}

fn use_color() -> bool {
    match std::env::var("NO_COLOR") {
        Ok(v) if !v.is_empty() => false,
        _ => true,
    }
}

#[cfg(windows)]
fn enable_windows_console() {
    const UTF8: u32 = 65001;
    const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 0x0004;
    const STD_OUTPUT_HANDLE: i32 = -11;

    #[link(name = "kernel32")]
    extern "system" {
        fn SetConsoleOutputCP(code_page: u32) -> i32;
        fn GetStdHandle(n: i32) -> *mut std::ffi::c_void;
        fn GetConsoleMode(h: *mut std::ffi::c_void, mode: *mut u32) -> i32;
        fn SetConsoleMode(h: *mut std::ffi::c_void, mode: u32) -> i32;
    }

    unsafe {
        SetConsoleOutputCP(UTF8);
        let h = GetStdHandle(STD_OUTPUT_HANDLE);
        if !h.is_null() && h != (-1isize as *mut std::ffi::c_void) {
            let mut mode = 0u32;
            if GetConsoleMode(h, &mut mode) != 0 {
                SetConsoleMode(h, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
            }
        }
    }
}

#[cfg(not(windows))]
fn enable_windows_console() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splash_has_brand_version_and_thanks() {
        let s = splash_text("0.1.0", "http://127.0.0.1:8232", "127.0.0.1:9067", false);
        assert!(s.contains("Zeaking"));
        assert!(s.contains("0.1.0"));
        assert!(s.contains("Thank you for using Zeaking"));
        assert!(s.contains("nosy people"));
        assert!(s.contains("Nozy"));
        assert!(s.contains("127.0.0.1:9067"));
    }

    #[test]
    fn color_art_uses_half_blocks() {
        assert!(BANNER_ANSI.contains('▀') || BANNER_ANSI.contains('▄'));
        assert!(BANNER_ANSI.contains("\x1b[38;2;"));
    }
}
