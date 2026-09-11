//! The DSP chain through the whole engine (SPEC §4, §12), rendered with the
//! offline sink. Fixtures are generated at test time, in a folder with
//! Japanese characters and spaces; the ReplayGain test tags FLAC files with
//! ffmpeg and is skipped with a message when ffmpeg is not on PATH.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use onsa_audio::dsp::db_to_gain;
use onsa_audio::wav::{write_wav, WavFormat};
use onsa_audio::{
    AnalysisFrame, AnalysisSettings, DspSettings, Engine, EqMode, Event, OfflineSink, PlayState,
    PlaybackSettings, QueueItem, ReplayGainMode, ReplayGainSettings,
};

const RATE: u32 = 48_000;

struct Fixtures {
    dir: PathBuf,
}

impl Fixtures {
    fn new(name: &str) -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "onsa テスト dsp {} {} {}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
            name
        ));
        std::fs::create_dir_all(&dir).expect("fixture folder");
        Self { dir }
    }

    fn wav(&self, name: &str, samples: &[f32]) -> PathBuf {
        let path = self.dir.join(name);
        write_wav(&path, RATE, 2, WavFormat::Float32, samples).expect("write fixture");
        path
    }
}

impl Drop for Fixtures {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn sine(frames: usize, freq: f32, amplitude: f32) -> Vec<f32> {
    (0..frames)
        .flat_map(|n| {
            let v = (std::f32::consts::TAU * freq * n as f32 / RATE as f32).sin() * amplitude;
            [v, v]
        })
        .collect()
}

fn peak(samples: &[f32]) -> f32 {
    samples.iter().fold(0.0f32, |max, v| max.max(v.abs()))
}

/// Largest second difference of the left channel. A sine's is bounded by
/// its curvature; a click is a spike far above it.
fn max_curvature(samples: &[f32]) -> f32 {
    let left: Vec<f32> = samples.iter().step_by(2).copied().collect();
    left.windows(3)
        .map(|w| (w[2] - 2.0 * w[1] + w[0]).abs())
        .fold(0.0, f32::max)
}

/// Starts an offline engine with `dsp` set before the first block.
fn start(dsp: DspSettings, items: Vec<QueueItem>) -> (Engine, OfflineSink) {
    let (engine, sink) = Engine::offline(RATE, 2, PlaybackSettings::default());
    engine.set_dsp(dsp).expect("dsp");
    engine.set_queue(items, 0, 0.0, true).expect("queue");
    assert!(sink.wait_for_audio(1024), "the engine never produced audio");
    (engine, sink)
}

#[test]
fn moving_the_eq_through_the_engine_does_not_click() {
    let fixtures = Fixtures::new("eq");
    let track = fixtures.wav("sine.wav", &sine(RATE as usize * 4, 1000.0, 0.2));
    let mut dsp = DspSettings {
        eq_enabled: true,
        ..DspSettings::default()
    };
    let (engine, mut sink) = start(dsp.clone(), vec![QueueItem::new(&track)]);
    let mut out = sink.render(RATE as usize);

    // A drastic move while the track plays: +12 dB at 1 kHz, -12 dB at 125 Hz.
    let mut gains = [0.0; 10];
    gains[5] = 12.0;
    gains[2] = -12.0;
    dsp.eq = EqMode::Graphic(gains);
    engine.set_dsp(dsp).expect("dsp");
    out.extend(sink.render(RATE as usize));

    let tail = &out[out.len() - 2 * 4800..];
    let level = peak(tail);
    assert!(
        (level - 0.2 * db_to_gain(12.0)).abs() < 0.02,
        "the EQ change never arrived: level {level}"
    );
    // After the opening fade, nothing is steeper than the loud sine itself.
    let body = &out[RATE as usize..];
    let (worst, steady) = (max_curvature(body), max_curvature(tail));
    assert!(worst <= steady * 1.05, "click: {worst} vs steady {steady}");
}

#[test]
fn the_limiter_holds_the_ceiling_through_the_engine() {
    let fixtures = Fixtures::new("limiter");
    let track = fixtures.wav("loud.wav", &sine(RATE as usize * 2, 1000.0, 0.9));
    let dsp = DspSettings {
        preamp_db: 12.0,
        ..DspSettings::default()
    };
    let (_engine, mut sink) = start(dsp, vec![QueueItem::new(&track)]);
    let out = sink.render(RATE as usize);
    let level = peak(&out);
    assert!(
        level <= db_to_gain(-0.1) + 1e-6,
        "over the ceiling: {level}"
    );
    assert!(level > 0.9, "the limiter squashed the signal: {level}");
}

#[test]
fn auto_preamp_keeps_a_boosted_full_scale_sine_below_clipping() {
    let fixtures = Fixtures::new("auto preamp");
    let track = fixtures.wav("full.wav", &sine(RATE as usize * 2, 1000.0, 1.0));
    let mut gains = [0.0; 10];
    gains[5] = 9.0;
    let boosted = DspSettings {
        eq_enabled: true,
        eq: EqMode::Graphic(gains),
        limiter_enabled: false,
        ..DspSettings::default()
    };

    let (_engine, mut sink) = start(
        DspSettings {
            auto_preamp: true,
            ..boosted.clone()
        },
        vec![QueueItem::new(&track)],
    );
    let level = peak(&sink.render(RATE as usize));
    assert!(level <= 1.0 + 1e-3, "auto preamp let it clip: {level}");

    // Without it the same settings go far over full scale.
    let (_engine, mut sink) = start(boosted, vec![QueueItem::new(&track)]);
    assert!(peak(&sink.render(RATE as usize)) > 2.0);
}

// ---------------------------------------------------------------- ReplayGain

fn ffmpeg() -> Command {
    let command = Command::new("ffmpeg");
    #[cfg(windows)]
    let command = {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let mut command = command;
        command.creation_flags(CREATE_NO_WINDOW);
        command
    };
    command
}

/// Encodes a WAV to FLAC with ReplayGain tags (Vorbis comments).
fn tagged_flac(input: &Path, output: &Path, tags: &[(&str, &str)]) -> bool {
    let mut command = ffmpeg();
    command
        .args(["-hide_banner", "-loglevel", "error", "-y", "-i"])
        .arg(input)
        .args(["-c:a", "flac"]);
    for (key, value) in tags {
        command.arg("-metadata").arg(format!("{key}={value}"));
    }
    match command.arg(output).status() {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}

/// Steady level of a track played with `replaygain`.
fn level_with(replaygain: ReplayGainMode, items: Vec<QueueItem>, limiter: bool) -> f32 {
    let dsp = DspSettings {
        replaygain: ReplayGainSettings {
            mode: replaygain,
            ..ReplayGainSettings::default()
        },
        limiter_enabled: limiter,
        ..DspSettings::default()
    };
    let (_engine, mut sink) = start(dsp, items);
    let out = sink.render(RATE as usize / 2);
    peak(&out[out.len() - 2 * 4800..])
}

#[test]
fn replaygain_follows_the_tags() {
    if Command::new("ffmpeg").arg("-version").output().is_err() {
        eprintln!("skipping: ffmpeg is not on PATH, so no tagged FLAC can be made");
        return;
    }
    let fixtures = Fixtures::new("replaygain");
    let wav = fixtures.wav("sine.wav", &sine(RATE as usize * 2, 1000.0, 0.5));
    let tagged = fixtures.dir.join("tagged.flac");
    let hot = fixtures.dir.join("hot.flac");
    assert!(tagged_flac(
        &wav,
        &tagged,
        &[
            ("REPLAYGAIN_TRACK_GAIN", "-6.02 dB"),
            ("REPLAYGAIN_TRACK_PEAK", "0.5"),
            ("REPLAYGAIN_ALBUM_GAIN", "-12.04 dB"),
            ("REPLAYGAIN_ALBUM_PEAK", "0.5"),
        ],
    ));
    assert!(tagged_flac(
        &wav,
        &hot,
        &[
            ("REPLAYGAIN_TRACK_GAIN", "+12.00 dB"),
            ("REPLAYGAIN_TRACK_PEAK", "0.5"),
        ],
    ));

    let close = |level: f32, expected: f32| (level - expected).abs() < expected * 0.01;
    let one = || vec![QueueItem::new(&tagged)];

    let off = level_with(ReplayGainMode::Off, one(), true);
    assert!(close(off, 0.5), "off: {off}");
    let track = level_with(ReplayGainMode::Track, one(), true);
    assert!(close(track, 0.25), "track: {track}");
    let album = level_with(ReplayGainMode::Album, one(), true);
    assert!(close(album, 0.125), "album: {album}");

    // The peak tag caps a hot gain: +12 dB on a 0.5 peak stops at 1.0.
    let capped = level_with(ReplayGainMode::Track, vec![QueueItem::new(&hot)], false);
    assert!(close(capped, 1.0), "clipping prevention: {capped}");

    // Auto: album gain for one album in order, track gain otherwise.
    let album_queue = (1..=2)
        .map(|number| QueueItem {
            album: Some("Lampu Kota".to_string()),
            track_number: Some(number),
            ..QueueItem::new(&tagged)
        })
        .collect();
    let auto_album = level_with(ReplayGainMode::Auto, album_queue, true);
    assert!(close(auto_album, 0.125), "auto, one album: {auto_album}");
    let auto_mixed = level_with(ReplayGainMode::Auto, one(), true);
    assert!(close(auto_mixed, 0.25), "auto, no album: {auto_mixed}");
}

// ------------------------------------------------------------------ analysis

/// Renders in small steps, roughly in real time, and collects analysis
/// frames for `span`.
fn frames_during(engine: &Engine, sink: &mut OfflineSink, span: Duration) -> Vec<AnalysisFrame> {
    let deadline = Instant::now() + span;
    let mut frames = Vec::new();
    while Instant::now() < deadline {
        sink.render(480);
        std::thread::sleep(Duration::from_millis(10));
        for event in engine.events().try_iter() {
            if let Event::Analysis(frame) = event {
                frames.push(frame);
            }
        }
    }
    frames
}

fn wait_for_state(engine: &Engine, sink: &mut OfflineSink, state: PlayState) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        assert!(Instant::now() < deadline, "never reached {state:?}");
        sink.render(64);
        if engine
            .events()
            .try_iter()
            .any(|event| event == Event::StateChanged(state))
        {
            return;
        }
    }
}

