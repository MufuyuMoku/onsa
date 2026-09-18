//! The theme model: what a theme file can say, and what every field falls
//! back to when the file does not say it (SPEC §9.3).

use serde::Serialize;

/// A validated theme, ready to be handed to the user interface.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Theme {
    /// Stable identifier, also the file stem of the theme file.
    pub id: String,
    /// Display name per user interface locale.
    pub name: LocalizedName,
    /// Schema version of the theme file.
    pub version: u32,
    /// Whether the theme ships with Onsa and is therefore read only.
    pub builtin: bool,
    /// Whether the theme is meant for a dark or a light window.
    pub base: Base,
    /// Font families the theme asks for.
    pub fonts: Fonts,
    /// Colour tokens, including the meaning based roles.
    pub color: Colors,
    /// Radii, density and hairline width.
    pub shape: Shape,
    /// Which component variant to use for meters, spectrum and so on.
    pub variants: Variants,
    /// Decorative effects.
    pub effects: Effects,
    /// How long things take to move, and how they move (SPEC §9.1).
    pub motion: Motion,
    /// Tone colour behaviour (SPEC §9.5).
    pub tone_color: ToneColor,
}

/// Display name of a theme in both user interface languages.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LocalizedName {
    /// Indonesian name.
    pub id: String,
    /// English name.
    pub en: String,
}

/// Font families a theme asks for. Values are family names only; the user
/// interface adds its own fallbacks and never loads a font from the internet.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Fonts {
    /// Family for ordinary interface text.
    pub ui: String,
    /// Family for numbers, meant to be tabular.
    pub numeric: String,
    /// Family for fixed labels; falls back to the interface family.
    pub label: String,
    /// Family used behind the others for Japanese, Korean and Chinese text.
    pub cjk_fallback: String,
}

/// All colour tokens of a theme.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Colors {
    /// Panel and window surfaces.
    pub surface: SurfaceColors,
    /// Text colours.
    pub text: TextColors,
    /// Meaning based roles. Components address these, never a raw colour.
    pub role: RoleColors,
    /// Colours of things that light up: displays, needles, indicator lamps.
    pub lit: LitColors,
    /// The two sides of a raised or sunken edge.
    pub edge: EdgeColors,
}

/// The two sides of an edge that catches light.
///
/// A bevel is drawn with a lit side and a shaded side. Every theme has them,
/// and a theme that draws nothing raised simply never uses them — which is
/// what lets a panel look moulded rather than printed without a single line
/// of code that knows which theme is on.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EdgeColors {
    /// The side the light comes from.
    pub light: String,
    /// The side away from it.
    pub dark: String,
}

/// Panel and window surfaces.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SurfaceColors {
    /// Area around the instrument panel.
    pub app: String,
    /// The panel body itself.
    pub body: String,
    /// Lit edge of the panel body.
    pub body_highlight: String,
    /// Recessed area a display sits in.
    pub well: String,
    /// Raised control surface.
    pub raised: String,
    /// Hairline separator.
    pub line: String,
}

/// Text colours.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TextColors {
    /// Main reading colour.
    pub primary: String,
    /// Supporting text.
    pub secondary: String,
    /// Printed-on-the-panel labels.
    pub silkscreen: String,
}

/// The meaning based colour roles every theme has to fill (SPEC §9.3).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RoleColors {
    /// Fixed label text.
    pub label: String,
    /// A value the user can change.
    pub adjustable: String,
    /// A stage or feature that is currently on.
    pub active: String,
    /// Playback position and playhead.
    pub position: String,
    /// A level approaching its limit.
    pub caution: String,
    /// Clipping or error.
    pub clip: String,
}

/// Colours of things that light up.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LitColors {
    /// A lit segment.
    pub on: String,
    /// An unlit segment that stays faintly visible.
    pub ghost: String,
    /// Glow around a lit element.
    pub glow: String,
    /// Second lit colour, such as the amber filter of Kaca asap.
    pub secondary: String,
    /// Face of a needle meter; falls back to the lit colour.
    pub meter_face: String,
    /// Needle of a needle meter; falls back to the second lit colour.
    pub needle: String,
    /// Metal of a knob; falls back to the primary text colour.
    pub knob: String,
    /// Indicator lamp; falls back to the active role.
    pub led: String,
}

