# ytaudio

A powerful CLI for extracting high-quality audio from YouTube with neural upscaling.

## Features

- **High-quality extraction**: Extracts best available audio (Opus 160-256kbps)
- **Neural upscaling**: AI-powered bandwidth extension using FlashSR (fast) or AudioSR (best quality)
- **Multi-format output**: FLAC, WAV, MP3, AAC, Opus
- **LUFS normalization**: EBU R128 loudness normalization
- **Batch processing**: Process playlists or URL lists in parallel
- **Metadata embedding**: Title, artist, artwork from YouTube

## Installation

### Prerequisites

```bash
# Install system dependencies
brew install yt-dlp ffmpeg

# Build ytaudio
cargo build --release

# Set up Python environment and neural models
./scripts/install-deps.sh
# or
./target/release/ytaudio update-models
```

### Verify installation

```bash
./target/release/ytaudio doctor
```

## Usage

### Basic extraction

```bash
# Extract to FLAC (default)
ytaudio "https://youtube.com/watch?v=..."

# Extract to MP3
ytaudio --format mp3 "https://youtube.com/watch?v=..."
```

### With neural upscaling

```bash
# Fast upscaling (FlashSR, ~0.4s per 5s audio)
ytaudio --enhance "https://youtube.com/watch?v=..."

# Best quality upscaling (AudioSR, ~2-5 min per track)
ytaudio --enhance --quality best "https://youtube.com/watch?v=..."
```

### With normalization

```bash
# Normalize to -14 LUFS (Spotify/YouTube standard)
ytaudio --normalize "https://youtube.com/watch?v=..."

# Custom LUFS target
ytaudio --normalize --lufs -16 "https://youtube.com/watch?v=..."
```

### Batch processing

```bash
# Create a file with URLs (one per line)
echo "https://youtube.com/watch?v=..." > urls.txt
echo "https://youtube.com/watch?v=..." >> urls.txt

# Process in parallel
ytaudio batch --input urls.txt --parallel 4 --enhance
```

### Full example

```bash
ytaudio --enhance --quality best --normalize --format flac -o ~/Music "https://youtube.com/watch?v=..."
```

## Output Formats

| Format | Codec | Quality |
|--------|-------|---------|
| FLAC | flac | Lossless, compression level 12 |
| WAV | pcm_s24le | Uncompressed 24-bit |
| MP3 | libmp3lame | VBR quality 0 (~245kbps) |
| AAC | aac | 256kbps |
| Opus | libopus | 192kbps |

## Neural Upscaling

ytaudio uses state-of-the-art neural models for audio super-resolution:

### FlashSR (default, `--quality fast`)
- 22x faster than traditional diffusion models
- ONNX-based inference
- ~0.4s to process 5s of audio
- Bandwidth extension from 16kHz to 48kHz

### AudioSR (`--quality best`)
- Diffusion-based model (ICASSP 2024)
- Highest quality reconstruction
- ~2-5 minutes per track
- Reconstructs frequencies up to 24kHz

## Configuration

Configuration can be set via:
1. Config file: `~/.config/ytaudio/config.toml`
2. Environment variables: `YTAUDIO_*`

Example config:

```toml
[output]
default_format = "flac"
default_directory = "~/Music"

[upscale]
default_quality = "fast"

[upscale.audiosr]
ddim_steps = 50
guidance_scale = 3.5

[normalize]
enabled = false
target_lufs = -14.0

[batch]
max_parallel = 4
```

## Commands

```
ytaudio                    # Extract audio (shorthand)
ytaudio extract <URL>      # Extract audio from URL
ytaudio batch              # Batch process URLs
ytaudio doctor             # Check dependencies
ytaudio update-models      # Download/update neural models
ytaudio config             # Show current configuration
```

## License

MIT
