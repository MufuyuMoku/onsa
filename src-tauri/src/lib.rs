//! Onsa application shell.
//!
//! This crate is the glue: it owns application state, exposes commands to the
//! user interface, forwards engine events, and holds the operating system
//! integration. The feature crates meet only here (SPEC §2).

#![warn(missing_docs)]

mod commands;
pub mod theme;

use anyhow::{Context, Result};

/// Starts the application: sets up logging, builds the window and runs until
/// the user closes it.
pub fn run() -> Result<()> {
    init_tracing();
    tracing::info!(version = env!("CARGO_PKG_VERSION"), "Onsa starting");

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::theme_list,
            commands::theme_get,
        ])
        .run(tauri::generate_context!())
        .context("the application window could not be started")
}

/// Sends log records to standard error, filtered by the `ONSA_LOG`
/// environment variable. Rolling log files land with the rest of the
/// operating system integration in M4 (SPEC §13).
fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_env("ONSA_LOG")
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
