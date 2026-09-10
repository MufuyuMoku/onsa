//! Onsa lyrics.
//!
//! Owns the LRC parser, the lyrics sources (local `.lrc` file, tags embedded
//! in the audio file, and LRCLIB) and the lyrics cache.
//!
//! This crate does not depend on the other feature crates (see SPEC §2).
//!
//! Lyrics are built in M8. Today the crate only carries its error type.

#![warn(missing_docs)]

use thiserror::Error;

/// Result alias for lyrics operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Anything the lyrics module can fail at.
#[derive(Debug, Error)]
pub enum Error {
    /// A lyrics file could not be read or written.
    #[error("lyrics i/o failed: {0}")]
    Io(#[from] std::io::Error),
    /// An LRC file could not be understood.
    #[error("malformed LRC at line {line}: {reason}")]
    MalformedLrc {
        /// One-based line number the parser gave up on.
        line: usize,
        /// Why the line could not be read.
        reason: String,
    },
    /// A remote lyrics source answered with something unusable.
    #[error("lyrics source failed: {0}")]
    Source(String),
}
