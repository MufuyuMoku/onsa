//! Command line front end for the Onsa audio engine.
//!
//! It exercises the engine without the user interface (SPEC §15, M1):
//! `devices` lists outputs, `play` and `queue` play through a device with
//! simple keyboard control, and `render-wav` renders through the offline
//! sink into a WAV file.

use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use onsa_audio::wav::{WavFormat, WavWriter};
use onsa_audio::{
    list_devices, BufferSize, CrossfadeCurve, DeviceChoice, Engine, Event, OutputRate,
    OutputSettings, PlayState, PlaybackSettings, QueueItem, ResamplerQuality,
};

#[derive(Parser)]
#[command(
    name = "onsa-cli",
    version,
    about = "Drive the Onsa audio engine without the user interface"
)]
struct Cli {
    /// Log filter, for example `info` or `onsa_audio=debug`.
    #[arg(long, global = true, env = "ONSA_LOG", default_value = "warn")]
    log: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// List the audio output devices.
    Devices,
    /// Play one file.
    Play {
        /// The file to play.
        file: PathBuf,
        /// Start this many seconds in.
        #[arg(long, default_value_t = 0.0)]
        start: f64,
        #[command(flatten)]
        playback: PlaybackArgs,
        #[command(flatten)]
        output: OutputArgs,
    },
    /// Play several files in order, gapless unless a crossfade is set.
    Queue {
        /// The files to play.
        files: Vec<PathBuf>,
        /// A text file with one path per line (blank lines and `#` comments
        /// are ignored; relative paths are relative to the list).
        #[arg(long)]
        list: Option<PathBuf>,
        #[command(flatten)]
        playback: PlaybackArgs,
        #[command(flatten)]
        output: OutputArgs,
    },
    /// Render files through the engine into a WAV file, without a device.
    RenderWav {
        /// The files to render, in order.
        #[arg(required = true)]
        files: Vec<PathBuf>,
        /// The WAV file to write.
        #[arg(short, long)]
        output: PathBuf,
        /// Output sample rate.
        #[arg(long, default_value_t = 48_000)]
        rate: u32,
        /// Output channel count.
        #[arg(long, default_value_t = 2)]
        channels: u16,
        /// Sample format of the WAV file.
        #[arg(long, value_enum, default_value_t = SampleArg::F32)]
        format: SampleArg,
        #[command(flatten)]
        playback: PlaybackArgs,
    },
}

#[derive(Args, Clone)]
struct PlaybackArgs {
    /// Crossfade between tracks in seconds (0 to 12, 0 is off).
    #[arg(long, default_value_t = 0.0)]
    crossfade: f32,
    /// Crossfade curve.
    #[arg(long, value_enum, default_value_t = CurveArg::EqualPower)]
    curve: CurveArg,
    /// Crossfade when skipping, in seconds.
    #[arg(long, default_value_t = 0.3)]
    skip_crossfade: f32,
    /// Crossfade even between consecutive tracks of one album.
    #[arg(long)]
    crossfade_albums: bool,
    /// Resampler quality.
    #[arg(long, value_enum, default_value_t = QualityArg::Balanced)]
    quality: QualityArg,
    /// Buffer between the engine and the device.
    #[arg(long, value_enum, default_value_t = BufferArg::Normal)]
    buffer: BufferArg,
}

#[derive(Args, Clone)]
struct OutputArgs {
    /// Output device identifier, as `devices` lists it. Default: the system
    /// default, followed when it changes.
    #[arg(long)]
    device: Option<String>,
    /// Open the device at this sample rate instead of its own.
    #[arg(long)]
    rate: Option<u32>,
}

#[derive(Clone, Copy, ValueEnum)]
enum CurveArg {
    EqualPower,
    Linear,
}

#[derive(Clone, Copy, ValueEnum)]
enum QualityArg {
    Fast,
    Balanced,
    Best,
}

#[derive(Clone, Copy, ValueEnum)]
enum BufferArg {
    Low,
    Normal,
    Large,
}

#[derive(Clone, Copy, ValueEnum)]
enum SampleArg {
    F32,
    I16,
}

