//! Engine tests through the offline sink (SPEC §12).
//!
//! Every fixture is generated here, at test time, into a folder whose name
//! holds Japanese characters and spaces. No music file is ever committed.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use onsa_audio::dsp::limiter::latency_frames;
use onsa_audio::wav::{write_wav, WavFormat};
use onsa_audio::{
    CrossfadeCurve, Engine, Event, OfflineSink, PlayState, PlaybackSettings, QueueItem, RepeatMode,
    MICRO_FADE_SECONDS,
};

const RATE: u32 = 48_000;

// ------------------------------------------------------------------ helpers

struct Fixtures {
    dir: PathBuf,
}

impl Fixtures {
    fn new(name: &str) -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "onsa テスト {} {} {}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
            name
        ));
        std::fs::create_dir_all(&dir).expect("fixture folder");
        Self { dir }
    }

    /// Writes a 32-bit float stereo WAV and returns its path.
    fn wav(&self, name: &str, rate: u32, samples: &[f32]) -> PathBuf {
        let path = self.dir.join(name);
        write_wav(&path, rate, 2, WavFormat::Float32, samples).expect("write fixture");
        path
    }
}

impl Drop for Fixtures {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// Stereo sine, both channels equal.
fn sine(frames: usize, rate: u32, freq: f64, amplitude: f32) -> Vec<f32> {
    (0..frames)
        .flat_map(|n| {
            let v = (n as f64 * freq * std::f64::consts::TAU / f64::from(rate)).sin() as f32
                * amplitude;
            [v, v]
        })
        .collect()
}

/// Constant level on one channel, silence on the other.
fn one_sided(frames: usize, left: f32, right: f32) -> Vec<f32> {
    (0..frames).flat_map(|_| [left, right]).collect()
}

fn fade_frames(rate: u32) -> usize {
    (rate as f32 * MICRO_FADE_SECONDS) as usize
}

fn max_step(samples: &[f32]) -> f32 {
    samples
        .chunks_exact(2)
        .zip(samples.chunks_exact(2).skip(1))
        .map(|(a, b)| (b[0] - a[0]).abs().max((b[1] - a[1]).abs()))
        .fold(0.0, f32::max)
}

fn offline(settings: PlaybackSettings) -> (Engine, OfflineSink) {
    Engine::offline(RATE, 2, settings)
}

fn play(engine: &Engine, sink: &OfflineSink, paths: &[&Path]) {
    let items = paths.iter().map(|path| QueueItem::new(*path)).collect();
    engine.set_queue(items, 0, 0.0, true).expect("queue");
    assert!(sink.wait_for_audio(1024), "the engine never produced audio");
}

/// Waits for an event matching `wanted`, rendering meanwhile when a sink is
/// given so the engine is never starved of a listener.
fn wait_for(
    engine: &Engine,
    mut sink: Option<(&mut OfflineSink, &mut Vec<f32>)>,
    wanted: impl Fn(&Event) -> bool,
) -> Event {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        while let Ok(event) = engine.events().try_recv() {
            if wanted(&event) {
                return event;
            }
        }
        assert!(Instant::now() < deadline, "event never arrived");
        match sink.as_mut() {
            Some((sink, out)) => out.extend(sink.render(64)),
            None => std::thread::sleep(Duration::from_millis(1)),
        }
    }
}

fn drain(engine: &Engine) -> Vec<Event> {
    engine.events().try_iter().collect()
}

/// Collects events until one matches `last` (included). Events are sent
/// from the engine thread, so they may trail the rendered audio a little.
fn collect_until(engine: &Engine, last: impl Fn(&Event) -> bool) -> Vec<Event> {
    let mut events = Vec::new();
    loop {
        match engine.events().recv_timeout(Duration::from_secs(10)) {
            Ok(event) => {
                let done = last(&event);
                events.push(event);
                if done {
                    return events;
                }
            }
            Err(_) => panic!("event never arrived; got {events:?}"),
        }
    }
}

// ------------------------------------------------------------------ gapless

