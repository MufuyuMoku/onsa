//! The engine handle: what `src-tauri` and `onsa-cli` hold on to.
//!
//! Commands go to the engine thread over a channel; events come back over
//! another. The engine never blocks the caller (SPEC §2, §3.1).

mod worker;

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use rtrb::RingBuffer;

use crate::analysis::{AnalysisFrame, AnalysisSettings};
use crate::dsp::chain::{ChainParams, DspSettings};
use crate::error::{Error, Result};
use crate::fade::CrossfadeCurve;
use crate::output::device::{DeviceChoice, DeviceFault, OutputRate};
use crate::output::offline::OfflineSink;
use crate::output::stage::{dsp_plumbing, OutputStage, Shared};
use crate::resample::ResamplerQuality;
use crate::source::TrackInfo;

pub use worker::queue_is_one_album;
use worker::{OutputTarget, Worker};

/// Identity of one queue entry.
///
/// A position in the queue cannot say which track is playing: the queue is
/// edited while it plays, so the entry at index three before an edit is a
/// different entry after it. Every entry carries an id instead, minted once
/// when the entry is made and kept as long as the entry lives, and the
/// engine follows the id (SPEC §6.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct QueueId(u64);

impl QueueId {
    /// A fresh id, unique for as long as the program runs.
    pub fn next() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        Self(NEXT.fetch_add(1, Ordering::Relaxed))
    }

    /// The number behind the id, for callers that have to send it elsewhere.
    pub fn get(self) -> u64 {
        self.0
    }
}

impl From<QueueId> for u64 {
    fn from(id: QueueId) -> Self {
        id.0
    }
}

/// One entry of the play queue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueItem {
    /// What tells this entry apart from every other one.
    pub id: QueueId,
    /// The file to play.
    pub path: PathBuf,
    /// Album, when the caller knows better than the file's tags (for example
    /// an edit kept in the library). `None` reads the tag.
    pub album: Option<String>,
    /// Track number, overriding the tag the same way.
    pub track_number: Option<u64>,
}

impl QueueItem {
    /// A queue entry that trusts the file's own tags.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            id: QueueId::next(),
            path: path.into(),
            album: None,
            track_number: None,
        }
    }
}

/// Interval of position events (SPEC §2: at most 10 Hz).
pub const DEFAULT_POSITION_INTERVAL: Duration = Duration::from_millis(100);

/// How far ahead of the device the engine decodes (SPEC §3.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BufferSize {
    /// About 50 ms: quick response, less tolerant of a busy system.
    Low,
    /// About 150 ms.
    #[default]
    Normal,
    /// About 500 ms: the most robust, and the power saving choice.
    Large,
}

impl BufferSize {
    /// Buffer length in seconds.
    pub fn seconds(self) -> f32 {
        match self {
            Self::Low => 0.05,
            Self::Normal => 0.15,
            Self::Large => 0.5,
        }
    }
}

/// What happens when a track ends (SPEC §6.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RepeatMode {
    /// The queue plays once and stops.
    #[default]
    Off,
    /// The queue starts over after the last track.
    All,
    /// The current track repeats, gaplessly.
    One,
}

/// Playback behaviour (SPEC §3.3, §3.4).
#[derive(Debug, Clone, PartialEq)]
pub struct PlaybackSettings {
    /// Crossfade between tracks, 0 to 12 seconds; 0 turns it off.
    pub crossfade_seconds: f32,
    /// Shape of the crossfade.
    pub curve: CrossfadeCurve,
    /// Crossfade used when the user skips (default 0.3 s).
    pub skip_crossfade_seconds: f32,
    /// Play consecutive tracks of the same album gaplessly even when a
    /// crossfade is set (default on).
    pub album_gapless: bool,
    /// Resampler preset.
    pub quality: ResamplerQuality,
    /// Buffer between the engine and the device.
    pub buffer: BufferSize,
    /// What happens when a track ends.
    pub repeat: RepeatMode,
    /// How often the playhead is reported. The power saving mode asks for
    /// fewer of them (SPEC §3.4); the default is ten a second (SPEC §2).
    pub position_interval: Duration,
}

impl Default for PlaybackSettings {
    fn default() -> Self {
        Self {
            crossfade_seconds: 0.0,
            curve: CrossfadeCurve::EqualPower,
            skip_crossfade_seconds: 0.3,
            album_gapless: true,
            quality: ResamplerQuality::Balanced,
            buffer: BufferSize::Normal,
            repeat: RepeatMode::Off,
            position_interval: DEFAULT_POSITION_INTERVAL,
        }
    }
}

/// Where and how to open the device.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OutputSettings {
    /// Device to play on.
    pub device: DeviceChoice,
    /// Sample rate to open it at.
    pub rate: OutputRate,
}

/// Whether anything is playing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayState {
    /// Nothing is loaded, or the queue has ended.
    Stopped,
    /// Audio is playing.
    Playing,
    /// Playback is held and can resume where it stopped.
    Paused,
}

