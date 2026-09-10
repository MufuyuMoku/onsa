//! Onsa audio engine.
//!
//! Owns decoding, resampling, the queue with gapless playback and crossfade,
//! the real-time DSP chain, the output backend and the analysis tap. The
//! engine runs on its own threads and knows nothing about the user interface;
//! `src-tauri` drives it with commands and forwards its events.
//!
//! This crate must not depend on any other Onsa crate (see SPEC §2).
//!
//! The engine itself is built in M1 and M2. Today the crate only carries the
//! error type that the rest of the engine will report through.

#![warn(missing_docs)]

use thiserror::Error;

/// Result alias for the audio engine.
pub type Result<T> = std::result::Result<T, Error>;

/// Anything the audio engine can fail at.
#[derive(Debug, Error)]
pub enum Error {
    /// The file could not be opened or read.
    #[error("cannot read audio source: {0}")]
    Source(#[from] std::io::Error),
    /// The container or codec is not supported by the decoder set.
    #[error("unsupported audio format: {0}")]
    UnsupportedFormat(String),
    /// No usable output device is available.
    #[error("no audio output device available")]
    NoOutputDevice,
}
