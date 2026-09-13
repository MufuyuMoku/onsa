//! Onsa audio engine.
//!
//! Owns decoding, resampling, the queue with gapless playback and crossfade,
//! the output stage and the offline sink. The engine runs on its own threads
//! and knows nothing about the user interface; `src-tauri` (or `onsa-cli`)
//! drives it with commands and listens to its events (SPEC §3).
//!
//! This crate must not depend on any other Onsa crate (SPEC §2).

#![warn(missing_docs)]

pub mod analysis;
mod channels;
pub mod dsp;
mod engine;
mod error;
mod fade;
mod lane;
mod mp4;
pub mod output;
mod resample;
mod source;
pub mod wav;

pub use analysis::{AnalysisFrame, AnalysisSettings};
pub use channels::ChannelMap;
pub use dsp::biquad::{Band, FilterKind};
pub use dsp::chain::{DspSettings, Stage};
pub use dsp::eq::EqMode;
pub use dsp::replaygain::{ReplayGainMode, ReplayGainSettings, ReplayGainTags};
pub use engine::{
    queue_is_one_album, BufferSize, Engine, Event, OutputSettings, PlayState, PlaybackSettings,
    QueueId, QueueItem, RepeatMode, DEFAULT_POSITION_INTERVAL,
};
pub use error::{Error, Result};
pub use fade::{CrossfadeCurve, MICRO_FADE_SECONDS};
pub use output::device::{list_devices, DeviceChoice, DeviceFault, DeviceInfo, OutputRate};
pub use output::offline::OfflineSink;
pub use resample::{ResamplerQuality, StreamResampler};
pub use source::{FileSource, TrackInfo};