#[test]
fn gapless_join_is_sample_exact() {
    let fixtures = Fixtures::new("gapless");
    let total = RATE as usize * 2;
    let split = RATE as usize + 777;
    let full = sine(total, RATE, 440.0, 0.5);
    let a = fixtures.wav("a.wav", RATE, &full[..split * 2]);
    let b = fixtures.wav("b.wav", RATE, &full[split * 2..]);

    let (engine, mut sink) = offline(PlaybackSettings::default());
    play(&engine, &sink, &[&a, &b]);
    let out = sink.render_to_end(total * 2);

    // The DSP chain delays everything by the limiter's look-ahead.
    let latency = latency_frames(RATE) * 2;
    assert!(
        out.len() >= total * 2 + latency,
        "output too short: {}",
        out.len() / 2
    );
    let start = fade_frames(RATE);
    let error = (start * 2..total * 2)
        .map(|i| (out[i + latency] - full[i]).abs())
        .fold(0.0f32, f32::max);
    assert!(
        error < 1e-6,
        "the two halves do not join into the whole: {error}"
    );

    let events = collect_until(&engine, |event| *event == Event::QueueEnded);
    let started: Vec<usize> = events
        .iter()
        .filter_map(|event| match event {
            Event::TrackStarted { index, .. } => Some(*index),
            _ => None,
        })
        .collect();
    assert_eq!(started, vec![0, 1]);
}

#[test]
fn gapless_join_survives_resampling() {
    let fixtures = Fixtures::new("gapless resample");
    let src_rate = 44_100;
    let total = src_rate as usize * 2;
    let split = src_rate as usize + 777;
    let full = sine(total, src_rate, 440.0, 0.5);
    let whole = fixtures.wav("whole.wav", src_rate, &full);
    let a = fixtures.wav("a.wav", src_rate, &full[..split * 2]);
    let b = fixtures.wav("b.wav", src_rate, &full[split * 2..]);

    let (engine, mut sink) = offline(PlaybackSettings::default());
    play(&engine, &sink, &[&a, &b]);
    let joined = sink.render_to_end(RATE as usize * 3);

    let (engine, mut sink) = offline(PlaybackSettings::default());
    play(&engine, &sink, &[&whole]);
    let reference = sink.render_to_end(RATE as usize * 3);

    let frames = joined.len().min(reference.len()) / 2;
    let expected = (total as f64 * f64::from(RATE) / f64::from(src_rate)).round() as usize;
    assert!(frames >= expected - 1, "{frames} < {expected}");
    let error = (0..frames * 2)
        .map(|i| (joined[i] - reference[i]).abs())
        .fold(0.0f32, f32::max);
    assert!(
        error < 1e-5,
        "resampled join differs from the whole file: {error}"
    );
}

// ---------------------------------------------------------------- crossfade

fn crossfade_render(curve: CrossfadeCurve, album: bool) -> Vec<f32> {
    let fixtures = Fixtures::new("crossfade");
    let frames = RATE as usize * 3;
    let a = fixtures.wav("a.wav", RATE, &one_sided(frames, 0.5, 0.0));
    let b = fixtures.wav("b.wav", RATE, &one_sided(frames, 0.0, 0.5));
    let settings = PlaybackSettings {
        crossfade_seconds: 1.0,
        curve,
        ..PlaybackSettings::default()
    };
    let (engine, mut sink) = offline(settings);
    let mut items = vec![QueueItem::new(&a), QueueItem::new(&b)];
    if album {
        for (number, item) in items.iter_mut().enumerate() {
            item.album = Some("Lampu Kota".to_string());
            item.track_number = Some(number as u64 + 1);
        }
    }
    engine.set_queue(items, 0, 0.0, true).unwrap();
    assert!(sink.wait_for_audio(1024));
    sink.render_to_end(frames * 3)
}

