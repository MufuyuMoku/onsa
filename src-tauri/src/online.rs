//! The commands behind looking a track up on the internet (SPEC §8, §14).
//!
//! This is the only way the interface can reach AcoustID, MusicBrainz or the
//! Cover Art Archive, and every one of these commands begins by asking
//! whether that is allowed at all. Looking things up is off until it is
//! turned on, and turning it off stops a run that is going.
//!
//! What these commands produce is a list of suggestions. Applying them is
//! not here: it goes through `tidy_preview_edits` and `tidy_apply_edits`
//! like every other change on the Tidy page, so a suggestion from the
//! internet passes exactly the same folder, the same summary and the same
//! undo as a value typed by hand. There is no second door.

use std::path::PathBuf;
use std::sync::Arc;

use onsa_downloader::{Program, Programs, Where};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

use crate::error::ErrorCode;
use crate::keys;
use crate::library::LibraryService;
use crate::matching::{self, Matching, Orders, Progress};
use crate::proposal::TrackMatch;
use crate::settings::{self, MetadataPrefs, ProgramPrefs, TidyPrefs};
use crate::tidy::{ReportDto, SummaryDto};
use crate::Folders;

/// Most tracks one matching run may look at.
///
/// The same cap the rest of the Tidy page keeps to, for the same reason: a
/// run has to be read through by a person afterwards.
use onsa_library::edits::MAX_BATCH;

/// What the interface may know about an outside program.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgramDto {
    /// Which one, by its name.
    pub key: String,
    /// Whether Onsa can run it.
    pub present: bool,
    /// Where it was found: `managed`, `chosen` or `system`.
    pub from: Option<String>,
    /// The file, so the listener can see which copy is in use.
    pub path: Option<String>,
    /// What it says its version is.
    pub version: Option<String>,
}

/// How a matching run stands, with everything it has found.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchStateDto {
    /// Whether a run is going, and how far along.
    #[serde(flatten)]
    pub progress: Progress,
    /// What has been found so far. Kept whole, whatever happened to the run.
    pub found: Vec<TrackMatch>,
    /// Whether looking things up is allowed at all.
    pub online: bool,
    /// Whether there is a key to look things up with.
    pub key: bool,
    /// Whether the fingerprinter is installed.
    pub fingerprinter: bool,
}

/// One cover the listener said yes to.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverPick {
    /// The track it goes on.
    pub track_id: i64,
    /// Which picture: the name the suggestion carried.
    pub key: String,
}

fn metadata(library: &LibraryService) -> Result<MetadataPrefs, ErrorCode> {
    library.read(|library| {
        Ok(settings::load::<MetadataPrefs>(
            library,
            settings::METADATA_KEY,
        ))
    })
}

fn program_prefs(library: &LibraryService) -> Result<ProgramPrefs, ErrorCode> {
    library.read(|library| {
        Ok(settings::load::<ProgramPrefs>(
            library,
            settings::PROGRAMS_KEY,
        ))
    })
}

/// The program manager, set up the way the listener asked for.
pub fn programs(app: &AppHandle) -> Result<Programs, ErrorCode> {
    let library = app.state::<LibraryService>();
    let prefs = program_prefs(&library)?;
    let bin = app.state::<Folders>().data.join("bin");
    let mut programs = Programs::new(bin).use_system(prefs.use_system);
    for (key, path) in &prefs.paths {
        if let Some(program) = Program::from_key(key) {
            programs.choose(program, PathBuf::from(path));
        }
    }
    Ok(programs)
}

/// The scope as the Tidy page has it.
fn scope(library: &LibraryService) -> Result<onsa_library::Scope, ErrorCode> {
    let prefs =
        library.read(|library| Ok(settings::load::<TidyPrefs>(library, settings::TIDY_KEY)))?;
    Ok(onsa_library::Scope {
        folder: prefs.folder.as_ref().map(PathBuf::from),
        limit: prefs.limit,
    }
    .take(prefs.limit))
}

/// Which outside programs are installed, and where they came from.
#[tauri::command]
pub async fn program_status(app: AppHandle) -> Result<Vec<ProgramDto>, ErrorCode> {
    // Asking a program its version means running it, which is not work for
    // the thread that answers the interface.
    tauri::async_runtime::spawn_blocking(move || {
        let programs = programs(&app)?;
        Ok(Program::ALL
            .into_iter()
            .map(|program| {
                let found = programs.find(program);
                ProgramDto {
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
                }
            })
            .collect())
    })
    .await
    .map_err(|_| ErrorCode::Library)?
}

