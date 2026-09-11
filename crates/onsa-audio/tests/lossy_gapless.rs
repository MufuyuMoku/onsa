//! Gapless joins for lossy codecs (SPEC §3.3, §12).
//!
//! MP3 and AAC encoders add silent priming frames at the start and padding
//! at the end. A gapless player must drop exactly those, using the LAME/Xing
//! header for MP3 and the edit list (or iTunSMPB) for MP4. Fixtures are made
//! at test time with ffmpeg; without ffmpeg on PATH these tests are skipped
//! with a message, because no encoded audio is ever committed.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use onsa_audio::wav::{write_wav, WavFormat};
use onsa_audio::{Engine, Event, PlaybackSettings, QueueItem, MICRO_FADE_SECONDS};

const RATE: u32 = 44_100;
const FREQ: f64 = 440.0;
const AMPLITUDE: f32 = 0.5;

struct Fixtures {
    dir: PathBuf,
}

impl Fixtures {
    fn new(name: &str) -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "onsa テスト lossy {} {} {}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
            name
        ));
        std::fs::create_dir_all(&dir).expect("fixture folder");
        Self { dir }
    }
}

impl Drop for Fixtures {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// ffmpeg, run with an argument array and never through a shell.
fn ffmpeg() -> Command {
    let command = Command::new("ffmpeg");
    // No console window on Windows (CLAUDE.md). Only this branch needs the
    // command to be mutable, so the binding changes here and nowhere else.
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

/// Whether ffmpeg is on PATH and has `encoder`.
fn has_encoder(encoder: &str) -> bool {
    let Ok(output) = ffmpeg().args(["-hide_banner", "-encoders"]).output() else {
        eprintln!("skipping: ffmpeg is not on PATH, so no {encoder} fixture can be made");
        return false;
    };
    let listed = String::from_utf8_lossy(&output.stdout)
        .lines()
        .any(|line| line.split_whitespace().nth(1) == Some(encoder));
    if !listed {
        eprintln!("skipping: this ffmpeg has no {encoder} encoder");
    }
    listed
}

fn encode(input: &Path, output: &Path, codec_args: &[&str]) {
    let status = ffmpeg()
        .args(["-hide_banner", "-loglevel", "error", "-y", "-i"])
        .arg(input)
        .args(codec_args)
        .arg(output)
        .status()
        .expect("run ffmpeg");
    assert!(
        status.success(),
        "ffmpeg failed to make {}",
        output.display()
    );
}

/// Decodes with ffmpeg to raw interleaved `f32` stereo. ffmpeg drops the
/// encoder priming itself (LAME header, MP4 edit list), which makes it the
/// reference decoder for the join.
fn decode_reference(input: &Path, frames: usize) -> Vec<f32> {
    let output = ffmpeg()
        .args(["-hide_banner", "-loglevel", "error", "-i"])
        .arg(input)
        .args(["-f", "f32le", "-ac", "2", "-"])
        .output()
        .expect("run ffmpeg");
    assert!(
        output.status.success(),
        "ffmpeg could not decode {}",
        input.display()
    );
    let mut samples: Vec<f32> = output
        .stdout
        .chunks_exact(4)
        .map(|bytes| f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
        .collect();
    // ffmpeg keeps the end padding; the original length is known.
    samples.truncate(frames * 2);
    samples
}

fn sine(frames: usize) -> Vec<f32> {
    (0..frames)
        .flat_map(|n| {
            let v = (n as f64 * FREQ * std::f64::consts::TAU / f64::from(RATE)).sin() as f32
                * AMPLITUDE;
            [v, v]
        })
        .collect()
}

/// Encodes two halves of one sine, plays them back to back and checks that
/// the join is gapless: exact lengths, alignment, no hole, and nothing
/// played past the end. Encoders add their own transient at a hard cut in the
/// source; that is the file's content and is allowed near the join.
fn check_lossy_join(name: &str, extension: &str, codec_args: &[&str]) {
    let fixtures = Fixtures::new(name);
    let total = RATE as usize * 4;
    let split = RATE as usize * 2 + 777;
    let full = sine(total);

    let mut parts = Vec::new();
    let mut reference = Vec::with_capacity(total * 2);
    for (index, range) in [(0, 0..split), (1, split..total)] {
        let wav = fixtures.dir.join(format!("part {index}.wav"));
        write_wav(
            &wav,
            RATE,
            2,
            WavFormat::Float32,
            &full[range.start * 2..range.end * 2],
        )
        .expect("write wav");
        let encoded = fixtures.dir.join(format!("part {index}.{extension}"));
        encode(&wav, &encoded, codec_args);
        reference.extend(decode_reference(&encoded, range.end - range.start));
        parts.push(encoded);
    }

    let (engine, mut sink) = Engine::offline(RATE, 2, PlaybackSettings::default());
    let items = parts.iter().map(QueueItem::new).collect();
    engine.set_queue(items, 0, 0.0, true).expect("queue");
    assert!(sink.wait_for_audio(1024), "the engine never produced audio");
    let out = sink.render_to_end(total * 2);

    // 1. The decoder knows each part's exact playable length, which only
    //    happens when the encoder delay and padding were read and removed.
    let mut lengths = Vec::new();
    for event in engine.events().try_iter() {
        match event {
            Event::TrackStarted { index, info, .. } => lengths.push((index, info.total_frames)),
            Event::TrackFailed { reason, .. } => panic!("{name}: track failed: {reason}"),
            _ => {}
        }
    }
    assert_eq!(
        lengths,
        vec![(0, Some(split as u64)), (1, Some((total - split) as u64))],
        "{name}: playable lengths do not match the originals"
    );

    let left: Vec<f32> = out.iter().step_by(2).copied().collect();
    assert!(
        left.len() >= total,
        "{name}: output too short: {}",
        left.len()
    );
    let error_at = |n: usize| (left[n] - full[n * 2]).abs();
    let fade = (RATE as f32 * MICRO_FADE_SECONDS) as usize;
    let join = split - 2048..split + 2048;

    // 2. Aligned: away from the join and the ends, the output is the original
    //    sine within the codec's noise. Priming left in, or padding played,
    //    would shift the second part and break this by the full amplitude.
    let steady = (fade..split - 2048)
        .chain(split + 2048..total - 2048)
        .map(error_at)
        .fold(0.0f32, f32::max);
    assert!(
        steady < 0.02,
        "{name}: signal misaligned, error {steady:.4}"
    );

    // 3. No hole at the join. Priming is silence, so an untrimmed join drops
    //    to almost nothing; an encoder transient at the hard cut only dips.
    //    (RMS of a sine at 0.5 is about 0.354.)
    let quietest = (split - 4096..split + 4096)
        .step_by(256)
        .map(|start| {
            let window = &left[start..start + 256];
            (window.iter().map(|v| v * v).sum::<f32>() / 256.0).sqrt()
        })
        .fold(f32::MAX, f32::min);
    assert!(
        quietest > 0.2,
        "{name}: level drops to {quietest:.3} at the join (frame {split})"
    );

    // 4. The join sounds exactly as a reference decoder plays the same two
    //    files back to back. Encoders add their own transient at a hard cut
    //    in the source; that is the files' content and appears in both, while
    //    a trim off by even one packet does not.
    assert_eq!(
        reference.len(),
        total * 2,
        "{name}: reference decode is short"
    );
    let (differ, differ_at) = (fade..total - 2048)
        .map(|n| ((left[n] - reference[n * 2]).abs(), n))
        .fold(
            (0.0f32, 0),
            |best, next| if next.0 > best.0 { next } else { best },
        );
    let near_join = join.map(error_at).fold(0.0f32, f32::max);
    eprintln!(
        "{name}: steady error {steady:.4}, quietest window {quietest:.3}, join vs sine {near_join:.4}, vs reference decoder {differ:.5} at frame {differ_at}"
    );
    assert!(
        differ < 0.005,
        "{name}: differs from the reference decoder by {differ:.5} at frame {differ_at} (join at {split})"
    );

    // 5. Playback stops where the original stops: after the closing fade
    //    there is only silence, so no encoder padding was played.
    let tail = &left[(total + fade + 64).min(left.len())..];
    let loudest_tail = tail.iter().fold(0.0f32, |max, v| max.max(v.abs()));
    assert!(
        loudest_tail == 0.0,
        "{name}: {} frames of sound after the end, peak {loudest_tail}",
        tail.iter().filter(|v| **v != 0.0).count()
    );
}

#[test]
fn mp3_join_drops_the_lame_delay_and_padding() {
    if !has_encoder("libmp3lame") {
        return;
    }
    check_lossy_join("mp3", "mp3", &["-c:a", "libmp3lame", "-b:a", "320k"]);
}

#[test]
fn m4a_join_drops_the_aac_priming_and_padding() {
    if !has_encoder("aac") {
        return;
    }
    check_lossy_join("m4a", "m4a", &["-c:a", "aac", "-b:a", "256k"]);
}
