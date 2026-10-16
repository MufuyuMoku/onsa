//! The theme model: what a theme file can say, and what every field falls
//! back to when the file does not say it (SPEC §9.3).

use std::collections::BTreeMap;

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
    /// Ornament text, as dictionary keys (SPEC §9.3, layer 7). The words
    /// themselves live in the dictionary, in both languages; a key the
    /// dictionary does not know is simply not shown.
    pub ornament: Vec<String>,
    /// Decorations the theme draws (SPEC §9.1): named from a closed list,
    /// so a theme chooses among what Onsa knows how to draw rather than
    /// bringing pictures of its own.
    pub decor: Vec<Decoration>,
    /// The theme's own arrangement of the window, if it has one (SPEC §9.3,
    /// layer 8). Data, in the same shape as the standard presets.
    pub signature: Option<Signature>,
}

/// A theme's signature layout: a name, and an arrangement.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Signature {
    /// Stable identifier of the layout.
    pub id: String,
    /// Its name, as the layout picker shows it.
    pub name: LocalizedName,
    /// What goes where.
    pub arrangement: Arrangement,
}

/// What goes where in the window (SPEC M16).
///
/// The three standard presets are values of this same shape, written in
/// the interface; a signature layout is one more value, read from a theme.
/// Nothing here names a theme: every field chooses from a closed list of
/// pieces Onsa knows how to draw, and any theme may choose any of them.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Arrangement {
    /// Whether the navigation has a column of its own.
    pub sidebar: SidebarPlace,
    /// Whether the queue may stand beside the content, or only over it.
    pub queue: QueuePlace,
    /// How much of the transport is drawn.
    pub transport: TransportStyle,
    /// Whether the signal path strip is shown.
    pub signal: bool,
    /// What the library opens on.
    pub library_home: LibraryHome,
    /// How the Now Playing screen is arranged.
    pub now_playing: NowPlayingStyle,
    /// How the cover is framed on it.
    pub cover: CoverFrame,
    /// How the play position is shown on it.
    pub position: PositionStyle,
    /// Whether the window is drawn as a window inside the window.
    pub chrome: Chrome,
    /// Whether parts of the screen carry name plates.
    pub plates: bool,
    /// Which Now Playing panels show, as `setPanels` takes them (M14).
    pub panels: BTreeMap<String, bool>,
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
    /// Family for titles and headings; falls back to the label family.
    pub display: String,
    /// Family for the words of a song; falls back to the interface family.
    pub lyrics: String,
    /// Family for a theme's own touches — a camcorder's time stamp, say;
    /// falls back to the numeric family.
    pub accent: String,
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
    /// A ground the whole window stands on, top to bottom, when the theme
    /// draws one: two to six colours of a vertical gradient. Empty means
    /// the window is the plain application surface.
    pub backdrop: Vec<String>,
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
    /// Colour of warning stripes, where a theme draws them.
    pub stripe: String,
    /// Indicator lamp; falls back to the active role.
    pub led: String,
    /// Behind the line being sung, where a theme marks it as if with a
    /// highlighter; transparent when it does not.
    pub highlight: String,
    /// The floor, for a theme that draws one under the transport (a
    /// carpet, a meadow); falls back to the raised surface.
    pub ground: String,
    /// Ornament text (SPEC §9.3, layer 7); falls back to the label role.
    /// Never the clip colour: ornament is not an error.
    pub ornament: String,
    /// Something pale standing in a scene — a door, a frame of plaster —
    /// and its edge; fall back to the panel and its hairline.
    pub paper: String,
    /// The edge of [`LitColors::paper`].
    pub paper_edge: String,
    /// The backlight of a liquid crystal display, where a theme draws
    /// one; falls back to the meter face.
    pub lcd: String,
    /// Its pixels; fall back to the well.
    pub lcd_ink: String,
    /// The frame round it; falls back to the hairline.
    pub lcd_edge: String,
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
    /// How the position bar of the transport is drawn.
    pub progress: ProgressStyle,
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
        /// A thin line that glows.
        Line => "line",
        /// Dark segments on a liquid crystal display.
        Lcd => "lcd",
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
        /// Columns of points of light, some in the secondary colour.
        Dots => "dots",
        /// Blades of grass that grow with the sound.
        Grass => "grass",
        /// Dark segments on a liquid crystal display.
        Lcd => "lcd",
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
        /// The label itself lights up.
        GlowText => "glow-text",
        /// The name in square brackets, in the active colour when on.
        Brackets => "brackets",
        /// Only the colour of the name changes.
        Text => "text",
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
        /// The backlight of a liquid crystal display; never its pixels.
        Backlight => "backlight",
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
        /// A thin ring and nothing inside it.
        Ring => "ring",
        /// A box with a dark edge and a hard shadow, the way an old web
        /// page drew one.
        Box => "box",
        /// Round and chromed; the play button is clear coloured plastic.
        Chrome => "chrome",
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
        /// A card laid on it, with a hard shadow.
        Card => "card",
        /// Clear coloured plastic with large rounded corners.
        Candy => "candy",
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

