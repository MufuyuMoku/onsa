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
const BUILTIN_SOURCES: [(&str, &str); 6] = [
    ("kaca-asap", include_str!("../../../themes/kaca-asap.json")),
    (
        "kokpit-kaca",
        include_str!("../../../themes/kokpit-kaca.json"),
    ),
    (
        "deck-malam",
        include_str!("../../../themes/deck-malam.json"),
    ),
    (
        "kubikel-biru",
        include_str!("../../../themes/kubikel-biru.json"),
    ),
    (
        "kilau-milenium",
        include_str!("../../../themes/kilau-milenium.json"),
    ),
    (
        "musim-dingin",
        include_str!("../../../themes/musim-dingin.json"),
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

/// A theme file that could not be used, and why.
///
/// The log has said this since M4, which helps whoever reads logs. The
/// window needs it too: somebody who writes a theme and mistypes one
/// bracket should not have to guess why their theme never appears.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Trouble {
    /// The file, by name.
    pub file: String,
    /// What was wrong with it, in the parser's own words.
    pub said: String,
}

/// The user's themes, and the files that could not be read.
pub fn read_user_dir(dir: &std::path::Path) -> (Vec<Theme>, Vec<Trouble>) {
    let mut troubles = Vec::new();
    let themes = user_themes_reporting(dir, &mut troubles);
    (themes, troubles)
}

/// Reads the user's own themes from `<app config>/themes/*.json`. A theme
/// that cannot be read is reported and left out; the rest still load.
pub fn user_themes(dir: &std::path::Path) -> Vec<Theme> {
    user_themes_reporting(dir, &mut Vec::new())
}

fn user_themes_reporting(dir: &std::path::Path, troubles: &mut Vec<Trouble>) -> Vec<Theme> {
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
            troubles.push(Trouble {
                file: file_name(&path),
                said: "a built-in theme already has that name".to_string(),
            });
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
                    troubles.push(Trouble {
                        file: file_name(&path),
                        said: format!("{error:#}"),
                    });
                }
            },
            Err(error) => {
                tracing::warn!(file = %path.display(), "theme cannot be read: {error}");
                troubles.push(Trouble {
                    file: file_name(&path),
                    said: error.to_string(),
                });
            }
        }
    }
    themes.sort_by(|a, b| a.id.cmp(&b.id));
    themes
}

/// A path as its file name alone: the window shows the file, not where
/// the listener's folders happen to live.
fn file_name(path: &std::path::Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}

