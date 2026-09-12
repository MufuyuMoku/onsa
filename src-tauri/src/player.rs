//! The player: the audio engine plus what the interface needs to know about
//! it (SPEC §2).
//!
//! Commands go straight to the engine. Its events arrive on a listener
//! thread that blocks until the engine has something to say, so a paused
//! player costs no CPU (SPEC §13.1); the listener keeps a mirror of the
//! player's state and turns events into interface events.

use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex, MutexGuard, Weak};

use onsa_audio::dsp::gain_to_db;
use onsa_audio::{
    queue_is_one_album, AnalysisFrame, AnalysisSettings, Engine, Event, PlayState, QueueItem,
    TrackInfo,
};
use onsa_library::TrackRow;
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::dto::TrackDto;
use crate::error::ErrorCode;
use crate::settings::{DspPrefs, EqKind, OutputPrefs, PlaybackPrefs, RgMode};

/// Everything but the playhead changed.
pub const STATE_EVENT: &str = "player://state";
/// The playhead moved (at most ten times a second).
pub const POSITION_EVENT: &str = "player://position";
/// Peak meters (while playing).
pub const METER_EVENT: &str = "player://meter";

/// Meter frames per second. The transport meters need no more.
const METER_FPS: u32 = 30;

/// The audio backend cpal uses on this system, for the signal path.
#[cfg(windows)]
const BACKEND: &str = "WASAPI";
#[cfg(target_os = "linux")]
const BACKEND: &str = "ALSA";
#[cfg(not(any(windows, target_os = "linux")))]
const BACKEND: &str = "cpal";

/// The player kept in the application state.
pub struct Player {
    shared: Arc<Shared>,
}

struct Shared {
    app: AppHandle,
    engine: Mutex<Option<Engine>>,
    state: Mutex<State>,
}

struct OutputInfo {
    name: String,
    sample_rate: u32,
    channels: usize,
}

struct State {
    queue: Vec<TrackRow>,
    one_album: bool,
    current: Option<usize>,
    play_state: PlayState,
    position: f64,
    duration: Option<f64>,
    source: Option<TrackInfo>,
    output: Option<OutputInfo>,
    output_error: bool,
    failed_track: Option<String>,
    output_prefs: OutputPrefs,
    playback: PlaybackPrefs,
    dsp: DspPrefs,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    // A panic elsewhere must not take playback down with it.
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Everything the interface shows about playback, minus the meters.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    /// `stopped`, `playing` or `paused`.
    pub state: &'static str,
    /// Queue position of the current track.
    pub current: Option<usize>,
    /// Number of queue entries.
    pub queue_length: usize,
    /// The current track.
    pub track: Option<TrackDto>,
    /// Seconds into the track.
    pub position: f64,
    /// Length of the track in seconds.
    pub duration: Option<f64>,
    /// Volume in dB.
    pub volume_db: f32,
    /// The signal path (SPEC §9.2).
    pub signal: SignalPath,
    /// Whether no audio output is open.
    pub output_error: bool,
    /// The last track that could not be played, if any.
    pub failed_track: Option<String>,
}

/// The stages the signal really goes through.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignalPath {
    /// The file.
    pub source: Option<SourceStage>,
    /// Present only while the rate is converted.
    pub resample: Option<ResampleStage>,
    /// ReplayGain.
    pub replaygain: ReplayGainStage,
    /// EQ and preamp.
    pub eq: EqStage,
    /// Peak limiter.
    pub limiter: LimiterStage,
    /// The device.
    pub output: Option<OutputStage>,
}

/// The file's format.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceStage {
    /// Codec or container.
    pub codec: Option<String>,
    /// Sample rate.
    pub sample_rate: u32,
    /// Bit depth, for lossless formats.
    pub bit_depth: Option<u8>,
    /// Channels.
    pub channels: usize,
}

/// Sample rate conversion.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResampleStage {
    /// From the file's rate.
    pub from: u32,
    /// To the device's rate.
    pub to: u32,
}

/// ReplayGain as applied to the current track.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayGainStage {
    /// The mode.
    pub mode: RgMode,
    /// Whether album gain applies.
    pub album: bool,
    /// The gain applied to the current track, in dB.
    pub gain_db: Option<f32>,
}

