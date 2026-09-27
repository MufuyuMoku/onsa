//! Deleting real files — to the Recycle Bin or the Trash, and never for
//! good (SPEC §15).
//!
//! The promise is Onsa's, not the operating system's. Windows recycles a
//! file only on a volume that keeps a Recycle Bin: a network share does not,
//! and neither do some removable drives, and there the shell deletes the
//! file outright. On Linux a freedesktop trash exists for the home
//! filesystem, while another mount has one only if somebody made it. In
//! both cases the call succeeds and the file is simply gone.
//!
//! So Onsa does not take the platform's word for it. Before the first
//! deletion in a place, it puts a small file of its own there, sends that to
//! the trash, and looks for it in the trash. If it is there, the place
//! honours the promise and the real deletion goes ahead; the probe is then
//! purged so nobody finds Onsa's litter in their Recycle Bin. If it is not
//! there, nothing of the listener's is deleted and the screen says why.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use crate::db::Library;
use crate::error::Result;

/// The name of the file Onsa deletes to find out whether deleting works.
const PROBE: &str = ".onsa-trash-check";

/// Why a file was not deleted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// The place has no Recycle Bin or Trash, so deleting there would be
    /// permanent. Onsa does not do that.
    NoTrashHere,
    /// Onsa could not find out whether deleting there is permanent, which
    /// counts the same: the file stays.
    CannotTell(String),
    /// There is no file at that path any more.
    NotThere,
    /// The system refused the deletion itself.
    Failed(String),
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoTrashHere => f.write_str("this place has no Recycle Bin or Trash"),
            Self::CannotTell(why) => write!(
                f,
                "it is not clear whether deleting here is permanent: {why}"
            ),
            Self::NotThere => f.write_str("there is no file there any more"),
            Self::Failed(why) => f.write_str(why),
        }
    }
}

/// What came of a deletion.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeleteReport {
    /// The files that are now in the Recycle Bin or the Trash.
    pub deleted: Vec<PathBuf>,
    /// The files that were left alone, and why.
    pub kept: Vec<(PathBuf, Refusal)>,
}

impl DeleteReport {
    /// How many files went to the Recycle Bin or the Trash.
    pub fn count(&self) -> usize {
        self.deleted.len()
    }

    /// Whether every file asked about was deleted.
    pub fn all_done(&self) -> bool {
        self.kept.is_empty()
    }
}

/// What Onsa has already found out about a place, so that deleting a hundred
/// files does not probe the trash a hundred times.
fn known() -> &'static Mutex<HashMap<PathBuf, std::result::Result<(), Refusal>>> {
    static KNOWN: OnceLock<Mutex<HashMap<PathBuf, std::result::Result<(), Refusal>>>> =
        OnceLock::new();
    KNOWN.get_or_init(|| Mutex::new(HashMap::new()))
}

/// What the answer is remembered against: the folder itself.
///
/// A Recycle Bin belongs to a volume and a freedesktop trash to a
/// filesystem, and it is tempting to remember the answer against the drive
/// letter instead — one probe for the whole of a drive. But a volume can be
/// mounted into a folder on Windows, and a share can be mapped into one on
/// both systems, so two folders under the same drive letter are not always
/// the same question. Getting that wrong would mean deleting somebody's
/// file for good, so the answer is remembered against the folder, and the
/// cost is one probe per folder per run of Onsa.
fn place_of(dir: &Path) -> PathBuf {
    dir.to_path_buf()
}

/// Whether two paths name the same folder, as the shell reports them.
fn same_folder(one: &Path, other: &Path) -> bool {
    if one == other {
        return true;
    }
    if cfg!(windows) {
        let text = |path: &Path| path.to_string_lossy().to_lowercase().replace('/', "\\");
        let (one, other) = (text(one), text(other));
        one.trim_end_matches('\\') == other.trim_end_matches('\\')
    } else {
        false
    }
}

/// Finds the probe in the trash and takes it out again.
///
/// Answers whether it was there, which is the whole question.
fn probe_landed_in_the_trash(dir: &Path) -> std::result::Result<bool, String> {
    let items = trash::os_limited::list().map_err(|error| error.to_string())?;
    let mine: Vec<trash::TrashItem> = items
        .into_iter()
        .filter(|item| item.name == PROBE && same_folder(&item.original_parent, dir))
        .collect();
    if mine.is_empty() {
        return Ok(false);
    }
    // Onsa's own litter does not stay in somebody's Recycle Bin.
    if let Err(error) = trash::os_limited::purge_all(&mine) {
        tracing::warn!("the trash probe could not be purged again: {error}");
    }
    Ok(true)
}

