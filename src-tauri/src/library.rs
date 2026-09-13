//! The library as the application uses it (SPEC §5).
//!
//! Two connections to one database: the reader answers the interface, the
//! writer belongs to the library thread that scans and applies what the
//! folder watcher sees. Browsing never waits for a scan, and playback never
//! waits for either (SPEC §13.1). The library thread sleeps on its job
//! channel, so an idle library costs no CPU.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, MutexGuard};

use onsa_library::{Change, FolderWatcher, Library, ScanProgress};
use serde::Serialize;
use tauri::http::{header, Response, StatusCode};
use tauri::{AppHandle, Emitter, Manager, Runtime, UriSchemeContext};

use crate::error::ErrorCode;

/// Scan progress for the interface.
pub const SCAN_EVENT: &str = "library://scan";
/// The library contents changed; lists should reload.
pub const CHANGED_EVENT: &str = "library://changed";
/// The set of playlists changed; the sidebar should reload.
pub const PLAYLISTS_EVENT: &str = "library://playlists";

/// Where a scan stands.
#[derive(Debug, Clone, Copy, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanStatus {
    /// Whether a scan is running.
    pub running: bool,
    /// Audio files found.
    pub seen: u64,
    /// Files whose tags were read.
    pub read: u64,
    /// Files skipped because they did not change.
    pub unchanged: u64,
    /// Files that could not be read.
    pub failed: u64,
    /// Tracks newly marked missing (after the scan).
    pub missing: u64,
    /// Duration of the last scan, in milliseconds.
    pub elapsed_ms: u64,
}

enum Job {
    AddFolder(PathBuf),
    Rescan,
    Changes(Vec<Change>),
}

/// The library service kept in the application state.
pub struct LibraryService {
    reader: Mutex<Library>,
    jobs: Sender<Job>,
    status: Arc<Mutex<ScanStatus>>,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    // A panic elsewhere must not take the library down with it.
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl LibraryService {
    /// Opens the database twice and starts the library thread. Folders
    /// already in the library are rescanned, which is quick for anything
    /// that did not change while Onsa was closed.
    pub fn start(app: AppHandle, db: &Path, cache: &Path) -> anyhow::Result<Self> {
        let reader = Library::open(db, cache)?;
        let writer = Library::open(db, cache)?;
        let (jobs, receive) = mpsc::channel();
        let status = Arc::new(Mutex::new(ScanStatus::default()));
        let worker = Worker {
            app,
            library: writer,
            watcher: None,
            jobs: jobs.clone(),
            status: status.clone(),
        };
        std::thread::Builder::new()
            .name("onsa-library".into())
            .spawn(move || worker.run(receive))?;
        if !reader.folder_paths()?.is_empty() {
            let _ = jobs.send(Job::Rescan);
        }
        Ok(Self {
            reader: Mutex::new(reader),
            jobs,
            status,
        })
    }

    /// Runs a query on the reader connection.
    pub fn read<T>(
        &self,
        query: impl FnOnce(&Library) -> onsa_library::Result<T>,
    ) -> Result<T, ErrorCode> {
        Ok(query(&lock(&self.reader))?)
    }

    /// Runs a change on the same connection the interface reads from.
    ///
    /// Playlist edits are the listener's own doing and must be visible the
    /// moment the command returns, so they do not go through the library
    /// thread. The database is in WAL mode with a busy timeout, so a scan
    /// running at the same time keeps reading while this short transaction
    /// commits.
    pub fn write<T>(
        &self,
        change: impl FnOnce(&mut Library) -> onsa_library::Result<T>,
    ) -> Result<T, ErrorCode> {
        Ok(change(&mut lock(&self.reader))?)
    }

    /// Adds a folder, then scans it.
    pub fn add_folder(&self, path: PathBuf) -> Result<(), ErrorCode> {
        self.jobs
            .send(Job::AddFolder(path))
            .map_err(|_| ErrorCode::Library)
    }

    /// Scans every folder again.
    pub fn rescan(&self) -> Result<(), ErrorCode> {
        self.jobs.send(Job::Rescan).map_err(|_| ErrorCode::Library)
    }

    /// Where the current or last scan stands.
    pub fn status(&self) -> ScanStatus {
        *lock(&self.status)
    }
}

/// The library thread.
struct Worker {
    app: AppHandle,
    library: Library,
    watcher: Option<FolderWatcher>,
    jobs: Sender<Job>,
    status: Arc<Mutex<ScanStatus>>,
}

impl Worker {
    fn run(mut self, jobs: Receiver<Job>) {
        self.watch_folders();
        for job in jobs {
            let result = match job {
                Job::AddFolder(path) => self.add_folder(&path),
                Job::Rescan => self.scan(),
                Job::Changes(changes) => self.apply(&changes),
            };
            if let Err(error) = result {
                tracing::error!("library: {error}");
            }
        }
    }

