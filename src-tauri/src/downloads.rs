//! The downloader's own commands (SPEC §7).
//!
//! This is where the programs Onsa runs are installed, and where downloads
//! will be run from. The rule from §7.1 that shapes the whole page: when the
//! downloader is first opened, Onsa **says what it would fetch, from where,
//! and roughly how large it is** — and nothing is fetched until somebody
//! presses the button.
//!
//! Fetching goes through the one HTTP client the project shares (SPEC §1).
//! What comes back is checked against the checksum the release published
//! before it is written anywhere, and that check lives in `onsa-downloader`,
//! which never speaks to the network itself.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use onsa_downloader::install::{self, MAX_BINARY};
use onsa_downloader::{Program, Where};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

use crate::error::ErrorCode;
use crate::net::{self, NetError};
use crate::online;
use crate::ytdlp;

/// What the interface listens for while a program is being fetched.
pub const BINARY_EVENT: &str = "downloads://binary";

/// Most a published list of checksums may weigh. It is a few lines of text.
const MAX_SUMS: u64 = 256 * 1024;

/// One program the downloader needs, as the interface lists it (SPEC §7.1).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BinaryDto {
    /// Which program, by its name.
    pub key: String,
    /// Whether Onsa can run it right now.
    pub present: bool,
    /// Where the copy in use came from: `managed`, `chosen` or `system`.
    pub from: Option<String>,
    /// The file in use.
    pub path: Option<String>,
    /// What it says its version is.
    pub version: Option<String>,
    /// Whether Onsa can install this one for itself yet.
    pub installable: bool,
    /// Where it would come from, so the listener can see before agreeing.
    pub url: Option<String>,
    /// Roughly how large that is, in bytes.
    pub about_bytes: Option<u64>,
    /// Whether this program is needed before anything can be downloaded.
    pub required: bool,
}

/// The programs, and whether a copy already on the system may be used.
///
/// The setting travels with the list rather than being asked for
/// separately: a page that draws the switch from a guess can disagree with
/// what is stored, and then pressing it once appears to do nothing.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BinariesDto {
    /// One entry per program the downloader needs.
    pub programs: Vec<BinaryDto>,
    /// Whether a copy already on the system may be used (SPEC §7.1).
    pub use_system: bool,
    /// Whether there is an ffmpeg for yt-dlp to use.
    ///
    /// Without one a download still works — Onsa asks for the audio as the
    /// site keeps it — but converting it, writing the tags in and putting
    /// the cover in are all ffmpeg's work, so the interface says so rather
    /// than offering what would fail.
    pub can_convert: bool,
}

/// How far along a fetch is.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchProgress {
    /// Which program.
    pub key: String,
    /// Bytes fetched so far.
    pub done: u64,
    /// How many there are altogether, when the server said.
    pub total: Option<u64>,
    /// Whether it has finished, one way or the other.
    pub finished: bool,
    /// What went wrong, as a fixed word the interface translates.
    pub failed: Option<String>,
    /// What the release or the client itself said about it.
    pub said: Option<String>,
}

/// One fetch at a time, and a way to stop it.
#[derive(Debug, Default)]
pub struct Fetching {
    busy: AtomicBool,
    stop: AtomicBool,
}

impl Fetching {
    /// Asks a fetch that is going to stop.
    pub fn stop(&self) {
        self.stop.store(true, Ordering::SeqCst);
    }
}

