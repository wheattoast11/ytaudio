//! Audio decoder using FFmpeg

use crate::error::DecodeError;
use std::path::{Path, PathBuf};
use tokio::process::Command;
use tracing::{debug, info};

#[derive(Debug)]
pub struct Decoder {
    ffmpeg_path: PathBuf,
}

impl Decoder {
    pub fn new(ffmpeg_path: PathBuf) -> Self {
        Self { ffmpeg_path }
    }

    /// Decode audio to 48kHz 24-bit PCM WAV for processing
    pub async fn decode_to_wav(&self, input: &Path, output: &Path) -> Result<(), DecodeError> {
        info!("Decoding {} to WAV", input.display());

        let status = Command::new(&self.ffmpeg_path)
            .args([
                "-hide_banner",
                "-loglevel", "error",
                "-i", input.to_str().unwrap(),
                // Output format: 48kHz 24-bit PCM
                "-c:a", "pcm_s24le",
                "-ar", "48000",
                // Overwrite output
                "-y",
                output.to_str().unwrap(),
            ])
            .status()
            .await?;

        if !status.success() {
            return Err(DecodeError::FfmpegFailed(status.code()));
        }

        debug!("Decoded to: {}", output.display());
        Ok(())
    }

    /// Get audio file info (sample rate, channels, duration)
    pub async fn get_audio_info(&self, input: &Path) -> Result<AudioInfo, DecodeError> {
        let output = Command::new(&self.ffmpeg_path)
            .args([
                "-hide_banner",
                "-i", input.to_str().unwrap(),
                "-f", "null",
                "-"
            ])
            .output()
            .await?;

        // FFmpeg outputs info to stderr
        let stderr = String::from_utf8_lossy(&output.stderr);

        let sample_rate = parse_sample_rate(&stderr).unwrap_or(48000);
        let channels = parse_channels(&stderr).unwrap_or(2);
        let duration = parse_duration(&stderr).unwrap_or(0.0);

        Ok(AudioInfo {
            sample_rate,
            channels,
            duration,
        })
    }
}

#[derive(Debug, Clone)]
pub struct AudioInfo {
    pub sample_rate: u32,
    pub channels: u8,
    pub duration: f64,
}

fn parse_sample_rate(ffmpeg_output: &str) -> Option<u32> {
    // Look for pattern like "48000 Hz" or "44100 Hz"
    let re = regex::Regex::new(r"(\d+) Hz").ok()?;
    let caps = re.captures(ffmpeg_output)?;
    caps.get(1)?.as_str().parse().ok()
}

fn parse_channels(ffmpeg_output: &str) -> Option<u8> {
    if ffmpeg_output.contains("stereo") {
        Some(2)
    } else if ffmpeg_output.contains("mono") {
        Some(1)
    } else if ffmpeg_output.contains("5.1") {
        Some(6)
    } else {
        Some(2) // Default to stereo
    }
}

fn parse_duration(ffmpeg_output: &str) -> Option<f64> {
    // Look for pattern like "Duration: 00:03:45.12"
    let re = regex::Regex::new(r"Duration: (\d+):(\d+):(\d+)\.(\d+)").ok()?;
    let caps = re.captures(ffmpeg_output)?;

    let hours: f64 = caps.get(1)?.as_str().parse().ok()?;
    let minutes: f64 = caps.get(2)?.as_str().parse().ok()?;
    let seconds: f64 = caps.get(3)?.as_str().parse().ok()?;
    let centiseconds: f64 = caps.get(4)?.as_str().parse().ok()?;

    Some(hours * 3600.0 + minutes * 60.0 + seconds + centiseconds / 100.0)
}