    /// (Re)starts the folder watcher on every library folder.
    fn watch_folders(&mut self) {
        self.watcher = None;
        let folders = match self.library.folder_paths() {
            Ok(folders) if !folders.is_empty() => folders,
            Ok(_) => return,
            Err(error) => {
                tracing::error!("library folders cannot be read: {error}");
                return;
            }
        };
        let jobs = self.jobs.clone();
        match FolderWatcher::start(&folders, move |changes| {
            let _ = jobs.send(Job::Changes(changes));
        }) {
            Ok(watcher) => {
                tracing::info!(folders = folders.len(), "watching library folders");
                self.watcher = Some(watcher);
            }
            Err(error) => tracing::warn!("folders are not watched: {error}"),
        }
    }

    fn add_folder(&mut self, path: &Path) -> onsa_library::Result<()> {
        self.library.add_folder(path)?;
        tracing::info!(folder = %path.display(), "folder added to the library");
        self.watch_folders();
        self.scan()
    }

    fn scan(&mut self) -> onsa_library::Result<()> {
        self.publish(ScanStatus {
            running: true,
            ..ScanStatus::default()
        });
        let app = self.app.clone();
        let status = self.status.clone();
        let result = self.library.scan(|progress: &ScanProgress| {
            let current = ScanStatus {
                running: true,
                seen: progress.seen,
                read: progress.read,
                unchanged: progress.unchanged,
                failed: progress.failed,
                ..ScanStatus::default()
            };
            *lock(&status) = current;
            let _ = app.emit(SCAN_EVENT, current);
        });
        let finished = match &result {
            Ok(report) => ScanStatus {
                running: false,
                seen: report.seen,
                read: report.read,
                unchanged: report.unchanged,
                failed: report.failed,
                missing: report.missing,
                elapsed_ms: report.elapsed.as_millis() as u64,
            },
            Err(_) => ScanStatus {
                running: false,
                ..*lock(&self.status)
            },
        };
        self.publish(finished);
        let _ = self.app.emit(CHANGED_EVENT, ());
        result.map(|_| ())
    }

    fn apply(&mut self, changes: &[Change]) -> onsa_library::Result<()> {
        let report = self.library.apply_changes(changes)?;
        tracing::debug!(?report, "folder changes applied");
        if report.updated + report.missing + report.moved > 0 {
            let _ = self.app.emit(CHANGED_EVENT, ());
        }
        Ok(())
    }

    fn publish(&self, status: ScanStatus) {
        *lock(&self.status) = status;
        let _ = self.app.emit(SCAN_EVENT, status);
    }
}

/// Serves `onsa://localhost/cover/<id>/<size>` (on Windows
/// `http://onsa.localhost/cover/<id>/<size>`): cover thumbnails straight from
/// the cache, never as base64 in an event (SPEC §2).
pub fn cover_protocol<R: Runtime>(
    context: UriSchemeContext<'_, R>,
    request: tauri::http::Request<Vec<u8>>,
) -> Response<Vec<u8>> {
    let mut parts = request.uri().path().trim_matches('/').split('/');
    let (Some("cover"), Some(id), size, None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return status(StatusCode::NOT_FOUND);
    };
    let Ok(id) = id.parse::<i64>() else {
        return status(StatusCode::NOT_FOUND);
    };
    let size = size
        .and_then(|size| size.parse::<u32>().ok())
        .unwrap_or(512);
    let Some(service) = context.app_handle().try_state::<LibraryService>() else {
        return status(StatusCode::SERVICE_UNAVAILABLE);
    };
    let file = match service.read(|library| library.cover_file(id, size)) {
        Ok(Some(file)) => file,
        Ok(None) | Err(_) => return status(StatusCode::NOT_FOUND),
    };
    match std::fs::read(&file) {
        Ok(bytes) => Response::builder()
            .header(header::CONTENT_TYPE, "image/jpeg")
            // A cover id always names the same image.
            .header(header::CACHE_CONTROL, "max-age=31536000, immutable")
            .body(bytes)
            .unwrap_or_else(|_| status(StatusCode::INTERNAL_SERVER_ERROR)),
        Err(error) => {
            tracing::warn!(file = %file.display(), "cover cannot be read: {error}");
            status(StatusCode::NOT_FOUND)
        }
    }
}

fn status(code: StatusCode) -> Response<Vec<u8>> {
    let mut response = Response::new(Vec::new());
    *response.status_mut() = code;
    response
}
