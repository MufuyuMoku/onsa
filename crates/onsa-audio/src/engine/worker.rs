//! The engine thread: decodes, resamples and mixes ahead of the device, and
//! handles every command (SPEC §3.1 "thread kontrol" and "thread decode").
//!
//! This thread is not real-time: it may allocate, log and block. It keeps
//! the ring buffer filled, and while playback is paused or stopped it sleeps
//! on the command channel, so an idle player costs no CPU (SPEC §13.1).

use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant};

use rtrb::Producer;

use super::{
    offline_output, Command, Event, OutputSettings, PlayState, PlaybackSettings, QueueId,
    QueueItem, RepeatMode,
};
use crate::analysis::{AnalysisSettings, AnalysisThread};
use crate::dsp::chain::{ChainParams, DspSettings};
use crate::dsp::replaygain::ReplayGainMode;
use crate::error::Result;
use crate::fade::MICRO_FADE_SECONDS;
use crate::lane::{Lane, LaneTrack};
use crate::output::device::{
    default_device_id, open_device, DeviceChoice, DeviceOutput, OutputRate,
};
use crate::output::stage::{DspPort, Shared};
use crate::source::{FileSource, TrackInfo};

/// Frames mixed and pushed per step.
const BLOCK_FRAMES: usize = 1024;
/// Longest wait for the output stage to finish a flush.
const FLUSH_TIMEOUT: Duration = Duration::from_secs(1);
/// Interval of position events (SPEC §2: at most 10 Hz).
const POSITION_INTERVAL: Duration = Duration::from_millis(100);
/// How often the system default device is checked while playing.
const DEFAULT_DEVICE_CHECK: Duration = Duration::from_secs(2);
/// Delay between attempts to reopen a lost output.
const OUTPUT_RETRY: Duration = Duration::from_millis(1000);
/// Past this point "previous" restarts the track instead of going back.
const PREVIOUS_RESTART_SECONDS: f64 = 3.0;
/// Time constant of a ReplayGain change between tracks.
const RG_SMOOTHING_SECONDS: f32 = 0.01;

/// Where the engine sends its audio.
pub(crate) enum OutputTarget {
    Device(OutputSettings),
    Offline,
}

/// An open output as the engine thread sees it.
pub(crate) struct Output {
    pub producer: Producer<f32>,
    pub shared: Arc<Shared>,
    pub sample_rate: u32,
    pub channels: usize,
    /// Keeps the device stream alive; `None` for the offline sink.
    pub device: Option<DeviceOutput>,
    pub name: String,
    /// The engine's end of the output's DSP plumbing.
    pub dsp: DspPort,
}

/// Where a pushed block of audio came from.
#[derive(Debug, Clone, Copy)]
struct Entry {
    out_start: u64,
    queue_index: usize,
    track_frame: u64,
    src_rate: u32,
}

/// A crossfade in progress.
#[derive(Debug, Clone, Copy)]
struct Fade {
    total: u64,
    done: u64,
}

/// The playhead.
#[derive(Debug, Clone, Copy)]
struct Playhead {
    queue_index: usize,
    frame: u64,
    src_rate: u32,
}

impl Playhead {
    fn seconds(self) -> f64 {
        self.frame as f64 / f64::from(self.src_rate)
    }
}

pub(crate) struct Worker {
    commands: Receiver<Command>,
    self_tx: Sender<Command>,
    events: Sender<Event>,
    settings: PlaybackSettings,
    target: OutputTarget,
    output: Option<Output>,
    queue: Vec<QueueItem>,
    infos: HashMap<usize, (PathBuf, TrackInfo)>,
    state: PlayState,
    /// The lane being heard (the incoming side of a crossfade).
    primary: Option<Lane>,
    /// The lane fading out.
    outgoing: Option<Lane>,
    fade: Option<Fade>,
    /// The next track, opened ahead of time (pre-roll).
    next: Option<LaneTrack>,
    /// Frames pushed into the current output.
    pushed: u64,
    timeline: VecDeque<Entry>,
    /// Track index the listener was last told about.
    reported: Option<usize>,
    /// Track index playback is on, for skips and seeks.
    current: usize,
    last_position: Instant,
    last_default_check: Instant,
    retry_at: Option<Instant>,
    /// Where to resume once a lost output is back.
    resume_at: Option<Playhead>,
    mix_a: Vec<f32>,
    mix_b: Vec<f32>,
    /// The last fill stopped on its per-round budget, not on a full buffer.
    more_to_fill: bool,
    /// DSP settings; the callback side reaches the output as ChainParams.
    dsp: DspSettings,
    /// ChainParams that did not fit in the output's queue yet.
    pending_params: Option<ChainParams>,
    /// Whether the queue is one album in order (ReplayGain auto mode).
    queue_is_one_album: bool,
    /// Rate of the track being played, for the match-source rate mode.
    source_rate: Option<u32>,
    analysis: AnalysisSettings,
    analysis_thread: Option<AnalysisThread>,
}

