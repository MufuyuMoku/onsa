//! User settings: their stored form, their defaults, and how they map onto
//! the engine's own types.
//!
//! Settings live as JSON in the library database (`settings` table), one key
//! per group. A value that cannot be read falls back to its default with a
//! warning in the log, so a damaged setting never stops the application.
//! Every value is clamped to a sane range on its way to the engine.

use std::collections::BTreeMap;
use std::time::Duration;

use onsa_audio::dsp::eq::MAX_BANDS;

use onsa_audio::{
    Band, BufferSize, CrossfadeCurve, DeviceChoice, DspSettings, EqMode, FilterKind, OutputRate,
    OutputSettings, PlaybackSettings, RepeatMode, ReplayGainMode, ReplayGainSettings,
    ResamplerQuality, DEFAULT_POSITION_INTERVAL,
};
use onsa_library::Library;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// Output device and rate.
pub const OUTPUT_KEY: &str = "output";
/// Crossfade, resampler and buffer.
pub const PLAYBACK_KEY: &str = "playback";
/// The DSP chain, volume included.
pub const DSP_KEY: &str = "dsp";
/// The chosen theme.
pub const THEME_KEY: &str = "theme";
/// The chosen interface language.
pub const LOCALE_KEY: &str = "locale";
/// Whether debug logging is on.
pub const LOG_DEBUG_KEY: &str = "logDebug";
/// Whether closing the window leaves Onsa in the tray.
pub const CLOSE_TO_TRAY_KEY: &str = "closeToTray";
/// What the interface shows beyond the theme.
pub const DISPLAY_KEY: &str = "display";
/// Key of the metadata settings.
pub const METADATA_KEY: &str = "metadata";

/// Reads one setting, or its default.
pub fn load<T: DeserializeOwned + Default>(library: &Library, key: &str) -> T {
    match library.setting(key) {
        Ok(Some(json)) => serde_json::from_str(&json).unwrap_or_else(|error| {
            tracing::warn!(key, "setting cannot be read, using the default: {error}");
            T::default()
        }),
        Ok(None) => T::default(),
        Err(error) => {
            tracing::warn!(key, "setting cannot be read, using the default: {error}");
            T::default()
        }
    }
}

/// Stores one setting.
pub fn save<T: Serialize>(library: &Library, key: &str, value: &T) -> onsa_library::Result<()> {
    match serde_json::to_string(value) {
        Ok(json) => library.set_setting(key, &json),
        Err(error) => {
            // Plain data always serialises; this is only a safety net.
            tracing::error!(key, "setting cannot be stored: {error}");
            Ok(())
        }
    }
}

/// Resampler preset (SPEC §3.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Quality {
    /// Cheapest.
    Fast,
    /// The default.
    #[default]
    Balanced,
    /// Highest quality.
    Best,
}

/// Buffer between engine and device (SPEC §3.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BufferChoice {
    /// About 50 ms.
    Low,
    /// About 150 ms.
    #[default]
    Normal,
    /// About 500 ms.
    Large,
}

/// Crossfade curve (SPEC §3.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Curve {
    /// Constant loudness through the fade.
    #[default]
    EqualPower,
    /// Straight lines.
    Linear,
}

/// What happens when a track ends (SPEC §6.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RepeatKind {
    /// The queue plays once and stops.
    #[default]
    Off,
    /// The queue starts over.
    All,
    /// The current track repeats.
    One,
}

impl From<RepeatKind> for RepeatMode {
    fn from(kind: RepeatKind) -> Self {
        match kind {
            RepeatKind::Off => Self::Off,
            RepeatKind::All => Self::All,
            RepeatKind::One => Self::One,
        }
    }
}

/// ReplayGain mode (SPEC §4.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RgMode {
    /// No ReplayGain.
    #[default]
    Off,
    /// Track gain.
    Track,
    /// Album gain.
    Album,
    /// Album gain for a whole album in order, otherwise track gain.
    Auto,
}

