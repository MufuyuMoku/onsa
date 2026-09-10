//! Onsa downloader.
//!
//! Manages the external programs the downloader needs (yt-dlp, ffmpeg, Deno)
//! and runs downloads as separate processes. Every process is spawned with an
//! argument array and never through a shell (see SPEC §7.3 and §14).
//!
//! This crate does not depend on the other feature crates; it only reports the
//! path of the finished file, and `src-tauri` hands that path to the library.
//!
//! The downloader is built in M10. Today the crate only carries its error type.

#![warn(missing_docs)]

use thiserror::Error;

/// Result alias for downloader operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Anything the downloader can fail at.
#[derive(Debug, Error)]
pub enum Error {
    /// Spawning or talking to an external process failed.
    #[error("cannot run external program: {0}")]
    Process(#[from] std::io::Error),
    /// A required external program is missing from the managed folder and PATH.
    #[error("missing external program: {0}")]
    MissingProgram(String),
    /// A downloaded binary did not match its published checksum.
    #[error("checksum mismatch for {0}")]
    ChecksumMismatch(String),
}