/// Frames where both tracks are audible, and the level where they cross.
fn overlap(out: &[f32]) -> (usize, f32) {
    let both: Vec<&[f32]> = out
        .chunks_exact(2)
        .filter(|frame| frame[0] > 0.005 && frame[1] > 0.005)
        .collect();
    let crossing = both
        .iter()
        .min_by(|a, b| {
            (a[0] - a[1])
                .abs()
                .partial_cmp(&(b[0] - b[1]).abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|frame| (frame[0] + frame[1]) / 2.0)
        .unwrap_or(0.0);
    (both.len(), crossing)
}

#[test]
fn crossfade_midpoint_follows_the_equal_power_curve() {
    let out = crossfade_render(CrossfadeCurve::EqualPower, false);
    let (length, crossing) = overlap(&out);
    let expected = 0.5 * std::f32::consts::FRAC_1_SQRT_2;
    assert!(
        (crossing - expected).abs() < 0.005,
        "crossing at {crossing}, expected {expected}"
    );
    // One second, give or take a block.
    assert!(
        (length as i64 - RATE as i64).abs() < 1100,
        "crossfade lasted {length} frames"
    );
}

#[test]
fn crossfade_midpoint_follows_the_linear_curve() {
    let (_, crossing) = overlap(&crossfade_render(CrossfadeCurve::Linear, false));
    assert!((crossing - 0.25).abs() < 0.005, "crossing at {crossing}");
}

#[test]
fn consecutive_album_tracks_stay_gapless() {
    let (length, _) = overlap(&crossfade_render(CrossfadeCurve::EqualPower, true));
    assert_eq!(length, 0, "album tracks were crossfaded");
}

// ---------------------------------------------------------------- micro-fade

#[test]
fn pause_and_resume_fade_without_jumps() {
    let fixtures = Fixtures::new("pause");
    let track = fixtures.wav("sine.wav", RATE, &sine(RATE as usize * 5, RATE, 440.0, 0.8));
    let (engine, mut sink) = offline(PlaybackSettings::default());
    play(&engine, &sink, &[&track]);

    let mut out = sink.render(RATE as usize / 2);
    engine.pause().unwrap();
    wait_for(&engine, None, |event| {
        *event == Event::StateChanged(PlayState::Paused)
    });
    out.extend(sink.render(RATE as usize * 3 / 10));
    engine.resume().unwrap();
    wait_for(&engine, None, |event| {
        *event == Event::StateChanged(PlayState::Playing)
    });
    out.extend(sink.render(RATE as usize / 2));

    // The steepest step of the sine itself, plus a little for the ramp.
    let limit = 0.8 * (std::f32::consts::TAU * 440.0 / RATE as f32) * 1.1;
    let step = max_step(&out);
    assert!(step < limit, "a jump of {step} (limit {limit})");

    let silent_run = out
        .chunks_exact(2)
        .fold((0usize, 0usize), |(run, best), frame| {
            let run = if frame[0] == 0.0 && frame[1] == 0.0 {
                run + 1
            } else {
                0
            };
            (run, best.max(run))
        })
        .1;
    assert!(silent_run > RATE as usize / 5, "pause was not silent");
}

#[test]
fn seek_is_smooth_and_sample_accurate() {
    let fixtures = Fixtures::new("seek");
    let source = sine(RATE as usize * 6, RATE, 441.3, 0.8);
    let track = fixtures.wav("sine.wav", RATE, &source);
    let (engine, mut sink) = offline(PlaybackSettings::default());
    play(&engine, &sink, &[&track]);

    let mut out = sink.render(RATE as usize / 2);
    let before = out.len() / 2;
    engine.seek(4.0).unwrap();
    wait_for(&engine, Some((&mut sink, &mut out)), |event| {
        matches!(event, Event::Seeked { .. })
    });
    out.extend(sink.render(RATE as usize / 2));

    let limit = 0.8 * (std::f32::consts::TAU * 441.3 / RATE as f32) * 1.1;
    let step = max_step(&out);
    assert!(step < limit, "a jump of {step} (limit {limit})");

    // Find where the new position fades in: the first sound after the
    // silence that follows the fade-out.
    let frames: Vec<&[f32]> = out.chunks_exact(2).collect();
    let silence = (before..frames.len())
        .find(|&i| frames[i][0] == 0.0)
        .expect("the seek never faded out");
    let start = (silence..frames.len())
        .find(|&i| frames[i][0] != 0.0)
        .expect("the seek never faded in");
    let target = RATE as usize * 4;
    let settle = fade_frames(RATE);
    let error = (settle..settle + 2000)
        .map(|j| (frames[start + j][0] - source[(target + j) * 2]).abs())
        .fold(0.0f32, f32::max);
    assert!(error < 1e-5, "seek landed off target: {error}");
}

// ------------------------------------------------------------- robustness

#[test]
fn a_new_output_resumes_where_the_old_one_stopped() {
    let fixtures = Fixtures::new("output change");
    let frames = RATE as usize * 10;
    // Left channel is a ramp: its value tells the position.
    let ramp: Vec<f32> = (0..frames)
        .flat_map(|n| [n as f32 / frames as f32, 0.0])
        .collect();
    let track = fixtures.wav("ramp.wav", RATE, &ramp);
    let (engine, mut sink) = offline(PlaybackSettings::default());
    play(&engine, &sink, &[&track]);
    let _ = sink.render(RATE as usize);

    let mut replacement = engine.replace_offline_output(44_100, 2).unwrap();
    assert!(replacement.wait_for_audio(1024));
    let out = replacement.render(44_100 / 2);
    let probe = fade_frames(44_100) + 100;
    let seconds = 1.0 + probe as f32 / 44_100.0;
    let expected = seconds / 10.0;
    let heard = out[probe * 2];
    assert!(
        (heard - expected).abs() < 0.003,
        "resumed at {heard}, expected about {expected}"
    );
}

#[test]
fn a_broken_file_is_reported_and_skipped() {
    let fixtures = Fixtures::new("broken");
    let broken = fixtures.dir.join("broken.flac");
    std::fs::write(&broken, b"this is not audio at all").unwrap();
    let good = fixtures.wav("good.wav", RATE, &sine(RATE as usize, RATE, 440.0, 0.5));

    let (engine, mut sink) = offline(PlaybackSettings::default());
    play(&engine, &sink, &[&broken, &good]);
    let out = sink.render_to_end(RATE as usize * 2);
    assert!(out.iter().any(|sample| sample.abs() > 0.4));

    let events = drain(&engine);
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::TrackFailed { index: 0, .. })),
        "{events:?}"
    );
    assert!(events
        .iter()
        .any(|event| matches!(event, Event::TrackStarted { index: 1, .. })));
}

