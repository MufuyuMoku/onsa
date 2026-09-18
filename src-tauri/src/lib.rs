//! Onsa application shell.
//!
//! This crate is the glue: it owns application state, exposes commands to the
//! user interface, forwards engine and library events, and holds the
//! operating system integration. The feature crates meet only here (SPEC §2).

#![warn(missing_docs)]

mod acoustid;
mod commands;
mod downloads;
mod dto;
mod editor;
mod error;
mod keys;
mod library;
mod logging;
mod lyrics;
mod matching;
mod media;
mod musicbrainz;
mod net;
mod online;
mod open;
mod player;
mod proposal;
mod queue;
mod session;
mod settings;
mod sleep;
pub mod theme;
mod tidy;
mod tray;
mod ytdlp;

use anyhow::{Context, Result};
use tauri::{AppHandle, Manager, WebviewWindow, WindowEvent};

use crate::library::LibraryService;
use crate::logging::Logging;
use crate::player::Player;
use crate::session::WindowMode;

/// Starts the application: sets up logging, the library and the player,
/// builds the window and runs until the user closes it.
pub fn run() -> Result<()> {
    tauri::Builder::default()
        // A second Onsa hands its files to the one already running (SPEC §13).
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            tray::show_window(app);
            let paths = open::from_arguments(argv);
            if !paths.is_empty() {
                open::accept(app, paths);
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .register_uri_scheme_protocol("onsa", library::cover_protocol)
        .setup(|app| setup(app).map_err(Into::into))
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::app_state,
            commands::theme_list,
            commands::theme_get,
            commands::theme_open_folder,
            commands::set_theme,
            commands::set_locale,
            commands::pick_folder,
            commands::library_add_folder,
            commands::library_rescan,
            commands::library_scan_status,
            commands::library_folders,
            commands::library_track_count,
            commands::library_tracks,
            commands::library_album_count,
            commands::library_albums,
            commands::library_album_tracks,
            commands::library_search,
            commands::library_artist_count,
            commands::library_artists,
            commands::library_artist_tracks,
            commands::library_genre_count,
            commands::library_genres,
            commands::library_genre_tracks,
            commands::library_folder_count,
            commands::library_directories,
            commands::library_folder_tracks,
            commands::player_play,
            commands::player_toggle,
            commands::player_next,
            commands::player_previous,
            commands::player_seek,
            commands::player_jump,
            commands::player_snapshot,
            commands::player_queue,
            commands::player_enqueue,
            commands::player_remove,
            commands::player_move,
            commands::player_clear,
            commands::player_shuffle,
            commands::settings_get,
            commands::settings_set_display,
            commands::player_watch,
            commands::settings_set_output,
            commands::settings_set_playback,
            commands::settings_set_dsp,
            commands::output_devices,
            commands::eq_curve,
            commands::autoeq_import,
            commands::autoeq_export,
            commands::window_set_mini,
            commands::tray_setup,
            commands::sleep_arm,
            commands::sleep_cancel,
            commands::sleep_status,
            commands::set_close_to_tray,
            commands::log_open_folder,
            commands::log_set_debug,
            commands::playlist_list,
            commands::playlist_get,
            commands::playlist_tracks,
            commands::playlist_create,
            commands::playlist_rename,
            commands::playlist_set_rules,
            commands::playlist_delete,
            commands::playlist_duplicate,
            commands::playlist_add,
            commands::playlist_remove,
            commands::playlist_move,
            commands::playlist_from_queue,
            commands::playlist_export,
            commands::playlist_import,
            commands::smart_preview,
            commands::metadata_get,
            commands::settings_set_metadata,
            commands::acoustid_test,
            tidy::tidy_scope,
            tidy::tidy_set_scope,
            tidy::tidy_pick_folder,
            tidy::tidy_tracks,
            tidy::tidy_auto,
            tidy::tidy_preview_edits,
            tidy::tidy_apply_edits,
            tidy::tidy_preview_writes,
            tidy::tidy_write,
            tidy::tidy_rename_plan,
            tidy::tidy_rename_apply,
            tidy::edit_history,
            tidy::edit_undo,
            online::program_status,
            online::program_choose,
            online::program_pick,
            online::match_state,
            online::match_start,
            online::match_stop,
            online::tidy_preview_covers,
            online::tidy_apply_covers,
            downloads::binaries_status,
            downloads::binary_install,
            downloads::binary_stop,
            downloads::binaries_use_system,
            downloads::download_probe,
            downloads::download_queue,
            downloads::download_start,
            downloads::download_stop,
            lyrics::lyrics_for,
            lyrics::lyrics_state,
            lyrics::lyrics_look_again,
            lyrics::lyrics_offset,
            lyrics::lyrics_write_beside,
            lyrics::lyrics_settings,
            lyrics::lyrics_set_settings,
            editor::track_fields,
            editor::track_preview,
            editor::track_apply,
            editor::track_write,
        ])
        .run(tauri::generate_context!())
        .context("the application window could not be started")
}

