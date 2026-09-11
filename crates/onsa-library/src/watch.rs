//! Following the library folders as files come and go (SPEC §5.2).
//!
//! The debouncer runs on its own thread and hands over batches of
//! [`Change`]s. Whoever owns the [`Library`] applies them with
//! [`Library::apply_changes`]; the library itself is never shared between
//! threads.

use std::path::{Path, PathBuf};
use std::time::Duration;

use notify_debouncer_full::notify::event::{ModifyKind, RenameMode};
use notify_debouncer_full::notify::{EventKind, RecommendedWatcher, RecursiveMode};
use notify_debouncer_full::{
    new_debouncer, DebounceEventResult, DebouncedEvent, Debouncer, RecommendedCache,
};

use crate::db::Library;
use crate::error::{Error, Result};
use crate::scan::ChangeReport;

/// How long the watcher waits for a burst of events to settle.
pub const DEBOUNCE: Duration = Duration::from_millis(750);

/// One thing that happened in a watched folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    /// A file or folder appeared or its contents changed.
    Changed(PathBuf),
    /// A file or folder went away.
    Removed(PathBuf),
    /// A file or folder moved or was renamed.
    Renamed {
        /// Where it was.
        from: PathBuf,
        /// Where it is now.
        to: PathBuf,
    },
}

/// Watches the library folders. Dropping it stops watching.
pub struct FolderWatcher {
    debouncer: Debouncer<RecommendedWatcher, RecommendedCache>,
}

impl FolderWatcher {
    /// Starts watching `folders`. `on_changes` runs on the watcher's thread
    /// with each settled batch; it should hand the batch to the thread that
    /// owns the library.
    pub fn start(
        folders: &[PathBuf],
        mut on_changes: impl FnMut(Vec<Change>) + Send + 'static,
    ) -> Result<Self> {
        let debouncer =
            new_debouncer(
                DEBOUNCE,
                None,
                move |result: DebounceEventResult| match result {
                    Ok(events) => {
                        let changes = to_changes(events);
                        if !changes.is_empty() {
                            on_changes(changes);
                        }
                    }
                    Err(errors) => {
                        for error in errors {
                            tracing::warn!("folder watcher: {error}");
                        }
                    }
                },
            )
            .map_err(|error| Error::Watch(error.to_string()))?;
        let mut watcher = Self { debouncer };
        for folder in folders {
            watcher.watch(folder)?;
        }
        Ok(watcher)
    }

    /// Starts following one more folder.
    pub fn watch(&mut self, folder: &Path) -> Result<()> {
        self.debouncer
            .watch(folder, RecursiveMode::Recursive)
            .map_err(|error| Error::Watch(format!("{}: {error}", folder.display())))
    }

    /// Stops following a folder.
    pub fn unwatch(&mut self, folder: &Path) -> Result<()> {
        self.debouncer
            .unwatch(folder)
            .map_err(|error| Error::Watch(format!("{}: {error}", folder.display())))
    }
}

/// Turns debounced file system events into library changes.
fn to_changes(events: Vec<DebouncedEvent>) -> Vec<Change> {
    let mut changes: Vec<Change> = Vec::new();
    for event in events {
        let paths = &event.paths;
        let new: Vec<Change> = match event.kind {
            EventKind::Modify(ModifyKind::Name(RenameMode::Both)) if paths.len() == 2 => {
                vec![Change::Renamed {
                    from: paths[0].clone(),
                    to: paths[1].clone(),
                }]
            }
            EventKind::Modify(ModifyKind::Name(RenameMode::From)) | EventKind::Remove(_) => {
                paths.iter().cloned().map(Change::Removed).collect()
            }
            EventKind::Create(_) | EventKind::Modify(_) => {
                paths.iter().cloned().map(Change::Changed).collect()
            }
            _ => Vec::new(),
        };
        for change in new {
            if changes.last() != Some(&change) {
                changes.push(change);
            }
        }
    }
    changes
}

impl Library {
    /// Applies what the watcher saw.
    pub fn apply_changes(&mut self, changes: &[Change]) -> Result<ChangeReport> {
        let mut report = ChangeReport::default();
        for change in self.pair_moves(changes)? {
            match &change {
                Change::Changed(path) => self.file_changed(path, &mut report)?,
                Change::Removed(path) => self.file_removed(path, &mut report)?,
                Change::Renamed { from, to } => self.file_renamed(from, to, &mut report)?,
            }
        }
        Ok(report)
    }

    /// Some moves arrive as a removal and an addition; on Windows every
    /// move between folders does. Within one batch, a removal and an
    /// addition of the same file (see [`Library::same_file_moved`]) become a
    /// move, so the track keeps its identity and statistics.
    fn pair_moves(&self, changes: &[Change]) -> Result<Vec<Change>> {
        let mut paired: Vec<Option<Change>> = changes.iter().cloned().map(Some).collect();
        for removed in 0..paired.len() {
            let Some(Change::Removed(from)) = paired[removed].clone() else {
                continue;
            };
            for added in 0..paired.len() {
                let Some(Change::Changed(to)) = &paired[added] else {
                    continue;
                };
                if self.same_file_moved(&from, to)? {
                    paired[removed] = Some(Change::Renamed {
                        from,
                        to: to.clone(),
                    });
                    paired[added] = None;
                    break;
                }
            }
        }
        Ok(paired.into_iter().flatten().collect())
    }
}
