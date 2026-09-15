//! Commands the user interface may call (SPEC §2).
//!
//! Errors cross this boundary as stable codes, never as sentences: all user
//! visible text comes from the interface dictionaries (SPEC §9.6). Dialog
//! titles and filter names are therefore passed in by the interface.

use std::path::PathBuf;

use onsa_audio::dsp::autoeq;
use onsa_audio::dsp::eq::{auto_preamp_db, response_curve, GRAPHIC_FREQS, MAX_BANDS};
use onsa_library::m3u::{read_m3u8, write_m3u8};
use onsa_library::{PathStyle, Rules};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_dialog::{DialogExt, FilePath};
use tauri_plugin_opener::OpenerExt;

use crate::dto::{
    AlbumDto, AutoEqDto, CurveDto, DeviceDto, ImportDto, NameCountDto, PlayContext, PlaylistDto,
    QueueEntryDto, QueuePlace, SearchDto, SortKey, TrackDto,
};
pub use crate::error::ErrorCode;
use crate::keys::{self, KeyStatus};
use crate::library::{LibraryService, ScanStatus, PLAYLISTS_EVENT};
use crate::logging::Logging;
use crate::net;
use crate::player::{Player, Snapshot, Watching};
use crate::session::{self, WindowMode};
use crate::settings::{
    self, BandPrefs, DisplayPrefs, DspPrefs, MetadataPrefs, OutputPrefs, PlaybackPrefs,
};
use crate::sleep::{self, SleepTimer};
use crate::theme::{self, Theme};
use crate::tray::{self, TrayLabels};

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

/// The folder the user's own themes live in.
fn theme_dir(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .app_config_dir()
        .ok()
        .map(|dir| theme::user_dir(&dir))
}

/// Lists the themes that can be chosen: the built-in ones, then the user's.
#[tauri::command]
pub fn theme_list(app: AppHandle) -> Vec<Theme> {
    let mut themes = theme::builtin_themes();
    if let Some(dir) = theme_dir(&app) {
        themes.extend(theme::user_themes(&dir));
    }
    themes
}

/// Reads one theme by identifier.
#[tauri::command]
pub fn theme_get(app: AppHandle, id: String) -> Result<Theme, ErrorCode> {
    if let Some(theme) = theme::builtin_theme(&id) {
        return Ok(theme);
    }
    theme_dir(&app)
        .map(|dir| theme::user_themes(&dir))
        .unwrap_or_default()
        .into_iter()
        .find(|theme| theme.id == id)
        .ok_or(ErrorCode::ThemeNotFound)
}

/// Opens the folder the user's own themes live in, creating it first so
/// there is somewhere to drop a theme (SPEC §9.3).
#[tauri::command]
pub fn theme_open_folder(app: AppHandle) -> Result<(), ErrorCode> {
    let Some(dir) = theme_dir(&app) else {
        return Err(ErrorCode::Io);
    };
    std::fs::create_dir_all(&dir).map_err(|error| {
        tracing::warn!(dir = %dir.display(), "theme folder cannot be made: {error}");
        ErrorCode::Io
    })?;
    app.opener()
        .open_path(dir.display().to_string(), None::<&str>)
        .map_err(|error| {
            tracing::warn!("theme folder cannot be opened: {error}");
            ErrorCode::Io
        })
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
    /// Whether the window is in mini player mode.
    pub mini: bool,
    /// Whether closing the window leaves Onsa in the tray.
    pub close_to_tray: bool,
}

/// Reports what the window restores at start.
#[tauri::command]
pub fn app_state(
    library: State<'_, LibraryService>,
    logging: State<'_, Logging>,
    mode: State<'_, WindowMode>,
) -> Result<AppState, ErrorCode> {
    let (theme_id, locale, has_folders) = library.read(|library| {
        Ok((
            settings::load::<Option<String>>(library, settings::THEME_KEY),
            settings::load::<Option<String>>(library, settings::LOCALE_KEY),
            !library.folder_paths()?.is_empty(),
        ))
    })?;
    Ok(AppState {
        theme_id,
        locale,
        has_folders,
        log_debug: logging.debug(),
        log_dir: logging.dir().display().to_string(),
        graphic_freqs: GRAPHIC_FREQS,
        max_bands: MAX_BANDS,
        mini: mode.mini(),
        close_to_tray: library
            .read(|library| Ok(settings::load::<bool>(library, settings::CLOSE_TO_TRAY_KEY)))?,
    })
}

