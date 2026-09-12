//! The player: the audio engine plus what the interface needs to know about
//! it (SPEC §2).
//!
//! Commands go straight to the engine. Its events arrive on a listener
//! thread that blocks until the engine has something to say, so a paused
//! player costs no CPU (SPEC §13.1); the listener keeps a mirror of the
//! player's state and turns events into interface events.

use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::time::{Duration, Instant};

use onsa_audio::dsp::gain_to_db;
use onsa_audio::{
    queue_is_one_album, AnalysisFrame, AnalysisSettings, Engine, Event, PlayState, QueueItem,
    TrackInfo,
};
use onsa_library::TrackRow;
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::dto::{QueuePlace, TrackDto};
use crate::error::ErrorCode;
use crate::session::{self, QueueState};
use crate::settings::{DspPrefs, EqKind, OutputPrefs, PlaybackPrefs, RepeatKind, RgMode};

/// Everything but the playhead changed.
pub const STATE_EVENT: &str = "player://state";
/// The playhead moved (at most ten times a second).
pub const POSITION_EVENT: &str = "player://position";
/// Peak meters (while playing).
pub const METER_EVENT: &str = "player://meter";
/// The queue changed; the interface reads it again.
pub const QUEUE_EVENT: &str = "player://queue";

/// Meter frames per second. The transport meters need no more.
const METER_FPS: u32 = 30;
/// How often the playhead is written down while a track plays (SPEC §13).
const REMEMBER_EVERY: Duration = Duration::from_secs(10);

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
    /// When the playhead was last written down.
    remembered: Mutex<Instant>,
}

struct OutputInfo {
    name: String,
    sample_rate: u32,
    channels: usize,
}

