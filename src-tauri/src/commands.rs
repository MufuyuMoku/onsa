//! Commands the user interface may call (SPEC §2).
//!
//! Errors cross this boundary as stable codes, never as sentences: all user
//! visible text comes from the interface dictionaries (SPEC §9.6).

use serde::Serialize;

use crate::theme::{self, Theme};

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

/// A failure the interface can translate.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case", tag = "code")]
pub enum ErrorCode {
    /// No theme with that identifier exists.
    ThemeNotFound,
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
}