#[test]
fn skipping_crossfades_without_jumps() {
    let fixtures = Fixtures::new("skip");
    let a = fixtures.wav("a.wav", RATE, &sine(RATE as usize * 4, RATE, 440.0, 0.5));
    let b = fixtures.wav("b.wav", RATE, &sine(RATE as usize * 4, RATE, 550.0, 0.5));
    let (engine, mut sink) = offline(PlaybackSettings::default());
    play(&engine, &sink, &[&a, &b]);

    let mut out = sink.render(RATE as usize);
    engine.next().unwrap();
    wait_for(&engine, Some((&mut sink, &mut out)), |event| {
        matches!(event, Event::TrackStarted { index: 1, .. })
    });
    out.extend(sink.render(RATE as usize / 2));

    let limit = 0.5 * (std::f32::consts::TAU * 550.0 / RATE as f32) * 1.3;
    let step = max_step(&out);
    assert!(step < limit, "a jump of {step} (limit {limit})");
}

// ------------------------------------------------------------------- repeat

#[test]
fn repeat_all_starts_the_queue_over() {
    let fixtures = Fixtures::new("repeat all");
    let fifth = RATE as usize / 5;
    let a = fixtures.wav("a.wav", RATE, &sine(fifth, RATE, 440.0, 0.3));
    let b = fixtures.wav("b.wav", RATE, &sine(fifth, RATE, 660.0, 0.3));

    let (engine, mut sink) = offline(PlaybackSettings {
        repeat: RepeatMode::All,
        ..PlaybackSettings::default()
    });
    play(&engine, &sink, &[&a, &b]);

    let mut started = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(10);
    while started.len() < 3 {
        assert!(
            Instant::now() < deadline,
            "the queue never came round: {started:?}"
        );
        sink.render(512);
        for event in drain(&engine) {
            match event {
                Event::TrackStarted { index, .. } => started.push(index),
                Event::QueueEnded => panic!("a repeating queue must not end"),
                _ => {}
            }
        }
    }
    assert_eq!(started, vec![0, 1, 0], "the queue did not start over");
}