impl Worker {
    pub fn new(
        commands: Receiver<Command>,
        self_tx: Sender<Command>,
        events: Sender<Event>,
        settings: PlaybackSettings,
        target: OutputTarget,
    ) -> Self {
        let now = Instant::now();
        Self {
            commands,
            self_tx,
            events,
            settings,
            target,
            output: None,
            queue: Vec::new(),
            infos: HashMap::new(),
            state: PlayState::Stopped,
            primary: None,
            outgoing: None,
            fade: None,
            next: None,
            pushed: 0,
            timeline: VecDeque::new(),
            reported: None,
            current: 0,
            last_position: now,
            last_default_check: now,
            retry_at: None,
            resume_at: None,
            mix_a: Vec::new(),
            mix_b: Vec::new(),
            more_to_fill: false,
            dsp: DspSettings::default(),
            pending_params: None,
            queue_is_one_album: false,
            source_rate: None,
            analysis: AnalysisSettings::default(),
            analysis_thread: None,
        }
    }

    fn emit(&self, event: Event) {
        // A caller that stopped listening is not an error.
        let _ = self.events.send(event);
    }

    // ---------------------------------------------------------------- output

    /// Opens the device described by the target.
    pub fn open_device_output(&mut self) -> Result<()> {
        let OutputTarget::Device(settings) = &self.target else {
            return Ok(());
        };
        // Match-source opens the device at the rate of the track being
        // played, when it knows one (SPEC §3.4).
        let rate = match settings.rate {
            OutputRate::MatchSource => match self.source_rate {
                Some(rate) => OutputRate::Fixed(rate),
                None => OutputRate::FollowDevice,
            },
            other => other,
        };
        let shared = Arc::new(Shared::default());
        let fault_tx = self.self_tx.clone();
        let dsp = self.dsp.clone();
        let (device, producer, port) = open_device(
            &settings.device,
            rate,
            self.settings.buffer.seconds(),
            |rate| dsp.chain_params(rate),
            shared.clone(),
            move |fault| {
                let _ = fault_tx.send(Command::DeviceFault(fault));
            },
        )?;
        tracing::info!(
            device = %device.name,
            rate = device.sample_rate,
            channels = device.channels,
            format = %device.sample_format,
            "output opened"
        );
        let output = Output {
            producer,
            shared,
            sample_rate: device.sample_rate,
            channels: device.channels,
            name: device.name.clone(),
            device: Some(device),
            dsp: port,
        };
        self.attach_output(output);
        Ok(())
    }

    /// Takes a freshly opened output into use.
    pub fn attach_output(&mut self, output: Output) {
        self.emit(Event::OutputOpened {
            name: output.name.clone(),
            sample_rate: output.sample_rate,
            channels: output.channels,
        });
        output
            .shared
            .paused
            .store(self.state == PlayState::Paused, Ordering::Release);
        self.output = Some(output);
        self.pushed = 0;
        self.timeline.clear();
        self.last_default_check = Instant::now();
        // The new stage was built with the current settings already.
        self.pending_params = None;
        self.restart_analysis();
    }

    /// Drops the output after a fault and remembers where playback was.
    fn lose_output(&mut self, reason: String) {
        if self.output.is_none() {
            return;
        }
        tracing::warn!("output lost: {reason}");
        self.resume_at = self.playhead().or(self.resume_at);
        self.drop_lanes();
        self.analysis_thread = None;
        self.output = None;
        self.retry_at = Some(Instant::now());
        self.emit(Event::OutputLost { reason });
    }

    /// Moves to another device or rate; playback resumes where it was.
    fn switch_output(&mut self, settings: OutputSettings) {
        let OutputTarget::Device(current) = &self.target else {
            return;
        };
        if *current == settings {
            return;
        }
        tracing::info!(?settings, "output settings changed, reopening the output");
        self.target = OutputTarget::Device(settings);
        self.reopen_output();
    }

    /// Opens the output again, picking playback up where it was.
    fn reopen_output(&mut self) {
        if self.output.is_some() {
            self.resume_at = self.playhead().or(self.resume_at);
            // Fade out on the old device before it closes.
            self.flush();
            self.drop_lanes();
            self.analysis_thread = None;
            self.output = None;
        }
        self.retry_at = Some(Instant::now());
        self.maintain_output();
    }

    /// Picks playback up after the output changed. Without a playhead the
    /// output was replaced before anything was heard (the match-source mode
    /// does that at the start of a track), so the track starts from its
    /// beginning rather than the queue ending.
    fn resume_after_output(&mut self, at: Option<Playhead>) {
        match at {
            Some(at) => self.restart_at(at.queue_index, at.frame),
            None if self.state == PlayState::Playing
                && self.primary.is_none()
                && self.current < self.queue.len() =>
            {
                self.start_from(self.current, 0.0);
            }
            None => {}
        }
    }

    /// Follows a track's own sample rate, when that mode is on (SPEC §3.4).
    fn match_rate(&mut self, rate: u32) {
        self.source_rate = Some(rate);
        if self
            .output
            .as_ref()
            .is_some_and(|output| output.sample_rate != rate)
        {
            tracing::info!(rate, "following the track's sample rate");
            self.reopen_output();
        }
    }

