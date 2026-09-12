//! Commands the user interface may call (SPEC §2).
//!
//! Errors cross this boundary as stable codes, never as sentences: all user
//! visible text comes from the interface dictionaries (SPEC §9.6). Dialog
//! titles and filter names are therefore passed in by the interface.

use std::path::PathBuf;

use onsa_audio::dsp::autoeq;
use onsa_audio::dsp::eq::{auto_preamp_db, response_curve, GRAPHIC_FREQS, MAX_BANDS};
use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::{DialogExt, FilePath};
use tauri_plugin_opener::OpenerExt;

use crate::dto::{
    AlbumDto, AutoEqDto, CurveDto, DeviceDto, NameCountDto, PlayContext, SearchDto, SortKey,
    TrackDto,
};
pub use crate::error::ErrorCode;
use crate::library::{LibraryService, ScanStatus};
use crate::logging::Logging;
use crate::player::{Player, Snapshot};
use crate::settings::{self, BandPrefs, DspPrefs, OutputPrefs, PlaybackPrefs};
use crate::theme::{self, Theme};

/// Points on an EQ response curve.
const CURVE_POINTS: usize = 240;
/// Rate the curve is drawn at while no output is open.
const CURVE_FALLBACK_RATE: u32 = 48_000;

/// What the interface needs to know about the running application before it
/// draws anything.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    /// Display name of the application.
    pub name: &'static str,
    /// Version of this build.
    pub version: &'static str,
    /// Identifier of the theme to use until the user picks one.
    pub default_theme_id: &'static str,
}

/// Reports name, version and starting theme.
#[tauri::command]
pub fn app_info() -> AppInfo {
    AppInfo {
        name: "Onsa",
        version: env!("CARGO_PKG_VERSION"),
        default_theme_id: theme::DEFAULT_THEME_ID,
    }
}

/// Lists the themes that can be chosen.
#[tauri::command]
pub fn theme_list() -> Vec<Theme> {
    theme::builtin_themes()
}

/// Reads one theme by identifier.
#[tauri::command]
pub fn theme_get(id: String) -> Result<Theme, ErrorCode> {
    theme::builtin_theme(&id).ok_or(ErrorCode::ThemeNotFound)
}

/// What the window restores at start.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    /// The chosen theme, if the user picked one.
    pub theme_id: Option<String>,
    /// The chosen language, if the user picked one.
    pub locale: Option<String>,
    /// Whether the library has a folder; without one the first screen shows.
    pub has_folders: bool,
    /// Whether debug logging is on.
    pub log_debug: bool,
    /// The log folder.
    pub log_dir: String,
    /// Centre frequencies of the graphic EQ.
    pub graphic_freqs: [f64; 10],
    /// Most parametric bands.
    pub max_bands: usize,
}

/// Reports what the window restores at start.
#[tauri::command]
pub fn app_state(
    library: State<'_, LibraryService>,
    logging: State<'_, Logging>,
) -> Result<AppState, ErrorCode> {
    let (theme_id, locale, has_folders) = library.read(|library| {
        Ok((
            settings::load::<Option<String>>(library, settings::THEME_KEY),
            settings::load::<Option<String>>(library, settings::LOCALE_KEY),
            !library.folder_paths()?.is_empty(),
        ))
    })?;
    Ok(AppState {
        theme_id: theme_id.filter(|id| theme::builtin_theme(id).is_some()),
        locale,
        has_folders,
        log_debug: logging.debug(),
        log_dir: logging.dir().display().to_string(),
        graphic_freqs: GRAPHIC_FREQS,
        max_bands: MAX_BANDS,
    })
}

/// Remembers the chosen theme.
#[tauri::command]
pub fn set_theme(library: State<'_, LibraryService>, id: String) -> Result<(), ErrorCode> {
    if theme::builtin_theme(&id).is_none() {
        return Err(ErrorCode::ThemeNotFound);
    }
    library.read(|library| settings::save(library, settings::THEME_KEY, &id))
}

/// Remembers the chosen language.
#[tauri::command]
pub fn set_locale(library: State<'_, LibraryService>, locale: String) -> Result<(), ErrorCode> {
    library.read(|library| settings::save(library, settings::LOCALE_KEY, &locale))
}

