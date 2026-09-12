//! Failures as the interface sees them.
//!
//! Errors cross the command boundary as stable codes, never as sentences:
//! all user visible text comes from the interface dictionaries (SPEC §9.6).
//! The details go to the log.

use serde::Serialize;

/// A failure the interface can translate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "code")]
pub enum ErrorCode {
    /// No theme with that identifier exists.
    ThemeNotFound,
    /// The library database failed.
    Library,
    /// The audio engine refused a command.
    Engine,
    /// No audio output could be opened.
    NoOutput,
    /// A file dialog could not be shown.
    Dialog,
    /// A file could not be read or written.
    Io,
    /// An EQ preset file held no usable filter.
    AutoEqEmpty,
}

impl From<onsa_library::Error> for ErrorCode {
    fn from(error: onsa_library::Error) -> Self {
        tracing::error!("library: {error}");
        Self::Library
    }
}

impl From<onsa_audio::Error> for ErrorCode {
    fn from(error: onsa_audio::Error) -> Self {
        tracing::error!("audio engine: {error}");
        Self::Engine
    }
}
