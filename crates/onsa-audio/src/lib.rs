//! Onsa audio engine.
//!
//! Owns decoding, resampling, the queue with gapless playback and crossfade,
//! the output stage and the offline sink. The engine runs on its own threads
//! and knows nothing about the user interface; `src-tauri` (or `onsa-cli`)
//! drives it with commands and listens to its events (SPEC §3).
//!
//! This crate must not depend on any other Onsa crate (SPEC §2).

#![warn(missing_docs)]

mod channels;
mod engine;
mod error;
mod fade;
mod lane;
mod mp4;
pub mod output;
mod resample;
mod source;
pub mod wav;

pub use channels::ChannelMap;
pub use engine::{
    BufferSize, Engine, Event, OutputSettings, PlayState, PlaybackSettings, QueueItem,
};
pub use error::{Error, Result};
pub use fade::{CrossfadeCurve, MICRO_FADE_SECONDS};
pub use output::device::{list_devices, DeviceChoice, DeviceFault, DeviceInfo, OutputRate};
pub use output::offline::OfflineSink;
pub use resample::{ResamplerQuality, StreamResampler};
pub use source::{FileSource, TrackInfo};