/// What the engine reports back.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// A track became audible.
    TrackStarted {
        /// Position in the queue.
        index: usize,
        /// The queue entry it came from, which survives queue edits.
        id: QueueId,
        /// The file.
        path: PathBuf,
        /// What the engine knows about it.
        info: TrackInfo,
    },
    /// The playhead, at most ten times a second while playing.
    Position {
        /// Position in the queue.
        index: usize,
        /// Seconds into the track.
        seconds: f64,
    },
    /// Playback state changed.
    StateChanged(PlayState),
    /// A seek has landed; playback continues from here.
    Seeked {
        /// Position in the queue.
        index: usize,
        /// Seconds into the track.
        seconds: f64,
    },
    /// A track could not be opened or decoded; playback moves on.
    TrackFailed {
        /// Position in the queue.
        index: usize,
        /// The file.
        path: PathBuf,
        /// What went wrong.
        reason: String,
    },
    /// The last track of the queue has finished.
    QueueEnded,
    /// An output was opened (at start, or after a device change).
    OutputOpened {
        /// Device name.
        name: String,
        /// Sample rate it runs at.
        sample_rate: u32,
        /// Channel count.
        channels: usize,
    },
    /// The output was lost; the engine keeps trying to reopen it.
    OutputLost {
        /// What the backend reported.
        reason: String,
    },
    /// An analysis frame (spectrum, meters, centroid): at most 60 a second,
    /// only while analysis is on and audio is playing (SPEC §4.4).
    Analysis(AnalysisFrame),
}

/// Commands from the handle to the engine thread.
pub(crate) enum Command {
    SetQueue {
        items: Vec<QueueItem>,
        start: usize,
        start_seconds: f64,
        play: bool,
    },
    UpdateQueue {
        items: Vec<QueueItem>,
    },
    Jump(usize),
    Pause,
    Resume,
    TogglePause,
    Stop,
    Next,
    Previous,
    Seek(f64),
    SetSettings(PlaybackSettings),
    SetDsp(DspSettings),
    SetAnalysis(AnalysisSettings),
    DeviceFault(DeviceFault),
    SetOutput(OutputSettings),
    MatchRate(u32),
    ReplaceOffline {
        sample_rate: u32,
        channels: usize,
        reply: Sender<OfflineSink>,
    },
    Shutdown,
}

/// The audio engine. Dropping it stops playback and joins its thread.
pub struct Engine {
    commands: Sender<Command>,
    events: Receiver<Event>,
    worker: Option<JoinHandle<()>>,
}