/// The programs the downloader needs, and what Onsa can do about each.
///
/// Two are listed, because two are what a listener can act on. yt-dlp is
/// the downloader itself, and Onsa fetches it. ffmpeg is what turns a
/// download into a tagged file with a cover on it, and what every
/// conversion needs; it is published as an archive, so in this version it
/// is installed by the listener rather than by Onsa.
///
/// ffprobe is not listed separately: it comes with ffmpeg, from the same
/// folder, and a second row for it would be a second thing to do that is
/// the same thing. Deno is not listed either — yt-dlp finds one for itself
/// if there is one, and Onsa neither fetches it nor has tried it.
#[tauri::command]
pub async fn binaries_status(app: AppHandle) -> Result<BinariesDto, ErrorCode> {
    // Asking a program its version means running it.
    tauri::async_runtime::spawn_blocking(move || {
        let use_system = {
            let library = app.state::<crate::library::LibraryService>();
            library
                .read(|library| {
                    Ok(crate::settings::load::<crate::settings::ProgramPrefs>(
                        library,
                        crate::settings::PROGRAMS_KEY,
                    ))
                })?
                .use_system
        };
        let programs = online::programs(&app)?;
        let listed = [Program::YtDlp, Program::Ffmpeg]
            .into_iter()
            .map(|program| {
                // Onsa runs yt-dlp itself, so Onsa's own rules decide where
                // it comes from. ffmpeg is run by yt-dlp, which searches
                // PATH on its own account: what matters for that row is
                // what yt-dlp will find, not what Onsa would have chosen.
                let found = if program == Program::Ffmpeg {
                    programs.reachable(program)
                } else {
                    programs.find(program)
                };
                // The same copy, found by a question Onsa was not allowed
                // to ask. Saying only "from the system" while the switch
                // below is off reads like a switch that does nothing, so
                // the row says whose copy it really is.
                let despite_switch = program == Program::Ffmpeg
                    && !use_system
                    && found.is_some()
                    && programs.find(program).is_none();
                let release = install::release_for(program);
                BinaryDto {
                    key: program.key().to_string(),
                    present: found.is_some(),
                    from: found.as_ref().map(|found| {
                        if despite_switch {
                            return "systemAnyway".to_string();
                        }
                        match found.from {
                            Where::Managed => "managed",
                            Where::Chosen => "chosen",
                            Where::System => "system",
                        }
                        .to_string()
                    }),
                    path: found.as_ref().map(|found| found.path.display().to_string()),
                    version: found.as_ref().and_then(|_| programs.version(program)),
                    installable: release.is_some(),
                    url: release.as_ref().map(|one| one.url.to_string()),
                    about_bytes: release.as_ref().map(|one| one.about_bytes),
                    required: program == Program::YtDlp,
                }
            })
            .collect();
        Ok(BinariesDto {
            programs: listed,
            use_system,
            can_convert: programs.reachable(Program::Ffmpeg).is_some(),
        })
    })
    .await
    .map_err(|_| ErrorCode::Library)?
}

/// Fetches one program and installs it, after the listener has agreed.
///
/// Three steps, in this order and no other: the published list of checksums,
/// then the file, then the check. Nothing reaches the folder Onsa runs
/// programs from until the checksum matches (SPEC §7.1).
#[tauri::command]
pub async fn binary_install(app: AppHandle, program: String) -> Result<BinaryDto, ErrorCode> {
    let program = Program::from_key(&program).ok_or(ErrorCode::Library)?;
    let release = install::release_for(program).ok_or(ErrorCode::Library)?;
    let state = app.state::<Arc<Fetching>>().inner().clone();
    if state
        .busy
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err(ErrorCode::Busy);
    }
    state.stop.store(false, Ordering::SeqCst);

    let bin_dir = app.state::<crate::Folders>().data.join("bin");
    let handle = app.clone();
    let key = program.key().to_string();

    // One handle for the worker, one kept here to clear the flag afterwards.
    let mine = state.clone();
    let outcome = tauri::async_runtime::spawn_blocking(move || {
        let say = |done: u64,
                   total: Option<u64>,
                   finished: bool,
                   failed: Option<String>,
                   said: Option<String>| {
            let _ = handle.emit(
                BINARY_EVENT,
                FetchProgress {
                    key: key.clone(),
                    done,
                    total,
                    finished,
                    failed,
                    said,
                },
            );
        };

        // The list first: fetching the file only to find there is nothing to
        // check it against would be a download spent for nothing.
        let sums = net::fetch(release.sums_url, MAX_SUMS, |_, _| true).map_err(trouble)?;
        let sums = String::from_utf8(sums).map_err(|error| {
            (
                "unreadable".to_string(),
                Some(format!("the list of checksums is not text: {error}")),
            )
        })?;
        let expected = install::checksum_for(&sums, release.listed_as).ok_or_else(|| {
            (
                "noChecksum".to_string(),
                Some(format!("no line in the list names {}", release.listed_as)),
            )
        })?;

        let bytes = net::fetch(release.url, MAX_BINARY, |done, total| {
            say(done, total, false, None, None);
            !mine.stop.load(Ordering::SeqCst)
        })
        .map_err(trouble)?;

        install::install(&bin_dir, program, &bytes, &expected).map_err(|error| {
            tracing::warn!("{} was not installed: {error}", program.key());
            let said = Some(error.to_string());
            match error {
                onsa_downloader::Error::ChecksumMismatch(_) => ("checksum".to_string(), said),
                _ => ("cannotWrite".to_string(), said),
            }
        })?;
        say(
            bytes.len() as u64,
            Some(bytes.len() as u64),
            true,
            None,
            None,
        );
        tracing::info!(
            program = program.key(),
            "installed from its official release"
        );
        Ok::<(), (String, Option<String>)>(())
    })
    .await;

    state.busy.store(false, Ordering::SeqCst);
    match outcome {
        Ok(Ok(())) => {}
        Ok(Err((code, said))) => {
            let _ = app.emit(
                BINARY_EVENT,
                FetchProgress {
                    key: program.key().to_string(),
                    done: 0,
                    total: None,
                    finished: true,
                    failed: Some(code),
                    said,
                },
            );
            return Err(ErrorCode::Download);
        }
        Err(_) => return Err(ErrorCode::Download),
    }

    let listed = binaries_status(app).await?;
    listed
        .programs
        .into_iter()
        .find(|one| one.key == program.key())
        .ok_or(ErrorCode::Library)
}