/// Points one program at a file the listener chose, or forgets the choice.
#[tauri::command]
pub fn program_choose(
    library: State<'_, LibraryService>,
    program: String,
    path: Option<String>,
) -> Result<(), ErrorCode> {
    if Program::from_key(&program).is_none() {
        return Err(ErrorCode::Library);
    }
    let mut prefs = program_prefs(&library)?;
    match path
        .map(|path| path.trim().to_string())
        .filter(|p| !p.is_empty())
    {
        Some(path) => prefs.paths.insert(program.clone(), path),
        None => prefs.paths.remove(&program),
    };
    library.read(|library| settings::save(library, settings::PROGRAMS_KEY, &prefs))?;
    tracing::info!(program, "where to find an outside program was set");
    Ok(())
}

/// How a matching run stands, and everything it has found.
#[tauri::command]
pub fn match_state(app: AppHandle) -> Result<MatchStateDto, ErrorCode> {
    let library = app.state::<LibraryService>();
    let prefs = metadata(&library)?;
    let (progress, found) = app.state::<Arc<Matching>>().state();
    let programs = programs(&app)?;
    Ok(MatchStateDto {
        progress,
        found,
        online: prefs.online,
        key: keys::acoustid(prefs.acoustid_key.as_deref()).is_some(),
        fingerprinter: programs.have(Program::Fpcalc),
    })
}

/// Starts looking the tracks in the folder up.
///
/// It comes back at once: the run goes on a thread of its own, and what it
/// finds arrives as it is found.
#[tauri::command]
pub fn match_start(app: AppHandle, tracks: Vec<i64>) -> Result<MatchStateDto, ErrorCode> {
    let library = app.state::<LibraryService>();
    let prefs = metadata(&library)?;
    if !prefs.online {
        // The switch is the switch. Nothing here goes round it.
        return Err(ErrorCode::Offline);
    }
    if app.state::<Arc<Matching>>().busy() {
        return match_state(app.clone());
    }

    let scope = scope(&library)?;
    let wanted: Vec<onsa_library::TrackRow> = library.read(|library| {
        let inside = library.tracks_in_scope(&scope, MAX_BATCH)?;
        Ok(inside
            .into_iter()
            .filter(|row| tracks.is_empty() || tracks.contains(&row.id))
            .take(scope.limit)
            .collect())
    })?;
    if wanted.is_empty() {
        return match_state(app.clone());
    }

    let covers_dir = library.read(|library| Ok(library.covers_dir().to_path_buf()))?;
    let key = keys::acoustid(prefs.acoustid_key.as_deref()).map(|key| key.value);
    let programs = programs(&app)?;
    tracing::info!(
        tracks = wanted.len(),
        fingerprinter = programs.have(Program::Fpcalc),
        "a matching run is starting"
    );
    matching::start(
        &app,
        Orders {
            key,
            programs,
            covers_dir,
            tracks: wanted,
        },
    );
    match_state(app.clone())
}

/// Asks a matching run to stop. What it found stays.
#[tauri::command]
pub fn match_stop(app: AppHandle) -> Result<MatchStateDto, ErrorCode> {
    app.state::<Arc<Matching>>().stop();
    tracing::info!("a matching run was asked to stop");
    match_state(app.clone())
}

/// What putting these covers on would do, without storing any of them.
#[tauri::command]
pub fn tidy_preview_covers(
    library: State<'_, LibraryService>,
    tracks: Vec<i64>,
) -> Result<SummaryDto, ErrorCode> {
    let scope = scope(&library)?;
    Ok(library
        .read(|library| library.preview_covers(&scope, &tracks))?
        .into())
}

/// Puts the chosen covers on, as one run that can be taken back.
///
/// The pictures are the ones already fetched and looked at; nothing is
/// downloaded again here.
#[tauri::command]
pub async fn tidy_apply_covers(
    app: AppHandle,
    note: String,
    picks: Vec<CoverPick>,
) -> Result<ReportDto, ErrorCode> {
    let outcome = tauri::async_runtime::spawn_blocking(move || {
        let library = app.state::<LibraryService>();
        let scope = scope(&library)?;
        let covers_dir = library.read(|library| Ok(library.covers_dir().to_path_buf()))?;
        let matching = app.state::<Arc<Matching>>();
        let mut ready: Vec<(i64, Vec<u8>)> = Vec::new();
        for pick in picks {
            match matching.proposed_cover(&covers_dir, &pick.key) {
                Some(bytes) => ready.push((pick.track_id, bytes)),
                // A picture that is no longer waiting is one this run never
                // offered. It is left out rather than fetched again.
                None => tracing::warn!("a chosen cover is no longer there"),
            }
        }
        library.write(|library| library.apply_covers(&note, &scope, &ready))
    })
    .await
    .map_err(|_| ErrorCode::Library)??;
    Ok(outcome.into())
}