/// Remembers the chosen theme.
#[tauri::command]
pub fn set_theme(
    app: AppHandle,
    library: State<'_, LibraryService>,
    id: String,
) -> Result<(), ErrorCode> {
    theme_get(app, id.clone())?;
    library.read(|library| settings::save(library, settings::THEME_KEY, &id))
}

/// Remembers the chosen language.
#[tauri::command]
pub fn set_locale(library: State<'_, LibraryService>, locale: String) -> Result<(), ErrorCode> {
    library.read(|library| settings::save(library, settings::LOCALE_KEY, &locale))
}

/// Changes and remembers what the interface shows beyond the theme.
#[tauri::command]
pub fn settings_set_display(
    library: State<'_, LibraryService>,
    display: DisplayPrefs,
) -> Result<(), ErrorCode> {
    library.read(|library| settings::save(library, settings::DISPLAY_KEY, &display))
}

/// Says which of the things that need the analysis tap are on screen. The
/// tap, the FFT and the frames all stop when none of them are (SPEC §4.4).
#[tauri::command]
pub fn player_watch(player: State<'_, Player>, watching: Watching) {
    player.watch(watching);
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

/// The tracks a context names, and where in them to start.
fn rows_for(
    library: &State<'_, LibraryService>,
    context: PlayContext,
) -> Result<(Vec<onsa_library::TrackRow>, usize), ErrorCode> {
    library.read(|library| {
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
            PlayContext::Artist { name, index } => (library.artist_tracks(&name)?, index),
            PlayContext::Genre { name, index } => (library.genre_tracks(&name)?, index),
            PlayContext::Folder { path, index } => (library.directory_tracks(&path)?, index),
            // The queue is only ever a source for a playlist: `tracks_of`
            // reads it from the player, and playing it is already happening.
            PlayContext::Queue => (Vec::new(), 0),
            PlayContext::Playlist { playlist_id, index } => {
                (library.playlist_tracks(playlist_id)?, index)
            }
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
    })
}

/// Replaces the queue and starts playing.
#[tauri::command]
pub fn player_play(
    library: State<'_, LibraryService>,
    player: State<'_, Player>,
    context: PlayContext,
) -> Result<(), ErrorCode> {
    let (rows, index) = rows_for(&library, context)?;
    player.play(rows, index)
}

/// Adds tracks to the queue, after the playing one or at the end.
#[tauri::command]
pub fn player_enqueue(
    library: State<'_, LibraryService>,
    player: State<'_, Player>,
    context: PlayContext,
    place: QueuePlace,
) -> Result<(), ErrorCode> {
    let (rows, _) = rows_for(&library, context)?;
    player.enqueue(rows, place)
}

/// Removes one queue entry, named by its id.
#[tauri::command]
pub fn player_remove(player: State<'_, Player>, entry: u64) -> Result<(), ErrorCode> {
    player.remove(entry)
}

/// Moves a queue entry to another place in the queue.
#[tauri::command]
pub fn player_move(player: State<'_, Player>, entry: u64, to: usize) -> Result<(), ErrorCode> {
    player.move_entry(entry, to)
}

/// Empties the queue.
#[tauri::command]
pub fn player_clear(player: State<'_, Player>) -> Result<(), ErrorCode> {
    player.clear()
}

/// Shuffles the queue, or puts the listed order back.
#[tauri::command]
pub fn player_shuffle(player: State<'_, Player>, shuffle: bool) -> Result<(), ErrorCode> {
    player.set_shuffle(shuffle)
}

/// The tracks of one artist.
#[tauri::command]
pub fn library_artist_tracks(
    library: State<'_, LibraryService>,
    name: String,
) -> Result<Vec<TrackDto>, ErrorCode> {
    library.read(|library| {
        Ok(library
            .artist_tracks(&name)?
            .iter()
            .map(TrackDto::from)
            .collect())
    })
}

/// The tracks of one genre.
#[tauri::command]
pub fn library_genre_tracks(
    library: State<'_, LibraryService>,
    name: String,
) -> Result<Vec<TrackDto>, ErrorCode> {
    library.read(|library| {
        Ok(library
            .genre_tracks(&name)?
            .iter()
            .map(TrackDto::from)
            .collect())
    })
}

/// The tracks inside one folder.
#[tauri::command]
pub fn library_folder_tracks(
    library: State<'_, LibraryService>,
    path: String,
) -> Result<Vec<TrackDto>, ErrorCode> {
    library.read(|library| {
        Ok(library
            .directory_tracks(&path)?
            .iter()
            .map(TrackDto::from)
            .collect())
    })
}

/// Number of artists.
#[tauri::command]
pub fn library_artist_count(library: State<'_, LibraryService>) -> Result<u64, ErrorCode> {
    library.read(|library| library.artist_count())
}

/// One page of artists.
#[tauri::command]
pub fn library_artists(
    library: State<'_, LibraryService>,
    offset: usize,
    limit: usize,
) -> Result<Vec<NameCountDto>, ErrorCode> {
    library.read(|library| {
        Ok(library
            .artists_page(offset, limit.min(1000))?
            .iter()
            .map(NameCountDto::from)
            .collect())
    })
}

/// Number of genres.
#[tauri::command]
pub fn library_genre_count(library: State<'_, LibraryService>) -> Result<u64, ErrorCode> {
    library.read(|library| library.genre_count())
}

/// One page of genres.
#[tauri::command]
pub fn library_genres(
    library: State<'_, LibraryService>,
    offset: usize,
    limit: usize,
) -> Result<Vec<NameCountDto>, ErrorCode> {
    library.read(|library| {
        Ok(library
            .genres_page(offset, limit.min(1000))?
            .iter()
            .map(NameCountDto::from)
            .collect())
    })
}

/// Number of folders that hold tracks.
#[tauri::command]
pub fn library_folder_count(library: State<'_, LibraryService>) -> Result<u64, ErrorCode> {
    library.read(|library| library.directory_count())
}

/// One page of the folders tracks sit in.
#[tauri::command]
pub fn library_directories(
    library: State<'_, LibraryService>,
    offset: usize,
    limit: usize,
) -> Result<Vec<NameCountDto>, ErrorCode> {
    library.read(|library| {
        Ok(library
            .directories_page(offset, limit.min(1000))?
            .iter()
            .map(NameCountDto::from)
            .collect())
    })
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

/// Plays another queue entry, named by its id.
#[tauri::command]
pub fn player_jump(player: State<'_, Player>, entry: u64) -> Result<(), ErrorCode> {
    player.jump(entry)
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
    pub items: Vec<QueueEntryDto>,
    /// Id of the current entry.
    pub current: Option<u64>,
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
    /// What the interface shows beyond the theme.
    pub display: DisplayPrefs,
}

/// Reads the settings.
#[tauri::command]
pub fn settings_get(
    library: State<'_, LibraryService>,
    player: State<'_, Player>,
) -> Result<SettingsDto, ErrorCode> {
    let (output, playback, dsp) = player.prefs();
    let display = library.read(|library| {
        Ok(settings::load::<DisplayPrefs>(
            library,
            settings::DISPLAY_KEY,
        ))
    })?;
    Ok(SettingsDto {
        output,
        playback,
        dsp,
        display,
    })
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

/// Sets the sleep timer (SPEC §3.5).
#[tauri::command]
pub fn sleep_arm(
    app: AppHandle,
    timer: State<'_, SleepTimer>,
    plan: sleep::Plan,
) -> Result<(), ErrorCode> {
    timer.arm(&app, plan);
    Ok(())
}

/// Cancels the sleep timer.
#[tauri::command]
pub fn sleep_cancel(app: AppHandle, timer: State<'_, SleepTimer>) -> Result<(), ErrorCode> {
    timer.cancel_and_publish(&app);
    Ok(())
}

/// Where the sleep timer stands.
#[tauri::command]
pub fn sleep_status(timer: State<'_, SleepTimer>) -> sleep::Status {
    timer.status()
}

/// Builds the tray icon with the interface's own words, and again when the
/// language changes (SPEC §13).
#[tauri::command]
pub fn tray_setup(app: AppHandle, labels: TrayLabels) -> Result<(), ErrorCode> {
    tray::build(&app, &labels).map_err(|error| {
        tracing::warn!("the tray icon cannot be built: {error}");
        ErrorCode::Io
    })
}

/// Chooses whether closing the window leaves Onsa in the tray.
#[tauri::command]
pub fn set_close_to_tray(
    library: State<'_, LibraryService>,
    enabled: bool,
) -> Result<(), ErrorCode> {
    library.read(|library| settings::save(library, settings::CLOSE_TO_TRAY_KEY, &enabled))
}

/// Switches the mini player on or off (SPEC §9.2). The answer is the mode
/// the window ended up in, which is what the interface shows.
#[tauri::command]
pub fn window_set_mini(app: AppHandle, mini: bool) -> Result<bool, ErrorCode> {
    session::set_mini(&app, mini).map_err(|error| {
        tracing::warn!("the window mode cannot change: {error}");
        ErrorCode::Io
    })
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

// ----------------------------------------------------------- playlists (M6)

/// Lists every playlist, manual and smart.
#[tauri::command]
pub fn playlist_list(library: State<'_, LibraryService>) -> Result<Vec<PlaylistDto>, ErrorCode> {
    library.read(|library| Ok(library.playlists()?.iter().map(PlaylistDto::from).collect()))
}

/// One playlist, if it is still there.
#[tauri::command]
pub fn playlist_get(
    library: State<'_, LibraryService>,
    id: i64,
) -> Result<Option<PlaylistDto>, ErrorCode> {
    library.read(|library| Ok(library.playlist(id)?.as_ref().map(PlaylistDto::from)))
}

/// The tracks of a playlist: the stored order for a manual one, and what the
/// rules match right now for a smart one.
#[tauri::command]
pub fn playlist_tracks(
    library: State<'_, LibraryService>,
    id: i64,
) -> Result<Vec<TrackDto>, ErrorCode> {
    library.read(|library| {
        Ok(library
            .playlist_tracks(id)?
            .iter()
            .map(TrackDto::from)
            .collect())
    })
}

/// Makes a playlist. With rules it is a smart one, without them a manual one.
#[tauri::command]
pub fn playlist_create(
    app: AppHandle,
    library: State<'_, LibraryService>,
    name: String,
    rules: Option<Rules>,
) -> Result<i64, ErrorCode> {
    let id = library.read(|library| match &rules {
        Some(rules) => library.create_smart_playlist(&name, rules),
        None => library.create_playlist(&name),
    })?;
    let _ = app.emit(PLAYLISTS_EVENT, ());
    Ok(id)
}

/// Renames a playlist.
#[tauri::command]
pub fn playlist_rename(
    app: AppHandle,
    library: State<'_, LibraryService>,
    id: i64,
    name: String,
) -> Result<(), ErrorCode> {
    library.read(|library| library.rename_playlist(id, &name))?;
    let _ = app.emit(PLAYLISTS_EVENT, ());
    Ok(())
}

/// Replaces the rules of a smart playlist.
#[tauri::command]
pub fn playlist_set_rules(
    app: AppHandle,
    library: State<'_, LibraryService>,
    id: i64,
    rules: Rules,
) -> Result<(), ErrorCode> {
    library.read(|library| library.set_playlist_rules(id, &rules))?;
    let _ = app.emit(PLAYLISTS_EVENT, ());
    Ok(())
}

/// Deletes a playlist. The tracks themselves stay in the library.
#[tauri::command]
pub fn playlist_delete(
    app: AppHandle,
    library: State<'_, LibraryService>,
    id: i64,
) -> Result<(), ErrorCode> {
    library.read(|library| library.delete_playlist(id))?;
    let _ = app.emit(PLAYLISTS_EVENT, ());
    Ok(())
}

/// Copies a playlist under another name.
#[tauri::command]
pub fn playlist_duplicate(
    app: AppHandle,
    library: State<'_, LibraryService>,
    id: i64,
    name: String,
) -> Result<i64, ErrorCode> {
    let copy = library.write(|library| library.duplicate_playlist(id, &name))?;
    let _ = app.emit(PLAYLISTS_EVENT, ());
    Ok(copy)
}

/// Adds whatever a context stands for to the end of a manual playlist.
#[tauri::command]
pub fn playlist_add(
    app: AppHandle,
    library: State<'_, LibraryService>,
    player: State<'_, Player>,
    id: i64,
    context: PlayContext,
) -> Result<usize, ErrorCode> {
    let tracks = tracks_of(&library, &player, context)?;
    library.write(|library| library.add_to_playlist(id, &tracks))?;
    let _ = app.emit(PLAYLISTS_EVENT, ());
    Ok(tracks.len())
}

/// Removes the entry at `position` from a manual playlist.
#[tauri::command]
pub fn playlist_remove(
    app: AppHandle,
    library: State<'_, LibraryService>,
    id: i64,
    position: usize,
) -> Result<(), ErrorCode> {
    library.write(|library| library.remove_from_playlist(id, position))?;
    let _ = app.emit(PLAYLISTS_EVENT, ());
    Ok(())
}

/// Moves an entry of a manual playlist, as a drag does.
#[tauri::command]
pub fn playlist_move(
    app: AppHandle,
    library: State<'_, LibraryService>,
    id: i64,
    from: usize,
    to: usize,
) -> Result<(), ErrorCode> {
    library.write(|library| library.move_in_playlist(id, from, to))?;
    let _ = app.emit(PLAYLISTS_EVENT, ());
    Ok(())
}

/// Saves the queue as it stands as a new manual playlist (SPEC §6.2).
#[tauri::command]
pub fn playlist_from_queue(
    app: AppHandle,
    library: State<'_, LibraryService>,
    player: State<'_, Player>,
    name: String,
) -> Result<i64, ErrorCode> {
    let tracks: Vec<i64> = player
        .queue()
        .0
        .iter()
        .map(|entry| entry.track.id)
        .collect();
    let id = library.write(|library| {
        let id = library.create_playlist(&name)?;
        library.add_to_playlist(id, &tracks)?;
        Ok(id)
    })?;
    let _ = app.emit(PLAYLISTS_EVENT, ());
    Ok(id)
}

/// The tracks a rule set matches right now, for the rule editor's preview.
#[tauri::command]
pub fn smart_preview(
    library: State<'_, LibraryService>,
    rules: Rules,
    limit: usize,
) -> Result<Vec<TrackDto>, ErrorCode> {
    library.read(|library| {
        Ok(library
            .smart_tracks(&rules)?
            .iter()
            .take(limit.min(500))
            .map(TrackDto::from)
            .collect())
    })
}

/// The track ids a context stands for, in order.
fn tracks_of(
    library: &State<'_, LibraryService>,
    player: &State<'_, Player>,
    context: PlayContext,
) -> Result<Vec<i64>, ErrorCode> {
    if matches!(context, PlayContext::Queue) {
        return Ok(player
            .queue()
            .0
            .iter()
            .map(|entry| entry.track.id)
            .collect());
    }
    let (rows, _) = rows_for(library, context)?;
    Ok(rows.iter().map(|row| row.id).collect())
}

/// Writes a playlist to an M3U8 file the listener picks (SPEC §6.4).
///
/// Paths are written relative to the file where they can be, so a playlist
/// saved beside the music travels with it.
#[tauri::command]
pub async fn playlist_export(
    app: AppHandle,
    id: i64,
    title: String,
    filter_name: String,
) -> Result<Option<String>, ErrorCode> {
    let (name, tracks) = app.state::<LibraryService>().read(|library| {
        let name = library
            .playlist(id)?
            .map(|row| row.name)
            .unwrap_or_else(|| "playlist".to_string());
        Ok((name, library.playlist_tracks(id)?))
    })?;
    let chooser = app.clone();
    let picked = tauri::async_runtime::spawn_blocking(move || {
        chooser
            .dialog()
            .file()
            .set_title(title)
            .set_file_name(format!("{}.m3u8", tidy_file_name(&name)))
            .add_filter(filter_name, &["m3u8", "m3u"])
            .blocking_save_file()
    })
    .await
    .map_err(|_| ErrorCode::Dialog)?;
    let Some(path) = into_path(picked) else {
        return Ok(None);
    };
    write_m3u8(&path, &tracks, PathStyle::Relative)?;
    Ok(Some(path.display().to_string()))
}

/// Reads an M3U8 file into a new manual playlist (SPEC §6.4).
///
/// Entries the library does not know are reported back rather than dropped
/// quietly, so the listener can see what did not come along.
#[tauri::command]
pub async fn playlist_import(
    app: AppHandle,
    title: String,
    filter_name: String,
) -> Result<Option<ImportDto>, ErrorCode> {
    let chooser = app.clone();
    let picked = tauri::async_runtime::spawn_blocking(move || {
        chooser
            .dialog()
            .file()
            .set_title(title)
            .add_filter(filter_name, &["m3u8", "m3u"])
            .blocking_pick_file()
    })
    .await
    .map_err(|_| ErrorCode::Dialog)?;
    let Some(path) = into_path(picked) else {
        return Ok(None);
    };
    let name = path
        .file_stem()
        .map(|stem| stem.to_string_lossy().to_string())
        .filter(|stem| !stem.trim().is_empty())
        .unwrap_or_else(|| "M3U8".to_string());

    let library = app.state::<LibraryService>();
    let report = library.read(|library| read_m3u8(&path, library))?;
    let playlist_id = if report.tracks.is_empty() {
        None
    } else {
        let name = name.clone();
        let tracks = report.tracks.clone();
        Some(library.write(|library| {
            let id = library.create_playlist(&name)?;
            library.add_to_playlist(id, &tracks)?;
            Ok(id)
        })?)
    };
    let _ = app.emit(PLAYLISTS_EVENT, ());
    Ok(Some(ImportDto {
        playlist_id,
        name,
        found: report.tracks.len(),
        missing: report.missing,
    }))
}

/// Characters no file name may hold on Windows, replaced on both systems so
/// a playlist file travels between them.
const FORBIDDEN: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

/// A playlist name made safe to suggest as a file name.
fn tidy_file_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|letter| {
            if FORBIDDEN.contains(&letter) {
                '_'
            } else {
                letter
            }
        })
        .collect();
    let cleaned = cleaned.trim().trim_matches('.').trim().to_string();
    if cleaned.is_empty() {
        "playlist".to_string()
    } else {
        cleaned
    }
}

// ------------------------------------------------------- metadata and keys

/// What the interface is told about the metadata settings.
///
/// The AcoustID key itself is deliberately not here. The interface can set
/// it and can ask whether there is one, but it never reads it back: a key
/// that never leaves the backend cannot end up in a log or a report.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataDto {
    /// Whether Onsa may ask the internet about metadata at all.
    pub online: bool,
    /// Whether there is an AcoustID key, and where it came from.
    pub acoustid: KeyStatus,
}

fn metadata_dto(prefs: &MetadataPrefs) -> MetadataDto {
    MetadataDto {
        online: prefs.online,
        acoustid: KeyStatus::from(keys::acoustid(prefs.acoustid_key.as_deref()).as_ref()),
    }
}

/// The metadata settings, and where the AcoustID key comes from.
#[tauri::command]
pub fn metadata_get(library: State<'_, LibraryService>) -> Result<MetadataDto, ErrorCode> {
    let prefs = library.read(|library| {
        Ok(settings::load::<MetadataPrefs>(
            library,
            settings::METADATA_KEY,
        ))
    })?;
    Ok(metadata_dto(&prefs))
}

/// Turns the internet on or off for metadata, and stores a key.
///
/// `acoustid_key` left out leaves the stored key alone; an empty one clears
/// it, which falls back to the environment or the build.
#[tauri::command]
pub fn settings_set_metadata(
    app: AppHandle,
    online: bool,
    acoustid_key: Option<String>,
) -> Result<MetadataDto, ErrorCode> {
    let library = app.state::<LibraryService>();
    if !online {
        // Off means off, including a run that is going at this moment. What
        // it has already found stays where it is; it simply stops asking.
        app.state::<std::sync::Arc<crate::matching::Matching>>()
            .stop();
    }
    let mut prefs = library.read(|library| {
        Ok(settings::load::<MetadataPrefs>(
            library,
            settings::METADATA_KEY,
        ))
    })?;
    prefs.online = online;
    if let Some(key) = acoustid_key {
        let key = key.trim().to_string();
        prefs.acoustid_key = (!key.is_empty()).then_some(key);
    }
    library.read(|library| settings::save(library, settings::METADATA_KEY, &prefs))?;
    // The log says that a key was stored, never which one.
    tracing::info!(
        online = prefs.online,
        acoustid = prefs.acoustid_key.is_some(),
        "metadata settings stored"
    );
    Ok(metadata_dto(&prefs))
}

/// What came of asking AcoustID whether it knows this key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum KeyTest {
    /// AcoustID accepted the key.
    Works,
    /// AcoustID has no such key.
    Refused,
    /// There is no key to try.
    Missing,
    /// Looking things up on the internet is switched off.
    Offline,
    /// AcoustID could not be reached.
    Unreachable,
}