/// EQ filter shape (SPEC §4.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FilterType {
    /// Bell.
    Peaking,
    /// Low shelf.
    LowShelf,
    /// High shelf.
    HighShelf,
    /// Low pass.
    LowPass,
    /// High pass.
    HighPass,
    /// Notch.
    Notch,
}

/// Which EQ editor is in use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EqKind {
    /// Ten fixed bands.
    #[default]
    Graphic,
    /// Free bands.
    Parametric,
}

impl From<FilterType> for FilterKind {
    fn from(kind: FilterType) -> Self {
        match kind {
            FilterType::Peaking => Self::Peaking,
            FilterType::LowShelf => Self::LowShelf,
            FilterType::HighShelf => Self::HighShelf,
            FilterType::LowPass => Self::LowPass,
            FilterType::HighPass => Self::HighPass,
            FilterType::Notch => Self::Notch,
        }
    }
}

impl From<FilterKind> for FilterType {
    fn from(kind: FilterKind) -> Self {
        match kind {
            FilterKind::Peaking => Self::Peaking,
            FilterKind::LowShelf => Self::LowShelf,
            FilterKind::HighShelf => Self::HighShelf,
            FilterKind::LowPass => Self::LowPass,
            FilterKind::HighPass => Self::HighPass,
            FilterKind::Notch => Self::Notch,
        }
    }
}

/// Where audio goes.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct OutputPrefs {
    /// Device identifier; `None` follows the system default.
    pub device_id: Option<String>,
    /// Fixed sample rate; `None` follows the device.
    pub sample_rate: Option<u32>,
    /// Follow each track's own rate instead, when the device supports it.
    pub match_source: bool,
}

impl OutputPrefs {
    /// The engine's form.
    pub fn engine(&self) -> OutputSettings {
        OutputSettings {
            device: match &self.device_id {
                Some(id) if !id.is_empty() => DeviceChoice::Id(id.clone()),
                _ => DeviceChoice::SystemDefault,
            },
            rate: match (self.match_source, self.sample_rate) {
                (true, _) => OutputRate::MatchSource,
                (false, Some(rate)) if (8_000..=768_000).contains(&rate) => OutputRate::Fixed(rate),
                _ => OutputRate::FollowDevice,
            },
        }
    }
}

/// Transitions and quality.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PlaybackPrefs {
    /// Crossfade between tracks, 0 to 12 seconds.
    pub crossfade_seconds: f32,
    /// Crossfade curve.
    pub curve: Curve,
    /// Crossfade when skipping.
    pub skip_crossfade_seconds: f32,
    /// No crossfade between consecutive tracks of one album.
    pub album_gapless: bool,
    /// Resampler preset.
    pub quality: Quality,
    /// Buffer size.
    pub buffer: BufferChoice,
    /// What happens when a track ends.
    pub repeat: RepeatKind,
    /// Power saving (SPEC §3.4): the largest buffer, the cheapest
    /// resampler, fewer analysis frames and fewer position events.
    pub power_save: bool,
}

impl Default for PlaybackPrefs {
    fn default() -> Self {
        let engine = PlaybackSettings::default();
        Self {
            crossfade_seconds: engine.crossfade_seconds,
            curve: Curve::EqualPower,
            skip_crossfade_seconds: engine.skip_crossfade_seconds,
            album_gapless: engine.album_gapless,
            quality: Quality::Balanced,
            buffer: BufferChoice::Normal,
            repeat: RepeatKind::Off,
            power_save: false,
        }
    }
}

/// Position events while saving power (SPEC §3.4: "fewer position events").
const SAVING_POSITION_INTERVAL: Duration = Duration::from_millis(500);
/// Analysis frames a second while saving power (SPEC §4.4).
pub const SAVING_FPS: u32 = 20;

