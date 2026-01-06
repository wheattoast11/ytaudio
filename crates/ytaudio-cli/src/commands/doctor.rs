use anyhow::Result;
use std::process::Command;
use which::which;

pub async fn run() -> Result<()> {
    println!("ytaudio dependency check\n");

    let mut all_ok = true;

    // Check yt-dlp
    print!("yt-dlp:        ");
    match which("yt-dlp") {
        Ok(path) => {
            let version = Command::new(&path).arg("--version").output();
            match version {
                Ok(out) => {
                    let v = String::from_utf8_lossy(&out.stdout);
                    println!("OK ({})", v.trim());
                }
                Err(_) => {
                    println!("FOUND but failed to get version");
                    all_ok = false;
                }
            }
        }
        Err(_) => {
            println!("NOT FOUND");
            println!("           Install with: brew install yt-dlp");
            all_ok = false;
        }
    }

    // Check FFmpeg
    print!("ffmpeg:        ");
    match which("ffmpeg") {
        Ok(path) => {
            let version = Command::new(&path).args(["-version"]).output();
            match version {
                Ok(out) => {
                    let first_line = String::from_utf8_lossy(&out.stdout)
                        .lines()
                        .next()
                        .unwrap_or("")
                        .to_string();
                    // Extract just version number
                    let version_part = first_line
                        .split_whitespace()
                        .nth(2)
                        .unwrap_or("unknown");
                    println!("OK ({})", version_part);
                }
                Err(_) => {
                    println!("FOUND but failed to get version");
                    all_ok = false;
                }
            }
        }
        Err(_) => {
            println!("NOT FOUND");
            println!("           Install with: brew install ffmpeg");
            all_ok = false;
        }
    }

    // Check Python
    print!("python3:       ");

    // Check for venv in multiple locations (macOS uses Application Support, Linux uses .local/share)
    let venv_paths = [
        dirs::data_dir().map(|d| d.join("ytaudio/venv/bin/python")),
        dirs::home_dir().map(|d| d.join(".local/share/ytaudio/venv/bin/python")),
    ];

    let venv_python = venv_paths
        .into_iter()
        .flatten()
        .find(|p| p.exists());

    if let Some(venv_python) = venv_python {
        let version = Command::new(&venv_python).arg("--version").output();
        match version {
            Ok(out) => {
                let v = String::from_utf8_lossy(&out.stdout);
                println!("OK ({}, venv)", v.trim().replace("Python ", ""));
            }
            Err(_) => {
                println!("VENV FOUND but failed to get version");
                all_ok = false;
            }
        }

        // Check AudioSR
        print!("  audiosr:     ");
        let check = Command::new(&venv_python)
            .args(["-c", "import audiosr; print('installed')"])
            .output();
        match check {
            Ok(out) if out.status.success() && String::from_utf8_lossy(&out.stdout).contains("installed") => {
                println!("OK");
            }
            _ => {
                println!("NOT INSTALLED");
                println!("               Run: ytaudio update-models");
                all_ok = false;
            }
        }

        // Check ONNX Runtime
        print!("  onnxruntime: ");
        let check = Command::new(&venv_python)
            .args(["-c", "import onnxruntime; print(onnxruntime.__version__)"])
            .output();
        match check {
            Ok(out) if out.status.success() => {
                println!("OK ({})", String::from_utf8_lossy(&out.stdout).trim());
            }
            _ => {
                println!("NOT INSTALLED");
                println!("               Run: ytaudio update-models");
                all_ok = false;
            }
        }

        // Check librosa
        print!("  librosa:     ");
        let check = Command::new(&venv_python)
            .args(["-c", "import librosa; print(librosa.__version__)"])
            .output();
        match check {
            Ok(out) if out.status.success() => {
                println!("OK ({})", String::from_utf8_lossy(&out.stdout).trim());
            }
            _ => {
                println!("NOT INSTALLED");
                println!("               Run: ytaudio update-models");
                all_ok = false;
            }
        }
    } else {
        // Check system Python
        match which("python3") {
            Ok(path) => {
                let version = Command::new(&path).arg("--version").output();
                match version {
                    Ok(out) => {
                        let v = String::from_utf8_lossy(&out.stdout);
                        println!("OK ({}, system)", v.trim().replace("Python ", ""));
                        println!("           Note: Virtual environment not set up");
                        println!("           Run: ytaudio update-models");
                    }
                    Err(_) => {
                        println!("FOUND but failed to get version");
                    }
                }
            }
            Err(_) => {
                println!("NOT FOUND");
                println!("           Install with: brew install python@3.11");
            }
        }
        all_ok = false;
    }

    // Check FlashSR model
    print!("FlashSR model: ");
    // HuggingFace Hub uses ~/.cache on macOS, not ~/Library/Caches
    let model_paths = [
        dirs::home_dir().map(|d| d.join(".cache/huggingface/hub/models--YatharthS--FlashSR")),
        dirs::cache_dir().map(|d| d.join("huggingface/hub/models--YatharthS--FlashSR")),
    ];
    let model_found = model_paths.into_iter().flatten().any(|p| p.exists());
    if model_found {
        println!("OK (cached)");
    } else {
        println!("NOT DOWNLOADED");
        println!("               Run: ytaudio update-models");
        all_ok = false;
    }

    println!();
    if all_ok {
        println!("All dependencies OK!");
    } else {
        println!("Some dependencies are missing. See above for installation instructions.");
    }

    Ok(())
}