impl PlaybackArgs {
    fn settings(&self) -> Result<PlaybackSettings> {
        if !(0.0..=12.0).contains(&self.crossfade) {
            bail!("--crossfade must be between 0 and 12 seconds");
        }
        Ok(PlaybackSettings {
            crossfade_seconds: self.crossfade,
            curve: match self.curve {
                CurveArg::EqualPower => CrossfadeCurve::EqualPower,
                CurveArg::Linear => CrossfadeCurve::Linear,
            },
            skip_crossfade_seconds: self.skip_crossfade.max(0.0),
            album_gapless: !self.crossfade_albums,
            quality: match self.quality {
                QualityArg::Fast => ResamplerQuality::Fast,
                QualityArg::Balanced => ResamplerQuality::Balanced,
                QualityArg::Best => ResamplerQuality::Best,
            },
            buffer: match self.buffer {
                BufferArg::Low => BufferSize::Low,
                BufferArg::Normal => BufferSize::Normal,
                BufferArg::Large => BufferSize::Large,
            },
        })
    }
}

impl OutputArgs {
    fn settings(&self) -> OutputSettings {
        OutputSettings {
            device: match &self.device {
                Some(id) => DeviceChoice::Id(id.clone()),
                None => DeviceChoice::SystemDefault,
            },
            rate: match self.rate {
                Some(rate) => OutputRate::Fixed(rate),
                None => OutputRate::FollowDevice,
            },
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(&cli.log))
        .with_writer(std::io::stderr)
        .init();
    tracing::debug!(version = env!("CARGO_PKG_VERSION"), "onsa-cli started");

    match cli.command {
        Command::Devices => devices(),
        Command::Play {
            file,
            start,
            playback,
            output,
        } => play(vec![file], start, &playback, &output),
        Command::Queue {
            mut files,
            list,
            playback,
            output,
        } => {
            if let Some(list) = list {
                files.extend(read_list(&list)?);
            }
            if files.is_empty() {
                bail!("nothing to play: give files or --list");
            }
            play(files, 0.0, &playback, &output)
        }
        Command::RenderWav {
            files,
            output,
            rate,
            channels,
            format,
            playback,
        } => render_wav(&files, &output, rate, channels, format, &playback),
    }
}

fn devices() -> Result<()> {
    let devices = list_devices().context("cannot list output devices")?;
    if devices.is_empty() {
        println!("No output devices found.");
    }
    for device in devices {
        let marker = if device.is_default { "*" } else { " " };
        println!(
            "{marker} {}\n    id: {}\n    {} Hz, {} channels",
            device.name, device.id, device.sample_rate, device.channels
        );
    }
    Ok(())
}

/// Reads a list of paths, one per line.
fn read_list(list: &Path) -> Result<Vec<PathBuf>> {
    let text =
        std::fs::read_to_string(list).with_context(|| format!("cannot read {}", list.display()))?;
    let base = list.parent().unwrap_or(Path::new("."));
    Ok(text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let path = PathBuf::from(line);
            if path.is_absolute() {
                path
            } else {
                base.join(path)
            }
        })
        .collect())
}

const KEYS: &str = "keys: [Enter] pause/resume · n next · b previous · s SECONDS seek · +N/-N skip seconds · q quit";

/// What the keyboard thread asks for.
enum Key {
    Toggle,
    Next,
    Previous,
    Seek(f64),
    Relative(f64),
    Quit,
}