fn into_path(picked: Option<FilePath>) -> Option<PathBuf> {
    picked.and_then(|file| file.into_path().ok())
}

/// Asks for a music folder.
#[tauri::command]
pub async fn pick_folder(app: AppHandle, title: String) -> Result<Option<String>, ErrorCode> {
    let picked = tauri::async_runtime::spawn_blocking(move || {
        app.dialog().file().set_title(title).blocking_pick_folder()
    })
    .await
    .map_err(|_| ErrorCode::Dialog)?;
    Ok(into_path(picked).map(|path| path.display().to_string()))
}

/// Adds a folder to the library and scans it.
#[tauri::command]
pub fn library_add_folder(
    library: State<'_, LibraryService>,
    path: String,
) -> Result<(), ErrorCode> {
    library.add_folder(PathBuf::from(path))
}

/// Scans every folder again.
#[tauri::command]
pub fn library_rescan(library: State<'_, LibraryService>) -> Result<(), ErrorCode> {
    library.rescan()
}

/// Where the current or last scan stands.
#[tauri::command]
pub fn library_scan_status(library: State<'_, LibraryService>) -> ScanStatus {
    library.status()
}

/// The library folders with their track counts.
#[tauri::command]
pub fn library_folders(library: State<'_, LibraryService>) -> Result<Vec<NameCountDto>, ErrorCode> {
    library.read(|library| Ok(library.folders()?.iter().map(NameCountDto::from).collect()))
}

/// Number of tracks.
#[tauri::command]
pub fn library_track_count(library: State<'_, LibraryService>) -> Result<u64, ErrorCode> {
    library.read(|library| library.track_count())
}

/// One page of the track list.
#[tauri::command]
pub fn library_tracks(
    library: State<'_, LibraryService>,
    sort: SortKey,
    descending: bool,
    offset: usize,
    limit: usize,
) -> Result<Vec<TrackDto>, ErrorCode> {
    library.read(|library| {
        Ok(library
            .tracks_page(sort.into(), descending, offset, limit.min(1000))?
            .iter()
            .map(TrackDto::from)
            .collect())
    })
}

/// Number of albums.
#[tauri::command]
pub fn library_album_count(library: State<'_, LibraryService>) -> Result<u64, ErrorCode> {
    library.read(|library| library.album_count())
}

/// One page of albums.
#[tauri::command]
pub fn library_albums(
    library: State<'_, LibraryService>,
    offset: usize,
    limit: usize,
) -> Result<Vec<AlbumDto>, ErrorCode> {
    library.read(|library| {
        Ok(library
            .albums_page(offset, limit.min(1000))?
            .iter()
            .map(AlbumDto::from)
            .collect())
    })
}

/// The tracks of an album.
#[tauri::command]
pub fn library_album_tracks(
    library: State<'_, LibraryService>,
    album_id: i64,
) -> Result<Vec<TrackDto>, ErrorCode> {
    library.read(|library| {
        Ok(library
            .album_tracks(album_id)?
            .iter()
            .map(TrackDto::from)
            .collect())
    })
}

/// Searches the library.
#[tauri::command]
pub fn library_search(
    library: State<'_, LibraryService>,
    query: String,
    limit: usize,
) -> Result<SearchDto, ErrorCode> {
    library.read(|library| Ok(SearchDto::from(&library.search(&query, limit.min(500))?)))
}

/// Replaces the queue and starts playing.
#[tauri::command]
pub fn player_play(
    library: State<'_, LibraryService>,
    player: State<'_, Player>,
    context: PlayContext,
) -> Result<(), ErrorCode> {
    let (rows, index) = library.read(|library| {
        Ok(match context {
            PlayContext::Library {
                sort,
                descending,
                index,
            } => (
                library.tracks_page(sort.into(), descending, 0, i64::MAX as usize)?,
                index,
            ),
            PlayContext::Album { album_id, index } => (library.album_tracks(album_id)?, index),
            PlayContext::Tracks { ids, index } => {
                let mut rows = Vec::with_capacity(ids.len());
                for id in ids {
                    if let Some(row) = library.track(id)? {
                        rows.push(row);
                    }
                }
                (rows, index)
            }
        })
    })?;
    player.play(rows, index)
}

