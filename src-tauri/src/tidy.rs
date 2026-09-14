//! Tidying metadata by hand, with the rails on (SPEC §8).
//!
//! Everything here goes through the library's own guards: a run is held to
//! a folder and to a count, it is summarised before there is anything to
//! press, and it is written down so it can be taken back. Nothing in this
//! module touches the internet; the matching that will comes later and hands
//! its proposals to the same commands.

use std::path::PathBuf;

use onsa_library::edits::{Change, Scope, Summary};
use onsa_library::overrides::Field;
use onsa_library::{Batch, BatchReport, RenamePlan, UndoReport};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;

use crate::dto::TrackDto;
use crate::error::ErrorCode;
use crate::library::LibraryService;
use crate::settings::{self, TidyPrefs};

/// Most tracks the interface asks for at a time.
const PAGE: usize = 500;

/// The folder a run is held to, and how many tracks it may take.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopeDto {
    /// The folder, when there is one. Nothing outside it can be touched.
    pub folder: Option<String>,
    /// Most tracks one run may change.
    pub limit: usize,
    /// What the cap is when nobody says otherwise.
    pub default_limit: usize,
    /// The largest cap Onsa will accept.
    pub max_limit: usize,
    /// How many tracks are inside the folder right now.
    pub tracks: usize,
}

/// One field a run would set.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeDto {
    /// The track.
    pub track_id: i64,
    /// The field, by the name the overrides table uses.
    pub field: String,
    /// The value; leaving it out clears the edit.
    pub value: Option<String>,
}

/// What a run would do, before there is a button to press.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SummaryDto {
    /// Tracks it would touch.
    pub tracks: usize,
    /// Field values it would change.
    pub fields: usize,
    /// Files whose tags it would rewrite.
    pub files_written: usize,
    /// Files it would move or rename.
    pub files_moved: usize,
    /// Tracks it would put a new cover on.
    pub covers: usize,
    /// What it would leave alone, and why.
    pub skipped: Vec<SkipDto>,
    /// Whether pressing apply would do anything at all.
    pub any: bool,
}

/// One reason a track would be left alone, and how many that is.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkipDto {
    /// A fixed word the interface translates.
    pub reason: String,
    /// How many tracks it covers.
    pub count: usize,
}

impl From<Summary> for SummaryDto {
    fn from(summary: Summary) -> Self {
        Self {
            tracks: summary.tracks,
            fields: summary.fields,
            files_written: summary.files_written,
            files_moved: summary.files_moved,
            covers: summary.covers,
            any: summary.would_do_anything(),
            skipped: summary
                .skipped
                .iter()
                .map(|(reason, count)| SkipDto {
                    reason: reason.name().to_string(),
                    count: *count,
                })
                .collect(),
        }
    }
}

/// What a run actually did.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportDto {
    /// The run, for taking it back.
    pub batch: Option<i64>,
    /// Fields or files changed.
    pub changed: usize,
    /// Left alone because there was nothing to do.
    pub unchanged: usize,
    /// Left alone because they sit outside the folder.
    pub out_of_scope: usize,
    /// Left over because the run hit its cap.
    pub over_limit: usize,
    /// What could not be done, with the reason. Those files are untouched.
    pub failed: Vec<String>,
}

impl From<BatchReport> for ReportDto {
    fn from(report: BatchReport) -> Self {
        Self {
            batch: report.batch,
            changed: report.changed,
            unchanged: report.unchanged,
            out_of_scope: report.out_of_scope,
            over_limit: report.over_limit,
            failed: report.failed,
        }
    }
}

/// One file's move, as the dry run lists it.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveDto {
    /// The track.
    pub track_id: i64,
    /// Where the file is now.
    pub from: String,
    /// Where the pattern would put it.
    pub to: String,
    /// Why it cannot be done, if it cannot.
    pub clash: Option<String>,
}

/// A whole dry run of a rename.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanDto {
    /// Every track the plan looked at.
    pub moves: Vec<MoveDto>,
    /// What it adds up to.
    pub summary: SummaryDto,
}

/// A run as the history lists it.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchDto {
    /// Its id.
    pub id: i64,
    /// What the run was.
    pub note: String,
    /// The folder it was held to.
    pub scope: Option<String>,
    /// When it ran.
    pub created_at: i64,
    /// When it was taken back, if it was.
    pub undone_at: Option<i64>,
    /// How many steps it holds.
    pub steps: usize,
}