fn play(
    files: Vec<PathBuf>,
    start: f64,
    playback: &PlaybackArgs,
    output: &OutputArgs,
) -> Result<()> {
    let total = files.len();
    let engine = Engine::with_device(output.settings(), playback.settings()?)
        .context("cannot open the audio output")?;
    let items = files.iter().map(QueueItem::new).collect();
    engine.set_queue(items, 0, start, true)?;

    let (keys_tx, keys) = mpsc::channel();
    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        for line in stdin.lock().lines() {
            let Ok(line) = line else { break };
            let line = line.trim();
            let key = match line {
                "" | "p" => Key::Toggle,
                "n" => Key::Next,
                "b" => Key::Previous,
                "q" => Key::Quit,
                _ => {
                    if let Some(seconds) =
                        line.strip_prefix('s').and_then(|s| s.trim().parse().ok())
                    {
                        Key::Seek(seconds)
                    } else if let Ok(delta) = line.parse::<f64>() {
                        Key::Relative(delta)
                    } else {
                        eprintln!("{KEYS}");
                        continue;
                    }
                }
            };
            if keys_tx.send(key).is_err() {
                break;
            }
        }
    });

    eprintln!("{KEYS}");
    let mut position = 0.0;
    let mut duration = None;
    let mut current = 0;
    loop {
        while let Ok(key) = keys.try_recv() {
            match key {
                Key::Toggle => engine.toggle_pause()?,
                Key::Next => engine.next()?,
                Key::Previous => engine.previous()?,
                Key::Seek(seconds) => engine.seek(seconds)?,
                Key::Relative(delta) => engine.seek((position + delta).max(0.0))?,
                Key::Quit => {
                    engine.stop()?;
                    eprintln!();
                    return Ok(());
                }
            }
        }

        match engine.events().recv_timeout(Duration::from_millis(50)) {
            Ok(Event::TrackStarted { index, path, info }) => {
                current = index;
                duration = info.duration_seconds();
                eprintln!(
                    "\r▶ [{}/{total}] {}  ({} Hz, {} ch{})",
                    index + 1,
                    path.display(),
                    info.sample_rate,
                    info.channels,
                    duration
                        .map(|d| format!(", {}", clock(d)))
                        .unwrap_or_default()
                );
            }
            Ok(Event::Position { seconds, .. }) => {
                position = seconds;
                let length = duration.map(clock).unwrap_or_else(|| "--:--".into());
                eprint!("\r  {} / {length}   ", clock(seconds));
                let _ = std::io::stderr().flush();
            }
            Ok(Event::Seeked { seconds, .. }) => position = seconds,
            Ok(Event::StateChanged(PlayState::Paused)) => eprint!("\r  ⏸ paused      "),
            Ok(Event::StateChanged(_)) => {}
            Ok(Event::TrackFailed {
                index,
                path,
                reason,
            }) => {
                eprintln!("\r✗ [{}/{total}] {}: {reason}", index + 1, path.display());
            }
            Ok(Event::OutputOpened {
                name,
                sample_rate,
                channels,
            }) => eprintln!("\r♪ output: {name} ({sample_rate} Hz, {channels} ch)"),
            Ok(Event::OutputLost { reason }) => {
                eprintln!("\r! output lost: {reason}; waiting for a device")
            }
            Ok(Event::QueueEnded) => {
                eprintln!("\r■ end of queue (last track {})", current + 1);
                return Ok(());
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => bail!("the audio engine stopped"),
        }
    }
}

fn render_wav(
    files: &[PathBuf],
    output: &Path,
    rate: u32,
    channels: u16,
    format: SampleArg,
    playback: &PlaybackArgs,
) -> Result<()> {
    let (engine, mut sink) = Engine::offline(rate, usize::from(channels), playback.settings()?);
    let items = files.iter().map(QueueItem::new).collect();
    engine.set_queue(items, 0, 0.0, true)?;
    sink.wait_for_audio(1);

    let format = match format {
        SampleArg::F32 => WavFormat::Float32,
        SampleArg::I16 => WavFormat::Int16,
    };
    let mut writer = WavWriter::create(output, rate, channels, format)?;
    let step = rate as usize * 10;
    let mut frames = 0usize;
    loop {
        let block = sink.render_to_end(step);
        frames += block.len() / usize::from(channels);
        writer.write(&block)?;
        if block.len() / usize::from(channels) < step {
            break;
        }
    }
    writer.finish()?;

    for event in engine.events().try_iter() {
        if let Event::TrackFailed { path, reason, .. } = event {
            eprintln!("✗ {}: {reason}", path.display());
        }
    }
    println!(
        "wrote {} ({} frames, {} at {rate} Hz)",
        output.display(),
        frames,
        clock(frames as f64 / f64::from(rate))
    );
    Ok(())
}

/// Seconds as `m:ss`.
fn clock(seconds: f64) -> String {
    let total = seconds.max(0.0) as u64;
    format!("{}:{:02}", total / 60, total % 60)
}
