//! Neural audio upscaling bridge for ytaudio
//!
//! This crate provides a bridge to Python-based neural audio upscaling models:
//! - FlashSR: Fast ONNX-based upscaling (22x faster)
//! - AudioSR: High-quality diffusion-based upscaling

mod error;
mod flashsr;
mod audiosr;

pub use error::UpscaleError;
pub use flashsr::FlashSR;
pub use audiosr::AudioSR;

use std::path::{Path, PathBuf};
use tracing::info;

/// Upscaling method selection
#[derive(Debug, Clone)]
pub enum UpscaleMethod {
    /// FlashSR - Fast ONNX-based upscaling
    FlashSR,
    /// AudioSR - High-quality diffusion-based upscaling
    AudioSR {
        ddim_steps: u32,
        guidance_scale: f32,
    },
}

impl std::fmt::Display for UpscaleMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpscaleMethod::FlashSR => write!(f, "FlashSR"),
            UpscaleMethod::AudioSR { .. } => write!(f, "AudioSR"),
        }
    }
}

/// Neural audio upscaler
#[derive(Debug)]
pub struct Upscaler {
    python_path: PathBuf,
}

impl Upscaler {
    pub fn new(python_path: PathBuf) -> Self {
        Self { python_path }
    }

    /// Upscale audio using the specified method
    pub async fn upscale(
        &self,
        input: &Path,
        output: &Path,
        method: UpscaleMethod,
    ) -> Result<(), UpscaleError> {
        info!("Upscaling with {}", method);

        match method {
            UpscaleMethod::FlashSR => {
                FlashSR::new(self.python_path.clone())
                    .upscale(input, output)
                    .await
            }
            UpscaleMethod::AudioSR { ddim_steps, guidance_scale } => {
                AudioSR::new(self.python_path.clone())
                    .upscale(input, output, ddim_steps, guidance_scale)
                    .await
            }
        }
    }
}