/// Pauses or plays.
#[tauri::command]
pub fn player_toggle(player: State<'_, Player>) -> Result<(), ErrorCode> {
    player.toggle()
}

/// Next track.
#[tauri::command]
pub fn player_next(player: State<'_, Player>) -> Result<(), ErrorCode> {
    player.next()
}

/// Previous track.
#[tauri::command]
pub fn player_previous(player: State<'_, Player>) -> Result<(), ErrorCode> {
    player.previous()
}

/// Seeks in the current track.
#[tauri::command]
pub fn player_seek(player: State<'_, Player>, seconds: f64) -> Result<(), ErrorCode> {
    player.seek(seconds)
}

/// Plays another queue entry.
#[tauri::command]
pub fn player_jump(player: State<'_, Player>, index: usize) -> Result<(), ErrorCode> {
    player.jump(index)
}

/// What the player shows.
#[tauri::command]
pub fn player_snapshot(player: State<'_, Player>) -> Snapshot {
    player.snapshot()
}

/// The queue.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueDto {
    /// Entries in order.
    pub items: Vec<TrackDto>,
    /// The current entry.
    pub current: Option<usize>,
}

/// Reads the queue.
#[tauri::command]
pub fn player_queue(player: State<'_, Player>) -> QueueDto {
    let (items, current) = player.queue();
    QueueDto { items, current }
}

/// Every setting group.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsDto {
    /// Output.
    pub output: OutputPrefs,
    /// Playback.
    pub playback: PlaybackPrefs,
    /// DSP chain.
    pub dsp: DspPrefs,
}

/// Reads the settings.
#[tauri::command]
pub fn settings_get(player: State<'_, Player>) -> SettingsDto {
    let (output, playback, dsp) = player.prefs();
    SettingsDto {
        output,
        playback,
        dsp,
    }
}

/// Changes and remembers the output.
#[tauri::command]
pub fn settings_set_output(
    library: State<'_, LibraryService>,
    player: State<'_, Player>,
    output: OutputPrefs,
) -> Result<(), ErrorCode> {
    library.read(|library| settings::save(library, settings::OUTPUT_KEY, &output))?;
    player.set_output(output)
}

/// Changes and remembers playback behaviour.
#[tauri::command]
pub fn settings_set_playback(
    library: State<'_, LibraryService>,
    player: State<'_, Player>,
    playback: PlaybackPrefs,
) -> Result<(), ErrorCode> {
    library.read(|library| settings::save(library, settings::PLAYBACK_KEY, &playback))?;
    player.set_playback(playback)
}

/// Changes and remembers the DSP chain.
#[tauri::command]
pub fn settings_set_dsp(
    library: State<'_, LibraryService>,
    player: State<'_, Player>,
    dsp: DspPrefs,
) -> Result<(), ErrorCode> {
    library.read(|library| settings::save(library, settings::DSP_KEY, &dsp))?;
    player.set_dsp(dsp)
}

/// Lists the output devices.
#[tauri::command]
pub fn output_devices() -> Result<Vec<DeviceDto>, ErrorCode> {
    Ok(onsa_audio::list_devices()?
        .iter()
        .map(DeviceDto::from)
        .collect())
}

/// The response curve of the EQ as set in `dsp` (whether or not it is on),
/// at the output's rate, with the preamp the auto preamp would set.
#[tauri::command]
pub fn eq_curve(player: State<'_, Player>, dsp: DspPrefs) -> CurveDto {
    let rate = player.output_rate().unwrap_or(CURVE_FALLBACK_RATE);
    let bands = dsp.engine().eq.bands();
    CurveDto {
        points: response_curve(&bands, rate, CURVE_POINTS),
        auto_preamp_db: auto_preamp_db(&bands, rate),
    }
}