theme_enum!(
    /// Something a theme draws for its look alone (SPEC §9.1). The ones that
    /// move stop in power saving and under reduced motion.
    Decoration, default Rivets {
        /// Rivets in the corners of a panel. Still.
        Rivets => "rivets",
        /// Warning stripes under the navigation, in the stripe colour and
        /// the well. Still.
        HazardStripe => "hazard-stripe",
        /// Specks drifting very slowly down the Now Playing screen. Moves.
        MarineSnow => "marine-snow",
        /// A lure of light hanging from above on its line. Still.
        Lure => "lure",
        /// Striped wallpaper over the window's ground. Still.
        Wallpaper => "wallpaper",
        /// A speckled carpet under the transport. Still.
        Carpet => "carpet",
        /// A sign at the foot of the navigation, in the active colour, with
        /// the theme's `sign` ornament on it. A sign, not a button. Still.
        ExitSign => "exit-sign",
        /// Soft clouds drifting very slowly across the sky. Moves.
        Clouds => "clouds",
        /// A few small sparkles in the sky. Moves, gently.
        Sparkles => "sparkles",
        /// One small eye in the sky, drawn simply and faintly. Still: it
        /// never blinks and never follows the pointer.
        SkyEye => "sky-eye",
        /// Fine horizontal lines over the window's ground, like brushed
        /// metal. Still.
        BrushedLines => "brushed-lines",
        /// The application's name on a holographic sticker. Still.
        Hologram => "hologram",
    }
);

theme_enum!(
    /// Whether the navigation has a column of its own.
    SidebarPlace, default Shown {
        /// Beside the content, as wide as the window allows.
        Shown => "shown",
        /// Behind a button; it opens over the content.
        Hidden => "hidden",
    }
);

theme_enum!(
    /// Where the queue goes.
    QueuePlace, default Docked {
        /// In a column of its own when the window is wide enough.
        Docked => "docked",
        /// Always over the content, opened by a button.
        Floating => "floating",
    }
);

theme_enum!(
    /// How much of the transport is drawn.
    TransportStyle, default Full {
        /// Everything: cover, title, controls, position, volume, meters.
        Full => "full",
        /// One slim bar: cover, title, controls, position, volume.
        Bar => "bar",
        /// Only what playing needs: the controls and the position.
        Minimal => "minimal",
    }
);

theme_enum!(
    /// What the library opens on.
    LibraryHome, default Tracks {
        /// The list of songs.
        Tracks => "tracks",
        /// The grid of album covers.
        Albums => "albums",
    }
);

theme_enum!(
    /// How the Now Playing screen is arranged.
    NowPlayingStyle, default Classic {
        /// The panels side by side, as Onsa has always drawn them.
        Classic => "classic",
        /// A large cover and the words, and little else.
        Immersive => "immersive",
        /// Cover in a frame beside the title and words, with the position
        /// and the spectrum as instruments underneath.
        Instrument => "instrument",
        /// Cover and a large title side by side, the words under them, the
        /// spectrum along the bottom, and the position down the right.
        Dive => "dive",
        /// The cover beside one card holding the title and the words, and
        /// the spectrum on a card of its own under them.
        Memo => "memo",
        /// The title and the words floating in the sky, the cover standing
        /// on the ground to the right, the spectrum along the ground.
        Meadow => "meadow",
        /// A large disc to one side, a display with the track and the time
        /// beside it, the title and the words under the display, and the
        /// spectrum on a wide display along the bottom.
        Pocket => "pocket",
    }
);

theme_enum!(
    /// How the cover is framed on the Now Playing screen.
    CoverFrame, default Plain {
        /// Just the picture.
        Plain => "plain",
        /// In a thick instrument bezel, with a crosshair over it.
        Instrument => "instrument",
        /// Lit from one corner, darkening towards the far one — never by
        /// more than half, so the picture stays clear.
        Lit => "lit",
        /// A photograph stuck on: a paper border and a hard shadow.
        Photo => "photo",
        /// Seen through a door standing open on its own, with its shadow.
        Door => "door",
        /// The label of a disc in a chromed ring, with a hole in the middle
        /// and a rainbow on its face. It turns slowly while the song plays.
        Disc => "disc",
    }
);

theme_enum!(
    /// How the play position is shown on the Now Playing screen.
    PositionStyle, default Bar {
        /// A horizontal bar.
        Bar => "bar",
        /// A round dial with a hand that goes round once per song.
        Dial => "dial",
        /// A depth scale down the side: the start of the song at the top,
        /// its end at the bottom. It seeks, the way a bar does.
        Depth => "depth",
    }
);

theme_enum!(
    /// Whether the window is drawn as a window inside the window.
    Chrome, default None {
        /// No; the operating system's frame is the only frame.
        None => "none",
        /// Yes: a title bar, a menu bar, a task panel and a task bar.
        Window => "window",
        /// A ceiling of lamps above the window, and the play time stamped in
        /// the top corners the way a camcorder stamps it. One lamp may dim
        /// slowly now and then; it stops in power saving and under reduced
        /// motion.
        Corridor => "corridor",
        /// The window as a landscape: ground along the bottom of the
        /// content, and the navigation as boxes in the sky.
        Meadow => "meadow",
    }
);

theme_enum!(
    /// How the position bar of the transport is drawn.
    ProgressStyle, default Plain {
        /// The system's slider in the position colour.
        Plain => "plain",
        /// Striped, with a small star marking where the song is.
        Stars => "stars",
        /// A row of blocks, the way an old progress bar fills.
        Blocks => "blocks",
    }
);