/// EQ and preamp.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EqStage {
    /// Whether the EQ runs.
    pub enabled: bool,
    /// Graphic or parametric.
    pub kind: EqKind,
    /// Bands in use.
    pub bands: usize,
    /// Preamp in effect, in dB.
    pub preamp_db: f32,
    /// Whether the preamp follows the curve.
    pub auto_preamp: bool,
}

/// The limiter.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LimiterStage {
    /// Whether it limits.
    pub enabled: bool,
}

/// The open device.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputStage {
    /// Device name.
    pub name: String,
    /// Audio backend.
    pub backend: &'static str,
    /// Rate it runs at.
    pub sample_rate: u32,
    /// Channels.
    pub channels: usize,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
struct Position {
    index: usize,
    seconds: f64,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
struct Meter {
    peak_db: [f32; 2],
    clip: bool,
    limiting: bool,
}

impl Meter {
    fn from_frame(frame: &AnalysisFrame) -> Self {
        let left = frame.peak_db.first().copied().unwrap_or(-120.0);
        let right = frame.peak_db.get(1).copied().unwrap_or(left);
        Self {
            peak_db: [left, right],
            clip: frame.clip,
            limiting: frame.limiting,
        }
    }
}

impl Player {
    /// Starts the player on the configured output. When no output can be
    /// opened the player still starts, reports it, and tries again when
    /// asked to play or when the output setting changes.
    pub fn start(
        app: AppHandle,
        output: OutputPrefs,
        playback: PlaybackPrefs,
        dsp: DspPrefs,
    ) -> Self {
        let shared = Arc::new(Shared {
            app,
            engine: Mutex::new(None),
            state: Mutex::new(State {
                queue: Vec::new(),
                one_album: false,
                current: None,
                play_state: PlayState::Stopped,
                position: 0.0,
                duration: None,
                source: None,
                output: None,
                output_error: false,
                failed_track: None,
                output_prefs: output,
                playback,
                dsp,
            }),
        });
        shared.open_engine();
        Self { shared }
    }

    /// What the interface shows.
    pub fn snapshot(&self) -> Snapshot {
        lock(&self.shared.state).snapshot()
    }

    /// The queue and the current entry.
    pub fn queue(&self) -> (Vec<TrackDto>, Option<usize>) {
        let state = lock(&self.shared.state);
        (
            state.queue.iter().map(TrackDto::from).collect(),
            state.current,
        )
    }

    /// The settings in use.
    pub fn prefs(&self) -> (OutputPrefs, PlaybackPrefs, DspPrefs) {
        let state = lock(&self.shared.state);
        (
            state.output_prefs.clone(),
            state.playback.clone(),
            state.dsp.clone(),
        )
    }

    /// Rate of the open output, if any.
    pub fn output_rate(&self) -> Option<u32> {
        lock(&self.shared.state)
            .output
            .as_ref()
            .map(|output| output.sample_rate)
    }

    /// Replaces the queue with `rows` and plays from `index`. Tracks known to
    /// be missing are left out.
    pub fn play(&self, rows: Vec<TrackRow>, index: usize) -> Result<(), ErrorCode> {
        let clicked = rows.get(index).map(|row| row.id);
        let rows: Vec<TrackRow> = rows
            .into_iter()
            .filter(|row| row.status != "missing")
            .collect();
        if rows.is_empty() {
            return Ok(());
        }
        let index = clicked
            .and_then(|id| rows.iter().position(|row| row.id == id))
            .unwrap_or(0);
        // The library's values win over the file's tags: an edit kept in
        // Onsa counts for album gapless and ReplayGain too.
        let items: Vec<QueueItem> = rows
            .iter()
            .map(|row| QueueItem {
                path: row.path.clone().into(),
                album: row.album.clone(),
                track_number: row.track_number.map(u64::from),
            })
            .collect();
        {
            let mut state = lock(&self.shared.state);
            state.one_album = queue_is_one_album(&items);
            state.duration = rows[index].duration_ms.map(|ms| ms as f64 / 1000.0);
            state.queue = rows;
            state.current = Some(index);
            state.position = 0.0;
            state.source = None;
            state.failed_track = None;
        }
        if lock(&self.shared.engine).is_none() {
            self.shared.open_engine();
        }
        let result = self
            .shared
            .command(|engine| engine.set_queue(items, index, 0.0, true));
        self.shared.emit_state();
        result
    }

    /// Pauses when playing, plays otherwise.
    pub fn toggle(&self) -> Result<(), ErrorCode> {
        self.shared.command(Engine::toggle_pause)
    }

