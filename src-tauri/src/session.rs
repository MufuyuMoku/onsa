//! What the application picks up again when it opens (SPEC §13).
//!
//! The queue, the track and position it was on, and the window's size and
//! place are kept in the library database alongside the settings. Playback
//! comes back paused: opening Onsa never starts making noise on its own.

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewWindow};

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
/// The window changed between the full panel and the mini player.
pub const MODE_EVENT: &str = "window://mode";

/// Size of the mini player (SPEC §9.2): one line of instrument panel.
pub const MINI_SIZE: (f64, f64) = (660.0, 146.0);
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

/// Where a window is and how big it is.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Geometry {
    /// Width in physical pixels.
    pub width: f64,
    /// Height in physical pixels.
    pub height: f64,
    /// Left edge, when the window has ever been placed.
    pub x: Option<f64>,
    /// Top edge.
    pub y: Option<f64>,
    /// Whether the window is maximised. Then the size is the maximised one,
    /// which the system works out again anyway.
    pub maximized: bool,
}

/// What a window should look like after a change of mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shape {
    /// One line of instrument panel, always on top.
    Mini,
    /// The full panel, back to what it was.
    Full(Geometry),
}

/// The window mode, and the window the mini player borrowed (SPEC §9.2).
///
/// This is the one place that knows whether Onsa is in mini player mode.
/// Every way in — the button, Ctrl+M, the session as it is restored — goes
/// through here, and the interface is told what came out rather than
/// deciding for itself.
#[derive(Debug, Default)]
pub struct WindowMode(std::sync::Mutex<Mode>);

#[derive(Debug, Default)]
struct Mode {
    mini: bool,
    /// The window as it was before the mini player took its place.
    full: Option<Geometry>,
    /// The last full window seen while it was not maximised: where it goes
    /// back to when the listener presses restore.
    plain: Option<Geometry>,
}

/// The full window when nothing better is known.
const DEFAULT_FULL: (f64, f64) = (1180.0, 760.0);

/// Whether a remembered window can serve as the full panel.
fn usable(full: &Geometry) -> bool {
    full.maximized || (full.width >= FULL_MIN_SIZE.0 && full.height >= FULL_MIN_SIZE.1)
}

impl WindowMode {
    fn lock(&self) -> std::sync::MutexGuard<'_, Mode> {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Whether the mini player is on.
    pub fn mini(&self) -> bool {
        self.lock().mini
    }

    /// What the window should become, or `None` when it is already there.
    /// `now` is the window as it stands, which is what the mini player
    /// gives back later.
    pub fn plan(&self, mini: bool, now: Geometry) -> Option<Shape> {
        let mode = self.lock();
        if mode.mini == mini {
            return None;
        }
        Some(if mini {
            Shape::Mini
        } else {
            // A remembered window too small to be the full panel is no use:
            // it can only come from a stored state written by another
            // version, or from a mode that once got out of step.
            Shape::Full(mode.full.filter(usable).unwrap_or(Geometry {
                width: DEFAULT_FULL.0,
                height: DEFAULT_FULL.1,
                ..now
            }))
        })
    }

    /// Records the mode the window is now in. Going into the mini player
    /// remembers the window it replaces.
    pub fn record(&self, mini: bool, now: Geometry) {
        let mut mode = self.lock();
        if mini && !mode.mini {
            mode.full = Some(now);
            if !now.maximized {
                mode.plain = Some(now);
            }
        }
        mode.mini = mini;
    }