/// Where Onsa keeps its own things, for the parts that need more than the
/// library: the outside programs live under the data folder, and the
/// suggested covers under the cache one.
#[derive(Debug, Clone)]
pub struct Folders {
    /// The database, the settings, and `bin/` (SPEC §7.1).
    pub data: std::path::PathBuf,
    /// The covers, and anything else that can be thrown away.
    pub cache: std::path::PathBuf,
}

/// A whole set of folders somewhere else, when `ONSA_DATA_DIR` names one.
///
/// Onsa keeps the listener's library where the system says applications
/// keep things. This is the way past that, and it exists for two reasons:
/// trying something out against a library that can be thrown away without
/// going anywhere near the real one, and a portable build later (SPEC §15,
/// M12). One variable moves the database, the covers and the log together,
/// because a library without its covers is not the same library.
fn folders_from_env() -> Option<(std::path::PathBuf, std::path::PathBuf, std::path::PathBuf)> {
    let root = std::env::var_os("ONSA_DATA_DIR")?;
    let root = std::path::PathBuf::from(root);
    if root.as_os_str().is_empty() {
        return None;
    }
    Some((root.join("data"), root.join("cache"), root.join("logs")))
}

/// Everything that needs the application's folders: logging, the library
/// with the stored settings, then the player.
fn setup(app: &mut tauri::App) -> Result<()> {
    let paths = app.path();
    let (data_dir, cache_dir, log_dir) = match folders_from_env() {
        Some(folders) => folders,
        None => (
            paths.app_data_dir().context("no data folder")?,
            paths.app_cache_dir().context("no cache folder")?,
            paths.app_log_dir().context("no log folder")?,
        ),
    };

    let logging = Logging::init(&log_dir);
    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        data = %data_dir.display(),
        "Onsa starting"
    );

    let library = LibraryService::start(
        app.handle().clone(),
        &data_dir.join("library.db"),
        &cache_dir,
    )
    .context("the library cannot be opened")?;
    let (output, playback, dsp, log_debug) = library
        .read(|library| {
            Ok((
                settings::load(library, settings::OUTPUT_KEY),
                settings::load(library, settings::PLAYBACK_KEY),
                settings::load(library, settings::DSP_KEY),
                settings::load::<bool>(library, settings::LOG_DEBUG_KEY),
            ))
        })
        .map_err(|code| anyhow::anyhow!("settings cannot be read: {code:?}"))?;
    if log_debug {
        logging.set_debug(true);
    }

    let player = Player::start(app.handle().clone(), output, playback, dsp);

    // The queue and the window come back as they were left (SPEC §13).
    let stored_queue: session::QueueState = session::load(&library, session::QUEUE_KEY);
    let position: f64 = session::load(&library, session::POSITION_KEY);
    if !stored_queue.ids.is_empty() {
        match library.read(|library| {
            let mut rows = Vec::with_capacity(stored_queue.ids.len());
            for id in &stored_queue.ids {
                if let Some(row) = library.track(*id)? {
                    rows.push(row);
                }
            }
            Ok(rows)
        }) {
            Ok(rows) => {
                tracing::info!(tracks = rows.len(), "queue restored, paused");
                let _ = player.restore(rows, stored_queue.current, position, stored_queue.shuffle);
            }
            Err(code) => tracing::warn!("the queue cannot be restored: {code:?}"),
        }
    }

    let window_state: session::WindowState = session::load(&library, session::WINDOW_KEY);

    app.manage(Folders {
        data: data_dir.clone(),
        cache: cache_dir.clone(),
    });
    app.manage(std::sync::Arc::new(matching::Matching::default()));
    app.manage(std::sync::Arc::new(downloads::Fetching::default()));
    app.manage(std::sync::Arc::new(downloads::Queue::default()));
    app.manage(std::sync::Arc::new(lyrics::Lookups::default()));
    app.manage(logging);
    app.manage(library);
    app.manage(player);
    app.manage(WindowMode::default());
    app.manage(sleep::SleepTimer::default());

    if let Some(window) = app.get_webview_window(session::MAIN_WINDOW) {
        session::restore_window(&window, &window_state, &app.state::<WindowMode>());
        remember_window(app.handle().clone(), window);
    }
    // The system's media keys and media display (SPEC §13).
    media::start(app.handle());

    // Files given on the command line, as "Open with Onsa" does.
    let opened = open::from_arguments(std::env::args().collect::<Vec<_>>());
    if !opened.is_empty() {
        open::accept(app.handle(), opened);
    }
    Ok(())
}

