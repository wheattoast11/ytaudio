//! AudioSR diffusion-based audio upscaling

use crate::UpscaleError;
use std::path::{Path, PathBuf};
use tokio::process::Command;
use tracing::{debug, info};

/// AudioSR upscaler (highest quality, slower)
#[derive(Debug)]
pub struct AudioSR {
    python_path: PathBuf,
}

impl AudioSR {
    pub fn new(python_path: PathBuf) -> Self {
        Self { python_path }
    }

    /// Upscale audio using AudioSR diffusion model
    pub async fn upscale(
        &self,
        input: &Path,
        output: &Path,
        ddim_steps: u32,
        guidance_scale: f32,
    ) -> Result<(), UpscaleError> {
        info!("Running AudioSR upscaling (ddim_steps={}, guidance_scale={})", ddim_steps, guidance_scale);

        // Inline Python script for AudioSR
        let script = format!(
            r#"
import sys
import os

# Suppress warnings
os.environ['TF_CPP_MIN_LOG_LEVEL'] = '3'

try:
    import torch
    import soundfile as sf
    from audiosr import build_model, super_resolution
except ImportError as e:
    print(f"Missing dependency: {{e}}", file=sys.stderr)
    sys.exit(1)

# Select device
if torch.backends.mps.is_available():
    device = "mps"
elif torch.cuda.is_available():
    device = "cuda"
else:
    device = "cpu"

print(f"Using device: {{device}}", file=sys.stderr)

# Build model
try:
    audiosr = build_model(model_name="basic", device=device)
except Exception as e:
    print(f"Failed to build model: {{e}}", file=sys.stderr)
    sys.exit(2)

# Run super-resolution
try:
    waveform = super_resolution(
        audiosr,
        "{input}",
        seed=42,
        guidance_scale={guidance_scale},
        ddim_steps={ddim_steps},
        latent_t_per_second=12.8
    )
except Exception as e:
    print(f"Inference failed: {{e}}", file=sys.stderr)
    sys.exit(3)

# Save output
try:
    sf.write("{output}", waveform.squeeze(), samplerate=48000, subtype='PCM_24')
    print("Upscaling complete")
except Exception as e:
    print(f"Failed to save output: {{e}}", file=sys.stderr)
    sys.exit(4)
"#,
            input = input.display(),
            output = output.display(),
            ddim_steps = ddim_steps,
            guidance_scale = guidance_scale,
        );

        let result = Command::new(&self.python_path)
            .args(["-c", &script])
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&result.stdout);
        let stderr = String::from_utf8_lossy(&result.stderr);

        if !stdout.is_empty() {
            debug!("AudioSR stdout: {}", stdout);
        }
        if !stderr.is_empty() {
            debug!("AudioSR stderr: {}", stderr);
        }

        if !result.status.success() {
            let exit_code = result.status.code().unwrap_or(-1);
            let error_msg = match exit_code {
                1 => "Missing Python dependencies. Run: ytaudio update-models".to_string(),
                2 => format!("Failed to build AudioSR model: {}", stderr.trim()),
                3 => format!("AudioSR inference failed: {}", stderr.trim()),
                4 => format!("Failed to save output: {}", stderr.trim()),
                _ => format!("AudioSR failed: {}", stderr.trim()),
            };
            return Err(UpscaleError::AudioSRFailed(error_msg));
        }

        info!("AudioSR upscaling complete");
        Ok(())
    }
}
