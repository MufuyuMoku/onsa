//! Output to a real audio device through cpal (SPEC §3.4).

use std::sync::Arc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{
    Device, ErrorKind, FromSample, OutputCallbackInfo, SampleFormat, SizedSample, Stream,
    StreamConfig, SupportedStreamConfig,
};
use rtrb::{Producer, RingBuffer};

use super::stage::{OutputStage, Shared};
use crate::error::{Error, Result};

/// Largest callback the stage handles in one pass; bigger callbacks are
/// served in several passes over the same preallocated scratch buffer.
const MAX_PASS_FRAMES: usize = 8192;

/// Which device to play on.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum DeviceChoice {
    /// The system default, following it when it changes.
    #[default]
    SystemDefault,
    /// A specific device, by the identifier [`list_devices`] reports.
    Id(String),
}

/// Which sample rate to open the device at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputRate {
    /// Whatever the device is set to (SPEC §3.4 default).
    #[default]
    FollowDevice,
    /// A fixed rate, when the device supports it; otherwise the device rate.
    Fixed(u32),
}

/// An output device as reported to the user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    /// Stable identifier, usable with [`DeviceChoice::Id`].
    pub id: String,
    /// Human readable name.
    pub name: String,
    /// Whether this is the current system default.
    pub is_default: bool,
    /// Default sample rate of the device.
    pub sample_rate: u32,
    /// Default channel count of the device.
    pub channels: u16,
}

/// Something went wrong with a running stream.
#[derive(Debug, Clone)]
pub struct DeviceFault {
    /// Whether the stream is gone and has to be rebuilt.
    pub lost: bool,
    /// What the backend reported.
    pub message: String,
}

/// A running device stream. Dropping it stops the stream.
pub struct DeviceOutput {
    _stream: Stream,
    /// Name of the device.
    pub name: String,
    /// Identifier of the device, when the backend reports one.
    pub id: Option<String>,
    /// Sample rate the stream runs at.
    pub sample_rate: u32,
    /// Channel count of the stream.
    pub channels: usize,
    /// Sample format the device takes.
    pub sample_format: SampleFormat,
}

/// Lists the output devices of the default host.
pub fn list_devices() -> Result<Vec<DeviceInfo>> {
    let host = cpal::default_host();
    let default_id = host
        .default_output_device()
        .and_then(|device| device.id().ok())
        .map(|id| id.to_string());
    let devices = host
        .output_devices()
        .map_err(|error| Error::Output(error.to_string()))?;
    let mut list = Vec::new();
    for device in devices {
        let Ok(config) = device.default_output_config() else {
            continue;
        };
        let id = device.id().map(|id| id.to_string()).unwrap_or_default();
        let name = device
            .description()
            .map(|description| description.to_string())
            .unwrap_or_else(|_| id.clone());
        list.push(DeviceInfo {
            is_default: default_id.as_deref() == Some(id.as_str()),
            id,
            name,
            sample_rate: config.sample_rate(),
            channels: config.channels(),
        });
    }
    Ok(list)
}

/// Identifier of the current system default output device.
pub fn default_device_id() -> Option<String> {
    cpal::default_host()
        .default_output_device()
        .and_then(|device| device.id().ok())
        .map(|id| id.to_string())
}

