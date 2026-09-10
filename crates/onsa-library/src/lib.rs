//! Onsa music library.
//!
//! Owns the SQLite database and its versioned migrations, the folder scanner
//! and watcher, tag reading and writing, the cover cache, playlists and smart
//! playlists, and full-text search.
//!
//! This crate does not depend on the other feature crates; `src-tauri` is the
//! only place where they meet (see SPEC §2).
//!
//! The library is built in M3. Today the crate only carries its error type.

#![warn(missing_docs)]

use thiserror::Error;

/// Result alias for library operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Anything the library can fail at.
#[derive(Debug, Error)]
pub enum Error {
    /// A file or directory could not be read or written.
    #[error("library i/o failed: {0}")]
    Io(#[from] std::io::Error),
    /// The database rejected an operation or is in an unexpected state.
    #[error("database error: {0}")]
    Database(String),
    /// The database schema is newer than this build understands.
    #[error("unsupported schema version: {0}")]
    UnsupportedSchema(u32),
}
