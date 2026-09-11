//! Errors of the audio engine.

use std::path::PathBuf;

use thiserror::Error;

/// Result alias for the audio engine.
pub type Result<T> = std::result::Result<T, Error>;

/// Anything the audio engine can fail at.
#[derive(Debug, Error)]
pub enum Error {
    /// A file could not be opened, read or written.
    #[error("i/o error on {path}: {source}")]
    Io {
        /// The file involved.
        path: PathBuf,
        /// What the operating system reported.
        source: std::io::Error,
    },
    /// The container or codec is not supported, or the file is not audio.
    #[error("unsupported audio in {path}: {reason}")]
    Unsupported {
        /// The file involved.
        path: PathBuf,
        /// Why it cannot be played.
        reason: String,
    },
    /// The file is audio but decoding failed beyond recovery.
    #[error("cannot decode {path}: {reason}")]
    Decode {
        /// The file involved.
        path: PathBuf,
        /// What went wrong.
        reason: String,
    },
    /// The resampler could not be built for the requested rates.
    #[error("cannot resample from {from} Hz to {to} Hz: {reason}")]
    Resampler {
        /// Source sample rate.
        from: u32,
        /// Output sample rate.
        to: u32,
        /// What the resampler reported.
        reason: String,
    },
    /// No usable output device is available.
    #[error("no audio output device available")]
    NoOutputDevice,
    /// The output device refused the stream or failed.
    #[error("audio output error: {0}")]
    Output(String),
    /// The engine thread is gone, so the command could not be delivered.
    #[error("the audio engine has stopped")]
    EngineStopped,
}
