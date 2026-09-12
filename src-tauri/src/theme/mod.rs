//! Theme loading (SPEC §9.3).
//!
//! Built-in themes are compiled into the binary from `themes/*.json` so the
//! window always has something to draw with. The user's own themes live in
//! `<app config>/themes/*.json`. Both go through the same tolerant parser,
//! and a damaged theme never stops the window from opening.

mod model;
mod parse;

pub use model::Theme;
pub use parse::{parse_theme, LoadedTheme};

/// The theme a fresh installation starts with, until the first-run screen
/// lets the user choose (SPEC §9.2).
pub const DEFAULT_THEME_ID: &str = "kaca-asap";

/// The three built-in themes, in the order the first-run screen shows them.
const BUILTIN_SOURCES: [(&str, &str); 3] = [
    ("kaca-asap", include_str!("../../../themes/kaca-asap.json")),
    (
        "kokpit-kaca",
        include_str!("../../../themes/kokpit-kaca.json"),
    ),
    (
        "deck-malam",
        include_str!("../../../themes/deck-malam.json"),
    ),
];

/// Reads every built-in theme.
///
/// Built-in themes are part of the binary, so a warning here is a bug in the
/// theme file rather than something the user can fix; it is logged and the
/// affected field falls back to its default.
pub fn builtin_themes() -> Vec<Theme> {
    BUILTIN_SOURCES
        .iter()
        .filter_map(|(id, source)| match parse_theme(id, source) {
            Ok(loaded) => {
                for warning in &loaded.warnings {
                    tracing::warn!(theme = id, "built-in theme: {warning}");
                }
                Some(loaded.theme)
            }
            Err(error) => {
                tracing::error!(theme = id, "built-in theme cannot be read: {error:#}");
                None
            }
        })
        .collect()
}

/// The folder the user's own themes live in.
pub fn user_dir(config_dir: &std::path::Path) -> std::path::PathBuf {
    config_dir.join("themes")
}

/// Reads the user's own themes from `<app config>/themes/*.json`. A theme
/// that cannot be read is reported and left out; the rest still load.
pub fn user_themes(dir: &std::path::Path) -> Vec<Theme> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut themes = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let Some(id) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        // A user theme never shadows a built-in one.
        if BUILTIN_SOURCES.iter().any(|(name, _)| *name == id) {
            tracing::warn!(theme = id, "user theme uses a built-in name, skipped");
            continue;
        }
        match std::fs::read_to_string(&path) {
            Ok(source) => match parse_theme(id, &source) {
                Ok(loaded) => {
                    for warning in &loaded.warnings {
                        tracing::info!(theme = id, "user theme: {warning}");
                    }
                    themes.push(loaded.theme);
                }
                Err(error) => {
                    tracing::warn!(theme = id, "user theme cannot be read: {error:#}");
                }
            },
            Err(error) => tracing::warn!(file = %path.display(), "theme cannot be read: {error}"),
        }
    }
    themes.sort_by(|a, b| a.id.cmp(&b.id));
    themes
}

/// Reads one built-in theme by identifier.
pub fn builtin_theme(id: &str) -> Option<Theme> {
    let (id, source) = BUILTIN_SOURCES.iter().find(|(name, _)| *name == id)?;
    match parse_theme(id, source) {
        Ok(loaded) => Some(loaded.theme),
        Err(error) => {
            tracing::error!(theme = id, "built-in theme cannot be read: {error:#}");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_builtin_theme_reads_without_a_single_correction() {
        for (id, source) in BUILTIN_SOURCES {
            let loaded = parse_theme(id, source).unwrap_or_else(|error| {
                panic!("built-in theme {id} is not readable: {error:#}");
            });
            assert!(
                loaded.warnings.is_empty(),
                "built-in theme {id} needed corrections: {:?}",
                loaded.warnings
            );
            assert_eq!(loaded.theme.id, id, "theme {id} names itself differently");
            assert!(loaded.theme.builtin, "theme {id} is not marked built-in");
        }
    }

    #[test]
    fn the_three_themes_of_the_specification_are_present() {
        let themes = builtin_themes();
        let ids: Vec<&str> = themes.iter().map(|theme| theme.id.as_str()).collect();
        assert_eq!(ids, ["kaca-asap", "kokpit-kaca", "deck-malam"]);
        assert!(ids.contains(&DEFAULT_THEME_ID));
    }

    #[test]
    fn every_theme_fills_every_colour_role() {
        // Components address roles, never a raw colour, so a role left empty
        // would leave a control invisible.
        for theme in builtin_themes() {
            let role = &theme.color.role;
            for (name, value) in [
                ("label", &role.label),
                ("adjustable", &role.adjustable),
                ("active", &role.active),
                ("position", &role.position),
                ("caution", &role.caution),
                ("clip", &role.clip),
            ] {
                assert!(
                    !value.trim().is_empty(),
                    "theme {} leaves role {name} empty",
                    theme.id
                );
            }
        }
    }

    #[test]
    fn an_unknown_identifier_has_no_theme() {
        assert!(builtin_theme("kaca-asap").is_some());
        assert!(builtin_theme("tema-yang-tidak-ada").is_none());
    }
}
