//! Onsa music library.
//!
//! Owns the SQLite database and its versioned migrations, the folder scanner
//! and watcher, tag reading, the cover cache, statistics and play history,
//! and full-text search (SPEC §5).
//!
//! This crate does not depend on the other feature crates; `src-tauri` is the
//! only place where they meet (see SPEC §2).

#![warn(missing_docs)]

pub mod cover;
mod db;
mod error;
pub mod overrides;
pub mod schema;
pub mod search;
pub mod stats;
pub mod tags;

pub use db::Library;
pub use error::{Error, Result};
pub use overrides::Field;
pub use search::{AlbumRow, NameCount, SearchResults, TrackRow, TrackSort};
pub use stats::{Play, TrackStats};