/// Asks a fetch that is going to stop. What was fetched is thrown away.
#[tauri::command]
pub fn binary_stop(app: AppHandle) {
    app.state::<Arc<Fetching>>().stop();
    tracing::info!("a program fetch was asked to stop");
}

/// Whether a copy already on the system may be used (SPEC §7.1).
#[tauri::command]
pub async fn binaries_use_system(app: AppHandle, allowed: bool) -> Result<BinariesDto, ErrorCode> {
    {
        let library = app.state::<crate::library::LibraryService>();
        let mut prefs = library.read(|library| {
            Ok(crate::settings::load::<crate::settings::ProgramPrefs>(
                library,
                crate::settings::PROGRAMS_KEY,
            ))
        })?;
        prefs.use_system = allowed;
        library.read(|library| {
            crate::settings::save(library, crate::settings::PROGRAMS_KEY, &prefs)
        })?;
    }
    tracing::info!(allowed, "using the system's own programs");
    binaries_status(app).await
}

/// A fixed word for what went wrong on the way, and the sentence that came
/// with it.
fn trouble(fetch: net::FetchTrouble) -> (String, Option<String>) {
    (why(fetch.kind), fetch.said)
}

/// A fixed word for what went wrong on the way.
fn why(error: NetError) -> String {
    match error {
        NetError::Unreachable => "unreachable".to_string(),
        NetError::TooLarge => "tooLarge".to_string(),
        NetError::Stopped => "stopped".to_string(),
    }
}

// ------------------------------------------------------ downloading (SPEC §7.2)

/// What the interface listens for while a download is going.
pub const DOWNLOAD_EVENT: &str = "downloads://progress";

/// One thing waiting to be downloaded, or being downloaded, or done.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemDto {
    /// Which one, for the interface to follow.
    pub id: u64,
    /// The URL of this one item.
    pub url: String,
    /// What it is called.
    pub title: String,
    /// `waiting`, `running`, `done`, `failed` or `stopped`.
    pub state: String,
    /// How far along, from nothing to one, when that is known.
    pub fraction: Option<f64>,
    /// Bytes a second, as yt-dlp reckons it.
    pub speed: Option<f64>,
    /// Seconds left, as yt-dlp reckons it.
    pub eta: Option<f64>,
    /// The file that arrived, once one has.
    pub file: Option<String>,
    /// Why it failed, as a fixed word the interface translates.
    pub failed: Option<String>,
    /// What yt-dlp itself said about it, kept for whoever looks into it.
    pub said: Option<String>,
}

/// The queue as it stands.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueDto {
    /// Whether anything is being downloaded right now.
    pub running: bool,
    /// Everything asked for in this run of Onsa, newest last.
    pub items: Vec<ItemDto>,
    /// Where finished files are being put.
    pub folder: Option<String>,
}

/// One row of the queue, as the application holds it.
#[derive(Debug, Clone)]
struct Item {
    id: u64,
    url: String,
    title: String,
    state: &'static str,
    fraction: Option<f64>,
    speed: Option<f64>,
    eta: Option<f64>,
    file: Option<String>,
    failed: Option<String>,
    said: Option<String>,
}

