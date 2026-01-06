//! Configuration management for ytaudio

use crate::error::ConfigError;
use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub paths: PathsConfig,
    pub output: OutputConfig,
    pub upscale: UpscaleConfig,
    pub normalize: NormalizeConfig,
    pub batch: BatchConfig,
    pub temp: TempConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathsConfig {
    /// Path to yt-dlp binary (auto-detected if not set)
    pub yt_dlp: Option<PathBuf>,
    /// Path to FFmpeg binary (auto-detected if not set)
    pub ffmpeg: Option<PathBuf>,
    /// Path to Python binary (auto-detected if not set)
    pub python: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Default output format
    pub default_format: String,
    /// Default output directory
    pub default_directory: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpscaleConfig {
    /// Default upscaling quality: "fast" (FlashSR) or "best" (AudioSR)
    pub default_quality: String,
    /// AudioSR-specific settings
    pub audiosr: AudioSRConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSRConfig {
    /// Number of DDIM steps (default: 50)
    pub ddim_steps: u32,
    /// Guidance scale (default: 3.5)
    pub guidance_scale: f32,
    /// Model variant: "basic" or "speech"
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizeConfig {
    /// Enable normalization by default
    pub enabled: bool,
    /// Target LUFS level (default: -14.0)
    pub target_lufs: f32,
    /// True peak limit (default: -1.0)
    pub true_peak: f32,
    /// Loudness range (default: 11.0)
    pub lra: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    /// Maximum parallel downloads
    pub max_parallel: usize,
    /// Continue on error
    pub continue_on_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TempConfig {
    /// Clean up temp files after processing
    pub cleanup: bool,
    /// Custom temp directory (uses system temp if not set)
    pub directory: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            paths: PathsConfig {
                yt_dlp: None,
                ffmpeg: None,
                python: None,
            },
            output: OutputConfig {
                default_format: "flac".to_string(),
                default_directory: PathBuf::from("."),
            },
            upscale: UpscaleConfig {
                default_quality: "fast".to_string(),
                audiosr: AudioSRConfig {
                    ddim_steps: 50,
                    guidance_scale: 3.5,
                    model: "basic".to_string(),
                },
            },
            normalize: NormalizeConfig {
                enabled: false,
                target_lufs: -14.0,
                true_peak: -1.0,
                lra: 11.0,
            },
            batch: BatchConfig {
                max_parallel: 4,
                continue_on_error: true,
            },
            temp: TempConfig {
                cleanup: true,
                directory: None,
            },
        }
    }
}

impl Config {
    /// Load configuration from file and environment
    pub fn load(config_file: Option<&Path>) -> Result<Self, ConfigError> {
        let mut figment = Figment::new()
            .merge(Serialized::defaults(Config::default()));

        // Load from default config directory
        if let Some(config_dir) = dirs::config_dir() {
            let default_config = config_dir.join("ytaudio/config.toml");
            if default_config.exists() {
                figment = figment.merge(Toml::file(&default_config));
            }
        }

        // Load from specified config file
        if let Some(path) = config_file {
            figment = figment.merge(Toml::file(path));
        }

        // Load from environment
        figment = figment.merge(Env::prefixed("YTAUDIO_").split("_"));

        figment.extract().map_err(|e| ConfigError::LoadError(e.to_string()))
    }

    /// Get yt-dlp path, auto-detecting if not configured
    pub fn yt_dlp_path(&self) -> Result<PathBuf, ConfigError> {
        if let Some(ref path) = self.paths.yt_dlp {
            Ok(path.clone())
        } else {
            which::which("yt-dlp")
                .map_err(|_| ConfigError::InvalidValue("yt-dlp not found in PATH".to_string()))
        }
    }

    /// Get FFmpeg path, auto-detecting if not configured
    pub fn ffmpeg_path(&self) -> Result<PathBuf, ConfigError> {
        if let Some(ref path) = self.paths.ffmpeg {
            Ok(path.clone())
        } else {
            which::which("ffmpeg")
                .map_err(|_| ConfigError::InvalidValue("ffmpeg not found in PATH".to_string()))
        }
    }

    /// Get Python path, preferring venv if available
    pub fn python_path(&self) -> Result<PathBuf, ConfigError> {
        if let Some(ref path) = self.paths.python {
            return Ok(path.clone());
        }

        // Check for venv Python in multiple locations
        let venv_paths = [
            // macOS standard (dirs::data_dir())
            dirs::data_dir().map(|d| d.join("ytaudio/venv/bin/python")),
            // XDG standard (~/.local/share)
            dirs::home_dir().map(|d| d.join(".local/share/ytaudio/venv/bin/python")),
        ];

        for path in venv_paths.into_iter().flatten() {
            if path.exists() {
                return Ok(path);
            }
        }

        // Fall back to system Python
        which::which("python3")
            .map_err(|_| ConfigError::InvalidValue("python3 not found in PATH".to_string()))
    }

    /// Get temp directory
    pub fn temp_dir(&self) -> PathBuf {
        self.temp.directory.clone().unwrap_or_else(std::env::temp_dir)
    }
}
