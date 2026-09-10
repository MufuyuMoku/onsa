//! Tolerant reading of a theme file.
//!
//! A theme must never be able to break the application (SPEC §9.3), so parsing
//! never rejects a field: anything missing, of the wrong type, or outside its
//! range is replaced by the default and reported as a warning. Only a file
//! that is not a JSON object at all is refused.

use anyhow::{anyhow, Context, Result};
use serde_json::Value;

use super::model::{
    Base, Colors, Density, Effects, Fonts, LabelCase, LitColors, LocalizedName, MeterVariant,
    PanelTexture, Radius, RoleColors, Shape, SpectrumVariant, StageIndicator, SurfaceColors,
    TextColors, Theme, TimeDisplay, ToneColor, ToneTarget, Variants,
};

/// A theme as read from a file, together with what had to be corrected.
#[derive(Debug, Clone)]
pub struct LoadedTheme {
    /// The theme, with defaults filled in for everything unusable.
    pub theme: Theme,
    /// One line per field that was missing, malformed or out of range.
    pub warnings: Vec<String>,
}

/// Reads a theme from JSON text.
///
/// `id_hint` is the file stem, used when the file does not name itself.
/// Fails only when the text is not a JSON object.
pub fn parse_theme(id_hint: &str, source: &str) -> Result<LoadedTheme> {
    let root: Value = serde_json::from_str(source).context("theme file is not valid JSON")?;
    if !root.is_object() {
        return Err(anyhow!("theme file is not a JSON object"));
    }

    let mut cx = Cx::default();
    let root = Some(&root);

    let id = {
        let id = cx.string(root, "id", "id", id_hint);
        if id.trim().is_empty() {
            cx.note("id", "empty");
            id_hint.to_string()
        } else {
            id
        }
    };

    let name_node = cx.child(root, "name", "name");
    let name = LocalizedName {
        id: cx.string(name_node, "id", "name.id", &id),
        en: cx.string(name_node, "en", "name.en", &id),
    };

    let fonts_node = cx.child(root, "fonts", "fonts");
    let ui_font = cx.string(fonts_node, "ui", "fonts.ui", DEFAULT_UI_FONT);
    let fonts = Fonts {
        numeric: cx.string(fonts_node, "numeric", "fonts.numeric", &ui_font),
        label: cx.string(fonts_node, "label", "fonts.label", &ui_font),
        cjk_fallback: cx.string(
            fonts_node,
            "cjkFallback",
            "fonts.cjkFallback",
            DEFAULT_CJK_FONT,
        ),
        ui: ui_font,
    };

    let color_node = cx.child(root, "color", "color");

    let surface_node = cx.child(color_node, "surface", "color.surface");
    let surface = SurfaceColors {
        app: cx.color(surface_node, "app", "color.surface.app", "#232326"),
        body: cx.color(surface_node, "body", "color.surface.body", "#161B26"),
        body_highlight: cx.color(
            surface_node,
            "bodyHighlight",
            "color.surface.bodyHighlight",
            "#1D2332",
        ),
        well: cx.color(surface_node, "well", "color.surface.well", "#06080E"),
        raised: cx.color(surface_node, "raised", "color.surface.raised", "#232A3A"),
        line: cx.color(surface_node, "line", "color.surface.line", "#2A3242"),
    };

    let text_node = cx.child(color_node, "text", "color.text");
    let text_primary = cx.color(text_node, "primary", "color.text.primary", "#EEF2F4");
    let text_secondary = cx.color(text_node, "secondary", "color.text.secondary", "#8E9AB0");
    let text = TextColors {
        silkscreen: cx.color(
            text_node,
            "silkscreen",
            "color.text.silkscreen",
            &text_secondary,
        ),
        primary: text_primary.clone(),
        secondary: text_secondary,
    };

    let role_node = cx.child(color_node, "role", "color.role");
    let role = RoleColors {
        label: cx.color(role_node, "label", "color.role.label", &text.secondary),
        adjustable: cx.color(role_node, "adjustable", "color.role.adjustable", "#35D0F2"),
        active: cx.color(role_node, "active", "color.role.active", "#3FE08A"),
        position: cx.color(role_node, "position", "color.role.position", "#E04FD3"),
        caution: cx.color(role_node, "caution", "color.role.caution", "#FFB020"),
        clip: cx.color(role_node, "clip", "color.role.clip", "#FF4A4A"),
    };

    let lit_node = cx.child(color_node, "lit", "color.lit");
    let lit_on = cx.color(lit_node, "on", "color.lit.on", &role.active);
    let lit_secondary = cx.color(lit_node, "secondary", "color.lit.secondary", &role.caution);
    let lit = LitColors {
        ghost: cx.color(
            lit_node,
            "ghost",
            "color.lit.ghost",
            "rgba(255,255,255,0.06)",
        ),
        glow: cx.color(lit_node, "glow", "color.lit.glow", "rgba(0,0,0,0)"),
        meter_face: cx.color(lit_node, "meterFace", "color.lit.meterFace", &lit_on),
        needle: cx.color(lit_node, "needle", "color.lit.needle", &lit_secondary),
        knob: cx.color(lit_node, "knob", "color.lit.knob", &text_primary),
        led: cx.color(lit_node, "led", "color.lit.led", &role.active),
        on: lit_on,
        secondary: lit_secondary,
    };

    let shape_node = cx.child(root, "shape", "shape");
    let radius_node = cx.child(shape_node, "radius", "shape.radius");
    let shape = Shape {
        radius: Radius {
            sm: cx.number(radius_node, "sm", "shape.radius.sm", 2.0, 0.0, 64.0),
            md: cx.number(radius_node, "md", "shape.radius.md", 4.0, 0.0, 64.0),
            lg: cx.number(radius_node, "lg", "shape.radius.lg", 10.0, 0.0, 64.0),
        },
        density: cx.enumeration(shape_node, "density", "shape.density"),
        hairline: cx.number(shape_node, "hairline", "shape.hairline", 1.0, 0.5, 4.0),
    };

    let variants_node = cx.child(root, "variants", "variants");
    let variants = Variants {
        meter: cx.enumeration::<MeterVariant>(variants_node, "meter", "variants.meter"),
        spectrum: cx.enumeration::<SpectrumVariant>(variants_node, "spectrum", "variants.spectrum"),
        panel_texture: cx.enumeration::<PanelTexture>(
            variants_node,
            "panelTexture",
            "variants.panelTexture",
        ),
        stage_indicator: cx.enumeration::<StageIndicator>(
            variants_node,
            "stageIndicator",
            "variants.stageIndicator",
        ),
        time_display: cx.enumeration::<TimeDisplay>(
            variants_node,
            "timeDisplay",
            "variants.timeDisplay",
        ),
    };

    let effects_node = cx.child(root, "effects", "effects");
    let effects = Effects {
        glow: cx.boolean(effects_node, "glow", "effects.glow", false),
        glass_reflection: cx.boolean(
            effects_node,
            "glassReflection",
            "effects.glassReflection",
            false,
        ),
        grain: cx.number(effects_node, "grain", "effects.grain", 0.0, 0.0, 1.0),
        label_case: cx.enumeration::<LabelCase>(effects_node, "labelCase", "effects.labelCase"),
    };

    let tone_node = cx.child(root, "toneColor", "toneColor");
    let tone_color = ToneColor {
        enabled: cx.boolean(tone_node, "enabled", "toneColor.enabled", true),
        targets: cx.tone_targets(tone_node, "targets", "toneColor.targets"),
        range: cx.number(tone_node, "range", "toneColor.range", 70.0, 0.0, 180.0),
    };

    let theme = Theme {
        id,
        name,
        version: cx.number(root, "version", "version", 1.0, 1.0, 1000.0) as u32,
        builtin: cx.boolean(root, "builtin", "builtin", false),
        base: cx.enumeration::<Base>(root, "base", "base"),
        fonts,
        color: Colors {
            surface,
            text,
            role,
            lit,
        },
        shape,
        variants,
        effects,
        tone_color,
    };

    Ok(LoadedTheme {
        theme,
        warnings: cx.warnings,
    })
}

