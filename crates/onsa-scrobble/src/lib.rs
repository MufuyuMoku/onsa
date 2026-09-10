//! Onsa scrobbling.
//!
//! Owns the Last.fm session, the now-playing update, the scrobble rules and
//! the offline queue. Session secrets live in the OS credential store, never
//! in the database or a plain file (see SPEC §11).
//!
//! This crate does not depend on the other feature crates (see SPEC §2).
//!
//! Scrobbling is built in M9. Today the crate only carries its error type.

#![warn(missing_docs)]

use thiserror::Error;

/// Result alias for scrobble operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Anything the scrobbler can fail at.
#[derive(Debug, Error)]
pub enum Error {
    /// The user has not connected an account yet.
    #[error("not authenticated")]
    NotAuthenticated,
    /// The service answered with an error code.
    #[error("scrobble service error {code}: {message}")]
    Service {
        /// Error code as reported by the service.
        code: u32,
        /// Human readable message as reported by the service.
        message: String,
    },
    /// The credential store refused to store or return the session.
    #[error("credential store error: {0}")]
    CredentialStore(String),
}