impl PlaybackPrefs {
    /// The engine's form. Power saving overrides the buffer, the resampler
    /// and the rate of position events; the values the listener chose are
    /// left as they are, and come back when they switch it off.
    pub fn engine(&self) -> PlaybackSettings {
        PlaybackSettings {
            crossfade_seconds: finite(self.crossfade_seconds, 0.0).clamp(0.0, 12.0),
            curve: match self.curve {
                Curve::EqualPower => CrossfadeCurve::EqualPower,
                Curve::Linear => CrossfadeCurve::Linear,
            },
            skip_crossfade_seconds: finite(self.skip_crossfade_seconds, 0.3).clamp(0.0, 2.0),
            album_gapless: self.album_gapless,
            quality: match (self.power_save, self.quality) {
                (true, _) | (_, Quality::Fast) => ResamplerQuality::Fast,
                (_, Quality::Balanced) => ResamplerQuality::Balanced,
                (_, Quality::Best) => ResamplerQuality::Best,
            },
            buffer: match (self.power_save, self.buffer) {
                (true, _) | (_, BufferChoice::Large) => BufferSize::Large,
                (_, BufferChoice::Low) => BufferSize::Low,
                (_, BufferChoice::Normal) => BufferSize::Normal,
            },
            repeat: self.repeat.into(),
            position_interval: if self.power_save {
                SAVING_POSITION_INTERVAL
            } else {
                DEFAULT_POSITION_INTERVAL
            },
        }
    }
}

/// How far the colour follows the sound (SPEC §9.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ToneStrength {
    /// The theme's own colours, unmoved.
    Off,
    /// Only noticeable once you look for it.
    Subtle,
    /// The default: plain to see, still calm.
    #[default]
    Medium,
    /// As far as the theme's two colours go.
    Strong,
}

/// How one track list is laid out: how wide its columns are and which of
/// them the listener kept (SPEC §9.2).
///
/// Widths are in CSS pixels. A column the listener never touched is simply
/// absent, so a later change to a default reaches everyone who never had an
/// opinion about it.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ColumnPrefs {
    /// Width per column key, for the ones the listener dragged.
    pub widths: BTreeMap<String, f64>,
    /// Columns the listener took off this list.
    pub hidden: Vec<String>,
}

/// What Onsa may do over the internet for metadata, and the key it uses
/// (SPEC §8, §14).
///
/// Nothing here reaches the internet unless `online` is on: that is the
/// listener's consent, and it starts off.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MetadataPrefs {
    /// Whether Onsa may ask the internet about metadata at all.
    pub online: bool,
    /// The listener's own AcoustID application key, when they have one.
    /// Never sent back to the interface, never logged.
    pub acoustid_key: Option<String>,
}

/// What the interface shows beyond the theme itself.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DisplayPrefs {
    /// How far the colour follows the character of the sound. Power saving
    /// stops it whatever this says.
    pub tone_color: ToneStrength,
    /// Column widths and choices, one entry per list the listener has
    /// arranged. Lists they never touched are not in here at all.
    pub columns: BTreeMap<String, ColumnPrefs>,
}

/// One parametric band.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BandPrefs {
    /// Filter shape.
    pub kind: FilterType,
    /// Frequency in Hz.
    pub freq: f64,
    /// Gain in dB.
    pub gain_db: f64,
    /// Quality factor.
    pub q: f64,
    /// A disabled band passes the signal untouched.
    #[serde(default = "enabled")]
    pub enabled: bool,
}

fn enabled() -> bool {
    true
}

impl BandPrefs {
    /// The engine's form, clamped to what a biquad can take.
    pub fn engine(&self) -> Band {
        Band {
            kind: self.kind.into(),
            freq: finite(self.freq, 1000.0).clamp(10.0, 22_000.0),
            gain_db: finite(self.gain_db, 0.0).clamp(-24.0, 24.0),
            q: finite(self.q, 0.707).clamp(0.05, 20.0),
            enabled: self.enabled,
        }
    }

    /// From the engine's form.
    pub fn from_engine(band: &Band) -> Self {
        Self {
            kind: band.kind.into(),
            freq: band.freq,
            gain_db: band.gain_db,
            q: band.q,
            enabled: band.enabled,
        }
    }
}

