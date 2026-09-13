//! The analysis tap: spectrum, meters and spectral centroid (SPEC §4.4).
//!
//! The output callback copies post-DSP samples into a lock-free ring buffer,
//! but only while analysis is switched on; otherwise it costs nothing. A
//! separate thread turns them into compact frames at most 60 times a
//! second, and parks itself while there is nothing to analyse, so a paused
//! or hidden player does no analysis work at all (SPEC §13.1).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use realfft::num_complex::Complex;
use realfft::{RealFftPlanner, RealToComplex};
use rtrb::Consumer;

/// FFT length (SPEC §4.4 default).
pub const DEFAULT_FFT_SIZE: usize = 2048;
/// Spectrum bands (SPEC §4.4 default).
pub const DEFAULT_BANDS: usize = 64;
/// Highest frame rate (SPEC §4.4).
pub const MAX_FPS: u32 = 60;

/// Bottom of the displayed range, in dBFS; 0 is the top.
const FLOOR_DB: f32 = -90.0;
/// How long a band's peak stays up before it falls.
const HOLD_SECONDS: f32 = 0.5;
/// How fast a released peak falls, in dB per second.
const FALL_DB_PER_SECOND: f32 = 30.0;

/// Analysis behaviour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnalysisSettings {
    /// Whether frames are produced at all.
    pub enabled: bool,
    /// Frames per second, at most [`MAX_FPS`].
    pub fps: u32,
    /// Number of spectrum bands.
    pub bands: usize,
    /// Whether the spectrum is wanted. With no visualizer on screen the
    /// meters alone are wanted, and the FFT behind the spectrum and the
    /// spectral centroid is work nobody would see (SPEC §4.4).
    pub spectrum: bool,
}

impl Default for AnalysisSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            fps: MAX_FPS,
            bands: DEFAULT_BANDS,
            spectrum: true,
        }
    }
}

/// One analysis frame, compact enough to send to the interface 60 times a
/// second.
#[derive(Debug, Clone, PartialEq)]
pub struct AnalysisFrame {
    /// Spectrum per band, 0 = -90 dBFS or below, 255 = 0 dBFS.
    pub bands: Vec<u8>,
    /// Held peak per band, same scale.
    pub peaks: Vec<u8>,
    /// Peak level per channel since the last frame, in dBFS.
    pub peak_db: Vec<f32>,
    /// RMS level per channel since the last frame, in dBFS.
    pub rms_db: Vec<f32>,
    /// Whether a sample reached full scale since the last frame.
    pub clip: bool,
    /// Whether the limiter reduced the gain since the last frame.
    pub limiting: bool,
    /// Spectral centroid in Hz (0 in silence), for tone colour (SPEC §9.5).
    pub centroid_hz: f32,
}

/// Turns blocks of samples into analysis frames. Not real-time: it runs on
/// the analysis thread.
pub struct Analyzer {
    fft: Arc<dyn RealToComplex<f32>>,
    size: usize,
    sample_rate: u32,
    channels: usize,
    window: Vec<f32>,
    /// The last `size` mono samples, as a ring.
    history: Vec<f32>,
    history_pos: usize,
    input: Vec<f32>,
    spectrum: Vec<Complex<f32>>,
    /// FFT bin range of each band.
    band_bins: Vec<(usize, usize)>,
    hold_db: Vec<f32>,
    hold_age: Vec<f32>,
    peak: Vec<f32>,
    sum_squares: Vec<f64>,
    counted: usize,
    clip: bool,
}

