//! YouTube audio downloader using yt-dlp

use crate::error::DownloadError;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tokio::process::Command;
use tracing::{debug, info};

#[derive(Debug)]
pub struct Downloader {
    yt_dlp_path: PathBuf,
    temp_dir: PathBuf,
}

#[derive(Debug)]
pub struct DownloadResult {
    pub audio_path: PathBuf,
    pub metadata: VideoMetadata,
    pub thumbnail_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VideoMetadata {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub artist: Option<String>,
    #[serde(default)]
    pub album: Option<String>,
    #[serde(default)]
    pub uploader: Option<String>,
    #[serde(default)]
    pub upload_date: Option<String>,
    #[serde(default)]
    pub duration: Option<f64>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub ext: String,
}

impl Downloader {
    pub fn new(yt_dlp_path: PathBuf, temp_dir: PathBuf) -> Self {
        Self { yt_dlp_path, temp_dir }
    }

    /// Download audio from YouTube URL
    pub async fn download(&self, url: &str) -> Result<DownloadResult, DownloadError> {
        info!("Downloading audio from: {}", url);

        // Create output template
        let output_template = self.temp_dir.join("%(id)s.%(ext)s");

        // Run yt-dlp with JSON output
        let output = Command::new(&self.yt_dlp_path)
            .args([
                // Format selection: best audio, prefer Opus
                "-f", "bestaudio[acodec=opus]/bestaudio[acodec=aac]/bestaudio",
                // Extract audio without re-encoding (keep original codec)
                "--extract-audio",
                "--audio-format", "best",
                // Keep original codec to avoid quality loss
                "--postprocessor-args", "ExtractAudio:-acodec copy",
                // Get metadata
                "--write-info-json",
                // Get thumbnail
                "--write-thumbnail",
                "--convert-thumbnails", "jpg",
                // Output template
                "-o", output_template.to_str().unwrap(),
                // Print JSON to stdout for metadata parsing
                "--print-json",
                // Don't download if already exists
                "--no-overwrites",
                // URL
                url,
            ])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            debug!("yt-dlp stderr: {}", stderr);

            if stderr.contains("Video unavailable") || stderr.contains("Private video") {
                return Err(DownloadError::VideoUnavailable(url.to_string()));
            }
            if stderr.contains("is not a valid URL") {
                return Err(DownloadError::InvalidUrl(url.to_string()));
            }

            return Err(DownloadError::YtDlpFailed(output.status.code()));
        }

        // Parse JSON output
        let stdout = String::from_utf8_lossy(&output.stdout);
        let metadata: VideoMetadata = serde_json::from_str(&stdout)
            .map_err(|e| DownloadError::MetadataParse(e.to_string()))?;

        debug!("Downloaded: {} ({})", metadata.title, metadata.id);

        // Find the audio file
        let audio_path = self.find_audio_file(&metadata.id)?;

        // Find thumbnail if it exists
        let thumbnail_path = self.find_thumbnail(&metadata.id);

        Ok(DownloadResult {
            audio_path,
            metadata,
            thumbnail_path,
        })
    }

    fn find_audio_file(&self, video_id: &str) -> Result<PathBuf, DownloadError> {
        // Look for common audio extensions
        let extensions = ["opus", "m4a", "webm", "mp3", "ogg", "aac"];

        for ext in extensions {
            let path = self.temp_dir.join(format!("{}.{}", video_id, ext));
            if path.exists() {
                debug!("Found audio file: {}", path.display());
                return Ok(path);
            }
        }

        Err(DownloadError::NoAudioStream)
    }

    fn find_thumbnail(&self, video_id: &str) -> Option<PathBuf> {
        let path = self.temp_dir.join(format!("{}.jpg", video_id));
        if path.exists() {
            Some(path)
        } else {
            // Try other extensions
            for ext in ["png", "webp"] {
                let path = self.temp_dir.join(format!("{}.{}", video_id, ext));
                if path.exists() {
                    return Some(path);
                }
            }
            None
        }
    }
}

/// Validate that a string looks like a YouTube URL
pub fn validate_youtube_url(url: &str) -> bool {
    url.contains("youtube.com/watch")
        || url.contains("youtu.be/")
        || url.contains("youtube.com/playlist")
        || url.contains("youtube.com/shorts")
        || url.contains("music.youtube.com")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_youtube_url() {
        assert!(validate_youtube_url("https://www.youtube.com/watch?v=dQw4w9WgXcQ"));
        assert!(validate_youtube_url("https://youtu.be/dQw4w9WgXcQ"));
        assert!(validate_youtube_url("https://youtube.com/playlist?list=PLrAXtmErZgOeiKm4sgNOknGvNjby9efdf"));
        assert!(validate_youtube_url("https://music.youtube.com/watch?v=dQw4w9WgXcQ"));
        assert!(!validate_youtube_url("https://example.com/video"));
    }
}