/// Reads an Equalizer APO / AutoEQ preset chosen by the user. `None` when
/// the dialog was cancelled.
#[tauri::command]
pub async fn autoeq_import(
    app: AppHandle,
    title: String,
    filter_name: String,
) -> Result<Option<AutoEqDto>, ErrorCode> {
    let picked = tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .file()
            .set_title(title)
            .add_filter(filter_name, &["txt"])
            .blocking_pick_file()
    })
    .await
    .map_err(|_| ErrorCode::Dialog)?;
    let Some(path) = into_path(picked) else {
        return Ok(None);
    };
    let bytes = std::fs::read(&path).map_err(|error| {
        tracing::warn!(file = %path.display(), "EQ preset cannot be read: {error}");
        ErrorCode::Io
    })?;
    let import = autoeq::parse(&String::from_utf8_lossy(&bytes));
    for warning in &import.warnings {
        tracing::info!(file = %path.display(), "EQ preset line skipped: {warning}");
    }
    if import.preset.bands.is_empty() {
        return Err(ErrorCode::AutoEqEmpty);
    }
    tracing::info!(file = %path.display(), bands = import.preset.bands.len(), "EQ preset imported");
    Ok(Some(AutoEqDto {
        bands: import
            .preset
            .bands
            .iter()
            .map(BandPrefs::from_engine)
            .collect(),
        preamp_db: import.preset.preamp_db,
        skipped: import.warnings.len(),
    }))
}

/// Writes the EQ as it is set now (its bands and the preamp in effect) as an
/// Equalizer APO / AutoEQ preset where the user chooses. Returns whether a
/// file was written.
#[tauri::command]
pub async fn autoeq_export(
    app: AppHandle,
    title: String,
    filter_name: String,
) -> Result<bool, ErrorCode> {
    let (bands, preamp_db) = {
        let player = app.state::<Player>();
        let (_, _, dsp) = player.prefs();
        let settings = dsp.engine();
        let rate = player.output_rate().unwrap_or(CURVE_FALLBACK_RATE);
        (
            settings.eq.bands(),
            f64::from(settings.effective_preamp_db(rate)),
        )
    };
    let picked = tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .file()
            .set_title(title)
            .set_file_name("ParametricEQ.txt")
            .add_filter(filter_name, &["txt"])
            .blocking_save_file()
    })
    .await
    .map_err(|_| ErrorCode::Dialog)?;
    let Some(path) = into_path(picked) else {
        return Ok(false);
    };
    std::fs::write(&path, autoeq::export(preamp_db, &bands)).map_err(|error| {
        tracing::warn!(file = %path.display(), "EQ preset cannot be written: {error}");
        ErrorCode::Io
    })?;
    tracing::info!(file = %path.display(), "EQ preset exported");
    Ok(true)
}

/// Opens the log folder in the file manager.
#[tauri::command]
pub fn log_open_folder(app: AppHandle) -> Result<(), ErrorCode> {
    let logging = app.state::<Logging>();
    let dir = logging.dir().display().to_string();
    app.opener().open_path(dir, None::<&str>).map_err(|error| {
        tracing::warn!("log folder cannot be opened: {error}");
        ErrorCode::Io
    })
}

/// Switches debug logging on or off and remembers it.
#[tauri::command]
pub fn log_set_debug(
    library: State<'_, LibraryService>,
    logging: State<'_, Logging>,
    enabled: bool,
) -> Result<(), ErrorCode> {
    logging.set_debug(enabled);
    library.read(|library| settings::save(library, settings::LOG_DEBUG_KEY, &enabled))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_info_starts_on_a_theme_that_exists() {
        let info = app_info();
        assert!(theme_list()
            .iter()
            .any(|theme| theme.id == info.default_theme_id));
    }

    #[test]
    fn an_unknown_theme_is_reported_as_a_code() {
        let error = theme_get("tidak-ada".to_string()).expect_err("should not resolve");
        let json = serde_json::to_string(&error).expect("serialisable");
        assert_eq!(json, r#"{"code":"theme_not_found"}"#);
    }

    #[test]
    fn play_contexts_read_as_the_interface_sends_them() {
        let context: PlayContext = serde_json::from_str(
            r#"{"kind":"library","sort":"artist","descending":false,"index":3}"#,
        )
        .unwrap();
        assert!(matches!(
            context,
            PlayContext::Library {
                sort: SortKey::Artist,
                index: 3,
                ..
            }
        ));
        let context: PlayContext =
            serde_json::from_str(r#"{"kind":"album","albumId":7,"index":0}"#).unwrap();
        assert!(matches!(context, PlayContext::Album { album_id: 7, .. }));
    }
}
