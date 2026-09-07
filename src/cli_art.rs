//! Pixel-art banners for the Nozy CLI (zebra mark on start, shield on send).
//!
//! Skipped for `--quiet`, `--json`, and non-TTY stdout so ops pipes stay clean.

use std::io::IsTerminal;

/// Startup mark: left-facing zebra head with the heater-shield outline from the logo.
pub const STARTUP_LOGO: &str = "\x1b[0m\
           ▄▄                         ▄▄██████▄▄
         ▄█▀█▄                      ▄██▀      ▀██▄
       ▄█▀ ▄ ▀█▄                   ██            ██
      █▀▄█▀ ▀█▄ █                 ██              ██
     █ █▀ ▄▄  ▀██                 ██              ██
     ██ ▄█▀▀▀█▄ █                 ██              ██
     █ ▀█▄██▄█▀ █                 ██              ██
      █▄ ▀▀▀▀ ▄█                   ██            ██
       ▀██████▀                     ▀██        ██▀
                                      ▀██    ██▀
                                        ▀████▀

              N O Z Y   W A L L E T
           shielded orchard  ·  nozy lite";

/// Heater-shield pixel mark used for `nozy send`.
pub const SEND_SHIELD: &str = "\
           ▄▄████████████▄▄
         ▄██▀▀          ▀▀██▄
       ▄█▀    ▄██████▄    ▀█▄
      ██     ██▀▀  ▀▀██     ██
     ██      ██  ██  ██      ██
     ██      ██      ██      ██
     ██      ██▄    ▄██      ██
     ██       ▀██████▀       ██
      ██                    ██
       ██                  ██
        ██                ██
         ██              ██
          ██            ██
           ██          ██
            ██        ██
             ██      ██
              ██    ██
               ██  ██
                ████
                 ██
           ─ shielded send ─";

const GOLD: &str = "\x1b[38;5;178m";
const WHITE: &str = "\x1b[97m";
const DIM: &str = "\x1b[90m";
const RESET: &str = "\x1b[0m";

pub fn should_print_art(quiet: bool, json: bool) -> bool {
    if quiet || json {
        return false;
    }
    std::io::stdout().is_terminal()
}

fn use_color() -> bool {
    std::env::var_os("NO_COLOR").is_none() && std::io::stdout().is_terminal()
}

fn paint(ansi: &str, body: &str) -> String {
    if use_color() {
        format!("{ansi}{body}{RESET}")
    } else {
        body.to_string()
    }
}

pub fn print_startup_logo() {
    println!();
    println!("{}", paint(WHITE, STARTUP_LOGO));
    println!();
}

pub fn print_send_shield() {
    println!();
    let art = paint(GOLD, SEND_SHIELD);
    if use_color() {
        println!("{art}");
        println!("{DIM}           orchard  ·  fully shielded{RESET}");
    } else {
        println!("{art}");
        println!("           orchard  ·  fully shielded");
    }
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_logo_looks_like_the_mark() {
        assert!(STARTUP_LOGO.contains("N O Z Y   W A L L E T"));
        assert!(STARTUP_LOGO.contains('▄'));
        assert!(STARTUP_LOGO.lines().count() >= 10);
    }

    #[test]
    fn send_art_is_a_shield() {
        assert!(SEND_SHIELD.contains("shielded send"));
        let widths: Vec<usize> = SEND_SHIELD
            .lines()
            .filter(|l| l.contains('█') || l.contains('▄') || l.contains('▀'))
            .map(|l| l.trim_end().chars().count())
            .collect();
        assert!(widths.len() >= 12, "shield should be several pixel rows");
        let first = widths[0];
        let last = *widths.last().unwrap();
        let mid = widths[widths.len() / 2];
        assert!(
            first < mid && last < mid,
            "heater shield should be widest in the middle (top={first} mid={mid} tip={last})"
        );
    }

    #[test]
    fn quiet_and_json_suppress_art() {
        assert!(!should_print_art(true, false));
        assert!(!should_print_art(false, true));
    }
}