/// The DSP chain as the user sets it (SPEC §4).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DspPrefs {
    /// ReplayGain mode.
    pub replaygain_mode: RgMode,
    /// ReplayGain preamp in dB.
    pub replaygain_preamp_db: f32,
    /// Gain for tracks without ReplayGain tags, in dB.
    pub replaygain_fallback_db: f32,
    /// Keep ReplayGain from pushing a tagged peak past full scale.
    pub replaygain_prevent_clipping: bool,
    /// Whether the EQ runs.
    pub eq_enabled: bool,
    /// Graphic or parametric.
    pub eq_kind: EqKind,
    /// Gains of the ten graphic bands, in dB.
    pub graphic_gains: [f64; 10],
    /// Parametric bands, at most sixteen.
    pub parametric: Vec<BandPrefs>,
    /// Manual preamp in dB.
    pub preamp_db: f32,
    /// Preamp from the EQ curve.
    pub auto_preamp: bool,
    /// Whether the limiter limits.
    pub limiter_enabled: bool,
    /// Limiter release in milliseconds.
    pub limiter_release_ms: f32,
    /// Volume in dB (0 is unity).
    pub volume_db: f32,
    /// TPDF dither for 16-bit output.
    pub dither: bool,
}

impl Default for DspPrefs {
    fn default() -> Self {
        let engine = DspSettings::default();
        Self {
            replaygain_mode: RgMode::Off,
            replaygain_preamp_db: engine.replaygain.preamp_db,
            replaygain_fallback_db: engine.replaygain.fallback_db,
            replaygain_prevent_clipping: engine.replaygain.prevent_clipping,
            eq_enabled: engine.eq_enabled,
            eq_kind: EqKind::Graphic,
            graphic_gains: [0.0; 10],
            parametric: Vec::new(),
            preamp_db: engine.preamp_db,
            auto_preamp: engine.auto_preamp,
            limiter_enabled: engine.limiter_enabled,
            limiter_release_ms: engine.limiter_release_ms,
            volume_db: engine.volume_db,
            dither: engine.dither,
        }
    }
}

impl DspPrefs {
    /// The engine's form.
    pub fn engine(&self) -> DspSettings {
        let eq = match self.eq_kind {
            EqKind::Graphic => EqMode::Graphic(
                self.graphic_gains
                    .map(|gain| finite(gain, 0.0).clamp(-24.0, 24.0)),
            ),
            EqKind::Parametric => EqMode::Parametric(
                self.parametric
                    .iter()
                    .take(MAX_BANDS)
                    .map(BandPrefs::engine)
                    .collect(),
            ),
        };
        DspSettings {
            replaygain: ReplayGainSettings {
                mode: match self.replaygain_mode {
                    RgMode::Off => ReplayGainMode::Off,
                    RgMode::Track => ReplayGainMode::Track,
                    RgMode::Album => ReplayGainMode::Album,
                    RgMode::Auto => ReplayGainMode::Auto,
                },
                preamp_db: finite(self.replaygain_preamp_db, 0.0).clamp(-15.0, 15.0),
                fallback_db: finite(self.replaygain_fallback_db, 0.0).clamp(-15.0, 15.0),
                prevent_clipping: self.replaygain_prevent_clipping,
            },
            eq_enabled: self.eq_enabled,
            eq,
            preamp_db: finite(self.preamp_db, 0.0).clamp(-30.0, 30.0),
            auto_preamp: self.auto_preamp,
            limiter_enabled: self.limiter_enabled,
            limiter_release_ms: finite(self.limiter_release_ms, 80.0).clamp(1.0, 2000.0),
            volume_db: finite(self.volume_db, 0.0).clamp(-90.0, 12.0),
            dither: self.dither,
            ..DspSettings::default()
        }
    }
}

