//! Managing the folders the library is made of, and the files in them
//! (SPEC §15, M13).
//!
//! Three of these change nothing on disk: letting a folder go, following one
//! that moved, and leaving a subfolder out of scanning. The fourth does:
//! deleting files. It goes to the Recycle Bin or the Trash and never further
//! than that — a place that cannot promise as much is refused, with the
//! reason, rather than deleted from.

use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;

use crate::error::ErrorCode;
use crate::library::LibraryService;

/// A subfolder left out of scanning.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExclusionDto {
    /// Where it is.
    pub path: String,
    /// Whether it is still on disk.
    pub present: bool,
}

/// One file that was not deleted, and why.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeptDto {
    /// The file.
    pub path: String,
    /// The reason, as a code the interface has words for.
    pub reason: String,
    /// What the system said, when the reason carries it. The interface shows
    /// this next to its own sentence, not instead of it.
    pub detail: Option<String>,
}

/// What came of a deletion.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteDto {
    /// The files that are now in the Recycle Bin or the Trash.
    pub deleted: Vec<String>,
    /// The files that stayed, with the reason for each.
    pub kept: Vec<KeptDto>,
}

impl From<onsa_library::DeleteReport> for DeleteDto {
    fn from(report: onsa_library::DeleteReport) -> Self {
        use onsa_library::Refusal;
        Self {
            deleted: report
                .deleted
                .iter()
                .map(|path| path.display().to_string())
                .collect(),
            kept: report
                .kept
                .iter()
                .map(|(path, why)| KeptDto {
                    path: path.display().to_string(),
                    reason: match why {
                        Refusal::NoTrashHere => "no_trash",
                        Refusal::CannotTell(_) => "cannot_tell",
                        Refusal::NotThere => "not_there",
                        Refusal::Failed(_) => "failed",
                    }
                    .into(),
                    detail: match why {
                        Refusal::CannotTell(what) | Refusal::Failed(what) => Some(what.clone()),
                        _ => None,
                    },
                })
                .collect(),
        }
    }
}

/// Stops using a library folder. The files on disk are not touched.
#[tauri::command]
pub fn library_remove_folder(
    library: State<'_, LibraryService>,
    path: String,
) -> Result<bool, ErrorCode> {
    let gone = library.write(|library| library.remove_folder(std::path::Path::new(&path)))?;
    if gone {
        tracing::info!(folder = %path, "a library folder was let go");
        library.rewatch()?;
    }
    Ok(gone)
}

/// Follows a library folder to where it is now, keeping its songs.
#[tauri::command]
pub fn library_move_folder(
    library: State<'_, LibraryService>,
    from: String,
    to: String,
) -> Result<u64, ErrorCode> {
    let carried = library.write(|library| {
        library.move_folder(std::path::Path::new(&from), std::path::Path::new(&to))
    })?;
    library.refresh()?;
    Ok(carried)
}

/// The subfolders left out of scanning.
#[tauri::command]
pub fn library_exclusions(
    library: State<'_, LibraryService>,
) -> Result<Vec<ExclusionDto>, ErrorCode> {
    library.read(|library| {
        Ok(library
            .exclusions()?
            .into_iter()
            .map(|path| ExclusionDto {
                present: path.is_dir(),
                path: path.display().to_string(),
            })
            .collect())
    })
}

/// Leaves a subfolder out of scanning. Its songs leave the library; the
/// files stay on disk.
#[tauri::command]
pub fn library_exclude(library: State<'_, LibraryService>, path: String) -> Result<u64, ErrorCode> {
    let gone = library.write(|library| library.exclude(std::path::Path::new(&path)))?;
    library.changed();
    Ok(gone)
}

/// Looks in a subfolder again, and scans it back in.
#[tauri::command]
pub fn library_include(
    library: State<'_, LibraryService>,
    path: String,
) -> Result<bool, ErrorCode> {
    let back = library.write(|library| library.include(std::path::Path::new(&path)))?;
    if back {
        library.refresh()?;
    }
    Ok(back)
}

/// Asks for a folder, for the screens that manage them.
#[tauri::command]
pub async fn folders_pick(app: AppHandle, title: String) -> Result<Option<String>, ErrorCode> {
    let picked = tauri::async_runtime::spawn_blocking(move || {
        app.dialog().file().set_title(title).blocking_pick_folder()
    })
    .await
    .map_err(|_| ErrorCode::Dialog)?;
    Ok(picked
        .and_then(|file| file.into_path().ok())
        .map(|path| path.display().to_string()))
}

/// Whether deleting a file in this folder would go to the Recycle Bin or the
/// Trash, asked before there is a button to press.
///
/// The answer is proven rather than assumed, so it costs a moment; the
/// interface asks it once, when the question is put to the listener.
#[tauri::command]
pub async fn files_delete_allowed(folder: String) -> Result<KeptDto, ErrorCode> {
    let answer = tauri::async_runtime::spawn_blocking(move || {
        let where_it_is = std::path::PathBuf::from(&folder);
        let outcome = onsa_library::delete::trash_is_real(&where_it_is);
        (folder, outcome)
    })
    .await
    .map_err(|_| ErrorCode::Library)?;
    let (folder, outcome) = answer;
    Ok(match outcome {
        Ok(()) => KeptDto {
            path: folder,
            reason: "ok".into(),
            detail: None,
        },
        Err(onsa_library::Refusal::NoTrashHere) => KeptDto {
            path: folder,
            reason: "no_trash".into(),
            detail: None,
        },
        Err(why) => KeptDto {
            path: folder,
            reason: "cannot_tell".into(),
            detail: Some(why.to_string()),
        },
    })
}

/// Sends these files to the Recycle Bin or the Trash.
#[tauri::command]
pub async fn files_delete(app: AppHandle, tracks: Vec<i64>) -> Result<DeleteDto, ErrorCode> {
    // Asking the shell to delete a hundred files is not work for the thread
    // that answers the interface.
    let outcome = tauri::async_runtime::spawn_blocking(move || {
        let library = app.state::<LibraryService>();
        let report = library.write(|library| library.delete_files(&tracks));
        if let Ok(report) = &report {
            if report.count() > 0 {
                library.changed();
            }
        }
        report
    })
    .await
    .map_err(|_| ErrorCode::Library)??;
    Ok(outcome.into())
}