impl Analyzer {
    /// An analyser for `channels` channels at `sample_rate`.
    pub fn new(sample_rate: u32, channels: usize, bands: usize, size: usize) -> Self {
        let size = size.max(64).next_power_of_two();
        let bands = bands.clamp(1, 256);
        let channels = channels.max(1);
        let fft = RealFftPlanner::<f32>::new().plan_fft_forward(size);
        let window = (0..size)
            .map(|n| 0.5 - 0.5 * (std::f32::consts::TAU * n as f32 / size as f32).cos())
            .collect();
        let input = fft.make_input_vec();
        let spectrum = fft.make_output_vec();

        let top = 20_000f32.min(sample_rate as f32 * 0.5);
        let bin_of = |freq: f32| (freq * size as f32 / sample_rate as f32).round() as usize;
        let last_bin = size / 2;
        let band_bins = (0..bands)
            .map(|k| {
                let low = 20.0 * (top / 20.0).powf(k as f32 / bands as f32);
                let high = 20.0 * (top / 20.0).powf((k + 1) as f32 / bands as f32);
                let start = bin_of(low).clamp(1, last_bin);
                let end = bin_of(high).clamp(start + 1, last_bin + 1);
                (start, end)
            })
            .collect();

        Self {
            fft,
            size,
            sample_rate,
            channels,
            window,
            history: vec![0.0; size],
            history_pos: 0,
            input,
            spectrum,
            band_bins,
            hold_db: vec![FLOOR_DB; bands],
            hold_age: vec![0.0; bands],
            peak: vec![0.0; channels],
            sum_squares: vec![0.0; channels],
            counted: 0,
            clip: false,
        }
    }

    /// Takes interleaved samples in.
    pub fn push(&mut self, samples: &[f32]) {
        for frame in samples.chunks_exact(self.channels) {
            let mut mono = 0.0;
            for (channel, &sample) in frame.iter().enumerate() {
                let magnitude = sample.abs();
                self.peak[channel] = self.peak[channel].max(magnitude);
                self.sum_squares[channel] += f64::from(sample) * f64::from(sample);
                self.clip |= magnitude >= 1.0;
                mono += sample;
            }
            self.history[self.history_pos] = mono / self.channels as f32;
            self.history_pos = (self.history_pos + 1) % self.size;
            self.counted += 1;
        }
    }

    /// Produces a frame from what came in since the last one. `elapsed` is
    /// the time since the last frame, for the peak-hold fall. Without
    /// `spectrum` the FFT is skipped: the frame carries the meters alone.
    pub fn frame(&mut self, limiting: bool, elapsed: f32, spectrum: bool) -> AnalysisFrame {
        if !spectrum {
            return self.meters(limiting);
        }
        for (n, slot) in self.input.iter_mut().enumerate() {
            let sample = self.history[(self.history_pos + n) % self.size];
            *slot = sample * self.window[n];
        }
        // The lengths match by construction, so this cannot fail.
        let _ = self.fft.process(&mut self.input, &mut self.spectrum);

        // A full-scale sine through a Hann window peaks at size / 4.
        let scale = 4.0 / self.size as f32;
        let mut weighted = 0.0f64;
        let mut total = 0.0f64;
        for (bin, value) in self.spectrum.iter().enumerate().skip(1) {
            let magnitude = f64::from(value.norm() * scale);
            weighted += magnitude * bin as f64;
            total += magnitude;
        }
        let bin_hz = self.sample_rate as f64 / self.size as f64;
        let centroid_hz = if total > 1e-6 {
            (weighted / total * bin_hz) as f32
        } else {
            0.0
        };

        let mut bands = Vec::with_capacity(self.band_bins.len());
        let mut peaks = Vec::with_capacity(self.band_bins.len());
        for (index, &(start, end)) in self.band_bins.iter().enumerate() {
            let magnitude = self.spectrum[start..end]
                .iter()
                .map(|value| value.norm() * scale)
                .fold(0.0f32, f32::max);
            let db = 20.0 * magnitude.max(1e-9).log10();
            if db >= self.hold_db[index] {
                self.hold_db[index] = db;
                self.hold_age[index] = 0.0;
            } else {
                self.hold_age[index] += elapsed;
                if self.hold_age[index] > HOLD_SECONDS {
                    self.hold_db[index] =
                        (self.hold_db[index] - FALL_DB_PER_SECOND * elapsed).max(db);
                }
            }
            bands.push(to_byte(db));
            peaks.push(to_byte(self.hold_db[index]));
        }

        self.finish(bands, peaks, centroid_hz, limiting)
    }

    /// A frame with the meters only, for when nothing shows a spectrum.
    fn meters(&mut self, limiting: bool) -> AnalysisFrame {
        self.finish(Vec::new(), Vec::new(), 0.0, limiting)
    }