    /// The geometry worth storing for the next time Onsa opens.
    ///
    /// While the mini player is on, that is the full window waiting behind
    /// it, not the strip on screen. A maximised window is stored at the size
    /// it had before it was maximised, so opening Onsa again and pressing
    /// restore gives back the window the listener had, not one the size of
    /// the screen.
    pub fn for_session(&self, now: Geometry) -> Geometry {
        let mut mode = self.lock();
        if !now.maximized && !mode.mini {
            mode.plain = Some(now);
        }
        match (mode.mini, mode.full) {
            (true, Some(full)) => full,
            _ if now.maximized => Geometry {
                maximized: true,
                ..mode.plain.unwrap_or(now)
            },
            _ => now,
        }
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

impl WindowState {
    /// The full window it describes.
    pub fn geometry(&self) -> Geometry {
        Geometry {
            width: self.width,
            height: self.height,
            x: self.x,
            y: self.y,
            maximized: self.maximized,
        }
    }

    /// A stored state from a geometry and a mode.
    pub fn new(geometry: Geometry, mini: bool) -> Self {
        Self {
            width: geometry.width,
            height: geometry.height,
            x: geometry.x,
            y: geometry.y,
            maximized: geometry.maximized,
            mini,
        }
    }
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

/// Puts the window back where it was, in the mode it was left in.
///
/// The full window is laid out first even when the mini player is coming
/// back, so that leaving the mini player later gives back the window the
/// listener actually had.
pub fn restore_window(window: &WebviewWindow, state: &WindowState, mode: &WindowMode) {
    let full = state.geometry();
    apply_full(window, full);
    mode.record(false, full);
    if state.mini {
        if let Some(shape) = mode.plan(true, full) {
            let _ = apply_shape(window, shape);
            mode.record(true, full);
        }
    }
}

/// Lays the full window out: its size, its place, and whether it is
/// maximised.
fn apply_full(window: &WebviewWindow, full: Geometry) {
    if full.width >= FULL_MIN_SIZE.0 && full.height >= FULL_MIN_SIZE.1 {
        let _ = window.set_size(PhysicalSize::new(full.width, full.height));
    }
    if let (Some(x), Some(y)) = (full.x, full.y) {
        // Only inside a screen: an unplugged monitor must not hide the window.
        if on_a_screen(window, x, y) {
            let _ = window.set_position(PhysicalPosition::new(x, y));
        }
    }
    // After the size, so the window has somewhere sensible to go back to.
    if full.maximized {
        let _ = window.maximize();
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
        x >= f64::from(position.x) - 32.0
            && x < right
            && y >= f64::from(position.y) - 32.0
            && y < bottom
    })
}

/// Gives the window the shape it was planned into.
pub fn apply_shape(window: &WebviewWindow, shape: Shape) -> tauri::Result<()> {
    match shape {
        Shape::Mini => {
            window.unmaximize()?;
            window.set_min_size(Some(PhysicalSize::new(MINI_SIZE.0, MINI_SIZE.1)))?;
            window.set_size(PhysicalSize::new(MINI_SIZE.0, MINI_SIZE.1))?;
            window.set_always_on_top(true)?;
        }
        Shape::Full(full) => {
            window.set_always_on_top(false)?;
            window.set_min_size(Some(PhysicalSize::new(FULL_MIN_SIZE.0, FULL_MIN_SIZE.1)))?;
            // The size is set even for a window that is about to be
            // maximised: it is what the system gives back when the listener
            // presses restore, and leaving the mini player's strip there
            // would hand them a sliver of a window.
            window.unmaximize()?;
            window.set_size(PhysicalSize::new(
                full.width.max(FULL_MIN_SIZE.0),
                full.height.max(FULL_MIN_SIZE.1),
            ))?;
            if let (Some(x), Some(y)) = (full.x, full.y) {
                if on_a_screen(window, x, y) {
                    window.set_position(PhysicalPosition::new(x, y))?;
                }
            }
            if full.maximized {
                window.maximize()?;
            }
        }
    }
    Ok(())
}

/// Reads where the window is and how big it is.
///
/// The size is the inner one on purpose: on Windows `set_size` lands on the
/// inner size while `outer_size` reports the frame around it, so reading one
/// and writing the other makes the window grow by the width of its own
/// border every time it comes back from the mini player.
pub fn geometry(window: &WebviewWindow) -> Geometry {
    let size = window.inner_size().unwrap_or(PhysicalSize::new(1180, 760));
    let position = window.outer_position().ok();
    Geometry {
        width: f64::from(size.width),
        height: f64::from(size.height),
        x: position.map(|at| f64::from(at.x)),
        y: position.map(|at| f64::from(at.y)),
        maximized: window.is_maximized().unwrap_or(false),
    }
}

/// The window as it should be stored: the full window, and the mode.
pub fn window_state(window: &WebviewWindow, mode: &WindowMode) -> WindowState {
    WindowState::new(mode.for_session(geometry(window)), mode.mini())
}

/// Switches the window between the full panel and the mini player, from
/// wherever the change came from (SPEC §9.2). Nothing else changes the mode.
pub fn set_mini(app: &AppHandle, mini: bool) -> tauri::Result<bool> {
    let Some(window) = app.get_webview_window(MAIN_WINDOW) else {
        return Ok(false);
    };
    let mode = app.state::<WindowMode>();
    let mut now = geometry(&window);
    let Some(shape) = mode.plan(mini, now) else {
        return Ok(mode.mini());
    };
    if mini && now.maximized {
        // A maximised window is the size of the screen, which says nothing
        // about where it sits when it is not. Ask the system first, so that
        // is the window the mini player gives back.
        window.unmaximize()?;
        now = Geometry {
            maximized: true,
            ..geometry(&window)
        };
    }
    apply_shape(&window, shape)?;
    mode.record(mini, now);
    save(app, WINDOW_KEY, &window_state(&window, &mode));
    // The interface follows the window, never the other way round.
    if let Err(error) = app.emit(MODE_EVENT, mini) {
        tracing::debug!("the window mode was not delivered: {error}");
    }
    Ok(mini)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full() -> Geometry {
        Geometry {
            width: 1400.0,
            height: 900.0,
            x: Some(120.0),
            y: Some(64.0),
            maximized: false,
        }
    }

    fn mini_now() -> Geometry {
        Geometry {
            width: MINI_SIZE.0,
            height: MINI_SIZE.1,
            x: Some(300.0),
            y: Some(300.0),
            maximized: false,
        }
    }

    #[test]
    fn the_mini_player_gives_back_the_window_it_took() {
        let mode = WindowMode::default();
        assert!(!mode.mini());

        assert_eq!(mode.plan(true, full()), Some(Shape::Mini));
        mode.record(true, full());
        assert!(mode.mini());

        // Asking for the mode it is already in changes nothing.
        assert_eq!(mode.plan(true, mini_now()), None);

        assert_eq!(mode.plan(false, mini_now()), Some(Shape::Full(full())));
        mode.record(false, mini_now());
        assert!(!mode.mini());
    }

    #[test]
    fn a_maximised_window_comes_back_maximised() {
        let maximised = Geometry {
            maximized: true,
            ..full()
        };
        let mode = WindowMode::default();
        mode.record(true, maximised);
        assert_eq!(mode.plan(false, mini_now()), Some(Shape::Full(maximised)));
    }

    #[test]
    fn the_session_keeps_the_full_window_not_the_strip() {
        let mode = WindowMode::default();
        mode.record(true, full());
        // While the mini player is on, what is on screen is the strip; the
        // window worth coming back to is the one behind it.
        assert_eq!(mode.for_session(mini_now()), full());
        assert!(WindowState::new(mode.for_session(mini_now()), mode.mini()).mini);

        mode.record(false, mini_now());
        assert_eq!(mode.for_session(full()), full());
    }

    #[test]
    fn a_maximised_window_is_stored_at_the_size_it_had_before() {
        // Otherwise pressing restore after opening Onsa again would give a
        // window the size of the screen.
        let mode = WindowMode::default();
        let plain = full();
        assert_eq!(mode.for_session(plain), plain);

        let screen = Geometry {
            width: 2560.0,
            height: 1440.0,
            x: Some(0.0),
            y: Some(0.0),
            maximized: true,
        };
        assert_eq!(
            mode.for_session(screen),
            Geometry {
                maximized: true,
                ..plain
            }
        );
    }

    #[test]
    fn with_nothing_remembered_the_full_window_gets_a_sensible_size() {
        let mode = WindowMode::default();
        mode.record(true, Geometry::default());
        let Some(Shape::Full(back)) = mode.plan(false, mini_now()) else {
            panic!("leaving the mini player should give a full window");
        };
        assert_eq!(back.width, DEFAULT_FULL.0);
        assert_eq!(back.height, DEFAULT_FULL.1);
        assert_eq!(back.x, mini_now().x, "it stays where the listener put it");
    }
}