    /// Reopens a lost output, and follows the system default device.
    fn maintain_output(&mut self) {
        let OutputTarget::Device(settings) = &self.target else {
            return;
        };
        let follows_default = settings.device == DeviceChoice::SystemDefault;

        if self.output.is_none() {
            let due = self.retry_at.is_some_and(|at| Instant::now() >= at);
            if !due {
                return;
            }
            match self.open_device_output() {
                Ok(()) => {
                    self.retry_at = None;
                    let at = self.resume_at.take();
                    self.resume_after_output(at);
                }
                Err(error) => {
                    tracing::debug!("output still unavailable: {error}");
                    self.retry_at = Some(Instant::now() + OUTPUT_RETRY);
                }
            }
            return;
        }

        if follows_default
            && self.state == PlayState::Playing
            && self.last_default_check.elapsed() >= DEFAULT_DEVICE_CHECK
        {
            self.last_default_check = Instant::now();
            let current = self
                .output
                .as_ref()
                .and_then(|output| output.device.as_ref())
                .and_then(|device| device.id.clone());
            let default = default_device_id();
            if default.is_some() && current.is_some() && default != current {
                tracing::info!("system default output changed, following it");
                self.lose_output("default output device changed".to_string());
                self.maintain_output();
            }
        }
    }

    // ----------------------------------------------------------------- loop

    pub fn run(&mut self) {
        loop {
            let command = match self.wait_time() {
                None => match self.commands.recv() {
                    Ok(command) => Some(command),
                    Err(_) => return,
                },
                Some(timeout) => match self.commands.recv_timeout(timeout) {
                    Ok(command) => Some(command),
                    Err(RecvTimeoutError::Timeout) => None,
                    Err(RecvTimeoutError::Disconnected) => return,
                },
            };
            if let Some(command) = command {
                if !self.handle(command) {
                    return;
                }
                while let Ok(command) = self.commands.try_recv() {
                    if !self.handle(command) {
                        return;
                    }
                }
            }
            self.maintain_output();
            self.flush_params();
            self.fill();
            self.report();
        }
    }

    /// How long to sleep before the next round; `None` sleeps until a
    /// command arrives.
    fn wait_time(&self) -> Option<Duration> {
        if self.more_to_fill && self.output.is_some() {
            return Some(Duration::ZERO);
        }
        if self.pending_params.is_some() && self.output.is_some() {
            // Settings are waiting for room in the output's queue.
            return Some(Duration::from_millis(5));
        }
        if self.output.is_none() {
            return self
                .retry_at
                .map(|at| at.saturating_duration_since(Instant::now()));
        }
        match self.state {
            PlayState::Playing => {
                let quarter = Duration::from_secs_f32(self.settings.buffer.seconds() / 4.0);
                Some(quarter.clamp(Duration::from_millis(5), Duration::from_millis(50)))
            }
            PlayState::Paused | PlayState::Stopped => None,
        }
    }

    /// Returns false when the engine should shut down.
    fn handle(&mut self, command: Command) -> bool {
        match command {
            Command::SetQueue {
                items,
                start,
                start_seconds,
                play,
            } => {
                self.flush();
                self.drop_lanes();
                self.queue = items;
                self.queue_is_one_album = queue_is_one_album(&self.queue);
                self.infos.clear();
                self.reported = None;
                self.set_state(if play {
                    PlayState::Playing
                } else {
                    PlayState::Paused
                });
                self.start_from(start, start_seconds);
            }
            Command::UpdateQueue { items } => self.update_queue(items),
            Command::Jump(index) => self.skip_to(index, true),
            Command::Next => self.skip_to(self.after(self.current), true),
            Command::Previous => {
                let position = self.playhead().map(Playhead::seconds).unwrap_or(0.0);
                if position > PREVIOUS_RESTART_SECONDS {
                    self.seek(0.0);
                } else if self.current > 0 {
                    self.skip_to(self.current - 1, false);
                } else if self.settings.repeat == RepeatMode::All && !self.queue.is_empty() {
                    self.skip_to(self.queue.len() - 1, false);
                } else {
                    self.seek(0.0);
                }
            }
            Command::Pause => {
                if self.state == PlayState::Playing {
                    self.set_state(PlayState::Paused);
                }
            }
            Command::Resume => self.resume(),
            Command::TogglePause => match self.state {
                PlayState::Playing => self.set_state(PlayState::Paused),
                _ => self.resume(),
            },
            Command::Stop => {
                self.flush();
                self.drop_lanes();
                self.set_state(PlayState::Stopped);
            }
            Command::Seek(seconds) => self.seek(seconds),
            Command::SetSettings(settings) => self.settings = settings,
            Command::SetDsp(settings) => {
                self.dsp = settings;
                self.send_params();
            }
            Command::SetAnalysis(settings) => {
                self.analysis = settings;
                self.restart_analysis();
            }
            Command::DeviceFault(fault) => {
                if fault.lost {
                    self.lose_output(fault.message);
                } else {
                    tracing::debug!("output notice: {}", fault.message);
                    // The backend rerouted the stream itself; remember the
                    // new default so it is not "followed" a second time.
                    self.last_default_check = Instant::now();
                }
            }
            Command::SetOutput(settings) => self.switch_output(settings),
            Command::MatchRate(rate) => self.match_rate(rate),
            Command::ReplaceOffline {
                sample_rate,
                channels,
                reply,
            } => {
                let at = self.playhead();
                self.drop_lanes();
                self.analysis_thread = None;
                let params = self.dsp.chain_params(sample_rate);
                let (output, sink) =
                    offline_output(sample_rate, channels, self.settings.buffer, params);
                self.attach_output(output);
                self.resume_after_output(at);
                let _ = reply.send(sink);
            }
            Command::Shutdown => {
                self.flush();
                return false;
            }
        }
        true
    }

