//! Encoder delay and padding for MP4 audio (AAC in `.m4a`, `.mp4`, `.m4b`).
//!
//! AAC encoders put silent priming frames in front of the audio and pad the
//! end to a whole frame. MP4 files record how much through an edit list
//! (`elst`) or, in files from iTunes and friends, the `iTunSMPB` tag.
//! symphonia 0.6 reads neither, so without this module every M4A track would
//! start with about 23 ms of priming and a gapless album would stutter at each
//! join (SPEC §3.3).
//!
//! Only the handful of boxes needed are read: `moov/mvhd`, and for the first
//! audio track `mdia/hdlr`, `mdia/mdhd` and `edts/elst`.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// Largest `moov` box read into memory. Real files stay far below this.
const MAX_MOOV_BYTES: u64 = 64 * 1024 * 1024;

/// What to cut from an MP4 audio track, in frames at the track's rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mp4Trim {
    /// Priming frames to drop at the start.
    pub delay: u64,
    /// Playable frames after the delay, when known.
    pub playable: Option<u64>,
}

/// Reads the edit list of the first audio track of an MP4 file.
///
/// Returns `None` when the file has no edit list, no audio track, or nothing
/// to trim.
pub fn read_edit_list(path: &Path, sample_rate: u32) -> Option<Mp4Trim> {
    let moov = read_moov(path)?;
    trim_from_moov(&moov, sample_rate)
}

/// Parses an `iTunSMPB` value: `" 00000000 00000840 000001CA 00000000003F31F6 ..."`
/// holds the delay, the padding and the playable frame count in hex.
pub fn parse_itunsmpb(text: &str) -> Option<Mp4Trim> {
    let fields: Vec<&str> = text.split_whitespace().collect();
    let hex = |index: usize| {
        fields
            .get(index)
            .and_then(|f| u64::from_str_radix(f, 16).ok())
    };
    let delay = hex(1)?;
    let playable = hex(3).filter(|frames| *frames > 0);
    if delay == 0 && playable.is_none() {
        return None;
    }
    Some(Mp4Trim { delay, playable })
}

/// Finds the top-level `moov` box and returns its payload.
fn read_moov(path: &Path) -> Option<Vec<u8>> {
    let mut file = File::open(path).ok()?;
    let length = file.metadata().ok()?.len();
    let mut offset = 0u64;
    while offset + 8 <= length {
        file.seek(SeekFrom::Start(offset)).ok()?;
        let mut header = [0u8; 16];
        file.read_exact(&mut header[..8]).ok()?;
        let size32 = u32::from_be_bytes([header[0], header[1], header[2], header[3]]);
        let kind = [header[4], header[5], header[6], header[7]];
        let (size, header_len) = match size32 {
            0 => (length - offset, 8),
            1 => {
                file.read_exact(&mut header[8..16]).ok()?;
                (u64::from_be_bytes(header[8..16].try_into().ok()?), 16)
            }
            n => (u64::from(n), 8),
        };
        if size < header_len {
            return None;
        }
        if &kind == b"moov" {
            let payload = size - header_len;
            if payload > MAX_MOOV_BYTES {
                return None;
            }
            let mut data = vec![0u8; usize::try_from(payload).ok()?];
            file.read_exact(&mut data).ok()?;
            return Some(data);
        }
        offset = offset.checked_add(size)?;
    }
    None
}

/// Iterates the child boxes of a payload as `(type, payload)`.
fn boxes(data: &[u8]) -> impl Iterator<Item = (&[u8], &[u8])> {
    let mut offset = 0usize;
    std::iter::from_fn(move || {
        if offset + 8 > data.len() {
            return None;
        }
        let size32 = u32::from_be_bytes(data[offset..offset + 4].try_into().ok()?) as usize;
        let kind = &data[offset + 4..offset + 8];
        let (size, header) = match size32 {
            0 => (data.len() - offset, 8),
            1 => {
                let wide = data.get(offset + 8..offset + 16)?;
                (
                    usize::try_from(u64::from_be_bytes(wide.try_into().ok()?)).ok()?,
                    16,
                )
            }
            n => (n, 8),
        };
        if size < header || offset + size > data.len() {
            return None;
        }
        let payload = &data[offset + header..offset + size];
        offset += size;
        Some((kind, payload))
    })
}

fn child<'a>(data: &'a [u8], kind: &[u8]) -> Option<&'a [u8]> {
    boxes(data)
        .find(|(k, _)| *k == kind)
        .map(|(_, payload)| payload)
}

fn be_u32(data: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_be_bytes(data.get(at..at + 4)?.try_into().ok()?))
}

fn be_u64(data: &[u8], at: usize) -> Option<u64> {
    Some(u64::from_be_bytes(data.get(at..at + 8)?.try_into().ok()?))
}

/// `(timescale, duration)` of an `mvhd` or `mdhd` payload.
fn header_timing(payload: &[u8]) -> Option<(u32, u64)> {
    match payload.first()? {
        0 => Some((be_u32(payload, 12)?, u64::from(be_u32(payload, 16)?))),
        1 => Some((be_u32(payload, 20)?, be_u64(payload, 24)?)),
        _ => None,
    }
}