    /// Completes a frame with the meter levels, and starts the next one.
    fn finish(
        &mut self,
        bands: Vec<u8>,
        peaks: Vec<u8>,
        centroid_hz: f32,
        limiting: bool,
    ) -> AnalysisFrame {
        let counted = self.counted.max(1) as f64;
        let frame = AnalysisFrame {
            bands,
            peaks,
            peak_db: self.peak.iter().map(|peak| level_db(*peak)).collect(),
            rms_db: self
                .sum_squares
                .iter()
                .map(|sum| level_db((sum / counted).sqrt() as f32))
                .collect(),
            clip: self.clip,
            limiting,
            centroid_hz,
        };
        self.peak.fill(0.0);
        self.sum_squares.fill(0.0);
        self.counted = 0;
        self.clip = false;
        frame
    }
}

fn level_db(level: f32) -> f32 {
    20.0 * level.max(1e-6).log10()
}

fn to_byte(db: f32) -> u8 {
    ((db - FLOOR_DB) / -FLOOR_DB * 255.0).clamp(0.0, 255.0) as u8
}

/// Switches the analysis thread on and off without polling.
pub struct AnalysisControl {
    active: AtomicBool,
    stop: AtomicBool,
}

/// The running analysis thread. Dropping it stops the thread.
pub struct AnalysisThread {
    control: Arc<AnalysisControl>,
    /// Hands the tap back when the thread ends.
    handle: Option<JoinHandle<Consumer<f32>>>,
}

impl AnalysisThread {
    /// Starts the thread on the tap of one output. `limiting` reports
    /// whether the limiter is working; `emit` receives every frame.
    pub fn spawn(
        mut tap: Consumer<f32>,
        sample_rate: u32,
        channels: usize,
        settings: AnalysisSettings,
        limiting: impl Fn() -> bool + Send + 'static,
        emit: impl Fn(AnalysisFrame) + Send + 'static,
    ) -> Self {
        let control = Arc::new(AnalysisControl {
            active: AtomicBool::new(false),
            stop: AtomicBool::new(false),
        });
        let thread_control = control.clone();
        let interval = Duration::from_secs_f32(1.0 / settings.fps.clamp(1, MAX_FPS) as f32);
        let handle = std::thread::Builder::new()
            .name("onsa-analysis".into())
            .spawn(move || {
                let mut analyzer =
                    Analyzer::new(sample_rate, channels, settings.bands, DEFAULT_FFT_SIZE);
                let mut last = Instant::now();
                loop {
                    if thread_control.stop.load(Ordering::Acquire) {
                        return tap;
                    }
                    if !thread_control.active.load(Ordering::Acquire) {
                        std::thread::park();
                        last = Instant::now();
                        continue;
                    }
                    std::thread::sleep(interval);
                    let available = tap.slots();
                    if available == 0 {
                        continue;
                    }
                    if let Ok(chunk) = tap.read_chunk(available) {
                        let (first, second) = chunk.as_slices();
                        analyzer.push(first);
                        analyzer.push(second);
                        chunk.commit_all();
                    }
                    let elapsed = last.elapsed().as_secs_f32();
                    last = Instant::now();
                    emit(analyzer.frame(limiting(), elapsed, settings.spectrum));
                }
            })
            .ok();
        Self { control, handle }
    }

    /// Wakes the thread, or lets it park.
    pub fn set_active(&self, active: bool) {
        self.control.active.store(active, Ordering::Release);
        if active {
            if let Some(handle) = &self.handle {
                handle.thread().unpark();
            }
        }
    }

    /// Ends the thread and hands back the tap, so analysis can restart on
    /// the same output with other settings.
    pub fn stop(mut self) -> Option<Consumer<f32>> {
        self.control.stop.store(true, Ordering::Release);
        let handle = self.handle.take()?;
        handle.thread().unpark();
        handle.join().ok()
    }
}