    fn set_state(&mut self, state: PlayState) {
        if let Some(output) = &self.output {
            output
                .shared
                .paused
                .store(state == PlayState::Paused, Ordering::Release);
        }
        if self.state != state {
            self.state = state;
            self.emit(Event::StateChanged(state));
        }
        self.update_analysis_activity();
    }

    // ------------------------------------------------------------------ dsp

    /// Sends the callback-side DSP settings to the output.
    fn send_params(&mut self) {
        if let Some(output) = &self.output {
            self.pending_params = Some(self.dsp.chain_params(output.sample_rate));
        }
        self.flush_params();
    }

    /// Delivers settings that are waiting for room in the output's queue.
    fn flush_params(&mut self) {
        let Some(params) = self.pending_params else {
            return;
        };
        let Some(output) = self.output.as_mut() else {
            return;
        };
        if output.dsp.updates.push(params).is_ok() {
            self.pending_params = None;
        }
    }

    /// ReplayGain for a queue entry, as a linear gain (SPEC §4.1).
    fn rg_gain(&self, index: usize) -> f32 {
        let settings = &self.dsp.replaygain;
        if settings.mode == ReplayGainMode::Off {
            return 1.0;
        }
        let tags = self
            .infos
            .get(&index)
            .map(|(_, info)| info.replaygain)
            .unwrap_or_default();
        settings.gain(&tags, settings.uses_album(self.queue_is_one_album))
    }

    fn rg_coef(&self) -> f32 {
        let rate = self
            .output
            .as_ref()
            .map_or(48_000, |output| output.sample_rate) as f32;
        1.0 - (-1.0 / (RG_SMOOTHING_SECONDS * rate).max(1.0)).exp()
    }

    /// (Re)starts the analysis thread on the current output's tap.
    fn restart_analysis(&mut self) {
        let old_tap = self.analysis_thread.take().and_then(AnalysisThread::stop);
        let Some(output) = self.output.as_mut() else {
            return;
        };
        let tap = old_tap.or_else(|| output.dsp.tap.take());
        if !self.analysis.enabled {
            output.dsp.tap = tap;
            self.update_analysis_activity();
            return;
        }
        let Some(tap) = tap else {
            return;
        };
        let shared = output.shared.clone();
        let events = self.events.clone();
        self.analysis_thread = Some(AnalysisThread::spawn(
            tap,
            output.sample_rate,
            output.channels,
            self.analysis,
            move || shared.limiting.swap(false, Ordering::Relaxed),
            move |frame| {
                let _ = events.send(Event::Analysis(frame));
            },
        ));
        self.update_analysis_activity();
    }

    /// Analysis runs only while it is on and audio is playing: a paused or
    /// stopped player does no analysis work (SPEC §13.1).
    fn update_analysis_activity(&self) {
        let active =
            self.analysis.enabled && self.state == PlayState::Playing && self.output.is_some();
        if let Some(output) = &self.output {
            output.shared.analysis.store(active, Ordering::Release);
        }
        if let Some(thread) = &self.analysis_thread {
            thread.set_active(active);
        }
    }

    fn resume(&mut self) {
        match self.state {
            PlayState::Paused => self.set_state(PlayState::Playing),
            PlayState::Stopped if !self.queue.is_empty() => {
                self.set_state(PlayState::Playing);
                let start = if self.current < self.queue.len() {
                    self.current
                } else {
                    0
                };
                self.reported = None;
                self.start_from(start, 0.0);
            }
            _ => {}
        }
    }

    // ------------------------------------------------------------- playback

