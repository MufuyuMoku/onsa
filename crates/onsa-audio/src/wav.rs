//! Writing WAV files: the offline render target and the test fixtures.

use std::fs::File;
use std::io::{BufWriter, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

/// Sample format of a written WAV file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WavFormat {
    /// 16-bit integer PCM.
    Int16,
    /// 32-bit IEEE float.
    Float32,
}

/// A WAV file being written. The header is completed by [`WavWriter::finish`].
pub struct WavWriter {
    path: PathBuf,
    out: BufWriter<File>,
    format: WavFormat,
    channels: u16,
    data_bytes: u64,
}

impl WavWriter {
    /// Creates the file and writes a provisional header.
    pub fn create(path: &Path, sample_rate: u32, channels: u16, format: WavFormat) -> Result<Self> {
        let io = |source| Error::Io {
            path: path.to_path_buf(),
            source,
        };
        let file = File::create(path).map_err(io)?;
        let mut writer = Self {
            path: path.to_path_buf(),
            out: BufWriter::new(file),
            format,
            channels,
            data_bytes: 0,
        };
        writer.write_header(sample_rate).map_err(io)?;
        Ok(writer)
    }

    fn bytes_per_sample(&self) -> u16 {
        match self.format {
            WavFormat::Int16 => 2,
            WavFormat::Float32 => 4,
        }
    }

    fn write_header(&mut self, sample_rate: u32) -> std::io::Result<()> {
        let bytes = self.bytes_per_sample();
        let (tag, bits) = match self.format {
            WavFormat::Int16 => (1u16, 16u16),
            WavFormat::Float32 => (3u16, 32u16),
        };
        let block_align = self.channels * bytes;
        let out = &mut self.out;
        out.write_all(b"RIFF")?;
        out.write_all(&0u32.to_le_bytes())?;
        out.write_all(b"WAVEfmt ")?;
        out.write_all(&16u32.to_le_bytes())?;
        out.write_all(&tag.to_le_bytes())?;
        out.write_all(&self.channels.to_le_bytes())?;
        out.write_all(&sample_rate.to_le_bytes())?;
        out.write_all(&(sample_rate * u32::from(block_align)).to_le_bytes())?;
        out.write_all(&block_align.to_le_bytes())?;
        out.write_all(&bits.to_le_bytes())?;
        out.write_all(b"data")?;
        out.write_all(&0u32.to_le_bytes())
    }

    /// Appends interleaved samples. Integer output is clipped and rounded.
    pub fn write(&mut self, samples: &[f32]) -> Result<()> {
        let result = (|| -> std::io::Result<()> {
            for &sample in samples {
                match self.format {
                    WavFormat::Int16 => {
                        let value = (sample.clamp(-1.0, 1.0) * 32767.0).round() as i16;
                        self.out.write_all(&value.to_le_bytes())?;
                    }
                    WavFormat::Float32 => self.out.write_all(&sample.to_le_bytes())?,
                }
            }
            Ok(())
        })();
        result.map_err(|source| Error::Io {
            path: self.path.clone(),
            source,
        })?;
        self.data_bytes += samples.len() as u64 * u64::from(self.bytes_per_sample());
        Ok(())
    }

    /// Completes the header and flushes the file.
    pub fn finish(mut self) -> Result<()> {
        let data = u32::try_from(self.data_bytes).unwrap_or(u32::MAX);
        let result = (|| -> std::io::Result<()> {
            self.out.seek(SeekFrom::Start(4))?;
            self.out.write_all(&data.saturating_add(36).to_le_bytes())?;
            self.out.seek(SeekFrom::Start(40))?;
            self.out.write_all(&data.to_le_bytes())?;
            self.out.flush()
        })();
        result.map_err(|source| Error::Io {
            path: self.path.clone(),
            source,
        })
    }
}

/// Writes a whole buffer to a WAV file in one go.
pub fn write_wav(
    path: &Path,
    sample_rate: u32,
    channels: u16,
    format: WavFormat,
    samples: &[f32],
) -> Result<()> {
    let mut writer = WavWriter::create(path, sample_rate, channels, format)?;
    writer.write(samples)?;
    writer.finish()
}