impl Drop for AnalysisThread {
    fn drop(&mut self) {
        self.control.stop.store(true, Ordering::Release);
        if let Some(handle) = self.handle.take() {
            handle.thread().unpark();
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine(frames: usize, freq: f32, amplitude: f32) -> Vec<f32> {
        (0..frames)
            .flat_map(|n| {
                let v = (std::f32::consts::TAU * freq * n as f32 / 48_000.0).sin() * amplitude;
                [v, v]
            })
            .collect()
    }

    #[test]
    fn measures_a_sine() {
        let mut analyzer = Analyzer::new(48_000, 2, DEFAULT_BANDS, DEFAULT_FFT_SIZE);
        analyzer.push(&sine(4800, 1000.0, 0.5));
        let frame = analyzer.frame(false, 1.0 / 60.0, true);
        assert!(
            (frame.peak_db[0] + 6.02).abs() < 0.05,
            "peak {}",
            frame.peak_db[0]
        );
        assert!(
            (frame.rms_db[1] + 9.03).abs() < 0.05,
            "rms {}",
            frame.rms_db[1]
        );
        assert!(
            (frame.centroid_hz - 1000.0).abs() < 60.0,
            "centroid {}",
            frame.centroid_hz
        );
        assert!(!frame.clip);

        let loudest = frame
            .bands
            .iter()
            .enumerate()
            .max_by_key(|(_, value)| **value)
            .map(|(index, _)| index)
            .unwrap();
        let (start, end) = analyzer.band_bins[loudest];
        let bin = 1000.0 * DEFAULT_FFT_SIZE as f32 / 48_000.0;
        assert!((start as f32 - 1.0..end as f32 + 1.0).contains(&bin));
        // Within a dB or so of -6 dBFS on the 0..255 scale.
        let expected = to_byte(-6.02);
        assert!(
            frame.bands[loudest].abs_diff(expected) <= 4,
            "{} vs {expected}",
            frame.bands[loudest]
        );
    }

    #[test]
    fn silence_reads_as_silence() {
        let mut analyzer = Analyzer::new(48_000, 2, DEFAULT_BANDS, DEFAULT_FFT_SIZE);
        analyzer.push(&vec![0.0; 2 * 4800]);
        let frame = analyzer.frame(false, 0.1, true);
        assert!(frame.bands.iter().all(|value| *value == 0));
        assert_eq!(frame.centroid_hz, 0.0);
        assert!(frame.peak_db.iter().all(|db| *db <= -119.0));
    }

    #[test]
    fn meters_alone_skip_the_spectrum() {
        // With no visualizer on screen the levels still have to be right,
        // but the FFT behind the spectrum is not run at all (SPEC §4.4).
        let mut analyzer = Analyzer::new(48_000, 2, DEFAULT_BANDS, DEFAULT_FFT_SIZE);
        analyzer.push(&sine(4800, 1000.0, 0.5));
        let frame = analyzer.frame(false, 1.0 / 60.0, false);
        assert!(frame.bands.is_empty(), "no spectrum was asked for");
        assert!(frame.peaks.is_empty());
        assert_eq!(frame.centroid_hz, 0.0);
        assert!(
            (frame.peak_db[0] + 6.02).abs() < 0.05,
            "peak {}",
            frame.peak_db[0]
        );
    }

    #[test]
    fn peaks_hold_then_fall() {
        let mut analyzer = Analyzer::new(48_000, 2, DEFAULT_BANDS, DEFAULT_FFT_SIZE);
        analyzer.push(&sine(4800, 1000.0, 0.5));
        let loud = analyzer.frame(false, 0.1, true);
        let top = *loud.peaks.iter().max().unwrap();
        analyzer.push(&vec![0.0; 2 * 4800]);
        let held = analyzer.frame(false, 0.1, true);
        assert_eq!(*held.peaks.iter().max().unwrap(), top, "peak should hold");
        for _ in 0..20 {
            analyzer.push(&vec![0.0; 2 * 480]);
            analyzer.frame(false, 0.1, true);
        }
        let fallen = analyzer.frame(false, 0.1, true);
        assert!(
            *fallen.peaks.iter().max().unwrap() < top,
            "peak should fall"
        );
    }

    #[test]
    fn full_scale_counts_as_clipping() {
        let mut analyzer = Analyzer::new(48_000, 2, DEFAULT_BANDS, DEFAULT_FFT_SIZE);
        analyzer.push(&[1.0, 0.0]);
        assert!(analyzer.frame(true, 0.1, true).clip);
        assert!(
            !analyzer.frame(false, 0.1, true).clip,
            "clip resets each frame"
        );
    }
}
