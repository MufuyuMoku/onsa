//! One audio file, decoded to `f32` interleaved at its own sample rate and
//! mapped to the output channel count (SPEC §3.2).

use std::fs::File;
use std::path::{Path, PathBuf};

use symphonia::core::codecs::audio::{AudioDecoder, AudioDecoderOptions};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo, TrackType};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::{MetadataOptions, StandardTag};
use symphonia::core::units::{Time, TimeBase, Timestamp};

use crate::channels::ChannelMap;
use crate::dsp::replaygain::{parse_gain, parse_peak, ReplayGainTags};
use crate::error::{Error, Result};
use crate::mp4::{self, Mp4Trim};

/// Decode errors tolerated in a row before a file is given up on.
const MAX_CONSECUTIVE_DECODE_ERRORS: u32 = 64;

/// What the engine knows about an opened file.
#[derive(Debug, Clone, PartialEq)]
pub struct TrackInfo {
    /// Sample rate of the file.
    pub sample_rate: u32,
    /// Channel count of the file.
    pub channels: usize,
    /// Playable frames (encoder delay and padding excluded), when known.
    pub total_frames: Option<u64>,
    /// Album tag, when present.
    pub album: Option<String>,
    /// Track number tag, when present.
    pub track_number: Option<u64>,
    /// ReplayGain tags (SPEC §4.1).
    pub replaygain: ReplayGainTags,
}

impl TrackInfo {
    /// Playable length in seconds, when known.
    pub fn duration_seconds(&self) -> Option<f64> {
        self.total_frames
            .map(|frames| frames as f64 / f64::from(self.sample_rate))
    }
}

/// An opened, decodable audio file.
pub struct FileSource {
    path: PathBuf,
    format: Box<dyn FormatReader>,
    decoder: Box<dyn AudioDecoder>,
    track_id: u32,
    time_base: Option<TimeBase>,
    start_ts: Timestamp,
    info: TrackInfo,
    map: ChannelMap,
    out_channels: usize,
    /// Decoded samples at the file's channel count.
    scratch: Vec<f32>,
    /// Decoded samples at the output channel count, not yet handed out.
    pending: Vec<f32>,
    pending_pos: usize,
    /// Frames still to drop after a seek landed before its target.
    skip_frames: u64,
    /// Encoder priming the decoder does not drop by itself (MP4), in frames.
    lead: u64,
    /// Playable frames, when the decoder would otherwise play the padding.
    limit: Option<u64>,
    /// Frames handed out since the start of the track.
    position: u64,
    ended: bool,
}

