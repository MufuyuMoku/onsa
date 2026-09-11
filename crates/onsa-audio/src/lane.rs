//! A lane: tracks played back to back through one resampler.
//!
//! Consecutive tracks with the same sample rate share a lane, so the
//! resampler sees one continuous signal and the join between them is truly
//! gapless (SPEC §3.3). A seek, a skip, a crossfade or a change of sample rate
//! starts a new lane.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::resample::{ResamplerQuality, StreamResampler};
use crate::source::FileSource;

/// Frames read from a source per step.
const READ_FRAMES: usize = 1024;

/// A track waiting in, or playing from, a lane.
pub struct LaneTrack {
    /// Position of the track in the queue.
    pub queue_index: usize,
    /// The opened file.
    pub source: FileSource,
}

/// Where a track begins within the lane.
#[derive(Debug, Clone, Copy)]
struct Mark {
    queue_index: usize,
    /// Output frame of the lane at which the track starts.
    out_start: u64,
    /// Track frame (at the track's own rate) the lane starts it from.
    track_start: u64,
}

/// A block of output that belongs to a single track.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Block {
    /// Frames written.
    pub frames: usize,
    /// Track the frames belong to.
    pub queue_index: usize,
    /// Track frame (at the track's rate) of the first written frame.
    pub track_frame: u64,
}

/// A track that could not be decoded to its end.
pub struct Failure {
    /// Position of the track in the queue.
    pub queue_index: usize,
    /// What went wrong.
    pub error: Error,
}

/// Tracks played back to back at one source rate.
pub struct Lane {
    src_rate: u32,
    out_rate: u32,
    channels: usize,
    resampler: Option<StreamResampler>,
    /// Tracks still to be read; the front one is being read.
    tracks: VecDeque<LaneTrack>,
    /// Starts of every track that has entered the output.
    marks: Vec<Mark>,
    /// Output frames ready to hand out.
    pending: Vec<f32>,
    pending_pos: usize,
    /// Output frames handed out so far.
    handed: u64,
    /// Source frames read so far, across all tracks.
    fed: u64,
    scratch: Vec<f32>,
    failures: Vec<Failure>,
    input_done: bool,
}

impl Lane {
    /// Starts a lane with its first track, output at `out_rate`.
    pub fn new(
        first: LaneTrack,
        out_rate: u32,
        channels: usize,
        quality: ResamplerQuality,
    ) -> Result<Self> {
        let src_rate = first.source.info().sample_rate;
        let resampler = if src_rate == out_rate {
            None
        } else {
            Some(StreamResampler::new(src_rate, out_rate, channels, quality)?)
        };
        let track_start = first.source.position();
        let mut lane = Self {
            src_rate,
            out_rate,
            channels,
            resampler,
            tracks: VecDeque::new(),
            marks: vec![Mark {
                queue_index: first.queue_index,
                out_start: 0,
                track_start,
            }],
            pending: Vec::new(),
            pending_pos: 0,
            handed: 0,
            fed: 0,
            scratch: vec![0.0; READ_FRAMES * channels],
            failures: Vec::new(),
            input_done: false,
        };
        lane.tracks.push_back(first);
        Ok(lane)
    }

    /// Whether `source` can follow gaplessly inside this lane.
    pub fn accepts(&self, source: &FileSource) -> bool {
        !self.input_done && source.info().sample_rate == self.src_rate
    }

    /// Queues a track to follow the ones already in the lane.
    pub fn append(&mut self, track: LaneTrack) {
        self.tracks.push_back(track);
    }

    /// Queue index of the last track the lane will play.
    pub fn last_queue_index(&self) -> Option<usize> {
        self.tracks
            .back()
            .map(|track| track.queue_index)
            .or_else(|| self.marks.last().map(|mark| mark.queue_index))
    }

    /// Number of tracks waiting behind the one being read.
    pub fn tracks_waiting(&self) -> usize {
        self.tracks.len().saturating_sub(1)
    }

