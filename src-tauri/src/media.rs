//! System media controls (SPEC §13): SMTC on Windows, MPRIS on Linux.
//!
//! The controls belong to the window's own thread on Windows, so they live
//! in a thread local on the main thread and every update is posted there.
//! Button presses arrive on the system's thread and are handed to the
//! player like any other command.

use std::cell::RefCell;
use std::time::Duration;

use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition};
use tauri::{AppHandle, Manager};

use crate::player::{Player, Snapshot};
use crate::session::MAIN_WINDOW;

/// The name MPRIS lists Onsa under.
const BUS_NAME: &str = "onsa";
/// How the player shows up in the system's media display.
const DISPLAY_NAME: &str = "Onsa";
/// Seek step for the system's "seek forward/back" buttons.
const SEEK_STEP: f64 = 5.0;

thread_local! {
    /// Only ever touched on the main thread.
    static CONTROLS: RefCell<Option<MediaControls>> = const { RefCell::new(None) };
}

/// Starts the system media controls. Called on the main thread; a failure
/// only means the system keys stay quiet, never that the player stops.
pub fn start(app: &AppHandle) {
    let hwnd = window_handle(app);
    let config = souvlaki::PlatformConfig {
        dbus_name: BUS_NAME,
        display_name: DISPLAY_NAME,
        hwnd,
    };
    let mut controls = match MediaControls::new(config) {
        Ok(controls) => controls,
        Err(error) => {
            tracing::warn!("system media controls unavailable: {error:?}");
            return;
        }
    };
    let handle = app.clone();
    if let Err(error) = controls.attach(move |event| on_button(&handle, event)) {
        tracing::warn!("system media controls cannot be attached: {error:?}");
        return;
    }
    tracing::info!("system media controls ready");
    CONTROLS.with(|slot| *slot.borrow_mut() = Some(controls));
}

/// The window handle Windows needs to hang the controls on.
#[cfg(windows)]
fn window_handle(app: &AppHandle) -> Option<*mut std::ffi::c_void> {
    let window = app.get_webview_window(MAIN_WINDOW)?;
    window.hwnd().ok().map(|handle| handle.0)
}

#[cfg(not(windows))]
fn window_handle(_app: &AppHandle) -> Option<*mut std::ffi::c_void> {
    None
}

/// Applies a button press from the system.
fn on_button(app: &AppHandle, event: MediaControlEvent) {
    let Some(player) = app.try_state::<Player>() else {
        return;
    };
    let position = player.snapshot().position;
    let result = match event {
        MediaControlEvent::Play => player.resume(),
        MediaControlEvent::Pause => player.pause(),
        MediaControlEvent::Toggle => player.toggle(),
        MediaControlEvent::Next => player.next(),
        MediaControlEvent::Previous => player.previous(),
        MediaControlEvent::Stop | MediaControlEvent::Quit => player.stop(),
        MediaControlEvent::Seek(direction) => player.seek(stepped(position, direction, SEEK_STEP)),
        MediaControlEvent::SeekBy(direction, by) => {
            player.seek(stepped(position, direction, by.as_secs_f64()))
        }
        MediaControlEvent::SetPosition(MediaPosition(at)) => player.seek(at.as_secs_f64()),
        MediaControlEvent::Raise => {
            if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
                let _ = window.show();
                let _ = window.set_focus();
            }
            Ok(())
        }
        MediaControlEvent::OpenUri(_) | MediaControlEvent::SetVolume(_) => Ok(()),
    };
    if let Err(code) = result {
        tracing::debug!("media key ignored: {code:?}");
    }
}

fn stepped(position: f64, direction: souvlaki::SeekDirection, by: f64) -> f64 {
    match direction {
        souvlaki::SeekDirection::Forward => position + by,
        souvlaki::SeekDirection::Backward => (position - by).max(0.0),
    }
}

/// Tells the system what is playing. Safe to call from any thread.
pub fn update(app: &AppHandle, snapshot: &Snapshot) {
    let title = snapshot.track.as_ref().map(|track| {
        track
            .title
            .clone()
            .unwrap_or_else(|| file_name(&track.path))
    });
    let artist = snapshot
        .track
        .as_ref()
        .and_then(|track| track.artist.clone());
    let album = snapshot
        .track
        .as_ref()
        .and_then(|track| track.album.clone());
    let cover = snapshot
        .track
        .as_ref()
        .and_then(|track| track.cover_id)
        .and_then(|id| cover_url(app, id));
    let duration = snapshot.duration.map(Duration::from_secs_f64);
    let position = Duration::from_secs_f64(snapshot.position.max(0.0));
    let state = snapshot.state;

    let posted = app.run_on_main_thread(move || {
        CONTROLS.with(|slot| {
            let mut slot = slot.borrow_mut();
            let Some(controls) = slot.as_mut() else {
                return;
            };
            let progress = Some(MediaPosition(position));
            let playback = match state {
                "playing" => MediaPlayback::Playing { progress },
                "paused" => MediaPlayback::Paused { progress },
                _ => MediaPlayback::Stopped,
            };
            if let Err(error) = controls.set_metadata(MediaMetadata {
                title: title.as_deref(),
                album: album.as_deref(),
                artist: artist.as_deref(),
                cover_url: cover.as_deref(),
                duration,
            }) {
                tracing::debug!("media metadata not set: {error:?}");
            }
            if let Err(error) = controls.set_playback(playback) {
                tracing::debug!("media playback state not set: {error:?}");
            }
        });
    });
    if let Err(error) = posted {
        tracing::debug!("media update not posted: {error}");
    }
}

/// A `file://` URL for a cover thumbnail, which both SMTC and MPRIS read.
fn cover_url(app: &AppHandle, cover_id: i64) -> Option<String> {
    let library = app.try_state::<crate::library::LibraryService>()?;
    let file = library
        .read(|library| library.cover_file(cover_id, 512))
        .ok()??;
    let path = file.to_str()?.replace('\\', "/");
    Some(format!("file:///{}", path.trim_start_matches('/')))
}

fn file_name(path: &str) -> String {
    path.rsplit(['/', '\\']).next().unwrap_or(path).to_string()
}
