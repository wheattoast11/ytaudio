//! ytaudio-core: Core pipeline for YouTube audio extraction with neural upscaling

pub mod config;
pub mod decoder;
pub mod downloader;
pub mod encoder;
pub mod error;
pub mod metadata;
pub mod normalizer;
pub mod pipeline;

pub use config::Config;
pub use error::{YtAudioError, Result};
