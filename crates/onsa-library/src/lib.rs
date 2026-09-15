//! Onsa music library.
//!
//! Owns the SQLite database and its versioned migrations, the folder scanner
//! and watcher, tag reading, the cover cache, statistics and play history,
//! and full-text search (SPEC §5).
//!
//! This crate does not depend on the other feature crates; `src-tauri` is the
//! only place where they meet (see SPEC §2).

#![warn(missing_docs)]

pub mod auto;
pub mod clean;
pub mod cover;
mod db;
pub mod edits;
mod error;
pub mod m3u;
pub mod overrides;
pub mod playlist;
pub mod rename;
pub mod scan;
pub mod schema;
pub mod search;
pub mod smart;
pub mod stats;
pub mod tags;
pub mod watch;
pub mod write;

pub use auto::{Reason, Suggestion};
pub use db::Library;
pub use edits::{Batch, BatchReport, Scope, Skipped, Summary, UndoReport};
pub use error::{Error, Result};
pub use m3u::{ImportReport, PathStyle};
pub use overrides::Field;
pub use playlist::{PlaylistKind, PlaylistRow};
pub use rename::{Move, RenamePlan};
pub use scan::{ChangeReport, ScanProgress, ScanReport, SUPPORTED_EXTENSIONS};
pub use search::{AlbumRow, NameCount, SearchResults, TrackRow, TrackSort};
pub use smart::{Match, Rule, Rules, Sort, SortField};
pub use stats::{Play, TrackStats};
pub use watch::{Change, FolderWatcher};