impl Engine {
    /// Starts the engine on a real audio device.
    pub fn with_device(output: OutputSettings, settings: PlaybackSettings) -> Result<Self> {
        let (commands, command_rx) = mpsc::channel();
        let (event_tx, events) = mpsc::channel();
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);
        let self_tx = commands.clone();
        let worker = std::thread::Builder::new()
            .name("onsa-audio".into())
            .spawn(move || {
                let mut worker = Worker::new(
                    command_rx,
                    self_tx,
                    event_tx,
                    settings,
                    OutputTarget::Device(output),
                );
                let opened = worker.open_device_output();
                let ok = opened.is_ok();
                let _ = ready_tx.send(opened);
                if ok {
                    worker.run();
                }
            })
            .map_err(|error| Error::Output(error.to_string()))?;
        match ready_rx.recv() {
            Ok(Ok(())) => Ok(Self {
                commands,
                events,
                worker: Some(worker),
            }),
            Ok(Err(error)) => {
                let _ = worker.join();
                Err(error)
            }
            Err(_) => Err(Error::EngineStopped),
        }
    }

    /// Starts the engine on an offline sink, for tests and rendering.
    pub fn offline(
        sample_rate: u32,
        channels: usize,
        settings: PlaybackSettings,
    ) -> (Self, OfflineSink) {
        let (commands, command_rx) = mpsc::channel();
        let (event_tx, events) = mpsc::channel();
        let self_tx = commands.clone();
        let params = DspSettings::default().chain_params(sample_rate);
        let (output, sink) = offline_output(sample_rate, channels, settings.buffer, params);
        let worker = std::thread::Builder::new()
            .name("onsa-audio".into())
            .spawn(move || {
                let mut worker = Worker::new(
                    command_rx,
                    self_tx,
                    event_tx,
                    settings,
                    OutputTarget::Offline,
                );
                worker.attach_output(output);
                worker.run();
            })
            .ok();
        (
            Self {
                commands,
                events,
                worker,
            },
            sink,
        )
    }

    /// The events the engine reports.
    pub fn events(&self) -> &Receiver<Event> {
        &self.events
    }

    fn send(&self, command: Command) -> Result<()> {
        self.commands
            .send(command)
            .map_err(|_| Error::EngineStopped)
    }

    /// Replaces the queue and starts at `start`, `start_seconds` into it.
    /// With `play` false the engine loads paused, as when restoring state.
    pub fn set_queue(
        &self,
        items: Vec<QueueItem>,
        start: usize,
        start_seconds: f64,
        play: bool,
    ) -> Result<()> {
        self.send(Command::SetQueue {
            items,
            start,
            start_seconds,
            play,
        })
    }

    /// Replaces the queue without disturbing what is playing: entries added,
    /// removed, reordered or shuffled (SPEC §6.1).
    ///
    /// The engine finds the playing entry again by its [`QueueId`], so it
    /// needs no help saying where the track went. When the playing entry is
    /// no longer in the queue, playback moves to the first entry that
    /// followed it and survived, and stops when there is none.
    pub fn update_queue(&self, items: Vec<QueueItem>) -> Result<()> {
        self.send(Command::UpdateQueue { items })
    }

    /// Plays the queue entry at `index`.
    pub fn jump(&self, index: usize) -> Result<()> {
        self.send(Command::Jump(index))
    }

    /// Pauses with a short fade.
    pub fn pause(&self) -> Result<()> {
        self.send(Command::Pause)
    }

    /// Resumes with a short fade.
    pub fn resume(&self) -> Result<()> {
        self.send(Command::Resume)
    }

    /// Pauses when playing, resumes otherwise.
    pub fn toggle_pause(&self) -> Result<()> {
        self.send(Command::TogglePause)
    }

    /// Stops with a short fade.
    pub fn stop(&self) -> Result<()> {
        self.send(Command::Stop)
    }

    /// Skips to the next track.
    pub fn next(&self) -> Result<()> {
        self.send(Command::Next)
    }

    /// Restarts the track, or goes to the previous one near its start.
    pub fn previous(&self) -> Result<()> {
        self.send(Command::Previous)
    }

    /// Moves to `seconds` into the current track.
    pub fn seek(&self, seconds: f64) -> Result<()> {
        self.send(Command::Seek(seconds))
    }

    /// Changes playback behaviour. Buffer size applies the next time the
    /// output opens; everything else applies to the next transition.
    pub fn set_settings(&self, settings: PlaybackSettings) -> Result<()> {
        self.send(Command::SetSettings(settings))
    }

    /// Changes the DSP chain (SPEC §4). EQ, preamp, limiter, volume and
    /// dither change smoothly within milliseconds; ReplayGain applies from
    /// the next block the engine decodes.
    pub fn set_dsp(&self, settings: DspSettings) -> Result<()> {
        self.send(Command::SetDsp(settings))
    }

    /// Switches the analysis tap and its frames on or off (SPEC §4.4).
    pub fn set_analysis(&self, settings: AnalysisSettings) -> Result<()> {
        self.send(Command::SetAnalysis(settings))
    }

    /// Moves playback to another device or sample rate. The old output fades
    /// out, the new one opens, and playback continues from the same position.
    pub fn set_output(&self, output: OutputSettings) -> Result<()> {
        self.send(Command::SetOutput(output))
    }

    /// Hands the event receiver over, so a listener thread can block on it
    /// without sharing the engine (an idle listener then costs no CPU).
    /// Afterwards [`Engine::events`] reports nothing.
    pub fn take_events(&mut self) -> Receiver<Event> {
        std::mem::replace(&mut self.events, mpsc::channel().1)
    }

    /// Replaces an offline output with a new one, possibly at another rate
    /// or channel count, exactly as a device change would. Test support for
    /// the device recovery path.
    pub fn replace_offline_output(&self, sample_rate: u32, channels: usize) -> Result<OfflineSink> {
        let (reply, receive) = mpsc::channel();
        self.send(Command::ReplaceOffline {
            sample_rate,
            channels,
            reply,
        })?;
        receive.recv().map_err(|_| Error::EngineStopped)
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        let _ = self.commands.send(Command::Shutdown);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

/// Builds an offline output: the engine side and the sink side. The offline
/// sink renders float, so it never needs dither.
pub(crate) fn offline_output(
    sample_rate: u32,
    channels: usize,
    buffer: BufferSize,
    params: ChainParams,
) -> (worker::Output, OfflineSink) {
    let channels = channels.max(1);
    let capacity = ((sample_rate as f32 * buffer.seconds()) as usize).max(1024) * channels;
    let (producer, consumer) = RingBuffer::new(capacity);
    let shared = Arc::new(Shared::default());
    shared.silent.store(true, Ordering::Release);
    let (dsp, port) = dsp_plumbing(sample_rate, channels, None, params);
    let stage = OutputStage::new(consumer, shared.clone(), channels, sample_rate, dsp);
    let output = worker::Output {
        producer,
        shared: shared.clone(),
        sample_rate,
        channels,
        device: None,
        name: "offline".to_string(),
        dsp: port,
    };
    (output, OfflineSink::new(stage, shared, sample_rate))
}
