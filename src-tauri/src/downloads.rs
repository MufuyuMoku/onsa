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

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use onsa_downloader::install::{self, MAX_BINARY};
use onsa_downloader::{Program, Where};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::error::ErrorCode;
use crate::net::{self, NetError};
use crate::online;

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
/// yt-dlp is the one that must be there before anything can be downloaded.
/// ffmpeg and Deno make it work properly — Deno is what yt-dlp wants for
/// YouTube — and Onsa cannot install those for itself yet: they are
/// published as archives, and unpacking them waits for the rest of M10.
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
        let listed = Program::ALL
            .into_iter()
            .filter(|program| *program != Program::Fpcalc)
            .map(|program| {
                let found = programs.find(program);
                let release = install::release_for(program);
                BinaryDto {
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
                    required: program == Program::YtDlp,
                }
            })
            .collect();
        Ok(BinariesDto {
            programs: listed,
            use_system,
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
        let say = |done: u64, total: Option<u64>, finished: bool, failed: Option<String>| {
            let _ = handle.emit(
                BINARY_EVENT,
                FetchProgress {
                    key: key.clone(),
                    done,
                    total,
                    finished,
                    failed,
                },
            );
        };

        // The list first: fetching the file only to find there is nothing to
        // check it against would be a download spent for nothing.
        let sums = net::fetch(release.sums_url, MAX_SUMS, |_, _| true).map_err(why)?;
        let sums = String::from_utf8(sums).map_err(|_| "unreadable".to_string())?;
        let expected = install::checksum_for(&sums, release.listed_as)
            .ok_or_else(|| "noChecksum".to_string())?;

        let bytes = net::fetch(release.url, MAX_BINARY, |done, total| {
            say(done, total, false, None);
            !mine.stop.load(Ordering::SeqCst)
        })
        .map_err(why)?;

        install::install(&bin_dir, program, &bytes, &expected).map_err(|error| {
            tracing::warn!("{} was not installed: {error}", program.key());
            match error {
                onsa_downloader::Error::ChecksumMismatch(_) => "checksum".to_string(),
                _ => "cannotWrite".to_string(),
            }
        })?;
        say(bytes.len() as u64, Some(bytes.len() as u64), true, None);
        tracing::info!(
            program = program.key(),
            "installed from its official release"
        );
        Ok::<(), String>(())
    })
    .await;

    state.busy.store(false, Ordering::SeqCst);
    match outcome {
        Ok(Ok(())) => {}
        Ok(Err(why)) => {
            let _ = app.emit(
                BINARY_EVENT,
                FetchProgress {
                    key: program.key().to_string(),
                    done: 0,
                    total: None,
                    finished: true,
                    failed: Some(why),
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

/// A fixed word for what went wrong on the way.
fn why(error: NetError) -> String {
    match error {
        NetError::Unreachable => "unreachable".to_string(),
        NetError::TooLarge => "tooLarge".to_string(),
        NetError::Stopped => "stopped".to_string(),
    }
}
