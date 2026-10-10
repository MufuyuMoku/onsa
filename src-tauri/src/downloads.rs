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
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
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
    /// Its official release page, for the listener who has to fetch it
    /// themselves. Fixed in the code (SPEC §7.1).
    pub page: String,
    /// What the distribution calls it, where distributions are how software
    /// arrives. Nothing on Windows.
    pub system_package: Option<String>,
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
/// the same thing. Deno is listed, because yt-dlp needs one for the whole
/// of YouTube and cannot fetch it itself.
///
/// fpcalc is not listed here: it belongs to reading a track rather than to
/// downloading one, so it is offered on the Metadata page — through this
/// same command, from the same catalogue, under the same rules.
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
        let listed = [Program::YtDlp, Program::Ffmpeg, Program::Deno]
            .into_iter()
            .map(|program| {
                // Onsa runs yt-dlp itself, so Onsa's own rules decide where
                // it comes from. ffmpeg is run by yt-dlp, which searches
                // PATH on its own account: what matters for that row is
                // what yt-dlp will find, not what Onsa would have chosen.
                // Onsa runs yt-dlp itself. ffmpeg and Deno are run by
                // yt-dlp, which searches PATH on its own account: what
                // matters for those rows is what yt-dlp will find.
                let found = if program == Program::Ffmpeg || program == Program::Deno {
                    programs.reachable(program)
                } else {
                    programs.find(program)
                };
                // The same copy, found by a question Onsa was not allowed
                // to ask. Saying only "from the system" while the switch
                // below is off reads like a switch that does nothing, so
                // the row says whose copy it really is.
                let despite_switch = (program == Program::Ffmpeg || program == Program::Deno)
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
                    page: install::release_page(program).to_string(),
                    system_package: install::system_package(program).map(str::to_string),
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

        // Whatever this release is checked against comes first: fetching
        // the file only to find there is nothing to check it against would
        // be a download spent for nothing. A release Onsa carries the
        // fingerprint for has nothing to fetch, and says so by having no
        // checksum URL at all.
        let published = match install::checksum_url(&release) {
            Some(url) => {
                let sums = net::fetch(&net::release_address(url), MAX_SUMS, |_, _| true)
                    .map_err(trouble)?;
                String::from_utf8(sums).map_err(|error| {
                    (
                        "unreadable".to_string(),
                        Some(format!("the list of checksums is not text: {error}")),
                    )
                })?
            }
            None => String::new(),
        };
        let expected = install::expected_checksum(&release, &published).ok_or_else(|| {
            (
                "noChecksum".to_string(),
                Some(format!(
                    "nothing was published that names {}",
                    release.file_name
                )),
            )
        })?;

        let bytes = net::fetch(
            &net::release_address(release.url),
            MAX_BINARY,
            |done, total| {
                say(done, total, false, None, None);
                !mine.stop.load(Ordering::SeqCst)
            },
        )
        .map_err(trouble)?;

        install::install_release(&bin_dir, &release, &bytes, &expected).map_err(|error| {
            tracing::warn!("{} was not installed: {error}", program.key());
            let said = Some(error.to_string());
            match error {
                onsa_downloader::Error::ChecksumMismatch(_) => ("checksum".to_string(), said),
                onsa_downloader::Error::BadArchive(_) => ("badArchive".to_string(), said),
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

    // The row for the program that was just fetched. Not every program Onsa
    // fetches is on the Downloads page — fpcalc belongs to the Metadata
    // page — so a row that is not in that list is not a failure. It only
    // means this program is met somewhere else.
    let listed = binaries_status(app.clone()).await?;
    if let Some(row) = listed
        .programs
        .into_iter()
        .find(|one| one.key == program.key())
    {
        return Ok(row);
    }
    let found = tauri::async_runtime::spawn_blocking(move || {
        let programs = online::programs(&app)?;
        let found = programs.find(program);
        let release = install::release_for(program);
        Ok::<BinaryDto, ErrorCode>(BinaryDto {
            key: program.key().to_string(),
            present: found.is_some(),
            from: found.as_ref().map(|found| {
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
            page: install::release_page(program).to_string(),
            system_package: install::system_package(program).map(str::to_string),
            required: false,
        })
    })
    .await
    .map_err(|_| ErrorCode::Library)?;
    found
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
        NetError::Gone => "releaseGone".to_string(),
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

/// The queue as it stands, and where it puts what it finishes.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueDto {
    /// Whether anything is being downloaded right now.
    pub running: bool,
    /// Everything asked for, newest last: in this run of Onsa, and what was
    /// left unfinished when it was last closed.
    pub items: Vec<ItemDto>,
    /// Where finished files are being put, when there is anywhere.
    pub folder: Option<String>,
    /// Whether that folder was chosen, rather than the one inside the first
    /// library folder.
    pub folder_chosen: bool,
    /// Whether what lands in that folder appears in the library.
    pub folder_in_library: bool,
    /// Whether the listener has already said that folder may stay outside
    /// the library, so the question is not asked again (SPEC §7.3).
    pub outside_accepted: bool,
    /// How many downloads run at once (SPEC §7.3).
    pub at_once: u8,
}

/// One row of the queue, as the application holds it.
///
/// It carries its own format and folder: two batches asked for with two
/// formats share one set of workers, and a download picked up after a
/// restart goes where it was first asked to.
#[derive(Debug, Clone)]
struct Item {
    /// Its row in the database's queue, which outlives Onsa being closed.
    id: u64,
    url: String,
    title: String,
    format: ytdlp::Format,
    folder: PathBuf,
    state: &'static str,
    fraction: Option<f64>,
    speed: Option<f64>,
    eta: Option<f64>,
    file: Option<String>,
    failed: Option<String>,
    said: Option<String>,
}

impl Item {
    fn waiting(
        id: u64,
        url: String,
        title: String,
        format: ytdlp::Format,
        folder: PathBuf,
    ) -> Self {
        Self {
            id,
            url,
            title,
            format,
            folder,
            state: "waiting",
            fraction: None,
            speed: None,
            eta: None,
            file: None,
            failed: None,
            said: None,
        }
    }

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

/// The downloader's settings, as stored.
fn prefs(app: &AppHandle) -> crate::settings::DownloadPrefs {
    let library = app.state::<crate::library::LibraryService>();
    library
        .read(|library| {
            Ok(crate::settings::load::<crate::settings::DownloadPrefs>(
                library,
                crate::settings::DOWNLOADS_KEY,
            ))
        })
        .unwrap_or_default()
}

/// Changes the downloader's settings and stores them.
fn change_prefs(
    app: &AppHandle,
    change: impl FnOnce(&mut crate::settings::DownloadPrefs),
) -> Result<(), ErrorCode> {
    let mut stored = prefs(app);
    change(&mut stored);
    let library = app.state::<crate::library::LibraryService>();
    library.read(|library| crate::settings::save(library, crate::settings::DOWNLOADS_KEY, &stored))
}

/// Where yt-dlp works on one download before the finished file is moved
/// into the library: in the cache, outside every library folder, so the
/// watcher never sees a piece, a pre-conversion copy or a thumbnail.
fn work_folder(app: &AppHandle, id: u64) -> PathBuf {
    work_root(app).join(id.to_string())
}

/// The folder every download works in, one subfolder each.
fn work_root(app: &AppHandle) -> PathBuf {
    app.state::<crate::Folders>().cache.join("downloads")
}

/// The downloads asked for, and how many are going.
///
/// What has not finished is also in the database (SPEC §7.3), so closing
/// Onsa part way loses nothing: [`restore`] puts it back as waiting.
#[derive(Debug, Default)]
pub struct Queue {
    items: Mutex<Vec<Item>>,
    /// How many workers are alive. The queue is running while any is.
    working: AtomicUsize,
    stop: Arc<AtomicBool>,
}

impl Queue {
    fn snapshot(&self) -> (bool, Vec<ItemDto>) {
        let items = self.items.lock().unwrap_or_else(|e| e.into_inner());
        (
            self.working.load(Ordering::Relaxed) > 0,
            items.iter().map(Item::dto).collect(),
        )
    }

    /// Asks whatever is downloading to stop, and drops what is waiting.
    /// Answers with the rows that were waiting, so they can be forgotten.
    fn stop(&self) -> Vec<u64> {
        self.stop.store(true, Ordering::SeqCst);
        let mut items = self.items.lock().unwrap_or_else(|e| e.into_inner());
        let mut dropped = Vec::new();
        for item in items.iter_mut().filter(|one| one.state == "waiting") {
            item.state = "stopped";
            dropped.push(item.id);
        }
        dropped
    }
}

/// Puts back what was left unfinished when Onsa was last closed.
///
/// It comes back **waiting**, not running: opening Onsa does not start
/// fetching from the internet by itself. The page offers to carry on, and
/// one press does (DECISIONS 2026-10-10).
pub fn restore(app: &AppHandle) {
    // Nothing can be working yet, so whatever is in the work folder was left
    // by a run that ended part way. It is outside the library.
    let _ = std::fs::remove_dir_all(work_root(app));

    let library = app.state::<crate::library::LibraryService>();
    let rows = match library.read(|library| library.queued_downloads()) {
        Ok(rows) => rows,
        Err(code) => {
            tracing::warn!("the download queue cannot be read: {code:?}");
            return;
        }
    };
    let queue = app.state::<Arc<Queue>>();
    let mut items = queue.items.lock().unwrap_or_else(|e| e.into_inner());
    for row in rows {
        let (Some(format), Ok(id)) = (ytdlp::Format::from_name(&row.format), u64::try_from(row.id))
        else {
            // A row Onsa cannot read is one it cannot carry out either.
            let _ = library.write(|library| library.forget_queued_download(row.id));
            continue;
        };
        items.push(Item::waiting(
            id,
            row.url,
            row.title,
            format,
            PathBuf::from(row.folder),
        ));
    }
    if !items.is_empty() {
        tracing::info!(left = items.len(), "unfinished downloads restored, waiting");
    }
}

/// Where finished downloads go (SPEC §7.3).
///
/// The folder the listener chose, if they chose one. Otherwise `Unduhan`
/// inside the first library folder, so what arrives appears in the library.
/// With neither, Onsa refuses to download rather than inventing somewhere —
/// a file nobody asked for, somewhere nobody chose, is worse than a clear
/// refusal.
fn output_folder(app: &AppHandle) -> Result<PathBuf, ErrorCode> {
    if let Some(chosen) = prefs(app).folder.filter(|one| !one.trim().is_empty()) {
        return Ok(PathBuf::from(chosen));
    }
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
    let stored = prefs(&app);
    let folder = output_folder(&app).ok();
    let folder_in_library = match &folder {
        Some(path) => {
            let library = app.state::<crate::library::LibraryService>();
            library.read(|library| library.holds(path)).unwrap_or(false)
        }
        None => false,
    };
    let folder_text = folder.as_ref().map(|path| path.display().to_string());
    let outside_accepted = folder_text.is_some() && stored.outside_ok == folder_text;
    let (running, items) = app.state::<Arc<Queue>>().snapshot();
    Ok(QueueDto {
        running,
        items,
        folder: folder_text,
        folder_chosen: stored.folder.is_some(),
        folder_in_library,
        outside_accepted,
        at_once: u8::try_from(stored.at_once()).unwrap_or(2),
    })
}

/// Sets how many downloads run at once (SPEC §7.3).
///
/// Takes effect for the downloads asked for next; the ones already going
/// are left to finish rather than stopped part way.
#[tauri::command]
pub fn download_set_at_once(app: AppHandle, at_once: u8) -> Result<QueueDto, ErrorCode> {
    change_prefs(&app, |prefs| {
        prefs.at_once = at_once;
        // Stored as it will be used, so what is read back is what runs.
        prefs.at_once = u8::try_from(prefs.at_once()).unwrap_or(2);
    })?;
    tracing::info!(at_once, "downloads at once");
    download_queue(app)
}

/// Chooses where finished downloads go, or goes back to the folder inside
/// the first library folder when `path` is nothing (SPEC §7.3).
///
/// An earlier "leave it outside" is kept: it names its folder, so it only
/// ever answers the question for that folder.
#[tauri::command]
pub fn download_choose_folder(app: AppHandle, path: Option<String>) -> Result<QueueDto, ErrorCode> {
    let chosen = match path
        .map(|one| one.trim().to_string())
        .filter(|one| !one.is_empty())
    {
        Some(path) => {
            let folder = PathBuf::from(&path);
            if !folder.is_absolute() || !folder.is_dir() {
                return Err(ErrorCode::NoFolder);
            }
            Some(path)
        }
        None => None,
    };
    change_prefs(&app, |prefs| prefs.folder = chosen.clone())?;
    tracing::info!(chosen = chosen.is_some(), "where downloads go");
    download_queue(app)
}

/// Lets the folder downloads go into stay outside the library: the answer
/// "no" to SPEC §7.3's question, kept so it is not asked again for this
/// folder.
#[tauri::command]
pub fn download_keep_outside(app: AppHandle) -> Result<QueueDto, ErrorCode> {
    let folder = output_folder(&app)?.display().to_string();
    change_prefs(&app, |prefs| prefs.outside_ok = Some(folder))?;
    download_queue(app)
}

/// Adds items to the queue and starts working through them.
///
/// Each is written to the database before anything runs, so a download
/// asked for is a download that survives Onsa being closed.
#[tauri::command]
pub fn download_start(
    app: AppHandle,
    items: Vec<DownloadRequest>,
    format: String,
) -> Result<QueueDto, ErrorCode> {
    let format = ytdlp::Format::from_name(&format).ok_or(ErrorCode::Library)?;
    let into = output_folder(&app)?;
    let folder_text = into.display().to_string();
    let queue = app.state::<Arc<Queue>>().inner().clone();
    let library = app.state::<crate::library::LibraryService>();

    let mut added = Vec::new();
    for one in items {
        if !ytdlp::is_a_url(&one.url) {
            continue;
        }
        let row = library.write(|library| {
            library.queue_download(&one.url, &one.title, format.name(), &folder_text)
        })?;
        let id = u64::try_from(row).map_err(|_| ErrorCode::Library)?;
        added.push(Item::waiting(id, one.url, one.title, format, into.clone()));
    }
    queue
        .items
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .extend(added);

    tracing::info!(
        format = format.name(),
        folder = %into.display(),
        "downloads were asked for"
    );
    run(&app)?;
    download_queue(app)
}

/// Carries on with what is waiting: what was left unfinished when Onsa was
/// last closed, or what a stop left behind.
#[tauri::command]
pub fn download_resume(app: AppHandle) -> Result<QueueDto, ErrorCode> {
    tracing::info!("the waiting downloads were asked to carry on");
    run(&app)?;
    download_queue(app)
}

/// Starts as many workers as there is waiting work for, up to the setting,
/// counting the ones already going.
fn run(app: &AppHandle) -> Result<(), ErrorCode> {
    let queue = app.state::<Arc<Queue>>().inner().clone();
    // Asking for downloads is asking for them to run: a stop from a moment
    // ago was about what was in the queue then.
    queue.stop.store(false, Ordering::SeqCst);

    // Each worker takes the next waiting item under the lock, so they share
    // one queue without ever taking the same row.
    let wanted = {
        let items = queue.items.lock().unwrap_or_else(|e| e.into_inner());
        items.iter().filter(|one| one.state == "waiting").count()
    }
    .min(prefs(app).at_once());

    loop {
        let going = queue.working.load(Ordering::SeqCst);
        if going >= wanted {
            break;
        }
        if queue
            .working
            .compare_exchange(going, going + 1, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            continue;
        }
        let handle = app.clone();
        let mine = queue.clone();
        // A download takes minutes and reads a pipe the whole time; it
        // belongs on a thread of its own, not on the one answering the
        // interface.
        if let Err(error) = std::thread::Builder::new()
            .name(format!("onsa-downloads-{}", going + 1))
            .spawn(move || {
                work_through(&handle, &mine);
                mine.working.fetch_sub(1, Ordering::SeqCst);
                let _ = handle.emit(DOWNLOAD_EVENT, ());
            })
        {
            queue.working.fetch_sub(1, Ordering::SeqCst);
            tracing::error!("a download thread could not be started: {error}");
            if queue.working.load(Ordering::SeqCst) == 0 {
                return Err(ErrorCode::Download);
            }
            break;
        }
    }
    Ok(())
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
    let dropped = app.state::<Arc<Queue>>().stop();
    forget(&app, &dropped);
    tracing::info!("the downloads were asked to stop");
    download_queue(app)
}

/// Takes rows out of the database's queue, because they are over.
fn forget(app: &AppHandle, ids: &[u64]) {
    let library = app.state::<crate::library::LibraryService>();
    for id in ids {
        let Ok(row) = i64::try_from(*id) else {
            continue;
        };
        if let Err(code) = library.write(|library| library.forget_queued_download(row)) {
            tracing::warn!("a finished download stays in the stored queue: {code:?}");
        }
    }
}

/// Works through the queue, taking the next thing nobody else has taken.
fn work_through(app: &AppHandle, queue: &Queue) {
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
                    Some((item.id, item.url.clone(), item.format, item.folder.clone()))
                }
                None => None,
            }
        };
        let Some((id, url, format, into)) = next else {
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
        let work = work_folder(app, id);
        let outcome = ytdlp::download(
            &programs,
            &url,
            format,
            &into,
            &work,
            queue.stop.clone(),
            |step| {
                let mut renamed = None;
                {
                    let mut items = queue.items.lock().unwrap_or_else(|e| e.into_inner());
                    if let Some(item) = items.iter_mut().find(|one| one.id == id) {
                        item.fraction = step.fraction;
                        item.speed = step.speed;
                        item.eta = step.eta;
                        if let Some(name) = step.name {
                            if name != item.title {
                                item.title = name.clone();
                                renamed = Some(name);
                            }
                        }
                    }
                }
                // The real name is written down once, so a queue picked up
                // after a restart says what each item is.
                if let (Some(name), Ok(row)) = (renamed, i64::try_from(id)) {
                    let library = handle.state::<crate::library::LibraryService>();
                    let _ = library.write(|library| library.retitle_queued_download(row, &name));
                }
                let _ = handle.emit(DOWNLOAD_EVENT, ());
            },
        );
        // Whatever is left of the work — all of it, when the download failed
        // or was stopped — goes with it.
        if let Err(error) = std::fs::remove_dir_all(&work) {
            if error.kind() != std::io::ErrorKind::NotFound {
                tracing::debug!("a download's work folder stays: {error}");
            }
        }

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

/// Writes down how one item ended, takes it out of the stored queue, and
/// tells the interface.
///
/// Every ending is an ending: a failed download is not retried by itself
/// after a restart, and a stopped one was stopped on purpose.
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
    forget(app, &[id]);
    let _ = app.emit(DOWNLOAD_EVENT, ());
}

/// Hands a finished file to the library, so it appears there (SPEC §7.2).
///
/// When the folder downloads go into is inside a library folder the watcher
/// usually notices on its own; asking for a scan makes it immediate rather
/// than eventual. A folder outside the library is left alone: the listener
/// was asked, and said so.
fn hand_to_library(app: &AppHandle, file: &Path) {
    tracing::info!(file = %file.display(), "a download finished");
    let library = app.state::<crate::library::LibraryService>();
    if !library.read(|library| library.holds(file)).unwrap_or(false) {
        return;
    }
    if let Err(error) = library.rescan() {
        tracing::warn!("the library could not be told about a download: {error:?}");
    }
}