/// First non-empty edit: `(segment_duration, media_time)`.
fn first_edit(elst: &[u8]) -> Option<(u64, u64)> {
    let version = *elst.first()?;
    let count = be_u32(elst, 4)? as usize;
    let entry_len = if version == 1 { 20 } else { 12 };
    for index in 0..count {
        let at = 8 + index * entry_len;
        let (duration, media_time) = if version == 1 {
            (be_u64(elst, at)?, be_u64(elst, at + 8)? as i64)
        } else {
            (
                u64::from(be_u32(elst, at)?),
                i64::from(be_u32(elst, at + 4)? as i32),
            )
        };
        // media_time -1 marks an empty edit (a pause before the media).
        if media_time >= 0 {
            return Some((duration, media_time as u64));
        }
    }
    None
}

fn to_frames(value: u64, timescale: u32, sample_rate: u32) -> u64 {
    if timescale == 0 {
        return 0;
    }
    let scaled = u128::from(value) * u128::from(sample_rate);
    ((scaled + u128::from(timescale) / 2) / u128::from(timescale)) as u64
}

fn trim_from_moov(moov: &[u8], sample_rate: u32) -> Option<Mp4Trim> {
    let (movie_timescale, _) = header_timing(child(moov, b"mvhd")?)?;
    for (kind, trak) in boxes(moov) {
        if kind != b"trak" {
            continue;
        }
        let Some(mdia) = child(trak, b"mdia") else {
            continue;
        };
        let is_audio = child(mdia, b"hdlr")
            .and_then(|hdlr| hdlr.get(8..12))
            .is_some_and(|handler| handler == b"soun");
        if !is_audio {
            continue;
        }
        let (media_timescale, media_duration) = header_timing(child(mdia, b"mdhd")?)?;
        let (segment, media_time) = first_edit(child(child(trak, b"edts")?, b"elst")?)?;

        let delay = to_frames(media_time, media_timescale, sample_rate);
        let segment = to_frames(segment, movie_timescale, sample_rate);
        let after_delay =
            to_frames(media_duration, media_timescale, sample_rate).saturating_sub(delay);
        // The edit's own length is exact when the movie timescale is at least
        // as fine as the sample rate (iTunes writes it that way). ffmpeg uses
        // milliseconds there, so fall back to the media length after the
        // delay, which ffmpeg writes exactly, never past what the edit allows.
        let playable = if movie_timescale >= sample_rate {
            segment
        } else {
            let slack = u64::from(sample_rate.div_ceil(movie_timescale.max(1)));
            after_delay.min(segment + slack)
        };
        if delay == 0 && playable >= after_delay {
            return None;
        }
        return Some(Mp4Trim {
            delay,
            playable: Some(playable),
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn boxed(kind: &[u8], payload: &[u8]) -> Vec<u8> {
        let mut out = ((payload.len() + 8) as u32).to_be_bytes().to_vec();
        out.extend_from_slice(kind);
        out.extend_from_slice(payload);
        out
    }

    /// A `moov` payload shaped like ffmpeg's: millisecond movie timescale,
    /// 1024 frames of priming, media length = priming + audio.
    fn moov(movie_timescale: u32, segment: u32, media_time: i32, media_duration: u32) -> Vec<u8> {
        let mut mvhd = vec![0u8; 20];
        mvhd[12..16].copy_from_slice(&movie_timescale.to_be_bytes());
        let mut mdhd = vec![0u8; 20];
        mdhd[12..16].copy_from_slice(&44_100u32.to_be_bytes());
        mdhd[16..20].copy_from_slice(&media_duration.to_be_bytes());
        let mut hdlr = vec![0u8; 12];
        hdlr[8..12].copy_from_slice(b"soun");
        let mut elst = vec![0u8, 0, 0, 0];
        elst.extend_from_slice(&1u32.to_be_bytes());
        elst.extend_from_slice(&segment.to_be_bytes());
        elst.extend_from_slice(&media_time.to_be_bytes());
        elst.extend_from_slice(&[0, 1, 0, 0]);

        let mdia = [boxed(b"hdlr", &hdlr), boxed(b"mdhd", &mdhd)].concat();
        let edts = boxed(b"elst", &elst);
        let trak = [boxed(b"edts", &edts), boxed(b"mdia", &mdia)].concat();
        [boxed(b"mvhd", &mvhd), boxed(b"trak", &trak)].concat()
    }

    #[test]
    fn reads_an_ffmpeg_style_edit_list() {
        // 88 978 frames of audio behind 1024 frames of priming.
        let trim = trim_from_moov(&moov(1000, 2017, 1024, 90_002), 44_100).unwrap();
        assert_eq!(
            trim,
            Mp4Trim {
                delay: 1024,
                playable: Some(88_978)
            }
        );
    }

    #[test]
    fn trusts_a_fine_movie_timescale() {
        // iTunes style: the edit length itself is exact, the media is padded.
        let trim = trim_from_moov(&moov(44_100, 88_978, 2112, 92_160), 44_100).unwrap();
        assert_eq!(
            trim,
            Mp4Trim {
                delay: 2112,
                playable: Some(88_978)
            }
        );
    }

    #[test]
    fn nothing_to_trim_is_none() {
        assert_eq!(
            trim_from_moov(&moov(44_100, 88_978, 0, 88_978), 44_100),
            None
        );
    }

    #[test]
    fn parses_itunsmpb() {
        let value = " 00000000 00000840 000001CA 00000000003F31F6 00000000 00000000";
        assert_eq!(
            parse_itunsmpb(value),
            Some(Mp4Trim {
                delay: 0x840,
                playable: Some(0x3F31F6)
            })
        );
        assert_eq!(parse_itunsmpb("nonsense"), None);
    }
}