/// Whether a file deleted from this folder really goes to the Recycle Bin or
/// the Trash.
///
/// Proven by doing it, once per place per run of Onsa. A place that cannot
/// be asked counts as a place that does not keep the promise.
pub fn trash_is_real(dir: &Path) -> std::result::Result<(), Refusal> {
    let place = place_of(dir);
    if let Ok(known) = known().lock() {
        if let Some(answer) = known.get(&place) {
            return answer.clone();
        }
    }
    let answer = ask_the_place(dir);
    match &answer {
        Ok(()) => {
            tracing::info!(place = %place.display(), "deleting here goes to the Recycle Bin or Trash")
        }
        Err(why) => {
            tracing::warn!(place = %place.display(), "deleting here would be permanent: {why}")
        }
    }
    if let Ok(mut known) = known().lock() {
        known.insert(place, answer.clone());
    }
    answer
}

/// The probe itself, without the remembering.
fn ask_the_place(dir: &Path) -> std::result::Result<(), Refusal> {
    let probe = dir.join(PROBE);
    // A leftover from a run that was cut short would be found in the trash
    // and answer the question without asking it.
    let _ = trash::os_limited::list().map(|items| {
        let stale: Vec<trash::TrashItem> = items
            .into_iter()
            .filter(|item| item.name == PROBE && same_folder(&item.original_parent, dir))
            .collect();
        if !stale.is_empty() {
            let _ = trash::os_limited::purge_all(&stale);
        }
    });
    std::fs::write(
        &probe,
        b"Onsa checks that deleting here is not permanent.\n",
    )
    .map_err(|error| Refusal::CannotTell(error.to_string()))?;
    if let Err(error) = trash::delete(&probe) {
        let _ = std::fs::remove_file(&probe);
        return Err(Refusal::CannotTell(error.to_string()));
    }
    match probe_landed_in_the_trash(dir) {
        Ok(true) => Ok(()),
        Ok(false) => Err(Refusal::NoTrashHere),
        Err(why) => Err(Refusal::CannotTell(why)),
    }
}

impl Library {
    /// Sends the files of these tracks to the Recycle Bin or the Trash.
    ///
    /// A track whose place cannot keep the promise is left alone and listed
    /// in the report with the reason, so the screen can say it. Nothing is
    /// ever deleted for good, and a file that is deleted is still a track:
    /// it is marked missing, so that restoring it from the system's own
    /// Restore brings the song back with everything that hung from it.
    pub fn delete_files(&mut self, tracks: &[i64]) -> Result<DeleteReport> {
        let mut report = DeleteReport::default();
        for track_id in tracks {
            let Some(track) = self.track(*track_id)? else {
                continue;
            };
            let path = PathBuf::from(&track.path);
            let Some(dir) = path.parent() else {
                report.kept.push((path, Refusal::NotThere));
                continue;
            };
            if !path.is_file() {
                report.kept.push((path, Refusal::NotThere));
                continue;
            }
            if let Err(why) = trash_is_real(dir) {
                report.kept.push((path, why));
                continue;
            }
            match trash::delete(&path) {
                Ok(()) => {
                    self.mark_missing(*track_id)?;
                    tracing::info!(file = %path.display(), "a file went to the Recycle Bin or Trash");
                    report.deleted.push(path);
                }
                Err(error) => {
                    report.kept.push((path, Refusal::Failed(error.to_string())));
                }
            }
        }
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_folder_is_its_own_question() {
        // Two folders on one drive are not one answer: either of them can be
        // a mounted volume or a mapped share with no Recycle Bin of its own.
        let one = Path::new(r"C:\Users\someone\音楽\album");
        let other = Path::new(r"C:\Users\someone\音楽");
        assert_eq!(place_of(one), one);
        assert_ne!(place_of(one), place_of(other));
    }

    #[test]
    fn the_shell_may_spell_a_folder_its_own_way() {
        let one = Path::new(r"C:\Users\someone\音楽");
        assert!(same_folder(one, one));
        if cfg!(windows) {
            assert!(same_folder(one, Path::new(r"c:\users\someone\音楽\")));
        }
        assert!(!same_folder(one, Path::new(r"C:\Users\someone")));
    }

    #[test]
    fn a_refusal_says_what_it_is_in_words() {
        assert!(!Refusal::NoTrashHere.to_string().is_empty());
        assert!(Refusal::CannotTell("no".into()).to_string().contains("no"));
    }
}
