//! Error types for neural upscaling

use thiserror::Error;

#[derive(Error, Debug)]
pub enum UpscaleError {
    #[error("Python not found. Install Python 3.9+")]
    PythonNotFound,

    #[error("AudioSR not installed. Run: ytaudio update-models")]
    AudioSRNotInstalled,

    #[error("FlashSR/ONNX Runtime not installed. Run: ytaudio update-models")]
    FlashSRNotInstalled,

    #[error("Model not found. Run: ytaudio update-models")]
    ModelNotFound,

    #[error("AudioSR inference failed: {0}")]
    AudioSRFailed(String),

    #[error("FlashSR inference failed: {0}")]
    FlashSRFailed(String),

    #[error("Upscaling timeout after {0} seconds")]
    Timeout(u64),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