    /// Asks the output stage to fade out and discard what is buffered, and
    /// waits until it has.
    fn flush(&mut self) {
        let Some(output) = &self.output else {
            return;
        };
        let shared = &output.shared;
        if self.pushed <= shared.consumed.load(Ordering::Acquire) {
            // Nothing is waiting in the ring buffer: there is nothing to
            // discard, and nobody may be listening yet to confirm it.
            self.timeline.clear();
            return;
        }
        shared.flush_request.fetch_add(1, Ordering::AcqRel);
        let deadline = Instant::now() + FLUSH_TIMEOUT;
        while shared.flush_pending() {
            if Instant::now() >= deadline {
                tracing::warn!("output did not finish the flush in time");
                break;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        self.pushed = shared.consumed.load(Ordering::Acquire);
        self.timeline.clear();
    }

    fn drop_lanes(&mut self) {
        self.primary = None;
        self.outgoing = None;
        self.fade = None;
        self.next = None;
        if let Some(output) = &self.output {
            output.shared.producing.store(false, Ordering::Release);
        }
    }

    /// Opens the first playable track from `index` on, reporting the ones
    /// that fail.
    fn open_from(&mut self, mut index: usize) -> Option<LaneTrack> {
        let channels = self.output.as_ref()?.channels;
        while index < self.queue.len() {
            let path = self.queue[index].path.clone();
            match FileSource::open(&path, channels) {
                Ok(source) => {
                    self.infos.insert(index, (path, source.info().clone()));
                    return Some(LaneTrack {
                        queue_index: index,
                        source,
                    });
                }
                Err(error) => {
                    tracing::warn!(path = %path.display(), "cannot open track: {error}");
                    self.emit(Event::TrackFailed {
                        index,
                        path,
                        reason: error.to_string(),
                    });
                    index += 1;
                }
            }
        }
        None
    }

    /// Replaces the queue while playback carries on (SPEC §6.1).
    ///
    /// Queue positions mean nothing across an edit, so every position the
    /// engine remembers is placed again by the [`QueueId`] of the entry it
    /// pointed at. As long as the entries being played are still there, the
    /// listener hears no break. When one of them is gone, the engine cannot
    /// keep playing it: it fades out what is buffered and carries on from
    /// the first surviving entry after it, or stops when there is none.
    fn update_queue(&mut self, items: Vec<QueueItem>) {
        let places: HashMap<QueueId, usize> = items
            .iter()
            .enumerate()
            .map(|(index, item)| (item.id, index))
            .collect();
        // Where every old position went, by index: `None` means removed.
        let moved: Vec<Option<usize>> = self
            .queue
            .iter()
            .map(|item| places.get(&item.id).copied())
            .collect();
        let place = |index: usize| moved.get(index).copied().flatten();

        let old_current = self.current;
        let at = self.playhead();
        let audible = at.map(|at| at.queue_index);
        let mut playing: Vec<usize> = self
            .timeline
            .iter()
            .map(|entry| entry.queue_index)
            .collect();
        for lane in [self.primary.as_ref(), self.outgoing.as_ref()]
            .into_iter()
            .flatten()
        {
            playing.extend(lane.queue_positions());
        }
        let intact = playing.iter().all(|&index| place(index).is_some());

        self.queue = items;
        self.queue_is_one_album = queue_is_one_album(&self.queue);
        // What comes next may have changed; it is opened again when needed.
        self.next = None;

        if intact {
            for entry in &mut self.timeline {
                if let Some(to) = place(entry.queue_index) {
                    entry.queue_index = to;
                }
            }
            for lane in [self.primary.as_mut(), self.outgoing.as_mut()]
                .into_iter()
                .flatten()
            {
                lane.remap(place);
            }
            self.infos = std::mem::take(&mut self.infos)
                .into_iter()
                .filter_map(|(index, info)| place(index).map(|to| (to, info)))
                .collect();
            // Keeping this pointing at the audible track stops the listener
            // being told a second time that it started.
            self.reported = self
                .reported
                .and_then(place)
                .or_else(|| audible.and_then(place));
            self.current = place(old_current)
                .or_else(|| self.surviving_from(old_current, &moved))
                .unwrap_or(self.queue.len());
            return;
        }

        // A track being heard, or one already decoded ahead of it, has left
        // the queue. What is buffered cannot stand, so it is faded out and
        // dropped, exactly as a seek does, and playback set up again.
        let playing = self.state != PlayState::Stopped;
        self.infos.clear();
        self.flush();
        self.drop_lanes();
        self.reported = None;

        // The track being heard may itself have survived: only what was
        // decoded behind it went. Then it carries on from where the listener
        // had reached, rather than starting over.
        if let Some((index, frame)) =
            at.and_then(|at| place(at.queue_index).map(|to| (to, at.frame)))
        {
            self.current = index;
            if playing {
                self.restart_at(index, frame);
            }
            return;
        }
        match self.surviving_from(old_current, &moved) {
            Some(index) => {
                self.current = index;
                if playing {
                    self.start_from(index, 0.0);
                }
            }
            None => {
                self.current = self.queue.len();
                if playing {
                    self.set_state(PlayState::Stopped);
                    self.emit(Event::QueueEnded);
                }
            }
        }
    }

    /// Where playback goes when the entry it was on is removed: the first
    /// entry that followed it and is still in the queue. With the whole
    /// queue repeating, it wraps round to the first entry left.
    fn surviving_from(&self, from: usize, moved: &[Option<usize>]) -> Option<usize> {
        moved
            .iter()
            .skip(from)
            .flatten()
            .copied()
            .next()
            .or(match self.settings.repeat {
                RepeatMode::All if !self.queue.is_empty() => Some(0),
                _ => None,
            })
    }

    /// The track to play after `index`, following the repeat mode. `None`
    /// ends the queue. Repeat One returns the same index, so the track is
    /// pre-rolled again and loops without a gap (SPEC §6.1).
    fn following(&self, index: usize) -> Option<usize> {
        if self.queue.is_empty() {
            return None;
        }
        match self.settings.repeat {
            RepeatMode::One => Some(index),
            RepeatMode::All => Some((index + 1) % self.queue.len()),
            RepeatMode::Off => (index + 1 < self.queue.len()).then_some(index + 1),
        }
    }

    /// Where a manual skip forward goes. Unlike [`Worker::following`], repeat
    /// One still moves on: the listener asked for the next track.
    fn after(&self, index: usize) -> usize {
        match self.settings.repeat {
            RepeatMode::All if !self.queue.is_empty() => (index + 1) % self.queue.len(),
            _ => index + 1,
        }
    }

    /// Starts a fresh lane at `index`, `seconds` in.
    fn start_from(&mut self, index: usize, seconds: f64) {
        let Some(mut track) = self.open_from(index) else {
            self.drop_lanes();
            self.set_state(PlayState::Stopped);
            self.emit(Event::QueueEnded);
            return;
        };
        if seconds > 0.0 {
            let rate = f64::from(track.source.info().sample_rate);
            if let Err(error) = track.source.seek((seconds * rate) as u64) {
                tracing::warn!("cannot start mid-track: {error}");
            }
        }
        self.current = track.queue_index;
        self.begin_lane(track);
    }

    /// Restarts playback at a known playhead, after an output change.
    fn restart_at(&mut self, index: usize, frame: u64) {
        let Some(mut track) = self.open_from(index) else {
            return;
        };
        if track.queue_index == index {
            if let Err(error) = track.source.seek(frame) {
                tracing::warn!("cannot restore the position: {error}");
            }
        }
        self.current = track.queue_index;
        self.begin_lane(track);
    }

    fn begin_lane(&mut self, track: LaneTrack) {
        // In match-source mode a track at another rate reopens the output.
        // It goes through the command channel, so nothing recurses here.
        if let OutputTarget::Device(settings) = &self.target {
            let rate = track.source.info().sample_rate;
            if settings.rate == OutputRate::MatchSource
                && self
                    .output
                    .as_ref()
                    .is_some_and(|output| output.sample_rate != rate)
            {
                let _ = self.self_tx.send(Command::MatchRate(rate));
            }
        }
        let Some(output) = &self.output else {
            return;
        };
        match Lane::new(
            track,
            output.sample_rate,
            output.channels,
            self.settings.quality,
        ) {
            Ok(lane) => {
                self.primary = Some(lane);
                output.shared.producing.store(true, Ordering::Release);
            }
            Err(error) => tracing::error!("cannot start playback: {error}"),
        }
    }

    /// Jumps to another track. While playing this crossfades in the stream
    /// (the manual skip crossfade); otherwise it flushes and starts clean.
    fn skip_to(&mut self, index: usize, forward: bool) {
        if index >= self.queue.len() {
            if forward {
                self.flush();
                self.drop_lanes();
                self.set_state(PlayState::Stopped);
                self.emit(Event::QueueEnded);
            }
            return;
        }
        if self.state != PlayState::Playing || self.output.is_none() {
            self.flush();
            self.drop_lanes();
            if self.state == PlayState::Stopped {
                self.set_state(PlayState::Playing);
            }
            self.start_from(index, 0.0);
            return;
        }

        let Some(track) = self.open_from(index) else {
            return;
        };
        let Some(output) = &self.output else {
            return;
        };
        self.current = track.queue_index;
        self.next = None;
        let seconds = self.settings.skip_crossfade_seconds.max(MICRO_FADE_SECONDS);
        let total = (seconds * output.sample_rate as f32) as u64;
        let old = self.primary.take();
        self.begin_lane(track);
        self.outgoing = old;
        self.fade = self.outgoing.as_ref().map(|_| Fade {
            total: total.max(1),
            done: 0,
        });
    }

    fn seek(&mut self, seconds: f64) {
        let index = self
            .playhead()
            .map(|at| at.queue_index)
            .unwrap_or(self.current);
        if index >= self.queue.len() {
            return;
        }
        self.flush();
        self.drop_lanes();
        if self.state == PlayState::Stopped {
            self.set_state(PlayState::Paused);
        }
        let seconds = seconds.max(0.0);
        self.start_from(index, seconds);
        if self.primary.is_some() {
            self.emit(Event::Seeked {
                index: self.current,
                seconds,
            });
        }
    }

    // ----------------------------------------------------------------- mix

    /// Crossfade length for the transition from `from` to `to`, in seconds.
    fn crossfade_for(
        &self,
        from: &FileSource,
        from_index: usize,
        to: &FileSource,
        to_index: usize,
    ) -> f32 {
        let seconds = self.settings.crossfade_seconds.clamp(0.0, 12.0);
        if seconds <= 0.0 {
            return 0.0;
        }
        if self.settings.album_gapless && self.consecutive_album(from, from_index, to, to_index) {
            return 0.0;
        }
        seconds
    }

    fn consecutive_album(
        &self,
        from: &FileSource,
        from_index: usize,
        to: &FileSource,
        to_index: usize,
    ) -> bool {
        let album = |source: &FileSource, index: usize| {
            self.queue
                .get(index)
                .and_then(|item| item.album.clone())
                .or_else(|| source.info().album.clone())
                .map(|album| album.trim().to_lowercase())
                .filter(|album| !album.is_empty())
        };
        let number = |source: &FileSource, index: usize| {
            self.queue
                .get(index)
                .and_then(|item| item.track_number)
                .or(source.info().track_number)
        };
        match (album(from, from_index), album(to, to_index)) {
            (Some(a), Some(b)) if a == b => matches!(
                (number(from, from_index), number(to, to_index)),
                (Some(a), Some(b)) if b == a + 1
            ),
            _ => false,
        }
    }

    /// Opens the track after the lane's last one, and joins it gaplessly
    /// when no crossfade is due and the rates match.
    fn preroll(&mut self) {
        if self.next.is_some() {
            return;
        }
        let Some(primary) = &self.primary else {
            return;
        };
        if primary.tracks_waiting() > 0 {
            return;
        }
        let Some(last) = primary.last_queue_index() else {
            return;
        };
        let Some(upcoming) = self.following(last) else {
            return;
        };
        let Some(track) = self.open_from(upcoming) else {
            return;
        };
        let Some(primary) = &self.primary else {
            return;
        };
        let crossfade = primary
            .last_track()
            .map(|from| self.crossfade_for(from, last, &track.source, track.queue_index))
            .unwrap_or(0.0);
        if crossfade <= 0.0 && primary.accepts(&track.source) {
            if let Some(primary) = self.primary.as_mut() {
                primary.append(track);
            }
        } else {
            self.next = Some(track);
        }
    }

    /// Starts the crossfade into the pre-rolled track when the current lane
    /// is close enough to its end.
    fn maybe_crossfade(&mut self) {
        if self.outgoing.is_some() {
            return;
        }
        let (Some(primary), Some(next), Some(output)) = (&self.primary, &self.next, &self.output)
        else {
            return;
        };
        let Some(from) = primary.last_track() else {
            return;
        };
        let Some(from_index) = primary.last_queue_index() else {
            return;
        };
        let seconds = self.crossfade_for(from, from_index, &next.source, next.queue_index);
        if seconds <= 0.0 {
            return;
        }
        // Without a known length the end cannot be anticipated; the tracks
        // then follow each other without a crossfade.
        let Some(remaining) = primary.remaining_frames() else {
            return;
        };
        let total = (seconds * output.sample_rate as f32) as u64;
        if remaining > total {
            return;
        }
        let Some(track) = self.next.take() else {
            return;
        };
        let old = self.primary.take();
        self.begin_lane(track);
        self.outgoing = old;
        self.fade = Some(Fade {
            total: remaining.max(1),
            done: 0,
        });
    }

    /// Keeps the ring buffer full.
    fn fill(&mut self) {
        self.more_to_fill = false;
        let mut filled = 0usize;
        if self.state == PlayState::Stopped {
            return;
        }
        loop {
            let Some(output) = &self.output else {
                return;
            };
            let channels = output.channels;
            let capacity_frames = output.producer.buffer().capacity() / channels;
            if filled >= capacity_frames {
                self.more_to_fill = true;
                return;
            }
            let step = BLOCK_FRAMES.min(capacity_frames);
            if output.producer.slots() / channels < step {
                return;
            }

            self.preroll();
            self.maybe_crossfade();
            let Some((frames, entry)) = self.render_block(step) else {
                if let Some(output) = &self.output {
                    output.shared.producing.store(false, Ordering::Release);
                }
                return;
            };
            let Some(output) = self.output.as_mut() else {
                return;
            };
            let samples = &self.mix_a[..frames * channels];
            if output.producer.push_entire_slice(samples).is_err() {
                tracing::error!("ring buffer refused a block it had room for");
                return;
            }
            self.timeline.push_back(Entry {
                out_start: self.pushed,
                ..entry
            });
            self.pushed += frames as u64;
            filled += frames;
        }
    }

    /// Mixes one block into `mix_a`. Returns `None` when there is nothing
    /// left to play.
    fn render_block(&mut self, frames: usize) -> Option<(usize, Entry)> {
        let channels = self.output.as_ref()?.channels;
        self.mix_a.resize(frames * channels, 0.0);
        self.mix_b.resize(frames * channels, 0.0);

        let block = loop {
            let primary = self.primary.as_mut()?;
            if let Some(block) = primary.read(&mut self.mix_a[..frames * channels]) {
                break block;
            }
            self.report_failures();
            self.primary = None;
            match self.next.take() {
                Some(track) => self.begin_lane(track),
                None => {
                    self.outgoing = None;
                    self.fade = None;
                    return None;
                }
            }
        };
        self.report_failures();

        let src_rate = self
            .infos
            .get(&block.queue_index)
            .map(|(_, info)| info.sample_rate)
            .unwrap_or(1);
        let count = block.frames;

        // ReplayGain is per track, so it is applied here, per block (SPEC §3.1).
        let coef = self.rg_coef();
        let target = self.rg_gain(block.queue_index);
        if let Some(primary) = self.primary.as_mut() {
            apply_gain(
                &mut self.mix_a[..count * channels],
                channels,
                &mut primary.rg_gain,
                target,
                coef,
            );
        }
        let outgoing_target = self
            .outgoing
            .as_ref()
            .and_then(Lane::last_queue_index)
            .map_or(1.0, |index| self.rg_gain(index));

        if let (Some(outgoing), Some(fade)) = (self.outgoing.as_mut(), self.fade.as_mut()) {
            let mut filled = 0;
            while filled < count {
                match outgoing.read(&mut self.mix_b[filled * channels..count * channels]) {
                    Some(part) => filled += part.frames,
                    None => break,
                }
            }
            self.mix_b[filled * channels..count * channels].fill(0.0);
            apply_gain(
                &mut self.mix_b[..count * channels],
                channels,
                &mut outgoing.rg_gain,
                outgoing_target,
                coef,
            );
            for frame in 0..count {
                let t = (fade.done + frame as u64) as f32 / fade.total as f32;
                let (gain_out, gain_in) = self.settings.curve.gains(t);
                for channel in 0..channels {
                    let i = frame * channels + channel;
                    self.mix_a[i] = self.mix_a[i] * gain_in + self.mix_b[i] * gain_out;
                }
            }
            fade.done += count as u64;
            if fade.done >= fade.total {
                self.outgoing = None;
                self.fade = None;
            }
        }

        Some((
            count,
            Entry {
                out_start: 0,
                queue_index: block.queue_index,
                track_frame: block.track_frame,
                src_rate,
            },
        ))
    }

    fn report_failures(&mut self) {
        let failures = self
            .primary
            .as_mut()
            .map(Lane::take_failures)
            .unwrap_or_default();
        for failure in failures {
            let path = self
                .queue
                .get(failure.queue_index)
                .map(|item| item.path.clone())
                .unwrap_or_default();
            self.emit(Event::TrackFailed {
                index: failure.queue_index,
                path,
                reason: failure.error.to_string(),
            });
        }
    }

    // -------------------------------------------------------------- events

    /// Where the listener is right now.
    fn playhead(&self) -> Option<Playhead> {
        let output = self.output.as_ref()?;
        let consumed = output.shared.consumed.load(Ordering::Acquire);
        let entry = self
            .timeline
            .iter()
            .rev()
            .find(|entry| entry.out_start <= consumed)
            .or(self.timeline.front())?;
        let offset = consumed.saturating_sub(entry.out_start) as f64 * f64::from(entry.src_rate)
            / f64::from(output.sample_rate);
        Some(Playhead {
            queue_index: entry.queue_index,
            frame: entry.track_frame + offset as u64,
            src_rate: entry.src_rate,
        })
    }

    fn report(&mut self) {
        let Some(output) = &self.output else {
            return;
        };
        let consumed = output.shared.consumed.load(Ordering::Acquire);
        let producing = output.shared.producing.load(Ordering::Acquire);

        if let Some(at) = self.playhead() {
            // Every track the listener has reached since the last report, in
            // order, even one shorter than a round of the loop.
            let mut last = self.reported;
            let mut started = Vec::new();
            for entry in self
                .timeline
                .iter()
                .take_while(|entry| entry.out_start <= consumed)
            {
                if last != Some(entry.queue_index) {
                    last = Some(entry.queue_index);
                    started.push(entry.queue_index);
                }
            }
            if started.is_empty() && self.reported.is_none() {
                started.push(at.queue_index);
            }
            for index in started {
                self.reported = Some(index);
                self.current = index;
                if let (Some((path, info)), Some(item)) =
                    (self.infos.get(&index), self.queue.get(index))
                {
                    self.emit(Event::TrackStarted {
                        index,
                        id: item.id,
                        path: path.clone(),
                        info: info.clone(),
                    });
                }
            }
            if self.state == PlayState::Playing && self.last_position.elapsed() >= POSITION_INTERVAL
            {
                self.last_position = Instant::now();
                self.emit(Event::Position {
                    index: at.queue_index,
                    seconds: at.seconds(),
                });
            }
        }

        // Forget blocks the listener is past.
        while self.timeline.len() > 1 && self.timeline[1].out_start <= consumed {
            self.timeline.pop_front();
        }

        if self.state == PlayState::Playing
            && !producing
            && self.primary.is_none()
            && consumed >= self.pushed
            && output.shared.silent.load(Ordering::Acquire)
        {
            self.set_state(PlayState::Stopped);
            self.current = self.queue.len();
            self.emit(Event::QueueEnded);
        }
    }
}

/// Applies a gain that glides from its last value to `target`. A lane's
/// first block starts at the target: there is nothing to glide from. Unity
/// with nothing to glide leaves the samples untouched, bit for bit.
fn apply_gain(
    samples: &mut [f32],
    channels: usize,
    state: &mut Option<f32>,
    target: f32,
    coef: f32,
) {
    let mut gain = state.unwrap_or(target);
    if gain == target && target == 1.0 {
        *state = Some(1.0);
        return;
    }
    for frame in samples.chunks_exact_mut(channels) {
        gain += (target - gain) * coef;
        if (target - gain).abs() < 1e-6 {
            gain = target;
        }
        for sample in frame {
            *sample *= gain;
        }
    }
    *state = Some(gain);
}

/// Whether every queue entry names one album and the track numbers run in
/// order without gaps, the condition for album gain in ReplayGain's auto mode
/// (SPEC §4.1). The caller supplies album and number in each entry.
/// Whether the queue is one album in order: same album, consecutive track
/// numbers. ReplayGain's auto mode uses album gain exactly then (SPEC §4.1).
pub fn queue_is_one_album(queue: &[QueueItem]) -> bool {
    let normalise = |item: &QueueItem| {
        item.album
            .as_deref()
            .map(|album| album.trim().to_lowercase())
            .filter(|album| !album.is_empty())
    };
    let Some(album) = queue.first().and_then(normalise) else {
        return false;
    };
    let mut previous: Option<u64> = None;
    for item in queue {
        let Some(number) = item.track_number else {
            return false;
        };
        if normalise(item).as_deref() != Some(album.as_str())
            || previous.is_some_and(|last| number != last + 1)
        {
            return false;
        }
        previous = Some(number);
    }
    true
}