impl From<Batch> for BatchDto {
    fn from(batch: Batch) -> Self {
        Self {
            id: batch.id,
            note: batch.note,
            scope: batch.scope,
            created_at: batch.created_at,
            undone_at: batch.undone_at,
            steps: batch.steps,
        }
    }
}

/// What came of taking a run back.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoDto {
    /// Steps put back.
    pub restored: usize,
    /// Steps that could not be, with the reason.
    pub failed: Vec<String>,
}

impl From<UndoReport> for UndoDto {
    fn from(report: UndoReport) -> Self {
        Self {
            restored: report.restored,
            failed: report.failed,
        }
    }
}

/// The field a name stands for, refusing anything else.
fn field_of(name: &str) -> Result<Field, ErrorCode> {
    Ok(match name {
        "title" => Field::Title,
        "artist" => Field::Artist,
        "album" => Field::Album,
        "album_artist" => Field::AlbumArtist,
        "track_number" => Field::TrackNumber,
        "disc_number" => Field::DiscNumber,
        "year" => Field::Year,
        "genre" => Field::Genre,
        "composer" => Field::Composer,
        _ => return Err(ErrorCode::Library),
    })
}

fn changes_of(changes: Vec<ChangeDto>) -> Result<Vec<Change>, ErrorCode> {
    changes
        .into_iter()
        .map(|change| {
            Ok(Change {
                track_id: change.track_id,
                field: field_of(&change.field)?,
                value: change.value.filter(|value| !value.trim().is_empty()),
            })
        })
        .collect()
}

/// The stored scope, read back as the library's own type.
fn scope_of(prefs: &TidyPrefs) -> Scope {
    Scope {
        folder: prefs.folder.as_ref().map(PathBuf::from),
        limit: prefs.limit,
    }
    .take(prefs.limit)
}

fn prefs(library: &LibraryService) -> Result<TidyPrefs, ErrorCode> {
    library.read(|library| Ok(settings::load::<TidyPrefs>(library, settings::TIDY_KEY)))
}

/// The folder a run is held to, and how many tracks are inside it.
#[tauri::command]
pub fn tidy_scope(library: State<'_, LibraryService>) -> Result<ScopeDto, ErrorCode> {
    let prefs = prefs(&library)?;
    let scope = scope_of(&prefs);
    let tracks = library.read(|library| Ok(library.tracks_in_scope(&scope, PAGE)?.len()))?;
    Ok(ScopeDto {
        folder: prefs.folder.clone(),
        limit: scope.limit,
        default_limit: onsa_library::edits::DEFAULT_BATCH,
        max_limit: onsa_library::edits::MAX_BATCH,
        tracks,
    })
}

/// Holds runs to a folder, or lets them loose on the whole library.
#[tauri::command]
pub fn tidy_set_scope(
    library: State<'_, LibraryService>,
    folder: Option<String>,
    limit: usize,
) -> Result<ScopeDto, ErrorCode> {
    let folder = folder.filter(|path| !path.trim().is_empty());
    let next = TidyPrefs {
        folder: folder.clone(),
        limit,
    };
    library.read(|library| settings::save(library, settings::TIDY_KEY, &next))?;
    tracing::info!(
        folder = folder.as_deref().unwrap_or("<the whole library>"),
        limit,
        "the tidy scope changed"
    );
    tidy_scope(library)
}

/// Asks for the folder a run should be held to.
#[tauri::command]
pub async fn tidy_pick_folder(app: AppHandle, title: String) -> Result<Option<String>, ErrorCode> {
    let picked = tauri::async_runtime::spawn_blocking(move || {
        app.dialog().file().set_title(title).blocking_pick_folder()
    })
    .await
    .map_err(|_| ErrorCode::Dialog)?;
    Ok(picked
        .and_then(|file| file.into_path().ok())
        .map(|path| path.display().to_string()))
}

/// The tracks a run would be allowed to look at.
#[tauri::command]
pub fn tidy_tracks(library: State<'_, LibraryService>) -> Result<Vec<TrackDto>, ErrorCode> {
    let scope = scope_of(&prefs(&library)?);
    library.read(|library| {
        Ok(library
            .tracks_in_scope(&scope, PAGE)?
            .iter()
            .map(TrackDto::from)
            .collect())
    })
}

