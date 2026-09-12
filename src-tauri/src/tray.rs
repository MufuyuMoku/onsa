//! The tray icon (SPEC §13).
//!
//! All user text belongs to the interface dictionaries (SPEC §9.6), so the
//! interface hands the labels over once it knows the language, and again
//! whenever the language changes.

use serde::Deserialize;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};

use crate::player::Player;
use crate::session::MAIN_WINDOW;

/// Identifier of the one tray icon.
const TRAY_ID: &str = "onsa";

/// The words the tray menu shows.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrayLabels {
    /// Bring the window back.
    pub show: String,
    /// Play or pause.
    pub play: String,
    /// Previous track.
    pub previous: String,
    /// Next track.
    pub next: String,
    /// Leave Onsa.
    pub quit: String,
}

/// Builds the tray icon, replacing the one already there.
pub fn build(app: &AppHandle, labels: &TrayLabels) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", &labels.show, true, None::<&str>)?;
    let play = MenuItem::with_id(app, "play", &labels.play, true, None::<&str>)?;
    let previous = MenuItem::with_id(app, "previous", &labels.previous, true, None::<&str>)?;
    let next = MenuItem::with_id(app, "next", &labels.next, true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", &labels.quit, true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &show,
            &PredefinedMenuItem::separator(app)?,
            &play,
            &previous,
            &next,
            &PredefinedMenuItem::separator(app)?,
            &quit,
        ],
    )?;

    if app.remove_tray_by_id(TRAY_ID).is_some() {
        tracing::debug!("tray menu rebuilt");
    }
    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| tauri::Error::WebviewNotFound)?;
    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .tooltip("Onsa")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| on_menu(app, event.id().as_ref()))
        .on_tray_icon_event(|tray, event| {
            // A left click brings the window back, as every tray app does.
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_window(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}

fn on_menu(app: &AppHandle, id: &str) {
    let player = app.try_state::<Player>();
    let result = match (id, player) {
        ("show", _) => {
            show_window(app);
            return;
        }
        ("quit", _) => {
            tracing::info!("leaving from the tray");
            app.exit(0);
            return;
        }
        ("play", Some(player)) => player.toggle(),
        ("previous", Some(player)) => player.previous(),
        ("next", Some(player)) => player.next(),
        _ => return,
    };
    if let Err(code) = result {
        tracing::debug!("tray command ignored: {code:?}");
    }
}

/// Whether a tray icon is there to come back from.
pub fn exists(app: &AppHandle) -> bool {
    app.tray_by_id(TRAY_ID).is_some()
}

/// Shows the window and puts it in front.
pub fn show_window(app: &AppHandle) {
    let Some(window) = app.get_webview_window(MAIN_WINDOW) else {
        return;
    };
    let _ = window.show();
    let _ = window.unminimize();
    let _ = window.set_focus();
}
