//! Files and folders handed to Onsa from outside (SPEC §13): dropped on the
//! window, opened with "Open with Onsa", or given to a second instance that
//! then hands them over and goes away.

use std::path::{Path, PathBuf};

use onsa_library::{TrackRow, SUPPORTED_EXTENSIONS};
use tauri::{AppHandle, Manager};

use crate::library::LibraryService;
use crate::player::Player;

/// Takes what was handed over: folders join the library, audio files play.
pub fn accept(app: &AppHandle, paths: Vec<PathBuf>) {
    let (folders, files): (Vec<PathBuf>, Vec<PathBuf>) =
        paths.into_iter().partition(|path| path.is_dir());

    if let Some(library) = app.try_state::<LibraryService>() {
        for folder in &folders {
            tracing::info!(folder = %folder.display(), "folder handed to Onsa");
            if let Err(code) = library.add_folder(folder.clone()) {
                tracing::warn!("the folder cannot be added: {code:?}");
            }
        }
    }

    let files: Vec<PathBuf> = files.into_iter().filter(|path| is_audio(path)).collect();
    if files.is_empty() {
        return;
    }
    let Some(player) = app.try_state::<Player>() else {
        return;
    };
    tracing::info!(files = files.len(), "files handed to Onsa");
    let rows = files.iter().map(|path| row_for(app, path)).collect();
    if let Err(code) = player.play(rows, 0) {
        tracing::warn!("the files cannot be played: {code:?}");
    }
}

/// Whether the scanner, and so the engine, handles this file.
fn is_audio(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            SUPPORTED_EXTENSIONS
                .iter()
                .any(|known| extension.eq_ignore_ascii_case(known))
        })
}

/// The library's entry for a file, or a bare one for a file it never saw.
fn row_for(app: &AppHandle, path: &Path) -> TrackRow {
    if let Some(library) = app.try_state::<LibraryService>() {
        let found = library.read(|library| {
            let Some(id) = library.track_id(path)? else {
                return Ok(None);
            };
            library.track(id)
        });
        if let Ok(Some(row)) = found {
            return row;
        }
    }
    bare_row(path)
}

/// A file Onsa knows nothing about yet: it still plays, listed by its name.
fn bare_row(path: &Path) -> TrackRow {
    TrackRow {
        id: -1,
        path: path.to_string_lossy().into_owned(),
        title: path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned()),
        artist: None,
        album: None,
        album_artist: None,
        track_number: None,
        disc_number: None,
        year: None,
        genre: None,
        duration_ms: None,
        album_id: None,
        cover_id: None,
        status: "ok".to_string(),
        codec: None,
        sample_rate: None,
        bit_depth: None,
        channels: None,
    }
}

/// Paths given on the command line, without the program itself.
pub fn from_arguments<I: IntoIterator<Item = String>>(arguments: I) -> Vec<PathBuf> {
    arguments
        .into_iter()
        .skip(1)
        .filter(|argument| !argument.starts_with('-'))
        .map(PathBuf::from)
        .filter(|path| path.exists())
        .collect()
}