#[test]
fn repeat_one_loops_the_same_track_without_a_gap() {
    let fixtures = Fixtures::new("repeat one");
    // Forty-eight whole periods, so a seamless loop is one continuous sine.
    let frames = RATE as usize / 10;
    let loops = 3;
    let whole = sine(frames * loops, RATE, 480.0, 0.5);
    let file = fixtures.wav("loop.wav", RATE, &whole[..frames * 2]);

    let (engine, mut sink) = offline(PlaybackSettings {
        repeat: RepeatMode::One,
        ..PlaybackSettings::default()
    });
    play(&engine, &sink, &[&file]);

    let latency = latency_frames(RATE) * 2;
    let wanted = frames * loops * 2 + latency;
    let mut out = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(10);
    while out.len() < wanted {
        assert!(Instant::now() < deadline, "the track did not loop");
        out.extend(sink.render(512));
    }

    let start = fade_frames(RATE) * 2;
    let error = (start..frames * loops * 2)
        .map(|i| (out[i + latency] - whole[i]).abs())
        .fold(0.0f32, f32::max);
    assert!(error < 1e-6, "the loop is not seamless: {error}");

    // Three loops of audio arrived and matched one continuous sine, so the
    // track really repeated. A looped track stays the same track: the queue
    // never ends, and the playhead wraps instead of a new track starting.
    assert!(
        !drain(&engine).contains(&Event::QueueEnded),
        "a repeating track must not end the queue"
    );
}

// ------------------------------------------------------------- queue edits

#[test]
fn the_queue_can_be_edited_while_a_track_plays() {
    let fixtures = Fixtures::new("queue edit");
    let seconds = RATE as usize;
    let first = sine(seconds, RATE, 440.0, 0.4);
    let a = fixtures.wav("a.wav", RATE, &first);
    let b = fixtures.wav("b.wav", RATE, &sine(RATE as usize / 5, RATE, 660.0, 0.4));

    let (engine, mut sink) = offline(PlaybackSettings::default());
    play(&engine, &sink, &[&a]);
    let mut out = sink.render(RATE as usize / 4 * 2);

    // A track added to the end while the first one is still playing.
    engine
        .update_queue(vec![QueueItem::new(&a), QueueItem::new(&b)], 0)
        .expect("update");
    out.extend(sink.render_to_end(seconds * 4));

    // The first track played through, unbroken, despite the edit.
    let latency = latency_frames(RATE) * 2;
    let start = fade_frames(RATE) * 2;
    let error = (start..seconds * 2)
        .map(|i| (out[i + latency] - first[i]).abs())
        .fold(0.0f32, f32::max);
    assert!(error < 1e-6, "editing the queue broke playback: {error}");

    let events = collect_until(&engine, |event| *event == Event::QueueEnded);
    let started: Vec<usize> = events
        .iter()
        .filter_map(|event| match event {
            Event::TrackStarted { index, .. } => Some(*index),
            _ => None,
        })
        .collect();
    assert_eq!(started, vec![0, 1], "the added track did not follow");
}

#[test]
fn an_output_replaced_at_the_very_start_keeps_playing() {
    // Match-source reopens the output as soon as a track begins, before any
    // audio has been heard; playback must carry on from the start.
    let fixtures = Fixtures::new("output at start");
    let seconds = RATE as usize / 2;
    let whole = sine(seconds, RATE, 440.0, 0.4);
    let file = fixtures.wav("a.wav", RATE, &whole);

    let (engine, _sink) = offline(PlaybackSettings::default());
    engine
        .set_queue(vec![QueueItem::new(&file)], 0, 0.0, true)
        .expect("queue");
    let mut sink = engine
        .replace_offline_output(RATE, 2)
        .expect("the output should be replaced");

    let out = sink.render_to_end(seconds * 4);
    let latency = latency_frames(RATE) * 2;
    assert!(
        out.len() >= seconds * 2,
        "playback stopped after the output changed: {} frames",
        out.len() / 2
    );
    let start = fade_frames(RATE) * 2;
    let error = (start..seconds * 2 - latency)
        .map(|i| (out[i + latency] - whole[i]).abs())
        .fold(0.0f32, f32::max);
    assert!(
        error < 1e-6,
        "the track did not play from its start: {error}"
    );
}