impl FileSource {
    /// Opens a file and prepares its decoder. `out_channels` is the channel
    /// count every read will produce.
    pub fn open(path: &Path, out_channels: usize) -> Result<Self> {
        let file = File::open(path).map_err(|source| Error::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let stream = MediaSourceStream::new(Box::new(file), Default::default());

        let mut hint = Hint::new();
        if let Some(extension) = path.extension().and_then(|extension| extension.to_str()) {
            hint.with_extension(extension);
        }

        let unsupported = |reason: String| Error::Unsupported {
            path: path.to_path_buf(),
            reason,
        };

        let mut format = symphonia::default::get_probe()
            .probe(
                &hint,
                stream,
                FormatOptions::default(),
                MetadataOptions::default(),
            )
            .map_err(|error| unsupported(error.to_string()))?;

        let track = format
            .default_track(TrackType::Audio)
            .ok_or_else(|| unsupported("no audio track".to_string()))?;
        let params = track
            .codec_params
            .as_ref()
            .and_then(|params| params.audio())
            .ok_or_else(|| unsupported("no audio codec parameters".to_string()))?
            .clone();
        let track_id = track.id;
        let time_base = track.time_base;
        let start_ts = track.start_ts;
        let mut total_frames = track.num_frames;
        let decoder_trims = track.delay.unwrap_or(0) > 0;

        let sample_rate = params
            .sample_rate
            .ok_or_else(|| unsupported("unknown sample rate".to_string()))?;
        let channels = params
            .channels
            .as_ref()
            .map(|channels| channels.count())
            .unwrap_or(2)
            .max(1);

        // Gapless playback: the decoder trims encoder delay and padding
        // (SPEC §3.3). This is the default in symphonia 0.6; it is spelled
        // out because Onsa depends on it.
        let options = AudioDecoderOptions::default().gapless(true);
        let decoder = symphonia::default::get_codecs()
            .make_audio_decoder(&params, &options)
            .map_err(|error| unsupported(error.to_string()))?;

        let tags = read_tags(format.as_mut());

        // symphonia 0.6 ignores the MP4 edit list and iTunSMPB, so the AAC
        // priming would play. Read them here, iTunSMPB first.
        let mut trim = None;
        if !decoder_trims && is_mp4(path) {
            trim = tags
                .itunsmpb
                .as_deref()
                .and_then(mp4::parse_itunsmpb)
                .or_else(|| mp4::read_edit_list(path, sample_rate));
        }
        let (lead, limit) = match trim {
            Some(Mp4Trim { delay, playable }) => {
                tracing::debug!(path = %path.display(), delay, ?playable, "applying MP4 encoder trim");
                if playable.is_some() {
                    total_frames = playable;
                }
                (delay, playable)
            }
            None => (0, None),
        };

        Ok(Self {
            path: path.to_path_buf(),
            format,
            decoder,
            track_id,
            time_base,
            start_ts,
            info: TrackInfo {
                sample_rate,
                channels,
                total_frames,
                album: tags.album,
                track_number: tags.track_number,
                replaygain: tags.replaygain,
            },
            map: ChannelMap::new(channels, out_channels),
            out_channels: out_channels.max(1),
            scratch: Vec::new(),
            pending: Vec::new(),
            pending_pos: 0,
            skip_frames: lead,
            lead,
            limit,
            position: 0,
            ended: false,
        })
    }

    /// The file this source reads.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// What is known about the file.
    pub fn info(&self) -> &TrackInfo {
        &self.info
    }

    /// Frames handed out since the start of the track, at the file's rate.
    pub fn position(&self) -> u64 {
        self.position
    }

    /// Fills `out` with whole frames at the output channel count and returns
    /// how many frames were written. Zero means the track has ended.
    pub fn read(&mut self, out: &mut [f32]) -> Result<usize> {
        let channels = self.out_channels;
        let mut wanted = out.len() / channels;
        if let Some(limit) = self.limit {
            let left = limit.saturating_sub(self.position);
            wanted = wanted.min(usize::try_from(left).unwrap_or(usize::MAX));
        }
        let mut written = 0;
        while written < wanted {
            if self.pending_pos >= self.pending.len() {
                if self.ended || !self.decode_next()? {
                    break;
                }
                continue;
            }
            let available = (self.pending.len() - self.pending_pos) / channels;
            let frames = available.min(wanted - written);
            let from = self.pending_pos;
            out[written * channels..(written + frames) * channels]
                .copy_from_slice(&self.pending[from..from + frames * channels]);
            self.pending_pos += frames * channels;
            written += frames;
        }
        self.position += written as u64;
        Ok(written)
    }

    /// Moves to `frame` (at the file's rate), sample-accurately when the
    /// format allows. Returns the frame playback will continue from.
    pub fn seek(&mut self, frame: u64) -> Result<u64> {
        let rate = f64::from(self.info.sample_rate);
        let target = match self.info.total_frames {
            Some(total) => frame.min(total),
            None => frame,
        };
        let raw_target = target + self.lead;
        let time = Time::try_from_secs_f64(raw_target as f64 / rate).unwrap_or_default();
        let seeked = self.format.seek(
            SeekMode::Accurate,
            SeekTo::Time {
                time,
                track_id: Some(self.track_id),
            },
        );
        self.decoder.reset();
        self.pending.clear();
        self.pending_pos = 0;
        self.ended = false;

        match seeked {
            Ok(seeked) => {
                let landed = self
                    .timestamp_to_frame(seeked.actual_ts)
                    .unwrap_or(raw_target);
                self.skip_frames = raw_target.saturating_sub(landed);
                self.position = target;
                Ok(target)
            }
            Err(error) => {
                tracing::warn!(path = %self.path.display(), "seek failed: {error}");
                Err(Error::Decode {
                    path: self.path.clone(),
                    reason: format!("seek failed: {error}"),
                })
            }
        }
    }

    fn timestamp_to_frame(&self, ts: Timestamp) -> Option<u64> {
        let relative = Timestamp::new(ts.get().saturating_sub(self.start_ts.get()));
        let time = self.time_base?.calc_time(relative)?;
        let nanos = time.as_nanos().max(0) as f64;
        Some((nanos * f64::from(self.info.sample_rate) / 1e9).round() as u64)
    }

    /// Decodes packets until one yields samples. Returns `false` at the end
    /// of the track.
    fn decode_next(&mut self) -> Result<bool> {
        let mut errors = 0;
        loop {
            let packet = match self.format.next_packet() {
                Ok(Some(packet)) => packet,
                Ok(None) => {
                    self.ended = true;
                    return Ok(false);
                }
                Err(SymphoniaError::IoError(error))
                    if error.kind() == std::io::ErrorKind::UnexpectedEof =>
                {
                    self.ended = true;
                    return Ok(false);
                }
                Err(SymphoniaError::ResetRequired) => {
                    self.rebuild_decoder()?;
                    continue;
                }
                Err(error) => {
                    // A damaged tail should end the track, not the player.
                    tracing::warn!(path = %self.path.display(), "demuxing stopped: {error}");
                    self.ended = true;
                    return Ok(false);
                }
            };
            if packet.track_id != self.track_id {
                continue;
            }

            match self.decoder.decode(&packet) {
                Ok(buffer) => {
                    let channels = buffer.spec().channels().count().max(1);
                    buffer.copy_to_vec_interleaved(&mut self.scratch);
                    if channels != self.map.from() {
                        self.map = ChannelMap::new(channels, self.out_channels);
                    }
                }
                Err(SymphoniaError::DecodeError(reason)) => {
                    errors += 1;
                    tracing::debug!(path = %self.path.display(), "skipping bad packet: {reason}");
                    if errors >= MAX_CONSECUTIVE_DECODE_ERRORS {
                        return Err(Error::Decode {
                            path: self.path.clone(),
                            reason: reason.to_string(),
                        });
                    }
                    continue;
                }
                Err(SymphoniaError::ResetRequired) => {
                    self.rebuild_decoder()?;
                    continue;
                }
                Err(error) => {
                    return Err(Error::Decode {
                        path: self.path.clone(),
                        reason: error.to_string(),
                    })
                }
            }

            let mut frames = self.scratch.len() / self.map.from();
            let mut start = 0;
            if self.skip_frames > 0 {
                let skip = (self.skip_frames as usize).min(frames);
                self.skip_frames -= skip as u64;
                start = skip * self.map.from();
                frames -= skip;
            }
            if frames == 0 {
                continue;
            }
            self.pending.clear();
            self.pending_pos = 0;
            self.map.map_into(&self.scratch[start..], &mut self.pending);
            return Ok(true);
        }
    }

    fn rebuild_decoder(&mut self) -> Result<()> {
        let params = self.decoder.codec_params().clone();
        let options = AudioDecoderOptions::default().gapless(true);
        self.decoder = symphonia::default::get_codecs()
            .make_audio_decoder(&params, &options)
            .map_err(|error| Error::Decode {
                path: self.path.clone(),
                reason: error.to_string(),
            })?;
        Ok(())
    }
}

/// The tags the engine itself needs.
struct EngineTags {
    album: Option<String>,
    track_number: Option<u64>,
    /// MP4 gapless information written by iTunes and compatible encoders.
    itunsmpb: Option<String>,
    replaygain: ReplayGainTags,
}

/// Reads the album and track number tags, the two facts the crossfade rule
/// for consecutive album tracks needs (SPEC §3.3), the ReplayGain tags
/// (SPEC §4.1), and `iTunSMPB`.
fn read_tags(format: &mut dyn FormatReader) -> EngineTags {
    let mut found = EngineTags {
        album: None,
        track_number: None,
        itunsmpb: None,
        replaygain: ReplayGainTags::default(),
    };
    let mut metadata = format.metadata();
    if let Some(revision) = metadata.skip_to_latest() {
        let tags = revision.media.tags.iter().chain(
            revision
                .per_track
                .iter()
                .flat_map(|track| &track.metadata.tags),
        );
        for tag in tags {
            match &tag.std {
                Some(StandardTag::Album(name)) if found.album.is_none() => {
                    found.album = Some(name.to_string());
                }
                Some(StandardTag::TrackNumber(number)) if found.track_number.is_none() => {
                    found.track_number = Some(*number);
                }
                Some(StandardTag::ReplayGainTrackGain(value)) => {
                    let rg = &mut found.replaygain;
                    rg.track_gain_db = rg.track_gain_db.or(parse_gain(value));
                }
                Some(StandardTag::ReplayGainTrackPeak(value)) => {
                    let rg = &mut found.replaygain;
                    rg.track_peak = rg.track_peak.or(parse_peak(value));
                }
                Some(StandardTag::ReplayGainAlbumGain(value)) => {
                    let rg = &mut found.replaygain;
                    rg.album_gain_db = rg.album_gain_db.or(parse_gain(value));
                }
                Some(StandardTag::ReplayGainAlbumPeak(value)) => {
                    let rg = &mut found.replaygain;
                    rg.album_peak = rg.album_peak.or(parse_peak(value));
                }
                _ => {}
            }
            if found.itunsmpb.is_none() && tag.raw.key.to_ascii_lowercase().ends_with("itunsmpb") {
                found.itunsmpb = Some(tag.raw.value.to_string());
            }
        }
    }
    found
}

/// Whether the file is an MP4 container, by extension.
fn is_mp4(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "m4a" | "m4b" | "mp4" | "m4p"
            )
        })
}