impl Item {
    fn dto(&self) -> ItemDto {
        ItemDto {
            id: self.id,
            url: self.url.clone(),
            title: self.title.clone(),
            state: self.state.to_string(),
            fraction: self.fraction,
            speed: self.speed,
            eta: self.eta,
            file: self.file.clone(),
            failed: self.failed.clone(),
            said: self.said.clone(),
        }
    }
}

/// The downloads asked for, and whether one is going.
///
/// One at a time for now: running several at once, and remembering the queue
/// across restarts, belong to the rest of M10 (SPEC §7.3).
#[derive(Debug, Default)]
pub struct Queue {
    items: Mutex<Vec<Item>>,
    next_id: AtomicU64,
    running: AtomicBool,
    stop: Arc<AtomicBool>,
}

impl Queue {
    fn snapshot(&self, folder: Option<String>) -> QueueDto {
        let items = self.items.lock().unwrap_or_else(|e| e.into_inner());
        QueueDto {
            running: self.running.load(Ordering::Relaxed),
            items: items.iter().map(Item::dto).collect(),
            folder,
        }
    }

    /// Asks whatever is downloading to stop, and drops what is waiting.
    pub fn stop(&self) {
        self.stop.store(true, Ordering::SeqCst);
        let mut items = self.items.lock().unwrap_or_else(|e| e.into_inner());
        for item in items.iter_mut().filter(|one| one.state == "waiting") {
            item.state = "stopped";
        }
    }
}

/// Where finished downloads go: inside the library, so they appear in it.
///
/// The first library folder is used. Onsa refuses to download at all when
/// there is none rather than inventing somewhere — a file nobody asked for,
/// somewhere nobody chose, is worse than a clear refusal (SPEC §7.3).
fn output_folder(app: &AppHandle) -> Result<PathBuf, ErrorCode> {
    let library = app.state::<crate::library::LibraryService>();
    let folders = library.read(|library| library.folders())?;
    let first = folders.first().ok_or(ErrorCode::NoFolder)?;
    Ok(PathBuf::from(&first.name).join("Unduhan"))
}

/// Asks what is at a URL, without fetching any of it (SPEC §7.2).
#[tauri::command]
pub async fn download_probe(app: AppHandle, url: String) -> Result<ytdlp::Probe, ProbeTrouble> {
    tauri::async_runtime::spawn_blocking(move || {
        let programs = online::programs(&app).map_err(|_| ProbeTrouble {
            code: "cannotRun".to_string(),
            said: None,
        })?;
        ytdlp::probe(&programs, &url).map_err(|why| {
            tracing::info!(code = why.code, "a URL could not be looked at");
            ProbeTrouble {
                code: why.code,
                said: why.said,
            }
        })
    })
    .await
    .map_err(|_| ProbeTrouble {
        code: "cannotRun".to_string(),
        said: None,
    })?
}

/// Why a URL could not be looked at, on its way to the window.
///
/// An error of its own rather than an [`ErrorCode`], because two things
/// have to arrive: the word the window says, and the sentence yt-dlp said
/// under it. The field is named `code` so the interface reads it the same
/// way it reads every other failure.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeTrouble {
    pub code: String,
    pub said: Option<String>,
}

/// The queue as it stands.
#[tauri::command]
pub fn download_queue(app: AppHandle) -> Result<QueueDto, ErrorCode> {
    let folder = output_folder(&app)
        .ok()
        .map(|path| path.display().to_string());
    Ok(app.state::<Arc<Queue>>().snapshot(folder))
}

/// Adds items to the queue and starts working through them.
#[tauri::command]
pub fn download_start(
    app: AppHandle,
    items: Vec<DownloadRequest>,
    format: String,
) -> Result<QueueDto, ErrorCode> {
    let format = ytdlp::Format::from_name(&format).ok_or(ErrorCode::Library)?;
    let into = output_folder(&app)?;
    let queue = app.state::<Arc<Queue>>().inner().clone();

    {
        let mut waiting = queue.items.lock().unwrap_or_else(|e| e.into_inner());
        for one in items {
            if !ytdlp::is_a_url(&one.url) {
                continue;
            }
            let id = queue.next_id.fetch_add(1, Ordering::SeqCst) + 1;
            waiting.push(Item {
                id,
                url: one.url,
                title: one.title,
                state: "waiting",
                fraction: None,
                speed: None,
                eta: None,
                file: None,
                failed: None,
                said: None,
            });
        }
    }

    tracing::info!(
        format = format.name(),
        folder = %into.display(),
        "downloads were asked for"
    );

    if queue
        .running
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
    {
        queue.stop.store(false, Ordering::SeqCst);
        let handle = app.clone();
        let mine = queue.clone();
        // A download takes minutes and reads a pipe the whole time; it
        // belongs on a thread of its own, not on the one answering the
        // interface.
        if let Err(error) = std::thread::Builder::new()
            .name("onsa-downloads".into())
            .spawn(move || {
                work_through(&handle, &mine, format, &into);
                mine.running.store(false, Ordering::SeqCst);
                let _ = handle.emit(DOWNLOAD_EVENT, ());
            })
        {
            tracing::error!("the download thread could not be started: {error}");
            queue.running.store(false, Ordering::SeqCst);
            return Err(ErrorCode::Download);
        }
    }

    download_queue(app)
}

