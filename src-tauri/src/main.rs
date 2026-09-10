// A release build must not open a console window next to the player.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if let Err(error) = onsa::run() {
        // The window never opened, so the only place left to report is the
        // console and the log.
        tracing::error!("Onsa cannot start: {error:#}");
        eprintln!("Onsa cannot start: {error:#}");
        std::process::exit(1);
    }
}