/// Follows the window: its size and place are written down when it loses
/// focus or closes, files dropped on it are taken, closing can leave Onsa in
/// the tray (SPEC §13), and a window nobody can see stops the meters and the
/// visualizer from being analysed at all (SPEC §4.4).
fn remember_window(app: AppHandle, window: WebviewWindow) {
    let target = window.clone();
    window.on_window_event(move |event| match event {
        WindowEvent::DragDrop(tauri::DragDropEvent::Drop { paths, .. }) => {
            open::accept(&app, paths.clone());
        }
        WindowEvent::CloseRequested { api, .. } => {
            session::save(
                &app,
                session::WINDOW_KEY,
                &session::window_state(&target, &target.app_handle().state::<WindowMode>()),
            );
            if close_to_tray(&app) && tray::exists(&app) {
                api.prevent_close();
                let _ = target.hide();
                window_visible(&app, false);
            }
        }
        WindowEvent::Focused(false) => {
            session::save(
                &app,
                session::WINDOW_KEY,
                &session::window_state(&target, &target.app_handle().state::<WindowMode>()),
            );
        }
        // Minimising and restoring arrive as a resize; there is no event of
        // their own. Hiding to the tray is reported by the code that hides.
        WindowEvent::Resized(_) => {
            let shown = target.is_minimized().map(|small| !small).unwrap_or(true)
                && target.is_visible().unwrap_or(true);
            window_visible(&app, shown);
            // Every size the window passes through while it is on screen is
            // a size worth coming back to, so minimising it right before it
            // closes does not lose where the listener had it.
            if shown {
                let mode = app.state::<WindowMode>();
                mode.seen(session::geometry(&target));
            }
        }
        _ => {}
    });
}

/// Tells the player whether the window is on screen.
pub(crate) fn window_visible(app: &AppHandle, visible: bool) {
    if let Some(player) = app.try_state::<Player>() {
        player.set_window_visible(visible);
    }
}

/// Whether closing the window should only hide it.
fn close_to_tray(app: &AppHandle) -> bool {
    app.try_state::<LibraryService>()
        .map(|library| session::load::<bool>(&library, settings::CLOSE_TO_TRAY_KEY))
        .unwrap_or(false)
}