/// Opens a stream on the chosen device. A chosen device that no longer
/// exists falls back to the system default.
///
/// `buffer_seconds` sizes the ring buffer between the engine and the
/// callback. `on_fault` runs on a backend thread, never inside the audio
/// callback.
pub fn open_device(
    choice: &DeviceChoice,
    rate: OutputRate,
    buffer_seconds: f32,
    shared: Arc<Shared>,
    on_fault: impl Fn(DeviceFault) + Send + 'static,
) -> Result<(DeviceOutput, Producer<f32>)> {
    let host = cpal::default_host();
    let device = match choice {
        DeviceChoice::SystemDefault => None,
        DeviceChoice::Id(id) => id
            .parse()
            .ok()
            .and_then(|id| host.device_by_id(&id))
            .or_else(|| {
                tracing::warn!(device = %id, "chosen output device is gone, using the default");
                None
            }),
    }
    .or_else(|| host.default_output_device())
    .ok_or(Error::NoOutputDevice)?;

    let config = pick_config(&device, rate)?;
    let sample_rate = config.sample_rate();
    let channels = usize::from(config.channels());
    let sample_format = config.sample_format();

    let capacity = ((sample_rate as f32 * buffer_seconds) as usize).max(1024) * channels;
    let (producer, consumer) = RingBuffer::new(capacity);
    let stage = OutputStage::new(consumer, shared, channels, sample_rate);

    let stream_config: StreamConfig = config.config();
    let stream = match sample_format {
        SampleFormat::F32 => build::<f32>(&device, stream_config, stage, on_fault),
        SampleFormat::F64 => build::<f64>(&device, stream_config, stage, on_fault),
        SampleFormat::I16 => build::<i16>(&device, stream_config, stage, on_fault),
        SampleFormat::I32 => build::<i32>(&device, stream_config, stage, on_fault),
        SampleFormat::I24 => build::<cpal::I24>(&device, stream_config, stage, on_fault),
        SampleFormat::U16 => build::<u16>(&device, stream_config, stage, on_fault),
        SampleFormat::U8 => build::<u8>(&device, stream_config, stage, on_fault),
        SampleFormat::I8 => build::<i8>(&device, stream_config, stage, on_fault),
        other => Err(Error::Output(format!("unsupported sample format {other}"))),
    }?;
    stream
        .play()
        .map_err(|error| Error::Output(error.to_string()))?;

    let name = device
        .description()
        .map(|description| description.to_string())
        .unwrap_or_else(|_| "unknown device".to_string());
    let id = device.id().ok().map(|id| id.to_string());
    Ok((
        DeviceOutput {
            _stream: stream,
            name,
            id,
            sample_rate,
            channels,
            sample_format,
        },
        producer,
    ))
}

fn pick_config(device: &Device, rate: OutputRate) -> Result<SupportedStreamConfig> {
    let default = device
        .default_output_config()
        .map_err(|error| Error::Output(error.to_string()))?;
    let OutputRate::Fixed(wanted) = rate else {
        return Ok(default);
    };
    if default.sample_rate() == wanted {
        return Ok(default);
    }
    let ranges = device
        .supported_output_configs()
        .map_err(|error| Error::Output(error.to_string()))?;
    let matching = ranges
        .filter(|range| {
            range.channels() == default.channels()
                && range.sample_format() == default.sample_format()
        })
        .find_map(|range| range.try_with_sample_rate(wanted));
    Ok(matching.unwrap_or_else(|| {
        tracing::warn!(
            rate = wanted,
            "device cannot run at the fixed rate, using its own"
        );
        default
    }))
}

fn build<T>(
    device: &Device,
    config: StreamConfig,
    mut stage: OutputStage,
    on_fault: impl Fn(DeviceFault) + Send + 'static,
) -> Result<Stream>
where
    T: SizedSample + FromSample<f32>,
{
    let channels = usize::from(config.channels).max(1);
    // Allocated here, once; the callback only ever slices it.
    let mut scratch = vec![0.0f32; MAX_PASS_FRAMES * channels];

    let data = move |data: &mut [T], _: &OutputCallbackInfo| {
        for chunk in data.chunks_mut(scratch.len()) {
            let pass = &mut scratch[..chunk.len()];
            stage.process(pass);
            for (sample, value) in chunk.iter_mut().zip(pass.iter()) {
                *sample = T::from_sample(*value);
            }
        }
    };
    let error = move |error: cpal::Error| {
        let lost = !matches!(
            error.kind(),
            ErrorKind::DeviceChanged | ErrorKind::Xrun | ErrorKind::RealtimeDenied
        );
        on_fault(DeviceFault {
            lost,
            message: error.to_string(),
        });
    };
    device
        .build_output_stream(config, data, error, None)
        .map_err(|error| Error::Output(error.to_string()))
}