/// Radii, density and hairline width.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Shape {
    /// Corner radii in pixels.
    pub radius: Radius,
    /// How tightly rows are packed.
    pub density: Density,
    /// Hairline width in pixels.
    pub hairline: f32,
    /// How a control is cut and edged.
    pub control: ControlShape,
    /// How a panel is framed.
    pub frame: FrameStyle,
}

/// Corner radii in pixels.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Radius {
    /// Small radius, for chips and segments.
    pub sm: f32,
    /// Medium radius, for displays and controls.
    pub md: f32,
    /// Large radius, for the panel itself.
    pub lg: f32,
}

/// Component variants a theme picks (SPEC §9.3).
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Variants {
    /// How level meters are drawn.
    pub meter: MeterVariant,
    /// How the spectrum is drawn.
    pub spectrum: SpectrumVariant,
    /// Material of the panel.
    pub panel_texture: PanelTexture,
    /// How an active signal path stage is marked.
    pub stage_indicator: StageIndicator,
    /// How the time display is drawn.
    pub time_display: TimeDisplay,
    /// How the rows of a list are told apart.
    pub rows: RowStyle,
    /// How a scrollbar is drawn.
    pub scrollbar: Scrollbar,
    /// How a tooltip is drawn.
    pub tooltip: TooltipStyle,
    /// How a dialog is drawn.
    pub dialog: DialogStyle,
}

/// How long things take to move, and how they move (SPEC §9.1).
///
/// A theme may set the pace but not the manner: both durations are held
/// inside a range that stays quick, and the easings on offer all settle
/// rather than bounce.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Motion {
    /// A small change: a colour, an outline. Milliseconds.
    pub fast: f32,
    /// A larger one: a panel opening. Milliseconds.
    pub slow: f32,
    /// The curve both follow.
    pub ease: Easing,
}

/// Decorative effects.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Effects {
    /// Whether lit elements glow.
    pub glow: bool,
    /// Whether displays carry a thin glass reflection.
    pub glass_reflection: bool,
    /// Film grain over the panel, 0.0 to 1.0.
    pub grain: f32,
    /// How label text is cased.
    pub label_case: LabelCase,
}

/// Tone colour behaviour (SPEC §9.5).
///
/// The shift is a move between two colours the theme names rather than a
/// hue rotation: a rotation is easy to see on a saturated phosphor and
/// almost invisible on warm cream, and each theme knows best which way its
/// own colours should lean.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ToneColor {
    /// Whether the theme uses tone colour at all.
    pub enabled: bool,
    /// Which elements follow the tone.
    pub targets: Vec<ToneTarget>,
    /// Where heavy sound leans.
    pub warm: String,
    /// Where bright sound leans.
    pub cool: String,
}

