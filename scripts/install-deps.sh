#!/bin/bash
set -euo pipefail

echo "=== ytaudio dependency installer ==="
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

ok() { echo -e "${GREEN}OK${NC}"; }
fail() { echo -e "${RED}FAILED${NC}"; }
warn() { echo -e "${YELLOW}WARNING${NC}"; }

# Check for Homebrew
echo -n "Checking Homebrew... "
if ! command -v brew &> /dev/null; then
    fail
    echo "Homebrew not found. Install from https://brew.sh"
    exit 1
fi
ok

# Install system dependencies
echo ""
echo "Installing system dependencies..."

echo -n "  yt-dlp... "
if ! command -v yt-dlp &> /dev/null; then
    brew install yt-dlp > /dev/null 2>&1 && ok || fail
else
    ok
fi

echo -n "  ffmpeg... "
if ! command -v ffmpeg &> /dev/null; then
    brew install ffmpeg > /dev/null 2>&1 && ok || fail
else
    ok
fi

# Check for Python
echo ""
echo -n "Checking Python 3... "
if ! command -v python3 &> /dev/null; then
    fail
    echo "Python 3 not found. Installing..."
    brew install python@3.11
fi
PYTHON=$(command -v python3)
ok

# Create virtual environment
echo ""
VENV_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/ytaudio/venv"
echo "Setting up Python virtual environment..."
echo "  Location: $VENV_DIR"

mkdir -p "$(dirname "$VENV_DIR")"

if [ ! -d "$VENV_DIR" ]; then
    echo -n "  Creating venv... "
    python3 -m venv "$VENV_DIR" && ok || { fail; exit 1; }
else
    echo "  Venv already exists"
fi

# Upgrade pip
echo -n "  Upgrading pip... "
"$VENV_DIR/bin/pip" install --upgrade pip > /dev/null 2>&1 && ok || fail

# Install Python packages
echo ""
echo "Installing Python packages..."

packages=(
    "torch>=2.0.0"
    "audiosr==0.0.7"
    "onnxruntime>=1.16.0"
    "librosa>=0.10.0"
    "soundfile>=0.12.0"
    "huggingface-hub>=0.20.0"
    "numpy>=1.24.0"
)

for package in "${packages[@]}"; do
    name=$(echo "$package" | cut -d'>' -f1 | cut -d'=' -f1)
    echo -n "  $name... "
    "$VENV_DIR/bin/pip" install "$package" > /dev/null 2>&1 && ok || warn
done

# Download FlashSR model
echo ""
echo -n "Downloading FlashSR ONNX model... "
"$VENV_DIR/bin/python" -c "
from huggingface_hub import hf_hub_download
hf_hub_download(repo_id='YatharthS/FlashSR', filename='model.onnx', subfolder='onnx')
" > /dev/null 2>&1 && ok || warn

echo ""
echo "=== Installation complete! ==="
echo ""
echo "Run 'ytaudio doctor' to verify everything is working."
echo ""