/// Family used when a theme names no interface font. Present on Windows and
/// on most Linux desktops; the user interface adds generic fallbacks anyway.
const DEFAULT_UI_FONT: &str = "system-ui";
/// Family used when a theme names no fallback for Japanese, Korean or Chinese.
const DEFAULT_CJK_FONT: &str = "Noto Sans JP";

/// Collects the corrections made while reading one theme file.
#[derive(Default)]
struct Cx {
    warnings: Vec<String>,
}

impl Cx {
    fn note(&mut self, path: &str, reason: &str) {
        self.warnings
            .push(format!("{path}: {reason}, using default"));
    }

    /// Looks up a nested object. Reports a value that is present but not an
    /// object; a missing one is normal and stays quiet until its fields are
    /// read.
    fn child<'a>(&mut self, parent: Option<&'a Value>, key: &str, path: &str) -> Option<&'a Value> {
        match parent.and_then(|parent| parent.get(key)) {
            Some(value) if value.is_object() => Some(value),
            Some(_) => {
                self.note(path, "not an object");
                None
            }
            None => None,
        }
    }

    fn string(&mut self, parent: Option<&Value>, key: &str, path: &str, default: &str) -> String {
        match parent.and_then(|parent| parent.get(key)) {
            Some(Value::String(text)) => text.clone(),
            Some(_) => {
                self.note(path, "not a string");
                default.to_string()
            }
            None => default.to_string(),
        }
    }

    /// Reads a colour token. Anything that is not a CSS colour Onsa writes
    /// itself is refused, so a broken theme cannot smuggle in a value the
    /// stylesheet would choke on.
    fn color(&mut self, parent: Option<&Value>, key: &str, path: &str, default: &str) -> String {
        match parent.and_then(|parent| parent.get(key)) {
            Some(Value::String(text)) if is_css_color(text) => text.clone(),
            Some(Value::String(_)) => {
                self.note(path, "not a recognised colour");
                default.to_string()
            }
            Some(_) => {
                self.note(path, "not a string");
                default.to_string()
            }
            None => default.to_string(),
        }
    }

    fn number(
        &mut self,
        parent: Option<&Value>,
        key: &str,
        path: &str,
        default: f32,
        min: f32,
        max: f32,
    ) -> f32 {
        match parent.and_then(|parent| parent.get(key)) {
            Some(Value::Number(number)) => match number.as_f64() {
                Some(value) if value.is_finite() => {
                    let value = value as f32;
                    if value < min || value > max {
                        self.note(path, "out of range");
                        value.clamp(min, max)
                    } else {
                        value
                    }
                }
                _ => {
                    self.note(path, "not a finite number");
                    default
                }
            },
            Some(_) => {
                self.note(path, "not a number");
                default
            }
            None => default,
        }
    }

    fn boolean(&mut self, parent: Option<&Value>, key: &str, path: &str, default: bool) -> bool {
        match parent.and_then(|parent| parent.get(key)) {
            Some(Value::Bool(value)) => *value,
            Some(_) => {
                self.note(path, "not a boolean");
                default
            }
            None => default,
        }
    }

    fn enumeration<T: ThemeEnum>(&mut self, parent: Option<&Value>, key: &str, path: &str) -> T {
        match parent.and_then(|parent| parent.get(key)) {
            Some(Value::String(text)) => match T::parse(text) {
                Some(value) => value,
                None => {
                    self.note(path, "unknown value");
                    T::default()
                }
            },
            Some(_) => {
                self.note(path, "not a string");
                T::default()
            }
            None => T::default(),
        }
    }

    fn tone_targets(&mut self, parent: Option<&Value>, key: &str, path: &str) -> Vec<ToneTarget> {
        let Some(value) = parent.and_then(|parent| parent.get(key)) else {
            return Vec::new();
        };
        let Some(items) = value.as_array() else {
            self.note(path, "not an array");
            return Vec::new();
        };

        let mut targets = Vec::with_capacity(items.len());
        for (index, item) in items.iter().enumerate() {
            match item.as_str().and_then(ToneTarget::parse) {
                Some(target) if !targets.contains(&target) => targets.push(target),
                Some(_) => {}
                None => self
                    .warnings
                    .push(format!("{path}[{index}]: unknown value, ignored")),
            }
        }
        targets
    }
}