    /// Info of the last track the lane will play.
    pub fn last_track(&self) -> Option<&FileSource> {
        self.tracks.back().map(|track| &track.source)
    }

    /// Output frames still to come, when every remaining track knows its
    /// length. Used to start a crossfade on time.
    pub fn remaining_frames(&self) -> Option<u64> {
        let mut source_frames = 0u64;
        for track in &self.tracks {
            let total = track.source.info().total_frames?;
            source_frames += total.saturating_sub(track.source.position());
        }
        let ratio = f64::from(self.out_rate) / f64::from(self.src_rate);
        let pending = ((self.pending.len() - self.pending_pos) / self.channels) as u64;
        Some((source_frames as f64 * ratio).round() as u64 + pending)
    }

    /// Takes the failures collected since the last call.
    pub fn take_failures(&mut self) -> Vec<Failure> {
        std::mem::take(&mut self.failures)
    }

    /// Writes up to `out.len() / channels` frames, never crossing from one
    /// track into the next, so every block belongs to exactly one track.
    /// Returns `None` once the lane is finished.
    pub fn read(&mut self, out: &mut [f32]) -> Option<Block> {
        let channels = self.channels;
        while self.pending_pos >= self.pending.len() && !self.input_done {
            self.pending.clear();
            self.pending_pos = 0;
            self.pull();
        }
        if self.pending_pos >= self.pending.len() {
            return None;
        }

        let position = self.handed;
        let mark_index = self
            .marks
            .iter()
            .rposition(|mark| mark.out_start <= position)
            .unwrap_or(0);
        let mark = self.marks[mark_index];
        let mut frames = (self.pending.len() - self.pending_pos) / channels;
        frames = frames.min(out.len() / channels);
        if let Some(next) = self.marks.get(mark_index + 1) {
            frames = frames.min((next.out_start - position) as usize);
        }

        let from = self.pending_pos;
        out[..frames * channels].copy_from_slice(&self.pending[from..from + frames * channels]);
        self.pending_pos += frames * channels;
        self.handed += frames as u64;

        let offset_out = position - mark.out_start;
        let offset_src =
            (offset_out as f64 * f64::from(self.src_rate) / f64::from(self.out_rate)).round();
        Some(Block {
            frames,
            queue_index: mark.queue_index,
            track_frame: mark.track_start + offset_src as u64,
        })
    }

    /// Reads one step of source audio into `pending`.
    fn pull(&mut self) {
        let channels = self.channels;
        let Some(track) = self.tracks.front_mut() else {
            self.finish_input();
            return;
        };
        let frames = match track.source.read(&mut self.scratch) {
            Ok(frames) => frames,
            Err(error) => {
                tracing::warn!(path = %track.source.path().display(), "track stopped early: {error}");
                self.failures.push(Failure {
                    queue_index: track.queue_index,
                    error,
                });
                0
            }
        };

        if frames == 0 {
            // This track is done; the next one starts at the current source
            // position, which maps to an exact output frame.
            self.tracks.pop_front();
            match self.tracks.front() {
                Some(next) => {
                    let out_start = (self.fed as f64 * f64::from(self.out_rate)
                        / f64::from(self.src_rate))
                    .round() as u64;
                    self.marks.push(Mark {
                        queue_index: next.queue_index,
                        out_start,
                        track_start: next.source.position(),
                    });
                }
                None => self.finish_input(),
            }
            return;
        }

        self.fed += frames as u64;
        let input = &self.scratch[..frames * channels];
        match self.resampler.as_mut() {
            None => self.pending.extend_from_slice(input),
            Some(resampler) => {
                if let Err(error) = resampler.feed(input, &mut self.pending) {
                    tracing::error!("resampler failed: {error}");
                    self.finish_input();
                }
            }
        }
    }

    fn finish_input(&mut self) {
        if self.input_done {
            return;
        }
        self.input_done = true;
        if let Some(resampler) = self.resampler.as_mut() {
            if let Err(error) = resampler.finish(&mut self.pending) {
                tracing::error!("resampler flush failed: {error}");
            }
        }
    }
}
