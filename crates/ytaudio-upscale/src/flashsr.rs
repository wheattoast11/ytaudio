//! FlashSR ONNX-based audio upscaling

use crate::UpscaleError;
use std::path::{Path, PathBuf};
use tokio::process::Command;
use tracing::{debug, info};

/// FlashSR upscaler (22x faster than AudioSR)
#[derive(Debug)]
pub struct FlashSR {
    python_path: PathBuf,
}

impl FlashSR {
    pub fn new(python_path: PathBuf) -> Self {
        Self { python_path }
    }

    /// Upscale audio using FlashSR ONNX model
    pub async fn upscale(&self, input: &Path, output: &Path) -> Result<(), UpscaleError> {
        info!("Running FlashSR upscaling");

        // Inline Python script for FlashSR
        let script = format!(
            r#"
import sys
import numpy as np

try:
    import librosa
    import soundfile as sf
    import onnxruntime as ort
    from huggingface_hub import hf_hub_download
except ImportError as e:
    print(f"Missing dependency: {{e}}", file=sys.stderr)
    sys.exit(1)

# Download model if not cached
try:
    model_path = hf_hub_download(
        repo_id='YatharthS/FlashSR',
        filename='model.onnx',
        subfolder='onnx'
    )
except Exception as e:
    print(f"Failed to download model: {{e}}", file=sys.stderr)
    sys.exit(2)

# Load audio at 16kHz (FlashSR input requirement)
try:
    y, sr = librosa.load("{input}", sr=16000)
    lowres_wav = y[np.newaxis, :].astype(np.float32)
except Exception as e:
    print(f"Failed to load audio: {{e}}", file=sys.stderr)
    sys.exit(3)

# Run ONNX inference
try:
    session = ort.InferenceSession(model_path)
    output = session.run(
        ["reconstruction"],
        {{"audio_values": lowres_wav}}
    )[0]
except Exception as e:
    print(f"Inference failed: {{e}}", file=sys.stderr)
    sys.exit(4)

# Save at 48kHz
try:
    sf.write("{output}", output.squeeze(0), samplerate=48000, subtype='PCM_24')
    print("Upscaling complete")
except Exception as e:
    print(f"Failed to save output: {{e}}", file=sys.stderr)
    sys.exit(5)
"#,
            input = input.display(),
            output = output.display(),
        );

        let result = Command::new(&self.python_path)
            .args(["-c", &script])
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&result.stdout);
        let stderr = String::from_utf8_lossy(&result.stderr);

        if !stdout.is_empty() {
            debug!("FlashSR stdout: {}", stdout);
        }
        if !stderr.is_empty() {
            debug!("FlashSR stderr: {}", stderr);
        }

        if !result.status.success() {
            let exit_code = result.status.code().unwrap_or(-1);
            let error_msg = match exit_code {
                1 => "Missing Python dependencies. Run: ytaudio update-models".to_string(),
                2 => "Failed to download FlashSR model".to_string(),
                3 => format!("Failed to load audio: {}", stderr.trim()),
                4 => format!("ONNX inference failed: {}", stderr.trim()),
                5 => format!("Failed to save output: {}", stderr.trim()),
                _ => format!("FlashSR failed: {}", stderr.trim()),
            };
            return Err(UpscaleError::FlashSRFailed(error_msg));
        }

        info!("FlashSR upscaling complete");
        Ok(())
    }
}
