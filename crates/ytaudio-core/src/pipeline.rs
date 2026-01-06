//! Pipeline orchestration for audio extraction and processing

use crate::config::PathsConfig;
use crate::decoder::Decoder;
use crate::downloader::Downloader;
use crate::encoder::{self, Encoder};
use crate::error::YtAudioError;
use crate::metadata::{sanitize_filename, MetadataEmbedder};
use crate::normalizer::Normalizer;
use crate::Config;

use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tracing::{debug, info};
use ytaudio_upscale::{UpscaleMethod, Upscaler};

// Re-export args types for convenience
pub mod args {
    pub use crate::config::PathsConfig;

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

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum UpscaleQuality {
        Best,
        Fast,
    }
}

pub use args::{OutputFormat, UpscaleQuality};

/// Pipeline configuration
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub url: String,
    pub output_dir: PathBuf,
    pub format: OutputFormat,
    pub enhance: bool,
    pub upscale_quality: UpscaleQuality,
    pub normalize: bool,
    pub target_lufs: f32,
    pub keep_temp: bool,
    pub paths: PathsConfig,
}

/// Pipeline progress stages
#[derive(Debug, Clone)]
pub enum PipelineStage {
    Downloading { progress: f32, title: String },
    Decoding,
    Upscaling { method: String, progress: f32 },
    Normalizing { target_lufs: f32 },
    Encoding { format: String },
    EmbeddingMetadata,
    Complete { output: PathBuf, duration: Duration },
    Failed { stage: String, error: String },
}

/// Main processing pipeline
pub struct Pipeline {
    config: PipelineConfig,
    progress_tx: mpsc::Sender<PipelineStage>,
}

impl Pipeline {
    pub fn new(config: PipelineConfig, progress_tx: mpsc::Sender<PipelineStage>) -> Self {
        Self { config, progress_tx }
    }

