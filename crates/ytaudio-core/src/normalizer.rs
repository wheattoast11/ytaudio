//! LUFS loudness normalization using FFmpeg

use crate::error::NormalizeError;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tokio::process::Command;
use tracing::{debug, info};

#[derive(Debug)]
pub struct Normalizer {
    ffmpeg_path: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
struct LoudnormStats {
    input_i: String,
    input_tp: String,
    input_lra: String,
    input_thresh: String,
    target_offset: String,
}

impl Normalizer {
    pub fn new(ffmpeg_path: PathBuf) -> Self {
        Self { ffmpeg_path }
    }

    /// Apply EBU R128 loudness normalization (two-pass for accuracy)
    pub async fn normalize(
        &self,
        input: &Path,
        output: &Path,
        target_lufs: f32,
        true_peak: f32,
        lra: f32,
    ) -> Result<(), NormalizeError> {
        info!("Normalizing to {:.1} LUFS", target_lufs);

        // First pass: measure loudness
        let stats = self.measure_loudness(input, target_lufs, true_peak, lra).await?;

        // Second pass: apply normalization with measured values
        self.apply_normalization(input, output, target_lufs, true_peak, lra, &stats).await?;

        debug!("Normalized to: {}", output.display());
        Ok(())
    }

    async fn measure_loudness(
        &self,
        input: &Path,
        target_lufs: f32,
        true_peak: f32,
        lra: f32,
    ) -> Result<LoudnormStats, NormalizeError> {
        let filter = format!(
            "loudnorm=I={}:TP={}:LRA={}:print_format=json",
            target_lufs, true_peak, lra
        );

        let output = Command::new(&self.ffmpeg_path)
            .args([
                "-hide_banner",
                "-i", input.to_str().unwrap(),
                "-af", &filter,
                "-f", "null",
                "-"
            ])
            .output()
            .await?;

        // Parse JSON from stderr
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stats = parse_loudnorm_output(&stderr)?;

        debug!(
            "Measured loudness: I={}, TP={}, LRA={}",
            stats.input_i, stats.input_tp, stats.input_lra
        );

        Ok(stats)
    }

    async fn apply_normalization(
        &self,
        input: &Path,
        output: &Path,
        target_lufs: f32,
        true_peak: f32,
        lra: f32,
        stats: &LoudnormStats,
    ) -> Result<(), NormalizeError> {
        let filter = format!(
            "loudnorm=I={}:TP={}:LRA={}:\
             measured_I={}:measured_TP={}:measured_LRA={}:measured_thresh={}:\
             offset={}:linear=true",
            target_lufs, true_peak, lra,
            stats.input_i, stats.input_tp, stats.input_lra,
            stats.input_thresh, stats.target_offset
        );

        let status = Command::new(&self.ffmpeg_path)
            .args([
                "-hide_banner",
                "-loglevel", "error",
                "-i", input.to_str().unwrap(),
                "-af", &filter,
                // Keep as 48kHz 24-bit WAV
                "-c:a", "pcm_s24le",
                "-ar", "48000",
                "-y",
                output.to_str().unwrap(),
            ])
            .status()
            .await?;

        if !status.success() {
            return Err(NormalizeError::FfmpegFailed(status.code()));
        }

        Ok(())
    }
}

fn parse_loudnorm_output(stderr: &str) -> Result<LoudnormStats, NormalizeError> {
    // Find the JSON block in FFmpeg output
    // It looks like:
    // {
    //     "input_i" : "-14.52",
    //     "input_tp" : "-0.95",
    //     ...
    // }

    let json_start = stderr.find('{').ok_or(NormalizeError::LoudnessParseError)?;
    let json_end = stderr.rfind('}').ok_or(NormalizeError::LoudnessParseError)?;

    if json_end <= json_start {
        return Err(NormalizeError::LoudnessParseError);
    }

    let json_str = &stderr[json_start..=json_end];

    serde_json::from_str(json_str).map_err(|_| NormalizeError::LoudnessParseError)
}
