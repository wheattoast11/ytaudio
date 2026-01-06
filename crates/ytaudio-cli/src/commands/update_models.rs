use anyhow::{Context, Result};
use std::process::Command;
use which::which;

pub async fn run() -> Result<()> {
    println!("Setting up Python environment and neural models...\n");

    // Get data directory for venv
    let data_dir = dirs::data_dir()
        .context("Could not determine data directory")?
        .join("ytaudio");

    let venv_dir = data_dir.join("venv");
    let venv_python = venv_dir.join("bin/python");
    let venv_pip = venv_dir.join("bin/pip");

    // Find system Python
    let python = which("python3").context(
        "Python 3 not found. Install with: brew install python@3.11",
    )?;

    // Create venv if it doesn't exist
    if !venv_dir.exists() {
        println!("Creating virtual environment at {}...", venv_dir.display());
        std::fs::create_dir_all(&data_dir)?;

        let status = Command::new(&python)
            .args(["-m", "venv", venv_dir.to_str().unwrap()])
            .status()
            .context("Failed to create virtual environment")?;

        if !status.success() {
            anyhow::bail!("Failed to create virtual environment");
        }
        println!("Virtual environment created.\n");
    } else {
        println!("Virtual environment exists at {}\n", venv_dir.display());
    }

    // Upgrade pip
    println!("Upgrading pip...");
    let status = Command::new(&venv_pip)
        .args(["install", "--upgrade", "pip"])
        .status()
        .context("Failed to upgrade pip")?;

    if !status.success() {
        anyhow::bail!("Failed to upgrade pip");
    }

    // Install packages
    let packages = [
        "torch>=2.0.0",
        "audiosr==0.0.7",
        "onnxruntime>=1.16.0",
        "librosa>=0.10.0",
        "soundfile>=0.12.0",
        "huggingface-hub>=0.20.0",
        "numpy>=1.24.0",
    ];

    println!("\nInstalling Python packages...");
    for package in &packages {
        print!("  Installing {}... ", package.split(">=").next().unwrap_or(package));
        let status = Command::new(&venv_pip)
            .args(["install", package])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();

        match status {
            Ok(s) if s.success() => println!("OK"),
            _ => println!("FAILED"),
        }
    }

    // Download FlashSR model
    println!("\nDownloading FlashSR ONNX model...");
    let download_script = r#"
from huggingface_hub import hf_hub_download
path = hf_hub_download(repo_id='YatharthS/FlashSR', filename='model.onnx', subfolder='onnx')
print(f'Downloaded to: {path}')
"#;

    let status = Command::new(&venv_python)
        .args(["-c", download_script])
        .status()
        .context("Failed to download FlashSR model")?;

    if !status.success() {
        println!("Warning: Failed to download FlashSR model. It will be downloaded on first use.");
    }

    println!("\n=== Setup Complete ===");
    println!("Run 'ytaudio doctor' to verify installation.");

    Ok(())
}
