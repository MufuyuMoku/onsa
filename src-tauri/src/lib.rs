//! Onsa application shell.
//!
//! This crate is the glue: it owns application state, exposes commands to the
//! user interface, forwards engine and library events, and holds the
//! operating system integration. The feature crates meet only here (SPEC §2).

#![warn(missing_docs)]

mod commands;
mod dto;
mod error;
mod library;
mod logging;
mod player;
mod settings;
pub mod theme;

use anyhow::{Context, Result};
use tauri::Manager;

use crate::library::LibraryService;
use crate::logging::Logging;
use crate::player::Player;

/// Starts the application: sets up logging, the library and the player,
/// builds the window and runs until the user closes it.
pub fn run() -> Result<()> {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .register_uri_scheme_protocol("onsa", library::cover_protocol)
        .setup(|app| setup(app).map_err(Into::into))
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::app_state,
            commands::theme_list,
            commands::theme_get,
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
            commands::player_play,
            commands::player_toggle,
            commands::player_next,
            commands::player_previous,
            commands::player_seek,
            commands::player_jump,
            commands::player_snapshot,
            commands::player_queue,
            commands::settings_get,
            commands::settings_set_output,
            commands::settings_set_playback,
            commands::settings_set_dsp,
            commands::output_devices,
            commands::eq_curve,
            commands::autoeq_import,
            commands::autoeq_export,
            commands::log_open_folder,
            commands::log_set_debug,
        ])
        .run(tauri::generate_context!())
        .context("the application window could not be started")
}

/// Everything that needs the application's folders: logging, the library
/// with the stored settings, then the player.
fn setup(app: &mut tauri::App) -> Result<()> {
    let paths = app.path();
    let log_dir = paths.app_log_dir().context("no log folder")?;
    let data_dir = paths.app_data_dir().context("no data folder")?;
    let cache_dir = paths.app_cache_dir().context("no cache folder")?;

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
    app.manage(logging);
    app.manage(library);
    app.manage(player);
    Ok(())
}
