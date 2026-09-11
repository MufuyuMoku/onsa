//! ReplayGain (SPEC §4.1).
//!
//! The gain is per track, so it is applied on the engine thread, before the
//! callback-side chain. Tags come from ID3 TXXX, Vorbis comments, MP4 atoms
//! and APE, all mapped by symphonia to the same standard tags.

/// Which ReplayGain value to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReplayGainMode {
    /// No ReplayGain.
    #[default]
    Off,
    /// Per-track gain.
    Track,
    /// Per-album gain (falls back to the track gain when missing).
    Album,
    /// Album gain when the queue is one album in order, otherwise track gain.
    Auto,
}

/// ReplayGain behaviour.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReplayGainSettings {
    /// Which value to use.
    pub mode: ReplayGainMode,
    /// Added to the tagged gain, in dB.
    pub preamp_db: f32,
    /// Gain for tracks without tags, in dB.
    pub fallback_db: f32,
    /// Lower the gain so the tagged peak cannot clip.
    pub prevent_clipping: bool,
}

impl Default for ReplayGainSettings {
    fn default() -> Self {
        Self {
            mode: ReplayGainMode::Off,
            preamp_db: 0.0,
            fallback_db: 0.0,
            prevent_clipping: true,
        }
    }
}

/// The ReplayGain tags of one track.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ReplayGainTags {
    /// Track gain in dB.
    pub track_gain_db: Option<f32>,
    /// Track peak, linear (1.0 is full scale).
    pub track_peak: Option<f32>,
    /// Album gain in dB.
    pub album_gain_db: Option<f32>,
    /// Album peak, linear.
    pub album_peak: Option<f32>,
}

impl ReplayGainTags {
    /// Whether any gain is tagged.
    pub fn has_gain(&self) -> bool {
        self.track_gain_db.is_some() || self.album_gain_db.is_some()
    }
}

/// Reads a gain tag such as `-6.54 dB`, `+1.2 dB` or `-6.54`.
pub fn parse_gain(text: &str) -> Option<f32> {
    let number = text
        .trim()
        .trim_end_matches(|c: char| c.is_ascii_alphabetic() || c.is_whitespace())
        .trim()
        .trim_start_matches('+')
        .replace(',', ".");
    number
        .parse::<f32>()
        .ok()
        .filter(|db| db.is_finite() && db.abs() < 60.0)
}

/// Reads a peak tag such as `0.988251`.
pub fn parse_peak(text: &str) -> Option<f32> {
    text.trim()
        .replace(',', ".")
        .parse::<f32>()
        .ok()
        .filter(|peak| peak.is_finite() && *peak > 0.0 && *peak < 100.0)
}

impl ReplayGainSettings {
    /// Linear gain for a track. `album` says whether album gain applies
    /// (the caller resolves [`ReplayGainMode::Auto`]).
    pub fn gain(&self, tags: &ReplayGainTags, album: bool) -> f32 {
        if self.mode == ReplayGainMode::Off {
            return 1.0;
        }
        let (gain, peak) = if album {
            (
                tags.album_gain_db.or(tags.track_gain_db),
                tags.album_peak.or(tags.track_peak),
            )
        } else {
            (
                tags.track_gain_db.or(tags.album_gain_db),
                tags.track_peak.or(tags.album_peak),
            )
        };
        let db = match gain {
            Some(gain) => gain + self.preamp_db,
            None => self.fallback_db,
        };
        let mut linear = super::db_to_gain(db);
        if self.prevent_clipping && gain.is_some() {
            if let Some(peak) = peak {
                linear = linear.min(1.0 / peak);
            }
        }
        linear
    }

    /// Whether album gain applies, given whether the queue is one album in
    /// order.
    pub fn uses_album(&self, queue_is_one_album: bool) -> bool {
        match self.mode {
            ReplayGainMode::Album => true,
            ReplayGainMode::Auto => queue_is_one_album,
            ReplayGainMode::Off | ReplayGainMode::Track => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    #[test]
    fn reads_the_usual_spellings() {
        assert_eq!(parse_gain("-6.54 dB"), Some(-6.54));
        assert_eq!(parse_gain("+1.20 dB"), Some(1.2));
        assert_eq!(parse_gain(" -3 "), Some(-3.0));
        assert_eq!(parse_gain("-7,5 dB"), Some(-7.5));
        assert_eq!(parse_gain("loud"), None);
        assert_eq!(parse_peak("0.988251"), Some(0.988251));
        assert_eq!(parse_peak("0"), None);
    }

    #[test]
    fn picks_track_or_album_and_adds_the_preamp() {
        let tags = ReplayGainTags {
            track_gain_db: Some(-6.0),
            track_peak: Some(0.5),
            album_gain_db: Some(-8.0),
            album_peak: Some(0.5),
        };
        let mut settings = ReplayGainSettings {
            mode: ReplayGainMode::Track,
            preamp_db: 2.0,
            ..ReplayGainSettings::default()
        };
        assert!(close(
            settings.gain(&tags, false),
            crate::dsp::db_to_gain(-4.0)
        ));
        assert!(close(
            settings.gain(&tags, true),
            crate::dsp::db_to_gain(-6.0)
        ));
        settings.mode = ReplayGainMode::Off;
        assert_eq!(settings.gain(&tags, false), 1.0);
    }

    #[test]
    fn untagged_tracks_get_the_fallback() {
        let settings = ReplayGainSettings {
            mode: ReplayGainMode::Track,
            preamp_db: 5.0,
            fallback_db: -3.0,
            prevent_clipping: true,
        };
        assert!(close(
            settings.gain(&ReplayGainTags::default(), false),
            crate::dsp::db_to_gain(-3.0)
        ));
    }

    #[test]
    fn the_peak_prevents_clipping() {
        let tags = ReplayGainTags {
            track_gain_db: Some(6.0),
            track_peak: Some(0.8),
            ..ReplayGainTags::default()
        };
        let mut settings = ReplayGainSettings {
            mode: ReplayGainMode::Track,
            ..ReplayGainSettings::default()
        };
        assert!(close(settings.gain(&tags, false), 1.0 / 0.8));
        settings.prevent_clipping = false;
        assert!(close(
            settings.gain(&tags, false),
            crate::dsp::db_to_gain(6.0)
        ));
    }

    #[test]
    fn auto_follows_the_queue() {
        let settings = ReplayGainSettings {
            mode: ReplayGainMode::Auto,
            ..ReplayGainSettings::default()
        };
        assert!(settings.uses_album(true));
        assert!(!settings.uses_album(false));
    }
}
