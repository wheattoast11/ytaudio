//! Error types for ytaudio-core

use thiserror::Error;

pub type Result<T> = std::result::Result<T, YtAudioError>;

#[derive(Error, Debug)]
pub enum YtAudioError {
    #[error("Download failed: {0}")]
    Download(#[from] DownloadError),

    #[error("Decode failed: {0}")]
    Decode(#[from] DecodeError),

    #[error("Upscale failed: {0}")]
    Upscale(#[from] ytaudio_upscale::UpscaleError),

    #[error("Normalization failed: {0}")]
    Normalize(#[from] NormalizeError),

    #[error("Encode failed: {0}")]
    Encode(#[from] EncodeError),

    #[error("Metadata embedding failed: {0}")]
    Metadata(#[from] MetadataError),

    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Pipeline error: {0}")]
    Pipeline(String),
}

#[derive(Error, Debug)]
pub enum DownloadError {
    #[error("yt-dlp not found. Install with: brew install yt-dlp")]
    YtDlpNotFound,

    #[error("yt-dlp failed with exit code: {0:?}")]
    YtDlpFailed(Option<i32>),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Video unavailable or private: {0}")]
    VideoUnavailable(String),

    #[error("No audio stream available")]
    NoAudioStream,

    #[error("Failed to parse metadata: {0}")]
    MetadataParse(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("FFmpeg not found. Install with: brew install ffmpeg")]
    FfmpegNotFound,

    #[error("FFmpeg failed with exit code: {0:?}")]
    FfmpegFailed(Option<i32>),

    #[error("Unsupported audio format: {0}")]
    UnsupportedFormat(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum NormalizeError {
    #[error("FFmpeg not found")]
    FfmpegNotFound,

    #[error("FFmpeg normalization failed with exit code: {0:?}")]
    FfmpegFailed(Option<i32>),

    #[error("Failed to parse loudness stats")]
    LoudnessParseError,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum EncodeError {
    #[error("FFmpeg not found")]
    FfmpegNotFound,

    #[error("FFmpeg encoding failed with exit code: {0:?}")]
    FfmpegFailed(Option<i32>),

    #[error("Unsupported output format: {0}")]
    UnsupportedFormat(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum MetadataError {
    #[error("FFmpeg not found")]
    FfmpegNotFound,

    #[error("FFmpeg metadata embedding failed with exit code: {0:?}")]
    FfmpegFailed(Option<i32>),

    #[error("Missing metadata: {0}")]
    MissingMetadata(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to load config: {0}")]
    LoadError(String),

    #[error("Invalid config value: {0}")]
    InvalidValue(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
