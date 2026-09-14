//! Onsa downloader.
//!
//! Manages the external programs Onsa runs and, from M10, runs downloads.
//! Every process is spawned with an argument array and never through a shell
//! (see SPEC §7.3 and §14).
//!
//! The program manager ([`programs`]) is built once and shared: M7 uses it
//! for `fpcalc`, the fingerprinter AcoustID needs (SPEC §8), and M10 uses
//! the same code for yt-dlp, ffmpeg and Deno (SPEC §7.1). Putting it here
//! rather than in `src-tauri` keeps the rule of SPEC §2 — the feature crates
//! do not depend on each other, and this one knows nothing about the
//! library; it reports paths and output, and `src-tauri` decides what they
//! mean.

#![warn(missing_docs)]

pub mod platform;
pub mod programs;

pub use programs::{Found, Program, Programs, Ran, Where};

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
    /// The program is there but could not be started.
    #[error("{program} could not be started: {said}")]
    CannotRun {
        /// Which program.
        program: String,
        /// What the system said about it.
        said: String,
    },
    /// The program ran and said it failed.
    #[error("{program} failed: {said}")]
    ProgramFailed {
        /// Which program.
        program: String,
        /// Its exit code, when it had one.
        code: Option<i32>,
        /// Its own first line of complaint.
        said: String,
    },
    /// The program went past its deadline and was stopped.
    #[error("{program} did not finish within {seconds}s and was stopped")]
    TooSlow {
        /// Which program.
        program: String,
        /// How long it was given.
        seconds: u64,
    },
    /// The program answered in a way Onsa cannot read.
    #[error("{program} answered in a way Onsa cannot read")]
    Unreadable {
        /// Which program.
        program: String,
    },
    /// A downloaded binary did not match its published checksum.
    #[error("checksum mismatch for {0}")]
    ChecksumMismatch(String),
}