#[test]
fn analysis_runs_only_while_it_is_on_and_audio_plays() {
    let fixtures = Fixtures::new("analysis");
    let track = fixtures.wav("sine.wav", &sine(RATE as usize * 20, 1000.0, 0.5));
    let (engine, mut sink) = start(DspSettings::default(), vec![QueueItem::new(&track)]);

    // Off by default: no frames at all.
    assert!(frames_during(&engine, &mut sink, Duration::from_millis(300)).is_empty());

    let on = AnalysisSettings {
        enabled: true,
        ..AnalysisSettings::default()
    };
    engine.set_analysis(on).expect("analysis");
    let frames = frames_during(&engine, &mut sink, Duration::from_millis(500));
    let frame = frames.last().expect("no analysis frames while playing");
    assert!(
        (frame.centroid_hz - 1000.0).abs() < 100.0,
        "centroid {}",
        frame.centroid_hz
    );
    assert!(
        (frame.peak_db[0] + 6.02).abs() < 0.5,
        "peak {}",
        frame.peak_db[0]
    );

    // Paused: the tap closes and the thread parks. Allow a frame in flight.
    engine.pause().expect("pause");
    wait_for_state(&engine, &mut sink, PlayState::Paused);
    frames_during(&engine, &mut sink, Duration::from_millis(150));
    assert!(
        frames_during(&engine, &mut sink, Duration::from_millis(400)).is_empty(),
        "analysis kept running while paused"
    );

    // Playing again brings the frames back.
    engine.resume().expect("resume");
    wait_for_state(&engine, &mut sink, PlayState::Playing);
    assert!(!frames_during(&engine, &mut sink, Duration::from_millis(500)).is_empty());

    // Switched off while playing: nothing more.
    engine
        .set_analysis(AnalysisSettings::default())
        .expect("analysis");
    frames_during(&engine, &mut sink, Duration::from_millis(150));
    assert!(
        frames_during(&engine, &mut sink, Duration::from_millis(400)).is_empty(),
        "analysis kept running after it was switched off"
    );
}