/// What these edits would do, without doing any of it.
#[tauri::command]
pub fn tidy_preview_edits(
    library: State<'_, LibraryService>,
    changes: Vec<ChangeDto>,
) -> Result<SummaryDto, ErrorCode> {
    let scope = scope_of(&prefs(&library)?);
    let changes = changes_of(changes)?;
    Ok(library
        .read(|library| library.preview_edits(&scope, &changes))?
        .into())
}

/// Applies the edits, as one run that can be taken back.
#[tauri::command]
pub fn tidy_apply_edits(
    library: State<'_, LibraryService>,
    note: String,
    changes: Vec<ChangeDto>,
) -> Result<ReportDto, ErrorCode> {
    let scope = scope_of(&prefs(&library)?);
    let changes = changes_of(changes)?;
    Ok(library
        .write(|library| library.apply_edits(&note, &scope, &changes))?
        .into())
}

/// What writing the waiting edits into the files would do.
#[tauri::command]
pub fn tidy_preview_writes(
    library: State<'_, LibraryService>,
    tracks: Vec<i64>,
) -> Result<SummaryDto, ErrorCode> {
    let scope = scope_of(&prefs(&library)?);
    Ok(library
        .read(|library| library.preview_writes(&scope, &tracks))?
        .into())
}

/// Writes the waiting edits into the files themselves.
#[tauri::command]
pub async fn tidy_write(
    app: AppHandle,
    note: String,
    tracks: Vec<i64>,
) -> Result<ReportDto, ErrorCode> {
    // Reading and rewriting files is not work for the thread that answers
    // the interface.
    let outcome = tauri::async_runtime::spawn_blocking(move || {
        let library = app.state::<LibraryService>();
        let scope = scope_of(&prefs(&library)?);
        library.write(|library| library.write_to_files(&note, &scope, &tracks))
    })
    .await
    .map_err(|_| ErrorCode::Library)??;
    Ok(outcome.into())
}

/// Where a pattern would put each file, without moving anything.
#[tauri::command]
pub fn tidy_rename_plan(
    library: State<'_, LibraryService>,
    root: String,
    pattern: String,
    tracks: Vec<i64>,
) -> Result<PlanDto, ErrorCode> {
    let scope = scope_of(&prefs(&library)?);
    let plan: RenamePlan = library.read(|library| {
        library.rename_plan(std::path::Path::new(&root), &pattern, &scope, &tracks)
    })?;
    Ok(PlanDto {
        moves: plan
            .moves
            .iter()
            .map(|one| MoveDto {
                track_id: one.track_id,
                from: one.from.display().to_string(),
                to: one.to.display().to_string(),
                clash: one.clash.clone(),
            })
            .collect(),
        summary: plan.summary().into(),
    })
}

/// Carries out a rename, working the plan out again first.
///
/// The plan is not taken from the interface: it is made afresh from the same
/// pattern, so a name that has been taken in the meantime is caught now
/// rather than trusted from a minute ago.
#[tauri::command]
pub async fn tidy_rename_apply(
    app: AppHandle,
    note: String,
    root: String,
    pattern: String,
    tracks: Vec<i64>,
) -> Result<ReportDto, ErrorCode> {
    let outcome = tauri::async_runtime::spawn_blocking(move || {
        let library = app.state::<LibraryService>();
        let scope = scope_of(&prefs(&library)?);
        library.write(|library| {
            let plan =
                library.rename_plan(std::path::Path::new(&root), &pattern, &scope, &tracks)?;
            library.apply_rename(&note, &scope, &plan)
        })
    })
    .await
    .map_err(|_| ErrorCode::Library)??;
    Ok(outcome.into())
}

/// The runs Onsa has made, newest first.
#[tauri::command]
pub fn edit_history(
    library: State<'_, LibraryService>,
    limit: usize,
) -> Result<Vec<BatchDto>, ErrorCode> {
    library.read(|library| {
        Ok(library
            .batches(limit.clamp(1, 200))?
            .into_iter()
            .map(BatchDto::from)
            .collect())
    })
}

/// Takes a whole run back.
#[tauri::command]
pub async fn edit_undo(app: AppHandle, batch: i64) -> Result<UndoDto, ErrorCode> {
    let outcome = tauri::async_runtime::spawn_blocking(move || {
        let library = app.state::<LibraryService>();
        library.write(|library| library.undo_batch(batch))
    })
    .await
    .map_err(|_| ErrorCode::Library)??;
    tracing::info!(batch, restored = outcome.restored, "a run was taken back");
    Ok(outcome.into())
}
