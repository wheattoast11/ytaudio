# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

```bash
# Build
cargo build                    # Debug build
cargo build --release          # Release build

# Run (from project root)
cargo run -- [args]            # Run debug binary
./target/release/ytaudio       # Run release binary

# Test
cargo test                     # All tests
cargo test -p ytaudio-core     # Test specific crate

# Lint
cargo clippy                   # Linting
cargo fmt                      # Format code

# Dependency setup (first time)
./scripts/install-deps.sh      # Install yt-dlp, ffmpeg, Python venv, models
cargo run -- doctor            # Verify all dependencies are working
```

## Architecture

### Workspace Structure

This is a Rust workspace with three crates:

- **ytaudio-cli** (`crates/ytaudio-cli/`) - CLI interface using clap, handles arg parsing and commands
- **ytaudio-core** (`crates/ytaudio-core/`) - Core processing pipeline, orchestrates all audio operations
- **ytaudio-upscale** (`crates/ytaudio-upscale/`) - Neural upscaling bridge to Python (FlashSR/AudioSR)

### Processing Pipeline

The core pipeline (`ytaudio-core/src/pipeline.rs`) orchestrates:

1. **Download** (yt-dlp) → 2. **Decode** (ffmpeg→WAV) → 3. **Upscale** (optional, Python) → 4. **Normalize** (optional, ffmpeg) → 5. **Encode** (ffmpeg) → 6. **Embed metadata** (ffmpeg)

Each stage communicates progress via `tokio::sync::mpsc` channel using `PipelineStage` enum.

### External Dependencies

The tool shells out to:
- `yt-dlp` - YouTube downloading
- `ffmpeg` - Decode/encode/normalize audio
- Python venv at `~/.local/share/ytaudio/venv/` - Neural upscaling (audiosr, onnxruntime)

Path resolution in `config.rs`: auto-detects from PATH or uses config overrides.

### Configuration

Loads via figment with precedence: defaults → `~/.config/ytaudio/config.toml` → CLI config → env vars (`YTAUDIO_*`).

### Neural Upscaling

Two methods in `ytaudio-upscale`:
- **FlashSR** (`flashsr.rs`) - Fast ONNX inference, ~0.4s per 5s audio
- **AudioSR** (`audiosr.rs`) - Diffusion model, higher quality, slower

Both invoke Python scripts via subprocess.

## Key Types

- `PipelineConfig` - All settings for a processing job
- `PipelineStage` - Progress events (Downloading, Decoding, Upscaling, etc.)
- `OutputFormat` / `UpscaleQuality` - Enums defined in both CLI args and core (mapped at boundaries)
- `Config` - App configuration with nested structs for paths, output, upscale, normalize, batch, temp