/// A file name a theme copy can be given, from whatever was typed.
///
/// The identifier of a user theme is its file's name, so it has to be
/// something a file system will take: letters, digits and hyphens, and
/// never empty.
pub fn safe_stem(name: &str) -> String {
    let stem: String = name
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    let stem = stem.trim_matches('-').to_string();
    let stem: String = stem
        .split('-')
        .filter(|piece| !piece.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if stem.is_empty() {
        "tema-saya".to_string()
    } else {
        stem.chars().take(60).collect()
    }
}

/// The text of one built-in theme, for copying as a starting point.
pub fn builtin_source(id: &str) -> Option<&'static str> {
    BUILTIN_SOURCES
        .iter()
        .find(|(name, _)| *name == id)
        .map(|(_, source)| *source)
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
    use crate::theme::model::Base;

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
    fn the_themes_of_the_specification_are_present() {
        let themes = builtin_themes();
        let ids: Vec<&str> = themes.iter().map(|theme| theme.id.as_str()).collect();
        assert_eq!(
            ids,
            [
                "kaca-asap",
                "kokpit-kaca",
                "deck-malam",
                "kubikel-biru",
                "kilau-milenium",
                "musim-dingin"
            ],
            "SPEC §1: three dark, then three light"
        );
        assert!(ids.contains(&DEFAULT_THEME_ID));
        // The three dark ones come first, and Onsa opens on one of them.
        let dark: Vec<&str> = themes
            .iter()
            .filter(|theme| theme.base == Base::Dark)
            .map(|theme| theme.id.as_str())
            .collect();
        assert_eq!(dark, ["kaca-asap", "kokpit-kaca", "deck-malam"]);
    }

    #[test]
    fn a_light_theme_is_light_the_whole_way_through() {
        // A theme that says it is light and then paints a dark panel would
        // leave the window's own scrollbars and form controls fighting it.
        for theme in builtin_themes()
            .iter()
            .filter(|one| one.base == Base::Light)
        {
            assert!(
                lightness(&theme.color.surface.body) > 0.6,
                "theme {} says light but its panel is dark",
                theme.id
            );
            assert!(
                lightness(&theme.color.text.primary) < 0.4,
                "theme {} would write light text on a light panel",
                theme.id
            );
        }
    }

    /// What the tone colour has to manage before a theme may claim it.
    ///
    /// Written as numbers because "the colours are different" was not
    /// enough: three light themes passed that and still looked like nothing
    /// was happening. A colour one shade darker than the one it replaces is
    /// a different number and the same colour to look at.
    ///
    /// The distances are CIEDE2000, which is built to follow what the eye
    /// actually notices, and the hue angle is there because lightness alone
    /// is exactly what fooled the earlier check.
    mod tone {
        /// How far apart the two ends have to be.
        pub const ENDS_APART: f64 = 20.0;
        /// How far each end has to be from the colour it leans.
        pub const FROM_BASE: f64 = 10.0;
        /// And in which direction: a different colour, not a darker one.
        pub const HUE_APART: f64 = 20.0;
        /// Both ends stay legible against the well they are drawn on
        /// (WCAG 2.1 for a graphical object).
        pub const ON_WELL: f64 = 3.0;
    }

    /// A colour as CIE L*a*b*, which is where colour distance is measured.
    fn lab(hex: &str) -> (f64, f64, f64) {
        let channel = |part: &str| u8::from_str_radix(part, 16).unwrap_or(0) as f64 / 255.0;
        let hex = hex.trim_start_matches('#');
        let hex = if hex.len() == 3 {
            hex.chars().flat_map(|c| [c, c]).collect::<String>()
        } else {
            hex.to_string()
        };
        if hex.len() != 6 {
            return (0.0, 0.0, 0.0);
        }
        // sRGB, undone, to linear light.
        let linear = |c: f64| {
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        let r = linear(channel(&hex[0..2]));
        let g = linear(channel(&hex[2..4]));
        let b = linear(channel(&hex[4..6]));
        // Linear sRGB to XYZ, D65.
        let x = r * 0.412_456_4 + g * 0.357_576_1 + b * 0.180_437_5;
        let y = r * 0.212_672_9 + g * 0.715_152_2 + b * 0.072_175_0;
        let z = r * 0.019_333_9 + g * 0.119_192_0 + b * 0.950_304_1;
        let f = |t: f64| {
            if t > 216.0 / 24389.0 {
                t.cbrt()
            } else {
                (841.0 / 108.0) * t + 4.0 / 29.0
            }
        };
        let (fx, fy, fz) = (f(x / 0.950_47), f(y), f(z / 1.088_83));
        (116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz))
    }

    /// CIEDE2000, the distance the eye agrees with best.
    fn delta_e(one: (f64, f64, f64), two: (f64, f64, f64)) -> f64 {
        let (l1, a1, b1) = one;
        let (l2, a2, b2) = two;
        let c1 = a1.hypot(b1);
        let c2 = a2.hypot(b2);
        let cbar = (c1 + c2) / 2.0;
        let g = 0.5 * (1.0 - (cbar.powi(7) / (cbar.powi(7) + 25f64.powi(7))).sqrt());
        let (a1p, a2p) = ((1.0 + g) * a1, (1.0 + g) * a2);
        let (c1p, c2p) = (a1p.hypot(b1), a2p.hypot(b2));
        let angle = |a: f64, b: f64| {
            if a == 0.0 && b == 0.0 {
                0.0
            } else {
                b.atan2(a).to_degrees().rem_euclid(360.0)
            }
        };
        let (h1p, h2p) = (angle(a1p, b1), angle(a2p, b2));

        let dlp = l2 - l1;
        let dcp = c2p - c1p;
        let dhp = if c1p * c2p == 0.0 {
            0.0
        } else {
            let diff = h2p - h1p;
            if diff > 180.0 {
                diff - 360.0
            } else if diff < -180.0 {
                diff + 360.0
            } else {
                diff
            }
        };
        let big_dhp = 2.0 * (c1p * c2p).sqrt() * (dhp.to_radians() / 2.0).sin();

        let lbar = (l1 + l2) / 2.0;
        let cbarp = (c1p + c2p) / 2.0;
        let hbarp = if c1p * c2p == 0.0 {
            h1p + h2p
        } else {
            let diff = (h1p - h2p).abs();
            let total = h1p + h2p;
            if diff <= 180.0 {
                total / 2.0
            } else if total < 360.0 {
                (total + 360.0) / 2.0
            } else {
                (total - 360.0) / 2.0
            }
        };

        let t = 1.0 - 0.17 * (hbarp - 30.0).to_radians().cos()
            + 0.24 * (2.0 * hbarp).to_radians().cos()
            + 0.32 * (3.0 * hbarp + 6.0).to_radians().cos()
            - 0.20 * (4.0 * hbarp - 63.0).to_radians().cos();
        let dtheta = 30.0 * (-(((hbarp - 275.0) / 25.0).powi(2))).exp();
        let rc = 2.0 * (cbarp.powi(7) / (cbarp.powi(7) + 25f64.powi(7))).sqrt();
        let sl = 1.0 + (0.015 * (lbar - 50.0).powi(2)) / (20.0 + (lbar - 50.0).powi(2)).sqrt();
        let sc = 1.0 + 0.045 * cbarp;
        let sh = 1.0 + 0.015 * cbarp * t;
        let rt = -(2.0 * dtheta).to_radians().sin() * rc;

        ((dlp / sl).powi(2)
            + (dcp / sc).powi(2)
            + (big_dhp / sh).powi(2)
            + rt * (dcp / sc) * (big_dhp / sh))
            .sqrt()
    }

    /// The distance between two `#rrggbb` colours.
    fn distance(one: &str, two: &str) -> f64 {
        delta_e(lab(one), lab(two))
    }

    /// How far apart two colours are in hue alone, in degrees.
    fn hue_apart(one: &str, two: &str) -> f64 {
        let hue = |c: &str| {
            let (_, a, b) = lab(c);
            b.atan2(a).to_degrees().rem_euclid(360.0)
        };
        let diff = (hue(one) - hue(two)).abs().rem_euclid(360.0);
        diff.min(360.0 - diff)
    }

    /// WCAG contrast between two colours, as a ratio.
    fn contrast(one: &str, two: &str) -> f64 {
        // Back from L* to relative luminance, which is what WCAG asks for
        // and what L* was worked out from in the first place.
        let light = |hex: &str| {
            let (l, _, _) = lab(hex);
            if l > 8.0 {
                ((l + 16.0) / 116.0).powi(3)
            } else {
                l / 903.3
            }
        };
        let (a, b) = (light(one), light(two));
        let (hi, lo) = if a > b { (a, b) } else { (b, a) };
        (hi + 0.05) / (lo + 0.05)
    }

    #[test]
    fn the_colour_distance_agrees_with_the_published_figures() {
        // Sharma, Wu and Dalal's test data for CIEDE2000: if these come out
        // right, the arithmetic above is the arithmetic everybody means.
        let close = |got: f64, want: f64| {
            assert!(
                (got - want).abs() < 0.001,
                "expected {want}, worked out {got}"
            );
        };
        close(
            delta_e((50.0, 2.6772, -79.7751), (50.0, 0.0, -82.7485)),
            2.0425,
        );
        close(delta_e((50.0, 2.5, 0.0), (50.0, 0.0, -2.5)), 4.3065);
        close(
            delta_e((63.0109, -31.0961, -5.8663), (62.8187, -29.7946, -4.0864)),
            1.2630,
        );
        close(
            delta_e((60.2574, -34.0099, 36.2677), (60.4626, -34.1751, 39.4387)),
            1.2644,
        );
    }

    /// A theme that says it leans has to lean somewhere the eye can see.
    ///
    /// Three of the six said they used tone colour while naming a cool end
    /// that was the very colour being leaned; the check that caught that
    /// only asked whether the colours were different at all, and one of the
    /// replacements passed it while still being invisible — the same blue,
    /// a shade darker. What the eye reads is hue, so that is measured.
    #[test]
    fn a_theme_that_leans_leans_somewhere_you_can_see() {
        use crate::theme::model::ToneTarget;
        for theme in builtin_themes() {
            let tone = &theme.tone_color;
            if !tone.enabled {
                continue;
            }
            let id = &theme.id;
            assert!(
                !tone.targets.is_empty(),
                "theme {id} says it uses tone colour and then moves nothing"
            );
            assert!(
                !tone.targets.contains(&ToneTarget::Meter),
                "theme {id}: a meter never follows the tone — its green, yellow \
                 and red mean how close the level is to the limit"
            );
            let ends = distance(&tone.warm, &tone.cool);
            assert!(
                ends >= tone::ENDS_APART,
                "theme {id}: its two ends are only {ends:.1} apart, and want {:.0}",
                tone::ENDS_APART
            );
            for target in &tone.targets {
                let base = match target {
                    ToneTarget::Spectrum => &theme.color.lit.on,
                    ToneTarget::Progress => &theme.color.role.position,
                    ToneTarget::CoverGlow => &theme.color.lit.glow,
                    ToneTarget::Meter => continue,
                };
                for (which, end) in [("warm", &tone.warm), ("cool", &tone.cool)] {
                    let moved = distance(base, end);
                    assert!(
                        moved >= tone::FROM_BASE,
                        "theme {id}: {target:?} moves only {moved:.1} towards its {which} end"
                    );
                    let turned = hue_apart(base, end);
                    assert!(
                        turned >= tone::HUE_APART,
                        "theme {id}: {target:?} and its {which} end are {turned:.0}° apart in hue, \
                         which is the same colour a little darker rather than another colour"
                    );
                }
            }
            for (which, end) in [("warm", &tone.warm), ("cool", &tone.cool)] {
                let seen = contrast(end, &theme.color.surface.well);
                assert!(
                    seen >= tone::ON_WELL,
                    "theme {id}: its {which} end sits at {seen:.1}:1 against its own well"
                );
            }
        }
    }

    /// Roughly how light a `#rrggbb` colour is, from 0 to 1.
    fn lightness(hex: &str) -> f32 {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return 0.5;
        }
        let part =
            |at: usize| u8::from_str_radix(&hex[at..at + 2], 16).unwrap_or(128) as f32 / 255.0;
        // The eye is far more sensitive to green than to blue.
        0.2126 * part(0) + 0.7152 * part(2) + 0.0722 * part(4)
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
