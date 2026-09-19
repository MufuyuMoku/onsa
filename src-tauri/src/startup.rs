//! When the library cannot be opened at all.
//!
//! The database is the first thing Onsa needs and the one thing it cannot
//! do without. Until v1.0.1 a database that could not be opened ended the
//! application before the window existed: the listener double-clicked Onsa
//! and nothing happened at all — no window, no dialog, and a log with one
//! line in it that they had no reason to look for.
//!
//! So the failure is caught, and the window opens to say what happened,
//! where the file is, and what can be done about it. **Nothing here repairs
//! or deletes anything.** That database is the listener's own: which copy
//! is worth keeping, and whether to move it aside, is theirs to decide.

use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::{AppHandle, Manager};
use tauri_plugin_opener::OpenerExt;

use crate::error::ErrorCode;

/// Which kind of trouble the database is in.
///
/// The two are told apart because what the listener can do about them is
/// not the same: a database from a newer Onsa is intact and wants a newer
/// Onsa, while a damaged one wants to be moved aside.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Trouble {
    /// The file was written by a newer version of Onsa.
    FromNewerOnsa,
    /// The file could not be read as a library at all.
    Unreadable,
}

/// What the window is told when the library could not be opened.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TroubleDto {
    /// Which kind.
    pub kind: Trouble,
    /// The database file, so the listener can find it.
    pub database: String,
    /// The folder it sits in, which is what the button opens.
    pub folder: String,
    /// What the failure said, for the listener to copy into a report.
    pub said: String,
}

/// The trouble, kept in the application state when there is any.
pub struct Startup(pub TroubleDto);

impl Startup {
    /// Reads a library failure and works out which kind it is.
    pub fn from(error: &anyhow::Error, database: &Path) -> Self {
        let newer = matches!(
            error.downcast_ref::<onsa_library::Error>(),
            Some(onsa_library::Error::UnsupportedSchema { .. })
        ) || error
            .chain()
            .any(|cause| cause.to_string().contains("schema version"));
        Self(TroubleDto {
            kind: if newer {
                Trouble::FromNewerOnsa
            } else {
                Trouble::Unreadable
            },
            database: database.display().to_string(),
            folder: database.parent().unwrap_or(database).display().to_string(),
            said: format!("{error:#}"),
        })
    }
}

/// Whether the library could not be opened, and what to say about it.
///
/// Nothing at all is the ordinary answer: it means the library opened.
#[tauri::command]
pub fn startup_trouble(app: AppHandle) -> Option<TroubleDto> {
    app.try_state::<Startup>().map(|state| state.0.clone())
}

/// Opens the folder the database sits in, so the listener can see it.
#[tauri::command]
pub fn startup_open_folder(app: AppHandle) -> Result<(), ErrorCode> {
    let Some(trouble) = app.try_state::<Startup>() else {
        return Err(ErrorCode::Io);
    };
    let folder = PathBuf::from(&trouble.0.folder);
    app.opener()
        .open_path(folder.display().to_string(), None::<&str>)
        .map_err(|error| {
            tracing::warn!("the data folder cannot be opened: {error}");
            ErrorCode::Io
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_database_from_a_newer_onsa_is_told_apart_from_a_damaged_one() {
        let newer = anyhow::Error::new(onsa_library::Error::UnsupportedSchema {
            found: 99,
            known: 5,
        })
        .context("the library cannot be opened");
        assert_eq!(
            Startup::from(&newer, Path::new("/data/library.db")).0.kind,
            Trouble::FromNewerOnsa
        );

        // What a damaged file really produces, in the words SQLite uses;
        // `rusqlite` is the library crate's dependency, not this one's.
        let broken = anyhow::anyhow!("database disk image is malformed")
            .context("the library cannot be opened");
        assert_eq!(
            Startup::from(&broken, Path::new("/data/library.db")).0.kind,
            Trouble::Unreadable
        );
    }

    #[test]
    fn the_folder_is_the_one_the_database_sits_in() {
        let error = anyhow::anyhow!("anything");
        let trouble = Startup::from(&error, Path::new("/data/onsa/library.db")).0;
        assert_eq!(
            trouble.database,
            Path::new("/data/onsa/library.db").display().to_string()
        );
        assert_eq!(
            trouble.folder,
            Path::new("/data/onsa").display().to_string()
        );
        assert!(!trouble.said.is_empty(), "it says what happened");
    }
}
