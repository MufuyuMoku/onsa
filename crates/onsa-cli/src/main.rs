//! Command line front end for the Onsa audio engine.
//!
//! It exercises the engine without the user interface (SPEC §15, M1 and M2):
//! `devices` lists outputs, `play` and `queue` play through a device with
//! line-based keyboard control over transport and the DSP chain, `render-wav`
//! renders through the offline sink into a WAV file, and `eq-check` reads an
//! Equalizer APO / AutoEQ preset the way the engine would.

use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use onsa_audio::dsp::autoeq;
use onsa_audio::dsp::eq::{auto_preamp_db, response_curve, GRAPHIC_FREQS};
use onsa_audio::wav::{WavFormat, WavWriter};
use onsa_audio::{
    list_devices, AnalysisFrame, AnalysisSettings, BufferSize, CrossfadeCurve, DeviceChoice,
    DspSettings, Engine, EqMode, Event, FileSource, FilterKind, OutputRate, OutputSettings,
    PlayState, PlaybackSettings, QueueItem, ReplayGainMode, ReplayGainSettings, ResamplerQuality,
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
        dsp: DspArgs,
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
        dsp: DspArgs,
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
        #[command(flatten)]
        dsp: DspArgs,
    },
    /// Read an Equalizer APO / AutoEQ preset (ParametricEQ.txt) and show the
    /// filters, anything skipped, the auto preamp, and the file as Onsa writes it.
    EqCheck {
        /// The preset file.
        file: PathBuf,
        /// Sample rate to evaluate the curve at.
        #[arg(long, default_value_t = 48_000)]
        rate: u32,
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
struct DspArgs {
    /// Graphic EQ: ten gains in dB for 31 Hz to 16 kHz, comma separated,
    /// for example `3,2,0,0,0,0,0,1,2,3`.
    #[arg(long, allow_hyphen_values = true)]
    eq: Option<String>,
    /// Parametric EQ from an Equalizer APO / AutoEQ file (ParametricEQ.txt).
    #[arg(long, conflicts_with = "eq")]
    eq_file: Option<PathBuf>,
    /// Preamp in dB. An AutoEQ file's own preamp is used unless this is given.
    #[arg(long, allow_hyphen_values = true)]
    preamp: Option<f32>,
    /// Set the preamp from the EQ curve, so the EQ alone cannot clip.
    #[arg(long)]
    auto_preamp: bool,
    /// Switch the peak limiter off (its 5 ms delay stays).
    #[arg(long)]
    no_limiter: bool,
    /// Limiter release in milliseconds.
    #[arg(long, default_value_t = 80.0)]
    limiter_release: f32,
    /// Output volume in dB (0 is unity).
    #[arg(long, default_value_t = 0.0, allow_hyphen_values = true)]
    volume: f32,
    /// ReplayGain mode.
    #[arg(long, value_enum, default_value_t = RgArg::Off)]
    replaygain: RgArg,
    /// ReplayGain preamp in dB, for tracks with tags.
    #[arg(long, default_value_t = 0.0, allow_hyphen_values = true)]
    rg_preamp: f32,
    /// Gain in dB for tracks without ReplayGain tags.
    #[arg(long, default_value_t = 0.0, allow_hyphen_values = true)]
    rg_fallback: f32,
    /// No dither, even for 16-bit output.
    #[arg(long)]
    no_dither: bool,
    /// Show peak and RMS meters, limiter activity and the spectral centroid.
    #[arg(long)]
    meter: bool,
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

#[derive(Clone, Copy, ValueEnum)]
enum RgArg {
    Off,
    Track,
    Album,
    Auto,
}

impl From<RgArg> for ReplayGainMode {
    fn from(arg: RgArg) -> Self {
        match arg {
            RgArg::Off => Self::Off,
            RgArg::Track => Self::Track,
            RgArg::Album => Self::Album,
            RgArg::Auto => Self::Auto,
        }
    }
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

impl DspArgs {
    fn settings(&self) -> Result<DspSettings> {
        let mut settings = DspSettings::default();
        let mut file_preamp = None;
        if let Some(text) = &self.eq {
            settings.eq = EqMode::Graphic(parse_graphic(text)?);
            settings.eq_enabled = true;
        }
        if let Some(path) = &self.eq_file {
            let bytes =
                std::fs::read(path).with_context(|| format!("cannot read {}", path.display()))?;
            let import = autoeq::parse(&String::from_utf8_lossy(&bytes));
            for warning in &import.warnings {
                eprintln!("! {}: {warning}", path.display());
            }
            if import.preset.bands.is_empty() {
                bail!("{} has no usable filters", path.display());
            }
            file_preamp = import.preset.preamp_db;
            settings.eq = EqMode::Parametric(import.preset.bands);
            settings.eq_enabled = true;
        }
        settings.preamp_db = self
            .preamp
            .or(file_preamp.map(|db| db as f32))
            .unwrap_or(0.0);
        settings.auto_preamp = self.auto_preamp;
        settings.limiter_enabled = !self.no_limiter;
        settings.limiter_release_ms = self.limiter_release;
        settings.volume_db = self.volume;
        settings.dither = !self.no_dither;
        settings.replaygain = ReplayGainSettings {
            mode: self.replaygain.into(),
            preamp_db: self.rg_preamp,
            fallback_db: self.rg_fallback,
            prevent_clipping: true,
        };
        Ok(settings)
    }
}

/// Ten comma-separated gains in dB.
fn parse_graphic(text: &str) -> Result<[f64; 10]> {
    let values = text
        .split(',')
        .map(|value| value.trim().parse::<f64>())
        .collect::<std::result::Result<Vec<f64>, _>>()
        .context("--eq takes ten gains in dB, comma separated")?;
    let gains: [f64; 10] = values
        .try_into()
        .map_err(|values: Vec<f64>| anyhow!("--eq takes ten gains, got {}", values.len()))?;
    if gains.iter().any(|gain| gain.abs() > 24.0) {
        bail!("EQ gains must stay within ±24 dB");
    }
    Ok(gains)
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
            dsp,
            output,
        } => play(vec![file], start, &playback, &dsp, &output),
        Command::Queue {
            mut files,
            list,
            playback,
            dsp,
            output,
        } => {
            if let Some(list) = list {
                files.extend(read_list(&list)?);
            }
            if files.is_empty() {
                bail!("nothing to play: give files or --list");
            }
            play(files, 0.0, &playback, &dsp, &output)
        }
        Command::RenderWav {
            files,
            output,
            rate,
            channels,
            format,
            playback,
            dsp,
        } => render_wav(&files, &output, rate, channels, format, &playback, &dsp),
        Command::EqCheck { file, rate } => eq_check(&file, rate),
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

/// Queue entries. ReplayGain's auto mode decides album gain from the album
/// and track number of every entry, which the caller supplies; here they come
/// from each file's tags.
fn queue_items(files: &[PathBuf], read_tags: bool) -> Vec<QueueItem> {
    files
        .iter()
        .map(|path| {
            let mut item = QueueItem::new(path);
            if read_tags {
                if let Ok(source) = FileSource::open(path, 2) {
                    item.album = source.info().album.clone();
                    item.track_number = source.info().track_number;
                }
            }
            item
        })
        .collect()
}

const KEYS: &str = "keys (type, then Enter):
  [Enter]/p pause · n next · b previous · s SECONDS seek · +N/-N skip · q quit
  eq on|off|flat · eq BAND DB (band 1-10 = 31 Hz..16 kHz) · pre DB · auto
  vol DB · lim on|off · rg off|track|album|auto · m meters";

/// What the keyboard thread asks for.
enum Key {
    Toggle,
    Next,
    Previous,
    Seek(f64),
    Relative(f64),
    Meter,
    Dsp(DspKey),
    Quit,
}

/// A change to the DSP chain from the keyboard.
enum DspKey {
    EqEnabled(bool),
    EqFlat,
    EqBand(usize, f64),
    Preamp(f32),
    AutoPreamp,
    Volume(f32),
    Limiter(bool),
    ReplayGain(ReplayGainMode),
}

fn parse_key(line: &str) -> Option<Key> {
    let words: Vec<&str> = line.split_whitespace().collect();
    let number = |text: &str| text.parse::<f64>().ok();
    Some(match words.as_slice() {
        [] | ["p"] => Key::Toggle,
        ["n"] => Key::Next,
        ["b"] => Key::Previous,
        ["q"] => Key::Quit,
        ["m"] => Key::Meter,
        ["s", seconds] => Key::Seek(number(seconds)?),
        ["eq", "on"] => Key::Dsp(DspKey::EqEnabled(true)),
        ["eq", "off"] => Key::Dsp(DspKey::EqEnabled(false)),
        ["eq", "flat"] => Key::Dsp(DspKey::EqFlat),
        ["eq", band, gain] => {
            let band = band
                .parse::<usize>()
                .ok()
                .filter(|b| (1..=10).contains(b))?;
            Key::Dsp(DspKey::EqBand(band - 1, number(gain)?.clamp(-24.0, 24.0)))
        }
        ["pre", db] => Key::Dsp(DspKey::Preamp(number(db)? as f32)),
        ["auto"] => Key::Dsp(DspKey::AutoPreamp),
        ["vol", db] => Key::Dsp(DspKey::Volume(number(db)? as f32)),
        ["lim", "on"] => Key::Dsp(DspKey::Limiter(true)),
        ["lim", "off"] => Key::Dsp(DspKey::Limiter(false)),
        ["rg", mode] => Key::Dsp(DspKey::ReplayGain(match *mode {
            "off" => ReplayGainMode::Off,
            "track" => ReplayGainMode::Track,
            "album" => ReplayGainMode::Album,
            "auto" => ReplayGainMode::Auto,
            _ => return None,
        })),
        [word] if word.starts_with('s') && number(&word[1..]).is_some() => {
            Key::Seek(number(&word[1..])?)
        }
        [word] => Key::Relative(number(word)?),
        _ => return None,
    })
}

/// Applies a keyboard change. Returns a note when it cannot apply.
fn apply_dsp_key(settings: &mut DspSettings, key: DspKey) -> Option<&'static str> {
    match key {
        DspKey::EqEnabled(enabled) => settings.eq_enabled = enabled,
        DspKey::EqFlat => {
            settings.eq = EqMode::Graphic([0.0; 10]);
            settings.eq_enabled = true;
        }
        DspKey::EqBand(band, gain) => match &mut settings.eq {
            EqMode::Graphic(gains) => {
                gains[band] = gain;
                settings.eq_enabled = true;
            }
            EqMode::Parametric(_) => {
                return Some("the EQ is parametric (from a file); use eq flat to switch to graphic")
            }
        },
        DspKey::Preamp(db) => {
            settings.preamp_db = db.clamp(-30.0, 30.0);
            settings.auto_preamp = false;
        }
        DspKey::AutoPreamp => settings.auto_preamp = !settings.auto_preamp,
        DspKey::Volume(db) => settings.volume_db = db.clamp(-90.0, 12.0),
        DspKey::Limiter(enabled) => settings.limiter_enabled = enabled,
        DspKey::ReplayGain(mode) => settings.replaygain.mode = mode,
    }
    None
}

/// One line describing the chain as it now stands.
fn describe(settings: &DspSettings, sample_rate: u32) -> String {
    let eq = if !settings.eq_enabled {
        "EQ off".to_string()
    } else {
        match &settings.eq {
            EqMode::Graphic(gains) => format!(
                "EQ {}",
                gains
                    .iter()
                    .map(|gain| format!("{gain:+.0}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            EqMode::Parametric(bands) => format!("EQ parametric, {} filters", bands.len()),
        }
    };
    let preamp = if settings.auto_preamp {
        format!(
            "preamp {:+.1} dB (auto)",
            settings.effective_preamp_db(sample_rate)
        )
    } else {
        format!("preamp {:+.1} dB", settings.preamp_db)
    };
    let limiter = if settings.limiter_enabled {
        "limiter on"
    } else {
        "limiter off"
    };
    let replaygain = match settings.replaygain.mode {
        ReplayGainMode::Off => "ReplayGain off",
        ReplayGainMode::Track => "ReplayGain track",
        ReplayGainMode::Album => "ReplayGain album",
        ReplayGainMode::Auto => "ReplayGain auto",
    };
    format!(
        "{eq} · {preamp} · {limiter} · volume {:+.1} dB · {replaygain}",
        settings.volume_db
    )
}

/// Meters in one short line.
fn format_meter(frame: &AnalysisFrame) -> String {
    let channel = |index: usize| {
        format!(
            "{:>5.1}/{:>5.1}",
            frame
                .peak_db
                .get(index)
                .copied()
                .unwrap_or(-120.0)
                .max(-99.9),
            frame
                .rms_db
                .get(index)
                .copied()
                .unwrap_or(-120.0)
                .max(-99.9)
        )
    };
    format!(
        "L {} R {} dB {}{} {:>5.0} Hz",
        channel(0),
        channel(1),
        if frame.limiting { "LIM" } else { "   " },
        if frame.clip { " CLIP" } else { "" },
        frame.centroid_hz
    )
}

fn play(
    files: Vec<PathBuf>,
    start: f64,
    playback: &PlaybackArgs,
    dsp: &DspArgs,
    output: &OutputArgs,
) -> Result<()> {
    let total = files.len();
    let mut dsp_settings = dsp.settings()?;
    let engine = Engine::with_device(output.settings(), playback.settings()?)
        .context("cannot open the audio output")?;
    // Settings go first, so the very first block is already processed.
    engine.set_dsp(dsp_settings.clone())?;
    let meter_settings = AnalysisSettings {
        enabled: true,
        fps: 30,
        ..AnalysisSettings::default()
    };
    let mut meter_on = dsp.meter;
    if meter_on {
        engine.set_analysis(meter_settings)?;
    }
    let read_tags = dsp_settings.replaygain.mode == ReplayGainMode::Auto;
    engine.set_queue(queue_items(&files, read_tags), 0, start, true)?;

    let (keys_tx, keys) = mpsc::channel();
    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        for line in stdin.lock().lines() {
            let Ok(line) = line else { break };
            match parse_key(line.trim()) {
                Some(key) => {
                    if keys_tx.send(key).is_err() {
                        break;
                    }
                }
                None => eprintln!("{KEYS}"),
            }
        }
    });

    eprintln!("{KEYS}");
    let mut position = 0.0;
    let mut duration = None;
    let mut current = 0;
    let mut sample_rate = 48_000;
    let mut meter_text = String::new();
    let mut last_meter = Instant::now();
    loop {
        while let Ok(key) = keys.try_recv() {
            match key {
                Key::Toggle => engine.toggle_pause()?,
                Key::Next => engine.next()?,
                Key::Previous => engine.previous()?,
                Key::Seek(seconds) => engine.seek(seconds)?,
                Key::Relative(delta) => engine.seek((position + delta).max(0.0))?,
                Key::Meter => {
                    meter_on = !meter_on;
                    engine.set_analysis(AnalysisSettings {
                        enabled: meter_on,
                        ..meter_settings
                    })?;
                    meter_text.clear();
                }
                Key::Dsp(change) => {
                    if let Some(note) = apply_dsp_key(&mut dsp_settings, change) {
                        eprintln!("\r! {note}");
                    } else {
                        engine.set_dsp(dsp_settings.clone())?;
                        eprintln!("\r♪ {}", describe(&dsp_settings, sample_rate));
                    }
                }
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
                let rg = &info.replaygain;
                let tags = match (rg.track_gain_db, rg.album_gain_db) {
                    (None, None) => String::new(),
                    (track, album) => format!(
                        ", RG track {} album {}",
                        track.map_or("-".into(), |db| format!("{db:+.2} dB")),
                        album.map_or("-".into(), |db| format!("{db:+.2} dB"))
                    ),
                };
                eprintln!(
                    "\r▶ [{}/{total}] {}  ({} Hz, {} ch{}{tags})",
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
                eprint!("\r  {} / {length}  {meter_text}   ", clock(seconds));
                let _ = std::io::stderr().flush();
            }
            Ok(Event::Analysis(frame)) => {
                if meter_on && last_meter.elapsed() >= Duration::from_millis(100) {
                    last_meter = Instant::now();
                    meter_text = format_meter(&frame);
                }
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
                sample_rate: rate,
                channels,
            }) => {
                sample_rate = rate;
                eprintln!("\r♪ output: {name} ({rate} Hz, {channels} ch)");
                eprintln!("\r♪ {}", describe(&dsp_settings, sample_rate));
            }
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
    dsp: &DspArgs,
) -> Result<()> {
    let dsp_settings = dsp.settings()?;
    let read_tags = dsp_settings.replaygain.mode == ReplayGainMode::Auto;
    let (engine, mut sink) = Engine::offline(rate, usize::from(channels), playback.settings()?);
    engine.set_dsp(dsp_settings)?;
    engine.set_queue(queue_items(files, read_tags), 0, 0.0, true)?;
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

fn eq_check(file: &Path, rate: u32) -> Result<()> {
    let bytes = std::fs::read(file).with_context(|| format!("cannot read {}", file.display()))?;
    let import = autoeq::parse(&String::from_utf8_lossy(&bytes));
    let bands = &import.preset.bands;
    println!("{}", file.display());
    match import.preset.preamp_db {
        Some(db) => println!("  preamp in the file: {db:+.1} dB"),
        None => println!("  no preamp line"),
    }
    for (index, band) in bands.iter().enumerate() {
        println!(
            "  {:>2}. {:<10} {:>8.1} Hz  {:>+6.1} dB  Q {:.2}",
            index + 1,
            kind_name(band.kind),
            band.freq,
            band.gain_db,
            band.q
        );
    }
    if bands.is_empty() {
        println!("  no usable filters");
    }
    for warning in &import.warnings {
        println!("  ! {warning}");
    }
    let boost = response_curve(bands, rate, 1024)
        .into_iter()
        .map(|(_, db)| db)
        .fold(f64::MIN, f64::max);
    println!(
        "  highest point of the curve: {boost:+.1} dB; auto preamp would be {:+.1} dB",
        auto_preamp_db(bands, rate)
    );
    println!(
        "  graphic bands for reference: {} Hz",
        GRAPHIC_FREQS
            .iter()
            .map(|freq| format!("{freq}"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!(
        "\nAs Onsa writes it:\n{}",
        autoeq::export(import.preset.preamp_db.unwrap_or(0.0), bands)
    );
    Ok(())
}

fn kind_name(kind: FilterKind) -> &'static str {
    match kind {
        FilterKind::Peaking => "peaking",
        FilterKind::LowShelf => "low shelf",
        FilterKind::HighShelf => "high shelf",
        FilterKind::LowPass => "low pass",
        FilterKind::HighPass => "high pass",
        FilterKind::Notch => "notch",
    }
}

/// Seconds as `m:ss`.
fn clock(seconds: f64) -> String {
    let total = seconds.max(0.0) as u64;
    format!("{}:{:02}", total / 60, total % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_keyboard_commands() {
        assert!(matches!(parse_key(""), Some(Key::Toggle)));
        assert!(matches!(parse_key("s 90"), Some(Key::Seek(s)) if s == 90.0));
        assert!(matches!(parse_key("s90"), Some(Key::Seek(s)) if s == 90.0));
        assert!(matches!(parse_key("-10"), Some(Key::Relative(d)) if d == -10.0));
        assert!(matches!(
            parse_key("eq 6 +9"),
            Some(Key::Dsp(DspKey::EqBand(5, g))) if g == 9.0
        ));
        assert!(matches!(parse_key("vol -6"), Some(Key::Dsp(DspKey::Volume(v))) if v == -6.0));
        assert!(matches!(
            parse_key("rg album"),
            Some(Key::Dsp(DspKey::ReplayGain(ReplayGainMode::Album)))
        ));
        assert!(parse_key("eq 11 3").is_none());
        assert!(parse_key("dance").is_none());
    }

    #[test]
    fn reads_graphic_gains() {
        assert_eq!(parse_graphic("3,2,0,0,0,0,0,1,2,-3").unwrap()[9], -3.0);
        assert!(parse_graphic("1,2,3").is_err());
        assert!(parse_graphic("30,0,0,0,0,0,0,0,0,0").is_err());
    }
}