/// `value`, or `fallback` when it is NaN or infinite.
fn finite<T: Into<f64> + Copy>(value: T, fallback: T) -> T {
    if value.into().is_finite() {
        value
    } else {
        fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_saving_overrides_the_buffer_the_resampler_and_the_position_events() {
        let chosen = PlaybackPrefs {
            quality: Quality::Best,
            buffer: BufferChoice::Low,
            ..PlaybackPrefs::default()
        };
        let normal = chosen.engine();
        assert_eq!(normal.quality, ResamplerQuality::Best);
        assert_eq!(normal.buffer, BufferSize::Low);
        assert_eq!(normal.position_interval, DEFAULT_POSITION_INTERVAL);

        let saving = PlaybackPrefs {
            power_save: true,
            ..chosen.clone()
        }
        .engine();
        assert_eq!(saving.quality, ResamplerQuality::Fast);
        assert_eq!(saving.buffer, BufferSize::Large);
        assert_eq!(saving.position_interval, SAVING_POSITION_INTERVAL);

        // What the listener chose is kept, and comes back when they stop
        // saving power.
        assert_eq!(chosen.engine(), normal);
    }

    #[test]
    fn a_damaged_or_partial_setting_falls_back_field_by_field() {
        let dsp: DspPrefs = serde_json::from_str(r#"{"volumeDb": -12.5, "unknown": 1}"#).unwrap();
        assert_eq!(dsp.volume_db, -12.5);
        assert!(dsp.limiter_enabled, "missing fields keep their defaults");
        let playback: PlaybackPrefs = serde_json::from_str("{}").unwrap();
        assert_eq!(playback, PlaybackPrefs::default());
    }

    #[test]
    fn column_widths_survive_the_trip_through_json() {
        let stored = r#"{
            "toneColor": "strong",
            "columns": {
                "tracks": { "widths": { "title": 320.5, "artist": 180.0 }, "hidden": ["year"] }
            }
        }"#;
        let display: DisplayPrefs = serde_json::from_str(stored).unwrap();
        assert_eq!(display.tone_color, ToneStrength::Strong);
        let tracks = display.columns.get("tracks").expect("the list was stored");
        assert_eq!(tracks.widths.get("title"), Some(&320.5));
        assert_eq!(tracks.hidden, vec!["year".to_string()]);
        assert!(
            !display.columns.contains_key("album"),
            "untouched lists stay out"
        );

        let again: DisplayPrefs =
            serde_json::from_str(&serde_json::to_string(&display).unwrap()).unwrap();
        assert_eq!(again, display, "what is written reads back the same");
    }

    #[test]
    fn a_display_setting_from_before_columns_still_reads() {
        // What an older Onsa wrote: the tone colour and nothing else.
        let display: DisplayPrefs = serde_json::from_str(r#"{"toneColor":"subtle"}"#).unwrap();
        assert_eq!(display.tone_color, ToneStrength::Subtle);
        assert!(display.columns.is_empty(), "every list keeps its defaults");
    }

    #[test]
    fn values_are_clamped_on_the_way_to_the_engine() {
        let playback = PlaybackPrefs {
            crossfade_seconds: 60.0,
            ..PlaybackPrefs::default()
        };
        assert_eq!(playback.engine().crossfade_seconds, 12.0);
        let band = BandPrefs {
            kind: FilterType::Peaking,
            freq: f64::NAN,
            gain_db: 99.0,
            q: 0.0,
            enabled: true,
        }
        .engine();
        assert_eq!((band.freq, band.gain_db, band.q), (1000.0, 24.0, 0.05));
        let output = OutputPrefs {
            device_id: Some(String::new()),
            sample_rate: Some(1),
            match_source: false,
        };
        assert_eq!(output.engine(), OutputSettings::default());
    }

    #[test]
    fn both_eq_editors_keep_their_values() {
        let mut dsp = DspPrefs {
            eq_enabled: true,
            graphic_gains: [3.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -2.0],
            parametric: vec![BandPrefs {
                kind: FilterType::HighShelf,
                freq: 8000.0,
                gain_db: 4.0,
                q: 0.7,
                enabled: true,
            }],
            ..DspPrefs::default()
        };
        assert_eq!(dsp.engine().active_bands().len(), 10);
        dsp.eq_kind = EqKind::Parametric;
        assert_eq!(dsp.engine().active_bands().len(), 1);
        dsp.eq_kind = EqKind::Graphic;
        assert_eq!(dsp.engine().active_bands()[0].gain_db, 3.0);
    }
}
