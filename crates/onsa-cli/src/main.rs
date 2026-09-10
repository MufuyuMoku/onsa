//! Command line front end for the Onsa audio engine.
//!
//! It exists so the engine can be exercised without the user interface. The
//! real subcommands (`play`, `queue`, `devices`, `render-wav`) arrive with the
//! engine in M1; for now the binary only reports what it is.

use anyhow::Result;

const USAGE: &str = "\
onsa-cli: drive the Onsa audio engine without the user interface

Usage:
  onsa-cli --version
  onsa-cli --help

The playback subcommands (play, queue, devices, render-wav) land in M1
together with the audio engine.";

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_env("ONSA_LOG")
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::debug!(version = env!("CARGO_PKG_VERSION"), "onsa-cli started");

    let arg = std::env::args().nth(1);
    match arg.as_deref() {
        Some("--version" | "-V") => println!("onsa-cli {}", env!("CARGO_PKG_VERSION")),
        _ => println!("{USAGE}"),
    }
    Ok(())
}