/// One thing the interface asked to download.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadRequest {
    /// The URL of this one item.
    pub url: String,
    /// What to call it in the list while it waits.
    pub title: String,
}

/// Stops the download that is going, and drops what was waiting.
#[tauri::command]
pub fn download_stop(app: AppHandle) -> Result<QueueDto, ErrorCode> {
    app.state::<Arc<Queue>>().stop();
    tracing::info!("the downloads were asked to stop");
    download_queue(app)
}

/// Works through the queue, one item at a time.
fn work_through(app: &AppHandle, queue: &Queue, format: ytdlp::Format, into: &Path) {
    loop {
        if queue.stop.load(Ordering::SeqCst) {
            return;
        }
        // The next one waiting, marked as running before the lock is let go.
        let next = {
            let mut items = queue.items.lock().unwrap_or_else(|e| e.into_inner());
            match items.iter_mut().find(|one| one.state == "waiting") {
                Some(item) => {
                    item.state = "running";
                    Some((item.id, item.url.clone()))
                }
                None => None,
            }
        };
        let Some((id, url)) = next else {
            return;
        };
        let _ = app.emit(DOWNLOAD_EVENT, ());

        let programs = match online::programs(app) {
            Ok(programs) => programs,
            Err(_) => {
                finish(
                    app,
                    queue,
                    id,
                    "failed",
                    None,
                    Some("cannotRun".into()),
                    None,
                );
                continue;
            }
        };

        let handle = app.clone();
        let outcome = ytdlp::download(&programs, &url, format, into, queue.stop.clone(), |step| {
            {
                let mut items = queue.items.lock().unwrap_or_else(|e| e.into_inner());
                if let Some(item) = items.iter_mut().find(|one| one.id == id) {
                    item.fraction = step.fraction;
                    item.speed = step.speed;
                    item.eta = step.eta;
                    if let Some(name) = step.name {
                        item.title = name;
                    }
                }
            }
            let _ = handle.emit(DOWNLOAD_EVENT, ());
        });

        match outcome {
            Ok(files) => {
                let file = files.first().map(|path| path.display().to_string());
                if let Some(path) = files.first() {
                    hand_to_library(app, path);
                }
                finish(app, queue, id, "done", file, None, None);
            }
            Err(why) => {
                let state = if why.code == "stopped" {
                    "stopped"
                } else {
                    "failed"
                };
                finish(app, queue, id, state, None, Some(why.code), why.said);
            }
        }
    }
}

/// Writes down how one item ended, and tells the interface.
fn finish(
    app: &AppHandle,
    queue: &Queue,
    id: u64,
    state: &'static str,
    file: Option<String>,
    failed: Option<String>,
    said: Option<String>,
) {
    {
        let mut items = queue.items.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(item) = items.iter_mut().find(|one| one.id == id) {
            item.state = state;
            item.file = file;
            item.failed = failed;
            item.said = said;
            item.fraction = if state == "done" {
                Some(1.0)
            } else {
                item.fraction
            };
            item.speed = None;
            item.eta = None;
        }
    }
    let _ = app.emit(DOWNLOAD_EVENT, ());
}

/// Hands a finished file to the library, so it appears there (SPEC §7.2).
///
/// The folder downloads go into is inside a library folder, so the watcher
/// usually notices on its own; asking for a scan makes it immediate rather
/// than eventual.
fn hand_to_library(app: &AppHandle, file: &Path) {
    tracing::info!(file = %file.display(), "a download finished");
    let library = app.state::<crate::library::LibraryService>();
    if let Err(error) = library.rescan() {
        tracing::warn!("the library could not be told about a download: {error:?}");
    }
}