/// The closed vocabularies of the theme file, as read by [`Cx::enumeration`].
trait ThemeEnum: Default {
    /// Parses the spelling used in theme files.
    fn parse(text: &str) -> Option<Self>
    where
        Self: Sized;
}

macro_rules! impl_theme_enum {
    ($($name:ty),+ $(,)?) => {
        $(impl ThemeEnum for $name {
            fn parse(text: &str) -> Option<Self> {
                <$name>::parse(text)
            }
        })+
    };
}

impl_theme_enum!(
    Base,
    Density,
    LabelCase,
    MeterVariant,
    PanelTexture,
    SpectrumVariant,
    StageIndicator,
    TimeDisplay,
);

/// Whether a string is one of the CSS colour notations Onsa uses: a hex
/// colour, an `rgb`/`rgba`/`hsl`/`hsla` function, or `transparent`.
fn is_css_color(text: &str) -> bool {
    let text = text.trim();
    if text.eq_ignore_ascii_case("transparent") {
        return true;
    }
    if let Some(digits) = text.strip_prefix('#') {
        return matches!(digits.len(), 3 | 4 | 6 | 8)
            && digits.bytes().all(|byte| byte.is_ascii_hexdigit());
    }
    for prefix in ["rgb(", "rgba(", "hsl(", "hsla("] {
        if let Some(rest) = text.strip_prefix(prefix) {
            return rest.ends_with(')')
                && rest.bytes().all(|byte| {
                    byte.is_ascii_digit() || b".,%-+ )".contains(&byte) || byte == b'/'
                });
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_complete_theme() {
        let source = r##"{
            "id": "example",
            "name": { "id": "Contoh", "en": "Example" },
            "version": 1,
            "builtin": false,
            "base": "dark",
            "fonts": { "ui": "B612", "numeric": "B612 Mono", "cjkFallback": "Noto Sans JP" },
            "color": {
                "role": { "clip": "#FF4A4A" },
                "lit": { "on": "rgba(92,242,207,0.6)" }
            },
            "shape": { "radius": { "sm": 3 }, "density": "compact", "hairline": 1 },
            "variants": { "meter": "needle", "spectrum": "soft" },
            "effects": { "glow": true, "grain": 0.25, "labelCase": "uppercase" },
            "toneColor": { "enabled": true, "targets": ["spectrum", "meter"], "range": 50 }
        }"##;

        let loaded = parse_theme("ignored", source).expect("valid theme");

        assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
        assert_eq!(loaded.theme.id, "example");
        assert_eq!(loaded.theme.name.en, "Example");
        assert_eq!(loaded.theme.variants.meter, MeterVariant::Needle);
        assert_eq!(loaded.theme.shape.density, Density::Compact);
        assert_eq!(loaded.theme.effects.label_case, LabelCase::Uppercase);
        assert_eq!(
            loaded.theme.tone_color.targets,
            vec![ToneTarget::Spectrum, ToneTarget::Meter]
        );
        // A font the file leaves out follows the interface font.
        assert_eq!(loaded.theme.fonts.label, "B612");
    }

    #[test]
    fn falls_back_to_the_file_stem_when_the_theme_does_not_name_itself() {
        let loaded = parse_theme("from-file-name", "{}").expect("valid theme");
        assert_eq!(loaded.theme.id, "from-file-name");
        assert_eq!(loaded.theme.name.id, "from-file-name");
    }

    #[test]
    fn replaces_broken_fields_and_reports_them() {
        let source = r#"{
            "id": "broken",
            "name": "not an object",
            "base": "chartreuse",
            "color": { "role": { "active": "drop table", "clip": 12 } },
            "shape": { "radius": { "sm": 900 } },
            "effects": { "glow": "yes", "grain": 4 },
            "toneColor": { "targets": ["spectrum", "kitchen-sink"] }
        }"#;

        let loaded = parse_theme("broken", source).expect("valid theme");

        assert_eq!(loaded.theme.base, Base::Dark);
        assert_eq!(loaded.theme.color.role.active, "#3FE08A");
        assert_eq!(loaded.theme.color.role.clip, "#FF4A4A");
        assert_eq!(loaded.theme.shape.radius.sm, 64.0);
        assert!(!loaded.theme.effects.glow);
        assert_eq!(loaded.theme.effects.grain, 1.0);
        assert_eq!(loaded.theme.tone_color.targets, vec![ToneTarget::Spectrum]);

        let warnings = loaded.warnings.join("\n");
        for path in [
            "name",
            "base",
            "color.role.active",
            "color.role.clip",
            "shape.radius.sm",
            "effects.glow",
            "effects.grain",
            "toneColor.targets[1]",
        ] {
            assert!(
                warnings.contains(path),
                "missing warning for {path}:\n{warnings}"
            );
        }
    }

    #[test]
    fn refuses_text_that_is_not_a_theme_object() {
        assert!(parse_theme("x", "not json at all").is_err());
        assert!(parse_theme("x", "[1, 2, 3]").is_err());
    }

    #[test]
    fn accepts_only_colour_notations_onsa_writes() {
        for good in [
            "#fff",
            "#FFFFFF",
            "#12345678",
            "rgba(92,242,207,0.6)",
            "rgb(0, 0, 0)",
            "hsl(210 40% 12%)",
            "transparent",
        ] {
            assert!(is_css_color(good), "should accept {good}");
        }
        for bad in [
            "",
            "#ff",
            "#gggggg",
            "red",
            "url(evil.png)",
            "rgba(0,0,0,0);position:fixed",
        ] {
            assert!(!is_css_color(bad), "should refuse {bad}");
        }
    }
}