struct State {
    queue: Vec<TrackRow>,
    /// The order before shuffling, empty while shuffle is off.
    unshuffled: Vec<TrackRow>,
    shuffle: bool,
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
    /// Whether the queue is shuffled.
    pub shuffle: bool,
    /// What happens when a track ends.
    pub repeat: RepeatKind,
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
            remembered: Mutex::new(Instant::now()),
            engine: Mutex::new(None),
            state: Mutex::new(State {
                queue: Vec::new(),
                unshuffled: Vec::new(),
                shuffle: false,
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
        let rows = playable(rows);
        if rows.is_empty() {
            return Ok(());
        }
        let index = clicked
            .and_then(|id| rows.iter().position(|row| row.id == id))
            .unwrap_or(0);
        // With shuffle on, the chosen track plays first and the rest follow
        // in a random order; the order as listed is kept to go back to.
        let (rows, index, unshuffled) = if lock(&self.shared.state).shuffle {
            let mut rest = rows.clone();
            let chosen = rest.remove(index);
            shuffle_rows(&mut rest);
            let mut shuffled = Vec::with_capacity(rest.len() + 1);
            shuffled.push(chosen);
            shuffled.extend(rest);
            (shuffled, 0, rows)
        } else {
            (rows, index, Vec::new())
        };
        let items = queue_items(&rows);
        {
            let mut state = lock(&self.shared.state);
            state.one_album = queue_is_one_album(&items);
            state.duration = rows[index].duration_ms.map(|ms| ms as f64 / 1000.0);
            state.queue = rows;
            state.unshuffled = unshuffled;
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
        self.shared.emit_queue();
        result
    }

    /// Puts a stored queue back, paused where it was left (SPEC §13).
    pub fn restore(
        &self,
        rows: Vec<TrackRow>,
        current: usize,
        position: f64,
        shuffle: bool,
    ) -> Result<(), ErrorCode> {
        let rows = playable(rows);
        if rows.is_empty() {
            return Ok(());
        }
        let current = current.min(rows.len() - 1);
        let position = if position.is_finite() {
            position.max(0.0)
        } else {
            0.0
        };
        let items = queue_items(&rows);
        {
            let mut state = lock(&self.shared.state);
            state.one_album = queue_is_one_album(&items);
            state.shuffle = shuffle;
            state.duration = rows[current].duration_ms.map(|ms| ms as f64 / 1000.0);
            state.queue = rows;
            state.current = Some(current);
            state.position = position;
        }
        let result = self
            .shared
            .command(|engine| engine.set_queue(items, current, position, false));
        self.shared.emit_state();
        self.shared.emit_queue();
        ignore_no_output(result)
    }

    /// Adds tracks after the playing one or at the end, without disturbing
    /// playback. An empty queue simply starts playing them (SPEC §6.1).
    pub fn enqueue(&self, rows: Vec<TrackRow>, place: QueuePlace) -> Result<(), ErrorCode> {
        let rows = playable(rows);
        if rows.is_empty() {
            return Ok(());
        }
        let start_now = {
            let mut state = lock(&self.shared.state);
            if state.queue.is_empty() {
                true
            } else {
                let at = match place {
                    QueuePlace::Next => state.current.map_or(state.queue.len(), |at| at + 1),
                    QueuePlace::End => state.queue.len(),
                }
                .min(state.queue.len());
                if state.shuffle {
                    state.unshuffled.extend(rows.iter().cloned());
                }
                for (offset, row) in rows.iter().enumerate() {
                    state.queue.insert(at + offset, row.clone());
                }
                if let Some(current) = state.current.as_mut() {
                    if at <= *current {
                        *current += rows.len();
                    }
                }
                false
            }
        };
        if start_now {
            self.play(rows, 0)
        } else {
            self.push_queue()
        }
    }

    /// Removes one entry. Removing the track that is playing lets it finish
    /// and carries on from there.
    pub fn remove(&self, index: usize) -> Result<(), ErrorCode> {
        {
            let mut state = lock(&self.shared.state);
            if index >= state.queue.len() {
                return Ok(());
            }
            let gone = state.queue.remove(index);
            if let Some(at) = state.unshuffled.iter().position(|row| row.id == gone.id) {
                state.unshuffled.remove(at);
            }
            state.current = match state.current {
                _ if state.queue.is_empty() => None,
                Some(current) if index < current => Some(current - 1),
                Some(current) if index == current => Some(current.min(state.queue.len() - 1)),
                other => other,
            };
        }
        self.push_queue()
    }

    /// Moves an entry, as a drag in the queue panel does.
    pub fn move_entry(&self, from: usize, to: usize) -> Result<(), ErrorCode> {
        {
            let mut state = lock(&self.shared.state);
            let last = state.queue.len();
            if from >= last || to >= last || from == to {
                return Ok(());
            }
            let row = state.queue.remove(from);
            state.queue.insert(to, row);
            state.current = state.current.map(|current| moved_index(current, from, to));
        }
        self.push_queue()
    }

    /// Empties the queue and stops.
    pub fn clear(&self) -> Result<(), ErrorCode> {
        {
            let mut state = lock(&self.shared.state);
            state.queue.clear();
            state.unshuffled.clear();
            state.current = None;
            state.source = None;
            state.duration = None;
            state.position = 0.0;
        }
        let result = self.shared.command(Engine::stop);
        self.shared.emit_state();
        self.shared.emit_queue();
        ignore_no_output(result)
    }

    /// Shuffles the queue, or puts the listed order back. The playing track
    /// keeps playing either way (SPEC §6.1).
    pub fn set_shuffle(&self, shuffle: bool) -> Result<(), ErrorCode> {
        {
            let mut state = lock(&self.shared.state);
            if state.shuffle == shuffle {
                return Ok(());
            }
            state.shuffle = shuffle;
            if !state.queue.is_empty() {
                if shuffle {
                    state.unshuffled = state.queue.clone();
                    match state.current {
                        Some(current) if current < state.queue.len() => {
                            let playing = state.queue.remove(current);
                            shuffle_rows(&mut state.queue);
                            state.queue.insert(0, playing);
                            state.current = Some(0);
                        }
                        _ => shuffle_rows(&mut state.queue),
                    }
                } else {
                    let playing = state
                        .current
                        .and_then(|current| state.queue.get(current))
                        .map(|row| row.id);
                    let listed = std::mem::take(&mut state.unshuffled);
                    if !listed.is_empty() {
                        state.queue = listed;
                    }
                    state.current = playing
                        .and_then(|id| state.queue.iter().position(|row| row.id == id))
                        .or(state.current);
                }
            }
        }
        self.push_queue()
    }

    /// Hands the queue to the engine and tells the interface.
    fn push_queue(&self) -> Result<(), ErrorCode> {
        let (items, current) = {
            let mut state = lock(&self.shared.state);
            let items = queue_items(&state.queue);
            state.one_album = queue_is_one_album(&items);
            let current = state.current.unwrap_or(0);
            (items, current)
        };
        let result = self
            .shared
            .command(|engine| engine.update_queue(items, current));
        self.shared.emit_state();
        self.shared.emit_queue();
        ignore_no_output(result)
    }

    /// Pauses when playing, plays otherwise.
    pub fn toggle(&self) -> Result<(), ErrorCode> {
        self.shared.command(Engine::toggle_pause)
    }

    /// Pauses.
    pub fn pause(&self) -> Result<(), ErrorCode> {
        self.shared.command(Engine::pause)
    }

    /// Plays on from where it paused.
    pub fn resume(&self) -> Result<(), ErrorCode> {
        self.shared.command(Engine::resume)
    }

    /// Stops, keeping the queue.
    pub fn stop(&self) -> Result<(), ErrorCode> {
        self.shared.command(Engine::stop)
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

/// The library's values win over the file's tags: an edit kept in Onsa
/// counts for album gapless and ReplayGain too.
fn queue_items(rows: &[TrackRow]) -> Vec<QueueItem> {
    rows.iter()
        .map(|row| QueueItem {
            path: row.path.clone().into(),
            album: row.album.clone(),
            track_number: row.track_number.map(u64::from),
        })
        .collect()
}

/// Drops the tracks the library knows are gone.
fn playable(rows: Vec<TrackRow>) -> Vec<TrackRow> {
    rows.into_iter()
        .filter(|row| row.status != "missing")
        .collect()
}

/// Where `index` ends up once the entry at `from` moves to `to`.
fn moved_index(index: usize, from: usize, to: usize) -> usize {
    if index == from {
        to
    } else if from < index && index <= to {
        index - 1
    } else if to <= index && index < from {
        index + 1
    } else {
        index
    }
}

/// Fisher-Yates with a small generator seeded from the clock. Shuffling a
/// queue needs no cryptographic randomness, and this keeps the application
/// free of another dependency.
fn shuffle_rows(rows: &mut [TrackRow]) {
    let mut seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0x9E37_79B9_7F4A_7C15, |since| since.as_nanos() as u64 | 1);
    let mut next = move || {
        seed = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = seed;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    };
    for index in (1..rows.len()).rev() {
        rows.swap(index, (next() % (index as u64 + 1)) as usize);
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

    /// Writes the queue down, so it comes back when Onsa opens again.
    fn remember_queue(&self) {
        let session = {
            let state = lock(&self.state);
            QueueState {
                ids: state.queue.iter().map(|row| row.id).collect(),
                current: state.current.unwrap_or(0),
                shuffle: state.shuffle,
            }
        };
        session::save(&self.app, session::QUEUE_KEY, &session);
        self.remember_position(true);
    }

    /// Writes the playhead down, at most every [`REMEMBER_EVERY`] unless
    /// `now` says otherwise.
    fn remember_position(&self, now: bool) {
        {
            let mut last = lock(&self.remembered);
            if !now && last.elapsed() < REMEMBER_EVERY {
                return;
            }
            *last = Instant::now();
        }
        let position = lock(&self.state).position;
        session::save(&self.app, session::POSITION_KEY, &position);
    }

    fn emit_queue(&self) {
        if let Err(error) = self.app.emit(QUEUE_EVENT, ()) {
            tracing::debug!("queue change not delivered: {error}");
        }
        self.remember_queue();
    }

    fn emit_state(&self) {
        let snapshot = lock(&self.state).snapshot();
        crate::media::update(&self.app, &snapshot);
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
                self.remember_queue();
            }
            Event::Position { index, seconds } | Event::Seeked { index, seconds } => {
                lock(&self.state).position = seconds;
                let _ = self.app.emit(POSITION_EVENT, Position { index, seconds });
                self.remember_position(false);
            }
            Event::StateChanged(play_state) => {
                lock(&self.state).play_state = play_state;
                self.emit_state();
                self.remember_position(true);
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
            shuffle: self.shuffle,
            repeat: self.playback.repeat,
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

#[cfg(test)]
mod tests {
    use super::moved_index;

    #[test]
    fn a_moved_entry_carries_the_playhead_with_it() {
        // The playing entry itself follows the drag.
        assert_eq!(moved_index(2, 2, 5), 5);
        // Dragged from before the playing entry to after it.
        assert_eq!(moved_index(3, 1, 4), 2);
        // Dragged from after it to before it.
        assert_eq!(moved_index(3, 5, 1), 4);
        // Untouched on either side.
        assert_eq!(moved_index(1, 4, 6), 1);
        assert_eq!(moved_index(7, 4, 6), 7);
    }
}
