//! Audio encoder using FFmpeg

use crate::error::EncodeError;
use std::path::{Path, PathBuf};
use tokio::process::Command;
use tracing::{debug, info};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Flac,
    Wav,
    Mp3,
    Aac,
    Opus,
}

impl OutputFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            OutputFormat::Flac => "flac",
            OutputFormat::Wav => "wav",
            OutputFormat::Mp3 => "mp3",
            OutputFormat::Aac => "m4a",
            OutputFormat::Opus => "opus",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "flac" => Some(OutputFormat::Flac),
            "wav" => Some(OutputFormat::Wav),
            "mp3" => Some(OutputFormat::Mp3),
            "aac" | "m4a" => Some(OutputFormat::Aac),
            "opus" => Some(OutputFormat::Opus),
            _ => None,
        }
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Flac => write!(f, "FLAC"),
            OutputFormat::Wav => write!(f, "WAV"),
            OutputFormat::Mp3 => write!(f, "MP3"),
            OutputFormat::Aac => write!(f, "AAC"),
            OutputFormat::Opus => write!(f, "Opus"),
        }
    }
}

#[derive(Debug)]
pub struct Encoder {
    ffmpeg_path: PathBuf,
}

impl Encoder {
    pub fn new(ffmpeg_path: PathBuf) -> Self {
        Self { ffmpeg_path }
    }

    /// Encode audio to target format
    pub async fn encode(
        &self,
        input: &Path,
        output: &Path,
        format: OutputFormat,
    ) -> Result<(), EncodeError> {
        info!("Encoding to {} format", format);

        let codec_args = Self::get_codec_args(format);

        let mut cmd = Command::new(&self.ffmpeg_path);
        cmd.args(["-hide_banner", "-loglevel", "error"]);
        cmd.arg("-i").arg(input);

        for arg in codec_args {
            cmd.arg(arg);
        }

        cmd.arg("-y").arg(output);

        let status = cmd.status().await?;

        if !status.success() {
            return Err(EncodeError::FfmpegFailed(status.code()));
        }

        debug!("Encoded to: {}", output.display());
        Ok(())
    }

    fn get_codec_args(format: OutputFormat) -> Vec<&'static str> {
        match format {
            OutputFormat::Flac => vec![
                "-c:a", "flac",
                "-compression_level", "12",
            ],
            OutputFormat::Wav => vec![
                "-c:a", "pcm_s24le",
            ],
            OutputFormat::Mp3 => vec![
                "-c:a", "libmp3lame",
                "-q:a", "0",  // VBR highest quality (~245 kbps)
            ],
            OutputFormat::Aac => vec![
                "-c:a", "aac",
                "-b:a", "256k",
            ],
            OutputFormat::Opus => vec![
                "-c:a", "libopus",
                "-b:a", "192k",
            ],
        }
    }
}