    pub async fn run(&self) -> Result<PathBuf, YtAudioError> {
        let start_time = Instant::now();

        // Create temp directory
        let temp_dir = tempfile::tempdir()?;
        let temp_path = temp_dir.path().to_path_buf();

        info!("Starting pipeline for: {}", self.config.url);
        debug!("Temp directory: {}", temp_path.display());

        // Get tool paths
        let app_config = Config::load(None)?;
        let yt_dlp_path = app_config.yt_dlp_path()?;
        let ffmpeg_path = app_config.ffmpeg_path()?;
        let python_path = app_config.python_path()?;

        // 1. Download
        let _ = self.progress_tx.send(PipelineStage::Downloading {
            progress: 0.0,
            title: "Starting download...".to_string(),
        }).await;

        let downloader = Downloader::new(yt_dlp_path, temp_path.clone());
        let download_result = downloader.download(&self.config.url).await.map_err(|e| {
            let _ = self.progress_tx.try_send(PipelineStage::Failed {
                stage: "download".to_string(),
                error: e.to_string(),
            });
            e
        })?;

        let _ = self.progress_tx.send(PipelineStage::Downloading {
            progress: 1.0,
            title: download_result.metadata.title.clone(),
        }).await;

        // 2. Decode to WAV
        let _ = self.progress_tx.send(PipelineStage::Decoding).await;

        let decoder = Decoder::new(ffmpeg_path.clone());
        let decoded_wav = temp_path.join("decoded.wav");
        decoder.decode_to_wav(&download_result.audio_path, &decoded_wav).await.map_err(|e| {
            let _ = self.progress_tx.try_send(PipelineStage::Failed {
                stage: "decode".to_string(),
                error: e.to_string(),
            });
            e
        })?;

        // 3. Upscale (if enabled)
        let processed_audio = if self.config.enhance {
            let method_name = match self.config.upscale_quality {
                UpscaleQuality::Best => "AudioSR",
                UpscaleQuality::Fast => "FlashSR",
            };

            let _ = self.progress_tx.send(PipelineStage::Upscaling {
                method: method_name.to_string(),
                progress: 0.0,
            }).await;

            let upscaler = Upscaler::new(python_path);
            let upscaled_wav = temp_path.join("upscaled.wav");

            let method = match self.config.upscale_quality {
                UpscaleQuality::Best => UpscaleMethod::AudioSR {
                    ddim_steps: app_config.upscale.audiosr.ddim_steps,
                    guidance_scale: app_config.upscale.audiosr.guidance_scale,
                },
                UpscaleQuality::Fast => UpscaleMethod::FlashSR,
            };

            upscaler.upscale(&decoded_wav, &upscaled_wav, method).await.map_err(|e| {
                let _ = self.progress_tx.try_send(PipelineStage::Failed {
                    stage: "upscale".to_string(),
                    error: e.to_string(),
                });
                e
            })?;

            let _ = self.progress_tx.send(PipelineStage::Upscaling {
                method: method_name.to_string(),
                progress: 1.0,
            }).await;

            upscaled_wav
        } else {
            decoded_wav
        };

        // 4. Normalize (if enabled)
        let normalized_audio = if self.config.normalize {
            let _ = self.progress_tx.send(PipelineStage::Normalizing {
                target_lufs: self.config.target_lufs,
            }).await;

            let normalizer = Normalizer::new(ffmpeg_path.clone());
            let normalized_wav = temp_path.join("normalized.wav");

            normalizer.normalize(
                &processed_audio,
                &normalized_wav,
                self.config.target_lufs,
                app_config.normalize.true_peak,
                app_config.normalize.lra,
            ).await.map_err(|e| {
                let _ = self.progress_tx.try_send(PipelineStage::Failed {
                    stage: "normalize".to_string(),
                    error: e.to_string(),
                });
                e
            })?;

            normalized_wav
        } else {
            processed_audio
        };

        // 5. Encode to target format
        let format_str = self.config.format.to_string();
        let _ = self.progress_tx.send(PipelineStage::Encoding {
            format: format_str.clone(),
        }).await;

        let encoder = Encoder::new(ffmpeg_path.clone());
        let encoded_file = temp_path.join(format!(
            "encoded.{}",
            self.config.format.extension()
        ));

        let encoder_format = match self.config.format {
            args::OutputFormat::Flac => encoder::OutputFormat::Flac,
            args::OutputFormat::Wav => encoder::OutputFormat::Wav,
            args::OutputFormat::Mp3 => encoder::OutputFormat::Mp3,
            args::OutputFormat::Aac => encoder::OutputFormat::Aac,
            args::OutputFormat::Opus => encoder::OutputFormat::Opus,
        };

        encoder.encode(&normalized_audio, &encoded_file, encoder_format).await.map_err(|e| {
            let _ = self.progress_tx.try_send(PipelineStage::Failed {
                stage: "encode".to_string(),
                error: e.to_string(),
            });
            e
        })?;

        // 6. Embed metadata
        let _ = self.progress_tx.send(PipelineStage::EmbeddingMetadata).await;

        let safe_title = sanitize_filename(&download_result.metadata.title);
        let final_filename = format!("{}.{}", safe_title, self.config.format.extension());
        let final_path = self.config.output_dir.join(&final_filename);

        // Ensure output directory exists
        tokio::fs::create_dir_all(&self.config.output_dir).await?;

        let embedder = MetadataEmbedder::new(ffmpeg_path);
        embedder.embed(
            &encoded_file,
            &final_path,
            &download_result.metadata,
            download_result.thumbnail_path.as_deref(),
        ).await.map_err(|e| {
            let _ = self.progress_tx.try_send(PipelineStage::Failed {
                stage: "metadata".to_string(),
                error: e.to_string(),
            });
            e
        })?;

        let duration = start_time.elapsed();
        info!("Pipeline complete: {} ({:.1}s)", final_path.display(), duration.as_secs_f32());

        let _ = self.progress_tx.send(PipelineStage::Complete {
            output: final_path.clone(),
            duration,
        }).await;

        // Cleanup temp directory (unless keep_temp is set)
        if !self.config.keep_temp {
            drop(temp_dir);
        } else {
            // Prevent cleanup by forgetting the temp dir
            std::mem::forget(temp_dir);
            debug!("Temp files kept at: {}", temp_path.display());
        }

        Ok(final_path)
    }
}
