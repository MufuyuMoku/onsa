//! Tolerant reading of a theme file.
//!
//! A theme must never be able to break the application (SPEC §9.3), so parsing
//! never rejects a field: anything missing, of the wrong type, or outside its
//! range is replaced by the default and reported as a warning. Only a file
//! that is not a JSON object at all is refused.

use anyhow::{anyhow, Context, Result};
use serde_json::Value;

use std::collections::BTreeMap;

use super::model::{
    Arrangement, Base, Chrome, ChromeColors, Colors, ControlShape, CoverFrame, Decoration, Density,
    DialogStyle, Easing, EdgeColors, Effects, Fonts, FrameStyle, LabelCase, LibraryHome, LitColors,
    LocalizedName, MeterVariant, Motion, NowPlayingStyle, PanelTexture, PositionStyle,
    ProgressStyle, QueuePlace, Radius, RoleColors, RowStyle, Scrollbar, Shape, SidebarPlace,
    Signature, Sizes, SpectrumVariant, StageIndicator, SurfaceColors, TextColors, Theme,
    TimeDisplay, ToneColor, ToneTarget, TooltipStyle, TransportStyle, Variants,
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

    // Only a family Onsa bundles can be named (SPEC §9.3, layer 6): the
    // interface never loads a font from anywhere, so a family it does not
    // carry would quietly turn into whatever the system has.
    let fonts_node = cx.child(root, "fonts", "fonts");
    let ui_font = cx.font(fonts_node, "ui", "fonts.ui", DEFAULT_UI_FONT);
    let label_font = cx.font(fonts_node, "label", "fonts.label", &ui_font);
    let numeric_font = cx.font(fonts_node, "numeric", "fonts.numeric", &ui_font);
    let fonts = Fonts {
        accent: cx.font(fonts_node, "accent", "fonts.accent", &numeric_font),
        numeric: numeric_font,
        display: cx.font(fonts_node, "display", "fonts.display", &label_font),
        lyrics: cx.font(fonts_node, "lyrics", "fonts.lyrics", &ui_font),
        cjk_fallback: cx.font(
            fonts_node,
            "cjkFallback",
            "fonts.cjkFallback",
            DEFAULT_CJK_FONT,
        ),
        label: label_font,
        ui: ui_font,
    };

    let color_node = cx.child(root, "color", "color");

    let surface_node = cx.child(color_node, "surface", "color.surface");
    let raised_surface = cx.color(surface_node, "raised", "color.surface.raised", "#232A3A");
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
        raised: raised_surface.clone(),
        line: cx.color(surface_node, "line", "color.surface.line", "#2A3242"),
        backdrop: cx.backdrop(surface_node, "backdrop", "color.surface.backdrop"),
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
    let meter_face = cx.color(lit_node, "meterFace", "color.lit.meterFace", &lit_on);
    let lit = LitColors {
        ghost: cx.color(
            lit_node,
            "ghost",
            "color.lit.ghost",
            "rgba(255,255,255,0.06)",
        ),
        glow: cx.color(lit_node, "glow", "color.lit.glow", "rgba(0,0,0,0)"),
        meter_face: meter_face.clone(),
        needle: cx.color(lit_node, "needle", "color.lit.needle", &lit_secondary),
        knob: cx.color(lit_node, "knob", "color.lit.knob", &text_primary),
        led: cx.color(lit_node, "led", "color.lit.led", &role.active),
        stripe: cx.color(lit_node, "stripe", "color.lit.stripe", &role.caution),
        highlight: cx.color(lit_node, "highlight", "color.lit.highlight", "transparent"),
        highlight_text: cx.color(
            lit_node,
            "highlightText",
            "color.lit.highlightText",
            &role.position,
        ),
        ground: cx.color(lit_node, "ground", "color.lit.ground", &raised_surface),
        ornament: cx.color(lit_node, "ornament", "color.lit.ornament", &role.label),
        paper: cx.color(lit_node, "paper", "color.lit.paper", &surface.body),
        paper_edge: cx.color(lit_node, "paperEdge", "color.lit.paperEdge", &surface.line),
        lcd: cx.color(lit_node, "lcd", "color.lit.lcd", &meter_face),
        lcd_ink: cx.color(lit_node, "lcdInk", "color.lit.lcdInk", &surface.well),
        lcd_edge: cx.color(lit_node, "lcdEdge", "color.lit.lcdEdge", &surface.line),
        on: lit_on,
        secondary: lit_secondary,
    };

    let edge_node = cx.child(color_node, "edge", "color.edge");
    let edge = EdgeColors {
        // What a raised edge looks like when nobody says: a little light on
        // top, rather more shadow underneath.
        light: cx.color(
            edge_node,
            "light",
            "color.edge.light",
            "rgba(255,255,255,0.10)",
        ),
        dark: cx.color(edge_node, "dark", "color.edge.dark", "rgba(0,0,0,0.45)"),
    };

    let chrome_node = cx.child(color_node, "chrome", "color.chrome");
    let taskbar = cx.gradient(
        chrome_node,
        "taskbar",
        "color.chrome.taskbar",
        &surface.raised,
    );
    let chrome = ChromeColors {
        outer: cx.color(chrome_node, "outer", "color.chrome.outer", &surface.app),
        frame: cx.color(chrome_node, "frame", "color.chrome.frame", &edge.dark),
        title: cx.gradient(chrome_node, "title", "color.chrome.title", &role.adjustable),
        title_text: cx.color(
            chrome_node,
            "titleText",
            "color.chrome.titleText",
            &surface.body,
        ),
        pane: cx.gradient(chrome_node, "pane", "color.chrome.pane", &surface.raised),
        pane_box: cx.color(
            chrome_node,
            "paneBox",
            "color.chrome.paneBox",
            &surface.body,
        ),
        pane_text: cx.color(
            chrome_node,
            "paneText",
            "color.chrome.paneText",
            &role.adjustable,
        ),
        taskbar_text: cx.color(
            chrome_node,
            "taskbarText",
            "color.chrome.taskbarText",
            &text.primary,
        ),
        start: cx.gradient(chrome_node, "start", "color.chrome.start", &role.active),
        tray: match chrome_node.and_then(|node| node.get("tray")) {
            Some(_) => cx.gradient(chrome_node, "tray", "color.chrome.tray", &surface.raised),
            None => taskbar.clone(),
        },
        taskbar,
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
        control: cx.enumeration::<ControlShape>(shape_node, "control", "shape.control"),
        frame: cx.enumeration::<FrameStyle>(shape_node, "frame", "shape.frame"),
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
        rows: cx.enumeration::<RowStyle>(variants_node, "rows", "variants.rows"),
        scrollbar: cx.enumeration::<Scrollbar>(variants_node, "scrollbar", "variants.scrollbar"),
        tooltip: cx.enumeration::<TooltipStyle>(variants_node, "tooltip", "variants.tooltip"),
        dialog: cx.enumeration::<DialogStyle>(variants_node, "dialog", "variants.dialog"),
        progress: cx.enumeration::<ProgressStyle>(variants_node, "progress", "variants.progress"),
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

    // A theme may set the pace but not the manner: quick stays quick, and
    // every easing on offer settles rather than bounces (SPEC §9.1).
    let motion_node = cx.child(root, "motion", "motion");
    let motion = Motion {
        fast: cx.number(motion_node, "fast", "motion.fast", 100.0, 40.0, 220.0),
        slow: cx.number(motion_node, "slow", "motion.slow", 150.0, 60.0, 320.0),
        ease: cx.enumeration::<Easing>(motion_node, "ease", "motion.ease"),
    };

    let tone_node = cx.child(root, "toneColor", "toneColor");
    let tone_color = ToneColor {
        enabled: cx.boolean(tone_node, "enabled", "toneColor.enabled", true),
        targets: cx.tone_targets(tone_node, "targets", "toneColor.targets"),
        warm: cx.color(tone_node, "warm", "toneColor.warm", &lit.secondary),
        cool: cx.color(tone_node, "cool", "toneColor.cool", &lit.on),
    };

    let ornament = cx.ornament(root, "ornament", "ornament");
    let decor = cx.list::<Decoration>(root, "decor", "decor");
    let signature = cx.signature(root, "signature", "signature");

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
            edge,
            chrome,
        },
        shape,
        variants,
        effects,
        motion,
        tone_color,
        ornament,
        decor,
        signature,
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

/// Every family Onsa bundles in `ui/static/fonts/`, each with its licence
/// (SPEC §9.4, §14), plus the system's own interface font. A test keeps
/// this list and the stylesheet that declares them in step.
pub(crate) const BUNDLED_FONTS: &[&str] = &[
    "system-ui",
    "Noto Sans JP",
    "B612 Mono",
    "Saira Stencil One",
    "Cormorant Garamond",
    "Martian Mono",
    "Courier Prime",
    "VT323",
    "Gloock",
    "Pixelify Sans",
    "Orbitron",
    "Exo 2",
    "DejaVu Sans Condensed",
];

/// How a key of ornament text has to look: `ornament.<theme>.<name>`, the
/// same shape the dictionary uses, and nothing a key could hide in.
fn is_ornament_key(key: &str) -> bool {
    let mut parts = key.split('.');
    let well_formed = |part: &str| {
        !part.is_empty()
            && part
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
    };
    matches!(
        (parts.next(), parts.next(), parts.next(), parts.next()),
        (Some("ornament"), Some(theme), Some(name), None) if well_formed(theme) && well_formed(name)
    )
}

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

    /// Reads a gradient: one colour, or a list of one to six. Anything
    /// else is reported and the default stands alone.
    fn gradient(
        &mut self,
        parent: Option<&Value>,
        key: &str,
        path: &str,
        default: &str,
    ) -> Vec<String> {
        let Some(value) = parent.and_then(|parent| parent.get(key)) else {
            return vec![default.to_string()];
        };
        if let Some(one) = value.as_str().filter(|text| is_css_color(text)) {
            return vec![one.to_string()];
        }
        let colours: Option<Vec<String>> = value.as_array().and_then(|items| {
            items
                .iter()
                .map(|item| {
                    item.as_str()
                        .filter(|text| is_css_color(text))
                        .map(str::to_string)
                })
                .collect()
        });
        match colours {
            Some(colours) if (1..=6).contains(&colours.len()) => colours,
            _ => {
                self.note(path, "not one to six colours");
                vec![default.to_string()]
            }
        }
    }

    /// Reads the colours of a backdrop: two to six of them, each a colour
    /// Onsa writes. Anything else leaves the window without one.
    fn backdrop(&mut self, parent: Option<&Value>, key: &str, path: &str) -> Vec<String> {
        let Some(value) = parent.and_then(|parent| parent.get(key)) else {
            return Vec::new();
        };
        let colours: Option<Vec<String>> = value.as_array().and_then(|items| {
            items
                .iter()
                .map(|item| {
                    item.as_str()
                        .filter(|text| is_css_color(text))
                        .map(str::to_string)
                })
                .collect()
        });
        match colours {
            Some(colours) if (2..=6).contains(&colours.len()) => colours,
            _ => {
                self.note(path, "not two to six colours");
                Vec::new()
            }
        }
    }

    /// Reads a font family, which has to be one Onsa bundles.
    fn font(&mut self, parent: Option<&Value>, key: &str, path: &str, default: &str) -> String {
        let family = self.string(parent, key, path, default);
        if BUNDLED_FONTS.contains(&family.as_str()) {
            return family;
        }
        self.warnings.push(format!(
            "{path}: \"{family}\" is not a font Onsa carries, using {default}"
        ));
        default.to_string()
    }

    /// Reads a list of words from one of the closed vocabularies. An unknown
    /// word is reported and left out; the rest stand.
    fn list<T: ThemeEnum + PartialEq>(
        &mut self,
        parent: Option<&Value>,
        key: &str,
        path: &str,
    ) -> Vec<T> {
        let Some(value) = parent.and_then(|parent| parent.get(key)) else {
            return Vec::new();
        };
        let Some(items) = value.as_array() else {
            self.note(path, "not an array");
            return Vec::new();
        };
        let mut found = Vec::with_capacity(items.len());
        for (index, item) in items.iter().enumerate() {
            match item.as_str().and_then(T::parse) {
                Some(one) if !found.contains(&one) => found.push(one),
                Some(_) => {}
                None => self
                    .warnings
                    .push(format!("{path}[{index}]: unknown value, ignored")),
            }
        }
        found
    }

    /// Reads the keys of ornament text. A key that is not shaped like one is
    /// reported and left out.
    fn ornament(&mut self, parent: Option<&Value>, key: &str, path: &str) -> Vec<String> {
        let Some(value) = parent.and_then(|parent| parent.get(key)) else {
            return Vec::new();
        };
        let Some(items) = value.as_array() else {
            self.note(path, "not an array");
            return Vec::new();
        };
        let mut keys: Vec<String> = Vec::new();
        for (index, item) in items.iter().enumerate() {
            match item.as_str() {
                Some(text) if is_ornament_key(text) => {
                    if !keys.iter().any(|one| one == text) {
                        keys.push(text.to_string());
                    }
                }
                _ => self
                    .warnings
                    .push(format!("{path}[{index}]: not an ornament key, ignored")),
            }
        }
        keys
    }

    /// Reads a signature layout. A theme without one is an ordinary theme;
    /// one with a broken one keeps what can be read of it.
    fn signature(&mut self, parent: Option<&Value>, key: &str, path: &str) -> Option<Signature> {
        let node = Some(self.child(parent, key, path)?);
        let at = |field: &str| format!("{path}.{field}");
        let id = self.string(node, "id", &at("id"), "");
        if id.trim().is_empty() {
            self.note(&at("id"), "missing, so there is no signature layout");
            return None;
        }
        let name_node = self.child(node, "name", &at("name"));
        let name = LocalizedName {
            id: self.string(name_node, "id", &at("name.id"), &id),
            en: self.string(name_node, "en", &at("name.en"), &id),
        };
        let arrangement = Arrangement {
            sidebar: self.enumeration::<SidebarPlace>(node, "sidebar", &at("sidebar")),
            queue: self.enumeration::<QueuePlace>(node, "queue", &at("queue")),
            transport: self.enumeration::<TransportStyle>(node, "transport", &at("transport")),
            signal: self.boolean(node, "signal", &at("signal"), true),
            library_home: self.enumeration::<LibraryHome>(node, "libraryHome", &at("libraryHome")),
            now_playing: self.enumeration::<NowPlayingStyle>(node, "nowPlaying", &at("nowPlaying")),
            cover: self.enumeration::<CoverFrame>(node, "cover", &at("cover")),
            position: self.enumeration::<PositionStyle>(node, "position", &at("position")),
            chrome: self.enumeration::<Chrome>(node, "chrome", &at("chrome")),
            plates: self.boolean(node, "plates", &at("plates"), false),
            panels: self.panels(node, "panels", &at("panels")),
            sizes: self.sizes(node, "sizes", &at("sizes")),
        };
        Some(Signature {
            id,
            name,
            arrangement,
        })
    }

    /// Reads the sizes a layout sets. Each is optional; one out of range is
    /// brought into it and reported.
    fn sizes(&mut self, parent: Option<&Value>, key: &str, path: &str) -> Sizes {
        let Some(node) = self.child(parent, key, path) else {
            return Sizes::default();
        };
        let node = Some(node);
        let mut size = |name: &str, min: f32, max: f32| -> Option<u32> {
            node.and_then(|node| node.get(name))?;
            let at = format!("{path}.{name}");
            Some(self.number(node, name, &at, min, min, max).round() as u32)
        };
        Sizes {
            cover: size("cover", 96.0, 480.0),
            title: size("title", 16.0, 64.0),
            lyrics: size("lyrics", 96.0, 480.0),
            spectrum: size("spectrum", 60.0, 400.0),
        }
    }

    /// Reads which Now Playing panels a layout shows. Only the four panels
    /// of M14 are known; anything else is reported and left out.
    fn panels(&mut self, parent: Option<&Value>, key: &str, path: &str) -> BTreeMap<String, bool> {
        const PANELS: [&str; 4] = ["cover", "details", "lyrics", "visualizer"];
        let mut panels = BTreeMap::new();
        let Some(node) = self.child(parent, key, path) else {
            return panels;
        };
        if let Some(entries) = node.as_object() {
            for (name, shown) in entries {
                match (PANELS.contains(&name.as_str()), shown.as_bool()) {
                    (true, Some(shown)) => {
                        panels.insert(name.clone(), shown);
                    }
                    _ => self.warnings.push(format!(
                        "{path}.{name}: not a panel shown or hidden, ignored"
                    )),
                }
            }
        }
        panels
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
    Chrome,
    ControlShape,
    CoverFrame,
    Decoration,
    LibraryHome,
    NowPlayingStyle,
    PositionStyle,
    ProgressStyle,
    QueuePlace,
    SidebarPlace,
    TransportStyle,
    Density,
    DialogStyle,
    Easing,
    FrameStyle,
    LabelCase,
    MeterVariant,
    PanelTexture,
    RowStyle,
    Scrollbar,
    SpectrumVariant,
    StageIndicator,
    TimeDisplay,
    TooltipStyle,
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
            "fonts": { "ui": "Courier Prime", "numeric": "B612 Mono", "cjkFallback": "Noto Sans JP" },
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
        assert_eq!(loaded.theme.fonts.label, "Courier Prime");
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

    #[test]
    fn a_theme_that_says_nothing_about_shape_looks_the_way_onsa_always_did() {
        // Everything the deeper scheme added has to default to what was
        // there before it existed, or every theme written so far changes
        // the moment Onsa updates.
        let loaded = parse_theme("bare", "{}").expect("an object");
        let theme = loaded.theme;
        assert_eq!(theme.shape.control, ControlShape::Soft);
        assert_eq!(theme.shape.frame, FrameStyle::Hairline);
        assert_eq!(theme.variants.rows, RowStyle::Plain);
        assert_eq!(theme.variants.scrollbar, Scrollbar::Thin);
        assert_eq!(theme.variants.tooltip, TooltipStyle::Plain);
        assert_eq!(theme.variants.dialog, DialogStyle::Flat);
        assert_eq!(theme.motion.ease, Easing::Standard);
        assert_eq!(theme.motion.fast, 100.0);
        assert_eq!(theme.motion.slow, 150.0);
        assert!(theme.color.edge.light.starts_with("rgba("));
        assert!(theme.color.edge.dark.starts_with("rgba("));
    }

    #[test]
    fn a_theme_can_be_moulded_rather_than_printed() {
        let source = r##"{
            "id": "moulded",
            "color": { "edge": { "light": "#FFFFFF", "dark": "#404040" } },
            "shape": { "control": "bevel", "frame": "raised" },
            "variants": {
                "rows": "stripes",
                "scrollbar": "classic",
                "tooltip": "panel",
                "dialog": "titled"
            },
            "motion": { "fast": 60, "slow": 90, "ease": "snap" }
        }"##;
        let loaded = parse_theme("moulded", source).expect("an object");
        assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
        let theme = loaded.theme;
        assert_eq!(theme.shape.control, ControlShape::Bevel);
        assert_eq!(theme.shape.frame, FrameStyle::Raised);
        assert_eq!(theme.variants.rows, RowStyle::Stripes);
        assert_eq!(theme.variants.scrollbar, Scrollbar::Classic);
        assert_eq!(theme.variants.tooltip, TooltipStyle::Panel);
        assert_eq!(theme.variants.dialog, DialogStyle::Titled);
        assert_eq!(theme.color.edge.light, "#FFFFFF");
        assert_eq!(theme.motion.fast, 60.0);
        assert_eq!(theme.motion.ease, Easing::Snap);
    }

    #[test]
    fn a_theme_may_set_the_pace_but_not_the_manner() {
        // Onsa's animations are short and never bounce (SPEC §9.1). A theme
        // asking for half a second is brought back to what is still quick,
        // and the warning says so.
        let source = r##"{ "motion": { "fast": 900, "slow": 5, "ease": "boing" } }"##;
        let loaded = parse_theme("slow", source).expect("an object");
        assert!(
            loaded.theme.motion.fast <= 220.0,
            "{}",
            loaded.theme.motion.fast
        );
        assert!(
            loaded.theme.motion.slow >= 60.0,
            "{}",
            loaded.theme.motion.slow
        );
        assert_eq!(
            loaded.theme.motion.ease,
            Easing::Standard,
            "an unknown curve"
        );
        assert_eq!(loaded.warnings.len(), 3, "{:?}", loaded.warnings);

        // None of the curves on offer overshoot: a bezier whose control
        // points stay between 0 and 1 cannot go past where it is heading,
        // which is what bouncing looks like (SPEC §9.1).
        for ease in [Easing::Standard, Easing::Linear, Easing::Snap, Easing::Soft] {
            let css = ease.css();
            let Some(inside) = css
                .strip_prefix("cubic-bezier(")
                .and_then(|rest| rest.strip_suffix(')'))
            else {
                continue;
            };
            let points: Vec<f32> = inside
                .split(',')
                .filter_map(|part| part.trim().parse().ok())
                .collect();
            assert_eq!(points.len(), 4, "{css}");
            for height in [points[1], points[3]] {
                assert!((0.0..=1.0).contains(&height), "{css} overshoots");
            }
        }
    }

    #[test]
    fn a_shape_nobody_has_heard_of_falls_back_and_is_reported() {
        let source =
            r##"{ "shape": { "control": "octagonal" }, "variants": { "rows": "zebra" } }"##;
        let loaded = parse_theme("odd", source).expect("an object");
        assert_eq!(loaded.theme.shape.control, ControlShape::Soft);
        assert_eq!(loaded.theme.variants.rows, RowStyle::Plain);
        assert_eq!(loaded.warnings.len(), 2, "{:?}", loaded.warnings);
    }

    #[test]
    fn a_theme_from_before_m16_reads_without_a_word_about_what_it_never_said() {
        // An old user theme names its fonts and colours and nothing of the
        // new layers. That is not a mistake to report.
        let source = r##"{ "id": "lama", "fonts": { "ui": "Courier Prime" } }"##;
        let loaded = parse_theme("lama", source).expect("an object");
        assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
        let theme = loaded.theme;
        assert_eq!(
            theme.fonts.display, "Courier Prime",
            "titles fall back to the labels"
        );
        assert_eq!(
            theme.fonts.lyrics, "Courier Prime",
            "and the words to the interface"
        );
        assert!(theme.ornament.is_empty());
        assert!(theme.decor.is_empty());
        assert_eq!(theme.signature, None);
    }

    #[test]
    fn a_font_onsa_does_not_carry_is_not_named() {
        // The interface never loads a font from anywhere (SPEC §14), so a
        // family that is not bundled would turn into whatever the system has.
        let source = r##"{ "fonts": { "ui": "Comic Sans MS", "numeric": "B612 Mono" } }"##;
        let loaded = parse_theme("x", source).expect("an object");
        assert_eq!(loaded.theme.fonts.ui, DEFAULT_UI_FONT);
        assert_eq!(loaded.theme.fonts.numeric, "B612 Mono");
        assert_eq!(loaded.warnings.len(), 1, "{:?}", loaded.warnings);
        assert!(loaded.warnings[0].contains("Comic Sans MS"));
    }

    #[test]
    fn ornament_text_is_named_by_key_and_only_by_key() {
        let source = r##"{ "ornament": [
            "ornament.kokpit-tua.plate",
            "ornament.kokpit-tua.plate",
            "<b>not a key</b>",
            "nav.downloads",
            "ornament.a.b.c",
            42
        ] }"##;
        let loaded = parse_theme("x", source).expect("an object");
        assert_eq!(loaded.theme.ornament, ["ornament.kokpit-tua.plate"]);
        assert_eq!(
            loaded.warnings.len(),
            4,
            "the duplicate is quiet, the rest are reported: {:?}",
            loaded.warnings
        );
    }

    #[test]
    fn decorations_come_from_the_list_onsa_knows_how_to_draw() {
        let source = r##"{ "decor": ["rivets", "fireworks", "hazard-stripe", "rivets"] }"##;
        let loaded = parse_theme("x", source).expect("an object");
        assert_eq!(
            loaded.theme.decor,
            [Decoration::Rivets, Decoration::HazardStripe]
        );
        assert_eq!(loaded.warnings.len(), 1, "{:?}", loaded.warnings);
    }

    #[test]
    fn a_signature_layout_is_read_as_data() {
        let source = r##"{ "signature": {
            "id": "panel-instrumen",
            "name": { "id": "Panel Instrumen", "en": "Instrument Panel" },
            "queue": "floating",
            "nowPlaying": "instrument",
            "cover": "instrument",
            "position": "dial",
            "plates": true,
            "panels": { "details": false, "mixer": true }
        } }"##;
        let loaded = parse_theme("x", source).expect("an object");
        let signature = loaded.theme.signature.expect("a signature layout");
        assert_eq!(signature.id, "panel-instrumen");
        assert_eq!(signature.name.en, "Instrument Panel");
        let arrangement = signature.arrangement;
        assert_eq!(arrangement.queue, QueuePlace::Floating);
        assert_eq!(
            arrangement.sidebar,
            SidebarPlace::Shown,
            "what it does not say is the default"
        );
        assert_eq!(arrangement.now_playing, NowPlayingStyle::Instrument);
        assert_eq!(arrangement.cover, CoverFrame::Instrument);
        assert_eq!(arrangement.position, PositionStyle::Dial);
        assert!(arrangement.plates);
        assert!(arrangement.signal);
        assert_eq!(arrangement.panels.get("details"), Some(&false));
        assert!(
            !arrangement.panels.contains_key("mixer"),
            "an unknown panel is left out"
        );
        assert_eq!(loaded.warnings.len(), 1, "{:?}", loaded.warnings);
    }

    #[test]
    fn a_layout_sets_its_own_sizes_within_bounds() {
        let source = r##"{ "signature": {
            "id": "jendela", "name": { "id": "Jendela", "en": "Window" },
            "sizes": { "cover": 190, "title": 200 }
        } }"##;
        let loaded = parse_theme("x", source).expect("an object");
        let sizes = loaded.theme.signature.expect("a layout").arrangement.sizes;
        assert_eq!(sizes.cover, Some(190));
        assert_eq!(sizes.title, Some(64), "too large is brought into range");
        assert_eq!(loaded.warnings.len(), 1, "{:?}", loaded.warnings);

        let plain = r##"{ "signature": { "id": "x", "name": { "id": "X", "en": "X" } } }"##;
        let sizes = parse_theme("x", plain)
            .expect("an object")
            .theme
            .signature
            .expect("a layout")
            .arrangement
            .sizes;
        assert_eq!(sizes, Sizes::default(), "no sizes: the style's own stand");
    }

    #[test]
    fn a_signature_without_a_name_to_call_it_by_is_no_signature() {
        let source = r##"{ "signature": { "position": "dial" } }"##;
        let loaded = parse_theme("x", source).expect("an object");
        assert_eq!(loaded.theme.signature, None);
        assert_eq!(loaded.warnings.len(), 1, "{:?}", loaded.warnings);
    }
}