    /// Next track.
    pub fn next(&self) -> Result<(), ErrorCode> {
        self.shared.command(Engine::next)
    }

    /// Previous track, or the start of this one.
    pub fn previous(&self) -> Result<(), ErrorCode> {
        self.shared.command(Engine::previous)
    }

    /// Seeks within the current track.
    pub fn seek(&self, seconds: f64) -> Result<(), ErrorCode> {
        let seconds = if seconds.is_finite() {
            seconds.max(0.0)
        } else {
            0.0
        };
        self.shared.command(|engine| engine.seek(seconds))
    }

    /// Plays another queue entry.
    pub fn jump(&self, index: usize) -> Result<(), ErrorCode> {
        self.shared.command(|engine| engine.jump(index))
    }

    /// Changes the DSP chain.
    pub fn set_dsp(&self, dsp: DspPrefs) -> Result<(), ErrorCode> {
        let settings = dsp.engine();
        lock(&self.shared.state).dsp = dsp;
        let result = self.shared.command(|engine| engine.set_dsp(settings));
        self.shared.emit_state();
        ignore_no_output(result)
    }

    /// Changes crossfade, resampler and buffer.
    pub fn set_playback(&self, playback: PlaybackPrefs) -> Result<(), ErrorCode> {
        let settings = playback.engine();
        lock(&self.shared.state).playback = playback;
        ignore_no_output(self.shared.command(|engine| engine.set_settings(settings)))
    }

    /// Changes device or rate.
    pub fn set_output(&self, output: OutputPrefs) -> Result<(), ErrorCode> {
        let settings = output.engine();
        lock(&self.shared.state).output_prefs = output;
        let result = if lock(&self.shared.engine).is_some() {
            self.shared.command(|engine| engine.set_output(settings))
        } else if self.shared.open_engine() {
            Ok(())
        } else {
            Err(ErrorCode::NoOutput)
        };
        self.shared.emit_state();
        result
    }
}

/// Settings are kept even while no output is open; they apply once one is.
fn ignore_no_output(result: Result<(), ErrorCode>) -> Result<(), ErrorCode> {
    match result {
        Err(ErrorCode::NoOutput) => Ok(()),
        other => other,
    }
}

impl Shared {
    /// Opens the engine on the configured output. Returns whether it worked.
    fn open_engine(self: &Arc<Self>) -> bool {
        let (output, playback, dsp) = {
            let state = lock(&self.state);
            (
                state.output_prefs.engine(),
                state.playback.engine(),
                state.dsp.engine(),
            )
        };
        match Engine::with_device(output, playback) {
            Ok(mut engine) => {
                let events = engine.take_events();
                let _ = engine.set_dsp(dsp);
                let _ = engine.set_analysis(AnalysisSettings {
                    enabled: true,
                    fps: METER_FPS,
                    ..AnalysisSettings::default()
                });
                spawn_listener(Arc::downgrade(self), events);
                *lock(&self.engine) = Some(engine);
                lock(&self.state).output_error = false;
                true
            }
            Err(error) => {
                tracing::error!("no audio output can be opened: {error}");
                lock(&self.state).output_error = true;
                false
            }
        }
    }

    fn command(
        &self,
        send: impl FnOnce(&Engine) -> onsa_audio::Result<()>,
    ) -> Result<(), ErrorCode> {
        match lock(&self.engine).as_ref() {
            Some(engine) => Ok(send(engine)?),
            None => Err(ErrorCode::NoOutput),
        }
    }

    fn emit_state(&self) {
        let snapshot = lock(&self.state).snapshot();
        if let Err(error) = self.app.emit(STATE_EVENT, snapshot) {
            tracing::debug!("player state not delivered: {error}");
        }
    }