/// Declares one of the small closed vocabularies of the theme file: an enum
/// with a default, the spelling used in theme files and in data attributes,
/// and parsing that reports an unknown value instead of failing.
macro_rules! theme_enum {
    (
        $(#[$meta:meta])*
        $name:ident, default $default:ident {
            $($(#[$vmeta:meta])* $variant:ident => $text:literal),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
        #[serde(rename_all = "kebab-case")]
        pub enum $name {
            $($(#[$vmeta])* $variant),+
        }

        impl $name {
            /// The value as written in a theme file and in a data attribute.
            pub fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $text),+
                }
            }

            /// Parses a value from a theme file; `None` when it is unknown.
            pub fn parse(text: &str) -> Option<Self> {
                match text {
                    $($text => Some(Self::$variant),)+
                    _ => None,
                }
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::$default
            }
        }
    };
}

theme_enum!(
    /// Whether a theme is meant for a dark or a light window.
    Base, default Dark {
        /// Dark window. All three built-in themes are dark.
        Dark => "dark",
        /// Light window. No light theme is designed yet (SPEC §16).
        Light => "light",
    }
);

theme_enum!(
    /// How level meters are drawn.
    MeterVariant, default Bar {
        /// A continuous bar.
        Bar => "bar",
        /// A row of discrete segments.
        Segment => "segment",
        /// A VU needle.
        Needle => "needle",
    }
);

theme_enum!(
    /// How the spectrum is drawn.
    SpectrumVariant, default Bar {
        /// Solid bars.
        Bar => "bar",
        /// Stacked segments.
        Segment => "segment",
        /// Soft gradient bars.
        Soft => "soft",
    }
);

theme_enum!(
    /// Material of the panel.
    PanelTexture, default Bezel {
        /// Dark glass over a lit display.
        Glass => "glass",
        /// Layered bezel.
        Bezel => "bezel",
        /// Brushed metal.
        Brushed => "brushed",
    }
);

theme_enum!(
    /// How an active signal path stage is marked.
    StageIndicator, default OutlineChip {
        /// A chip that glows when the stage is on.
        GlowChip => "glow-chip",
        /// A chip whose outline changes colour.
        OutlineChip => "outline-chip",
        /// A small indicator lamp next to the label.
        Led => "led",
    }
);

theme_enum!(
    /// How the time display is drawn.
    TimeDisplay, default Plain {
        /// Unlit segments stay faintly visible behind the digits.
        GhostSegment => "ghost-segment",
        /// Plain digits.
        Plain => "plain",
    }
);

theme_enum!(
    /// How label text is cased.
    LabelCase, default AsWritten {
        /// Exactly as written in the dictionary.
        AsWritten => "as-written",
        /// Capitals, silkscreen style.
        Uppercase => "uppercase",
    }
);

theme_enum!(
    /// How tightly rows are packed.
    Density, default Comfortable {
        /// Roomy rows.
        Comfortable => "comfortable",
        /// Tight rows.
        Compact => "compact",
    }
);

theme_enum!(
    /// An element that can follow the tone colour.
    ToneTarget, default Spectrum {
        /// The spectrum analyser.
        Spectrum => "spectrum",
        /// Level meters.
        Meter => "meter",
        /// The progress bar and playhead.
        Progress => "progress",
        /// Glow around the cover art.
        CoverGlow => "cover-glow",
    }
);

theme_enum!(
    /// How a control is cut and edged.
    ControlShape, default Soft {
        /// Rounded by the theme's own radius.
        Soft => "soft",
        /// Square corners, whatever the radius says.
        Square => "square",
        /// Fully rounded ends.
        Pill => "pill",
        /// A moulded edge: lit on one side, shaded on the other.
        Bevel => "bevel",
    }
);

theme_enum!(
    /// How a panel is framed.
    FrameStyle, default Hairline {
        /// One thin line.
        Hairline => "hairline",
        /// Sunken into the surface around it.
        Inset => "inset",
        /// Standing out of it.
        Raised => "raised",
        /// Nothing at all.
        None => "none",
    }
);

theme_enum!(
    /// How the rows of a list are told apart.
    RowStyle, default Plain {
        /// Nothing between them but space, which is what Onsa has always
        /// done: a list of tracks is quiet until something is under the
        /// pointer.
        Plain => "plain",
        /// A hairline under each row.
        Lines => "lines",
        /// Every other row on a slightly different ground.
        Stripes => "stripes",
    }
);

theme_enum!(
    /// How a scrollbar is drawn.
    Scrollbar, default Thin {
        /// A narrow bar that stays out of the way.
        Thin => "thin",
        /// A wide bar with a visible track, the way a desktop draws one.
        Classic => "classic",
        /// No bar at all; the panel still scrolls.
        Hidden => "hidden",
    }
);

theme_enum!(
    /// How a tooltip is drawn.
    TooltipStyle, default Plain {
        /// A small dark box.
        Plain => "plain",
        /// The same material as a panel.
        Panel => "panel",
    }
);

theme_enum!(
    /// How a dialog is drawn.
    DialogStyle, default Flat {
        /// A panel with a hairline around it.
        Flat => "flat",
        /// Standing out of the window, with a shadow under it.
        Raised => "raised",
        /// With a bar across the top carrying its name.
        Titled => "titled",
    }
);

theme_enum!(
    /// The curve a change follows. None of them overshoot (SPEC §9.1).
    Easing, default Standard {
        /// Quick away, settling at the end.
        Standard => "standard",
        /// The same speed throughout.
        Linear => "linear",
        /// Away at once, then slowing.
        Snap => "snap",
        /// Gentle at both ends.
        Soft => "soft",
    }
);

impl Easing {
    /// The curve as CSS writes it.
    pub fn css(self) -> &'static str {
        match self {
            Self::Standard => "cubic-bezier(0.2, 0, 0, 1)",
            Self::Linear => "linear",
            Self::Snap => "cubic-bezier(0.4, 0, 1, 1)",
            Self::Soft => "cubic-bezier(0.4, 0, 0.2, 1)",
        }
    }
}
