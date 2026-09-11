//! Errors of the library.

use std::path::PathBuf;

use thiserror::Error;

/// Result alias for library operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Anything the library can fail at.
#[derive(Debug, Error)]
pub enum Error {
    /// A file or directory could not be read or written.
    #[error("i/o error on {path}: {source}")]
    Io {
        /// The file or directory involved.
        path: PathBuf,
        /// What the operating system reported.
        source: std::io::Error,
    },
    /// The database rejected an operation.
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    /// The database schema is newer than this build understands.
    #[error("the library database has schema version {found}; this build knows up to {known}")]
    UnsupportedSchema {
        /// Version found in the database.
        found: u32,
        /// Newest version this build knows.
        known: u32,
    },
    /// A path cannot be stored because it is not valid Unicode.
    #[error("path is not valid Unicode: {0}")]
    NonUnicodePath(PathBuf),
    /// The folder watcher could not start or follow a folder.
    #[error("folder watcher error: {0}")]
    Watch(String),
}

impl Error {
    /// An I/O error with the path it happened on.
    pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}
