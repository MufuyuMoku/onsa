//! Reading tags and stream properties with lofty (SPEC §5.2).

use std::path::Path;

use lofty::file::FileType;
use lofty::picture::PictureType;
use lofty::prelude::{Accessor, AudioFile, ItemKey, TaggedFileExt};

/// What the library keeps about one file, straight from the file.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TrackMeta {
    /// Length in milliseconds.
    pub duration_ms: Option<i64>,
    /// Container or codec name, for display.
    pub codec: Option<String>,
    /// Sample rate in Hz.
    pub sample_rate: Option<u32>,
    /// Bits per sample, for lossless formats.
    pub bit_depth: Option<u8>,
    /// Channel count.
    pub channels: Option<u8>,
    /// Bitrate in kbit/s.
    pub bitrate: Option<u32>,
    /// Title.
    pub title: Option<String>,
    /// Artist.
    pub artist: Option<String>,
    /// Album.
    pub album: Option<String>,
    /// Album artist.
    pub album_artist: Option<String>,
    /// Track number.
    pub track_number: Option<u32>,
    /// Tracks on the disc.
    pub track_total: Option<u32>,
    /// Disc number.
    pub disc_number: Option<u32>,
    /// Discs in the set.
    pub disc_total: Option<u32>,
    /// Year.
    pub year: Option<i32>,
    /// Genre.
    pub genre: Option<String>,
    /// Composer.
    pub composer: Option<String>,
    /// ReplayGain track gain in dB.
    pub rg_track_gain: Option<f64>,
    /// ReplayGain track peak, linear.
    pub rg_track_peak: Option<f64>,
    /// ReplayGain album gain in dB.
    pub rg_album_gain: Option<f64>,
    /// ReplayGain album peak, linear.
    pub rg_album_peak: Option<f64>,
    /// The embedded cover: the front cover when marked, else the first
    /// picture.
    pub picture: Option<Vec<u8>>,
}

/// Reads one file. The error is a readable reason, kept for the log; the
/// caller records the file as failed.
pub fn read(path: &Path) -> Result<TrackMeta, String> {
    let tagged = lofty::read_from_path(path).map_err(|error| error.to_string())?;
    let properties = tagged.properties();
    let duration = properties.duration().as_millis() as i64;
    let mut meta = TrackMeta {
        duration_ms: (duration > 0).then_some(duration),
        codec: Some(codec_name(tagged.file_type())),
        sample_rate: properties.sample_rate(),
        bit_depth: properties.bit_depth(),
        channels: properties.channels(),
        bitrate: properties
            .audio_bitrate()
            .or_else(|| properties.overall_bitrate()),
        ..TrackMeta::default()
    };

    if let Some(tag) = tagged.primary_tag().or_else(|| tagged.first_tag()) {
        meta.title = clean(tag.title().as_deref());
        meta.artist = clean(tag.artist().as_deref());
        meta.album = clean(tag.album().as_deref());
        meta.genre = clean(tag.genre().as_deref());
        meta.album_artist = clean(tag.get_string(ItemKey::AlbumArtist));
        meta.composer = clean(tag.get_string(ItemKey::Composer));
        meta.track_number = tag.track();
        meta.track_total = tag.track_total();
        meta.disc_number = tag.disk();
        meta.disc_total = tag.disk_total();
        meta.year = tag
            .date()
            .map(|date| i32::from(date.year))
            .or_else(|| tag.get_string(ItemKey::Year).and_then(leading_year));
        meta.rg_track_gain = tag.get_string(ItemKey::ReplayGainTrackGain).and_then(gain);
        meta.rg_track_peak = tag
            .get_string(ItemKey::ReplayGainTrackPeak)
            .and_then(number);
        meta.rg_album_gain = tag.get_string(ItemKey::ReplayGainAlbumGain).and_then(gain);
        meta.rg_album_peak = tag
            .get_string(ItemKey::ReplayGainAlbumPeak)
            .and_then(number);
    }

    // The picture may sit in any of the file's tags.
    meta.picture = tagged
        .tags()
        .iter()
        .flat_map(|tag| tag.pictures())
        .min_by_key(|picture| picture.pic_type() != PictureType::CoverFront)
        .map(|picture| picture.data().to_vec());
    Ok(meta)
}

fn codec_name(file_type: FileType) -> String {
    match file_type {
        FileType::Flac => "FLAC".into(),
        FileType::Mpeg => "MP3".into(),
        FileType::Mp4 => "MP4".into(),
        FileType::Aac => "AAC".into(),
        FileType::Vorbis => "Vorbis".into(),
        FileType::Opus => "Opus".into(),
        FileType::Wav => "WAV".into(),
        FileType::Aiff => "AIFF".into(),
        other => format!("{other:?}"),
    }
}

/// Trimmed text; nothing when empty.
fn clean(text: Option<&str>) -> Option<String> {
    text.map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_owned)
}

/// The year at the start of a date such as `2011-04-03`.
fn leading_year(text: &str) -> Option<i32> {
    text.trim()
        .get(..4)
        .and_then(|year| year.parse().ok())
        .filter(|year| (1000..=9999).contains(year))
}

/// A ReplayGain gain such as `-6.54 dB`.
fn gain(text: &str) -> Option<f64> {
    number(
        text.trim()
            .trim_end_matches(|c: char| c.is_ascii_alphabetic() || c.is_whitespace()),
    )
}

fn number(text: &str) -> Option<f64> {
    text.trim()
        .trim_start_matches('+')
        .replace(',', ".")
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_loose_values() {
        assert_eq!(leading_year("2011-04-03"), Some(2011));
        assert_eq!(leading_year("97"), None);
        assert_eq!(gain("-6.54 dB"), Some(-6.54));
        assert_eq!(gain("+1,5 dB"), Some(1.5));
        assert_eq!(number("0.988"), Some(0.988));
        assert_eq!(clean(Some("  ")), None);
        assert_eq!(clean(Some(" 夜明け ")), Some("夜明け".into()));
    }

    #[test]
    fn a_file_that_is_not_audio_is_an_error() {
        let path = std::env::temp_dir().join(format!("onsa tags {}.flac", std::process::id()));
        std::fs::write(&path, b"definitely not flac").unwrap();
        assert!(read(&path).is_err());
        let _ = std::fs::remove_file(&path);
    }
}