/// Asks AcoustID whether it knows the key Onsa would use (SPEC §14).
///
/// The question is a lookup by track id rather than by fingerprint, so no
/// audio has to be read: a well-formed request with a key AcoustID knows
/// comes back `ok` with nothing in it, and one with a key it does not know
/// comes back as an error naming the key. The key is sent to AcoustID and
/// nowhere else, and never appears in the answer or the log.
#[tauri::command]
pub async fn acoustid_test(app: AppHandle) -> Result<KeyTest, ErrorCode> {
    let prefs = app.state::<LibraryService>().read(|library| {
        Ok(settings::load::<MetadataPrefs>(
            library,
            settings::METADATA_KEY,
        ))
    })?;
    if !prefs.online {
        return Ok(KeyTest::Offline);
    }
    let Some(key) = keys::acoustid(prefs.acoustid_key.as_deref()) else {
        return Ok(KeyTest::Missing);
    };
    // The client blocks, so it must not run on the async runtime's thread.
    let outcome = tauri::async_runtime::spawn_blocking(move || net::acoustid_accepts(&key.value))
        .await
        .unwrap_or(KeyTest::Unreachable);
    tracing::info!(?outcome, "the AcoustID key was tried");
    Ok(outcome)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_info_starts_on_a_theme_that_exists() {
        let info = app_info();
        assert!(theme::builtin_themes()
            .iter()
            .any(|theme| theme.id == info.default_theme_id));
    }

    #[test]
    fn an_unknown_theme_is_reported_as_a_code() {
        assert!(theme::builtin_theme("tidak-ada").is_none());
        let json = serde_json::to_string(&ErrorCode::ThemeNotFound).expect("serialisable");
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
