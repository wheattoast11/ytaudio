use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "ytaudio")]
#[command(author, version, about = "YouTube audio extraction with neural upscaling")]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// YouTube URL to process (shorthand for `extract <URL>`)
    #[arg(value_name = "URL")]
    pub url: Option<String>,

    /// Enable neural upscaling (bandwidth extension)
    #[arg(short, long)]
    pub enhance: bool,

    /// Output format
    #[arg(short, long, value_enum, default_value = "flac")]
    pub format: OutputFormat,

    /// Apply LUFS normalization (-14 LUFS by default)
    #[arg(short, long)]
    pub normalize: bool,

    /// Target LUFS level (requires --normalize)
    #[arg(long, default_value = "-14.0", requires = "normalize")]
    pub lufs: f32,

    /// Upscaling quality: best (AudioSR) or fast (FlashSR)
    #[arg(short, long, value_enum, default_value = "fast")]
    pub quality: UpscaleQuality,

    /// Output directory
    #[arg(short, long, default_value = ".")]
    pub output: PathBuf,

    /// Verbose output (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Config file path
    #[arg(long)]
    pub config: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Extract audio from a single URL
    Extract {
        /// YouTube URL
        url: String,

        #[command(flatten)]
        options: ExtractOptions,
    },

    /// Batch process multiple URLs
    Batch {
        /// File containing URLs (one per line) or playlist URL
        #[arg(short, long)]
        input: PathBuf,

        /// Maximum parallel downloads
        #[arg(short, long, default_value = "4")]
        parallel: usize,

        #[command(flatten)]
        options: ExtractOptions,
    },

    /// Check and install dependencies
    Doctor,

    /// Download/update neural models
    UpdateModels,

    /// Show configuration
    Config,
}

#[derive(clap::Args, Clone)]
pub struct ExtractOptions {
    /// Enable neural upscaling (bandwidth extension)
    #[arg(short, long)]
    pub enhance: bool,

    /// Output format
    #[arg(short, long, value_enum, default_value = "flac")]
    pub format: OutputFormat,

    /// Apply LUFS normalization
    #[arg(short, long)]
    pub normalize: bool,

    /// Target LUFS level
    #[arg(long, default_value = "-14.0")]
    pub lufs: f32,

    /// Upscaling quality
    #[arg(short, long, value_enum, default_value = "fast")]
    pub quality: UpscaleQuality,

    /// Output directory
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Keep intermediate files (for debugging)
    #[arg(long)]
    pub keep_temp: bool,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// FLAC - Lossless compression (recommended)
    Flac,
    /// WAV - Uncompressed PCM
    Wav,
    /// MP3 - Lossy, widely compatible
    Mp3,
    /// AAC - Lossy, good quality/size ratio
    Aac,
    /// Opus - Lossy, best quality/size ratio
    Opus,
}

impl OutputFormat {
    #[allow(dead_code)]
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

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UpscaleQuality {
    /// AudioSR - Best quality, slower (~2-5 min/track)
    Best,
    /// FlashSR - 22x faster, near-equal quality
    Fast,
}

impl std::fmt::Display for UpscaleQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpscaleQuality::Best => write!(f, "best (AudioSR)"),
            UpscaleQuality::Fast => write!(f, "fast (FlashSR)"),
        }
    }
}