    fn on_event(&self, event: Event) {
        match event {
            Event::TrackStarted { index, info, path } => {
                tracing::debug!(index, path = %path.display(), "track started");
                {
                    let mut state = lock(&self.state);
                    state.current = Some(index);
                    state.position = 0.0;
                    state.duration = info.duration_seconds().or_else(|| {
                        state
                            .queue
                            .get(index)
                            .and_then(|row| row.duration_ms)
                            .map(|ms| ms as f64 / 1000.0)
                    });
                    state.source = Some(info);
                    state.failed_track = None;
                }
                self.emit_state();
            }
            Event::Position { index, seconds } | Event::Seeked { index, seconds } => {
                lock(&self.state).position = seconds;
                let _ = self.app.emit(POSITION_EVENT, Position { index, seconds });
            }
            Event::StateChanged(play_state) => {
                lock(&self.state).play_state = play_state;
                self.emit_state();
            }
            Event::TrackFailed {
                index,
                path,
                reason,
            } => {
                tracing::warn!(index, path = %path.display(), "track cannot be played: {reason}");
                lock(&self.state).failed_track = Some(path.display().to_string());
                self.emit_state();
            }
            Event::QueueEnded => {
                tracing::debug!("queue ended");
                {
                    let mut state = lock(&self.state);
                    state.play_state = PlayState::Stopped;
                    state.position = 0.0;
                }
                self.emit_state();
            }
            Event::OutputOpened {
                name,
                sample_rate,
                channels,
            } => {
                tracing::info!(device = %name, sample_rate, channels, "audio output opened");
                {
                    let mut state = lock(&self.state);
                    state.output = Some(OutputInfo {
                        name,
                        sample_rate,
                        channels,
                    });
                    state.output_error = false;
                }
                self.emit_state();
            }
            Event::OutputLost { reason } => {
                tracing::warn!("audio output lost: {reason}");
                {
                    let mut state = lock(&self.state);
                    state.output = None;
                    state.output_error = true;
                }
                self.emit_state();
            }
            Event::Analysis(frame) => {
                let _ = self.app.emit(METER_EVENT, Meter::from_frame(&frame));
            }
        }
    }
}

/// Forwards engine events until the engine goes away.
fn spawn_listener(shared: Weak<Shared>, events: Receiver<Event>) {
    let spawned = std::thread::Builder::new()
        .name("onsa-player-events".into())
        .spawn(move || {
            for event in events {
                let Some(shared) = shared.upgrade() else {
                    return;
                };
                shared.on_event(event);
            }
        });
    if let Err(error) = spawned {
        tracing::error!("player events cannot be followed: {error}");
    }
}

impl State {
    fn snapshot(&self) -> Snapshot {
        let row = self.current.and_then(|index| self.queue.get(index));
        let output_rate = self.output.as_ref().map(|output| output.sample_rate);
        let settings = self.dsp.engine();
        let album = settings.replaygain.uses_album(self.one_album);

        let source = self.source.as_ref().map(|info| SourceStage {
            codec: row.and_then(|row| row.codec.clone()),
            sample_rate: info.sample_rate,
            bit_depth: row.and_then(|row| row.bit_depth),
            channels: info.channels,
        });
        let resample = match (&self.source, output_rate) {
            (Some(info), Some(to)) if info.sample_rate != to => Some(ResampleStage {
                from: info.sample_rate,
                to,
            }),
            _ => None,
        };
        let gain_db = match (self.dsp.replaygain_mode, &self.source) {
            (RgMode::Off, _) | (_, None) => None,
            (_, Some(info)) => Some(gain_to_db(
                settings.replaygain.gain(&info.replaygain, album),
            )),
        };

        Snapshot {
            state: match self.play_state {
                PlayState::Stopped => "stopped",
                PlayState::Playing => "playing",
                PlayState::Paused => "paused",
            },
            current: self.current,
            queue_length: self.queue.len(),
            track: row.map(TrackDto::from),
            position: self.position,
            duration: self.duration,
            volume_db: self.dsp.volume_db,
            signal: SignalPath {
                source,
                resample,
                replaygain: ReplayGainStage {
                    mode: self.dsp.replaygain_mode,
                    album,
                    gain_db,
                },
                eq: EqStage {
                    enabled: self.dsp.eq_enabled,
                    kind: self.dsp.eq_kind,
                    bands: settings.eq.bands().len(),
                    preamp_db: settings.effective_preamp_db(output_rate.unwrap_or(48_000)),
                    auto_preamp: self.dsp.auto_preamp,
                },
                limiter: LimiterStage {
                    enabled: self.dsp.limiter_enabled,
                },
                output: self.output.as_ref().map(|output| OutputStage {
                    name: output.name.clone(),
                    backend: BACKEND,
                    sample_rate: output.sample_rate,
                    channels: output.channels,
                }),
            },
            output_error: self.output_error,
            failed_track: self.failed_track.clone(),
        }
    }
}
