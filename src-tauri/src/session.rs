//! What the application picks up again when it opens (SPEC §13).
//!
//! The queue, the track and position it was on, and the window's size and
//! place are kept in the library database alongside the settings. Playback
//! comes back paused: opening Onsa never starts making noise on its own.

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, WebviewWindow};

use crate::library::LibraryService;
use crate::settings;

/// The queue and where it was.
pub const QUEUE_KEY: &str = "queue";
/// Seconds into the track that was playing. Kept apart from the queue so
/// the playhead can be stored often without rewriting the whole queue.
pub const POSITION_KEY: &str = "position";
/// The window's size, place and mode.
pub const WINDOW_KEY: &str = "window";

/// Label of the one window Onsa opens.
pub const MAIN_WINDOW: &str = "main";

/// Size of the mini player (SPEC §9.2): one line of instrument panel.
pub const MINI_SIZE: (f64, f64) = (560.0, 148.0);
/// Smallest useful full window, matching `tauri.conf.json`.
pub const FULL_MIN_SIZE: (f64, f64) = (880.0, 560.0);

/// The queue as it was left.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct QueueState {
    /// Track ids in queue order.
    pub ids: Vec<i64>,
    /// Which entry was playing.
    pub current: usize,
    /// Whether the queue was shuffled.
    pub shuffle: bool,
}

/// Whether the window is in mini player mode (SPEC §9.2).
#[derive(Debug, Default)]
pub struct WindowMode(std::sync::atomic::AtomicBool);

impl WindowMode {
    /// Whether the mini player is on.
    pub fn mini(&self) -> bool {
        self.0.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Records the mode the window is in.
    pub fn set_mini(&self, mini: bool) {
        self.0.store(mini, std::sync::atomic::Ordering::Relaxed);
    }
}

/// The window as it was left.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct WindowState {
    /// Width in physical pixels.
    pub width: f64,
    /// Height in physical pixels.
    pub height: f64,
    /// Left edge, when it was ever moved.
    pub x: Option<f64>,
    /// Top edge.
    pub y: Option<f64>,
    /// Whether the window was maximised.
    pub maximized: bool,
    /// Whether it was in mini player mode.
    pub mini: bool,
}

/// Reads a stored session value, or its default.
pub fn load<T: serde::de::DeserializeOwned + Default>(library: &LibraryService, key: &str) -> T {
    library
        .read(|library| Ok(settings::load::<T>(library, key)))
        .unwrap_or_default()
}

/// Stores a session value. A failure is logged, never fatal: losing the
/// place in a queue must not stop anything.
pub fn save<T: Serialize>(app: &AppHandle, key: &str, value: &T) {
    let Some(library) = app.try_state::<LibraryService>() else {
        return;
    };
    if let Err(error) = library.read(|library| settings::save(library, key, value)) {
        tracing::warn!(key, "session state not stored: {error:?}");
    }
}

/// Puts the window back where it was.
pub fn restore_window(window: &WebviewWindow, state: &WindowState) {
    if state.width >= 200.0 && state.height >= 100.0 {
        let _ = window.set_size(PhysicalSize::new(state.width, state.height));
    }
    if let (Some(x), Some(y)) = (state.x, state.y) {
        // Only inside a screen: an unplugged monitor must not hide the window.
        if on_a_screen(window, x, y) {
            let _ = window.set_position(PhysicalPosition::new(x, y));
        }
    }
    if state.maximized && !state.mini {
        let _ = window.maximize();
    }
    if state.mini {
        let _ = apply_mini(window, true);
    }
}

/// Whether a point falls on one of the screens the system reports.
fn on_a_screen(window: &WebviewWindow, x: f64, y: f64) -> bool {
    let Ok(monitors) = window.available_monitors() else {
        return false;
    };
    monitors.iter().any(|monitor| {
        let position = monitor.position();
        let size = monitor.size();
        let right = f64::from(position.x) + f64::from(size.width);
        let bottom = f64::from(position.y) + f64::from(size.height);
        x >= f64::from(position.x) - 32.0 && x < right && y >= f64::from(position.y) - 32.0 && y < bottom
    })
}

/// Switches the window between the full panel and the mini player.
pub fn apply_mini(window: &WebviewWindow, mini: bool) -> tauri::Result<()> {
    if mini {
        window.unmaximize()?;
        window.set_min_size(Some(PhysicalSize::new(MINI_SIZE.0, MINI_SIZE.1)))?;
        window.set_size(PhysicalSize::new(MINI_SIZE.0, MINI_SIZE.1))?;
        window.set_always_on_top(true)?;
    } else {
        window.set_always_on_top(false)?;
        window.set_min_size(Some(PhysicalSize::new(FULL_MIN_SIZE.0, FULL_MIN_SIZE.1)))?;
        window.set_size(PhysicalSize::new(1180.0, 760.0))?;
    }
    Ok(())
}

/// Reads the window's current geometry, keeping the stored mode.
pub fn window_state(window: &WebviewWindow, mini: bool) -> WindowState {
    let size = window.outer_size().unwrap_or(PhysicalSize::new(1180, 760));
    let position = window.outer_position().ok();
    WindowState {
        width: f64::from(size.width),
        height: f64::from(size.height),
        x: position.map(|at| f64::from(at.x)),
        y: position.map(|at| f64::from(at.y)),
        maximized: window.is_maximized().unwrap_or(false),
        mini,
    }
}
