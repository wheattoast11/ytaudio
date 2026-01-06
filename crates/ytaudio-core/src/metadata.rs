//! Metadata and artwork embedding using FFmpeg

use crate::downloader::VideoMetadata;
use crate::error::MetadataError;
use std::path::{Path, PathBuf};
use tokio::process::Command;
use tracing::{debug, info};

#[derive(Debug)]
pub struct MetadataEmbedder {
    ffmpeg_path: PathBuf,
}

impl MetadataEmbedder {
    pub fn new(ffmpeg_path: PathBuf) -> Self {
        Self { ffmpeg_path }
    }

    /// Embed metadata and artwork into audio file
    pub async fn embed(
        &self,
        audio: &Path,
        output: &Path,
        metadata: &VideoMetadata,
        artwork: Option<&Path>,
    ) -> Result<(), MetadataError> {
        info!("Embedding metadata: {}", metadata.title);

        let mut cmd = Command::new(&self.ffmpeg_path);
        cmd.args(["-hide_banner", "-loglevel", "error"]);

        // Input audio
        cmd.arg("-i").arg(audio);

        // Input artwork if available
        if let Some(art) = artwork {
            cmd.arg("-i").arg(art);
            cmd.args(["-map", "0:a", "-map", "1:v"]);
            cmd.args(["-c:v", "mjpeg"]);
            cmd.args(["-disposition:v", "attached_pic"]);
        }

        // Add metadata tags
        cmd.args(["-metadata", &format!("title={}", metadata.title)]);

        // Use uploader as artist if artist not available
        if let Some(ref artist) = metadata.artist {
            cmd.args(["-metadata", &format!("artist={}", artist)]);
        } else if let Some(ref uploader) = metadata.uploader {
            cmd.args(["-metadata", &format!("artist={}", uploader)]);
        }

        if let Some(ref album) = metadata.album {
            cmd.args(["-metadata", &format!("album={}", album)]);
        }

        if let Some(ref date) = metadata.upload_date {
            // YouTube date format is YYYYMMDD, convert to YYYY-MM-DD
            let formatted_date = if date.len() == 8 {
                format!("{}-{}-{}", &date[0..4], &date[4..6], &date[6..8])
            } else {
                date.clone()
            };
            cmd.args(["-metadata", &format!("date={}", formatted_date)]);
        }

        // Add comment with video ID for reference
        cmd.args(["-metadata", &format!("comment=YouTube: {}", metadata.id)]);

        // Copy audio codec (no re-encoding)
        cmd.args(["-c:a", "copy"]);

        cmd.arg("-y").arg(output);

        let status = cmd.status().await?;

        if !status.success() {
            return Err(MetadataError::FfmpegFailed(status.code()));
        }

        debug!("Embedded metadata to: {}", output.display());
        Ok(())
    }

    /// Embed metadata only (no artwork) with simple approach
    pub async fn embed_simple(
        &self,
        input: &Path,
        output: &Path,
        metadata: &VideoMetadata,
    ) -> Result<(), MetadataError> {
        let mut cmd = Command::new(&self.ffmpeg_path);
        cmd.args(["-hide_banner", "-loglevel", "error"]);
        cmd.arg("-i").arg(input);

        // Add metadata
        cmd.args(["-metadata", &format!("title={}", metadata.title)]);

        if let Some(ref artist) = metadata.artist.as_ref().or(metadata.uploader.as_ref()) {
            cmd.args(["-metadata", &format!("artist={}", artist)]);
        }

        if let Some(ref date) = metadata.upload_date {
            let formatted_date = if date.len() == 8 {
                format!("{}-{}-{}", &date[0..4], &date[4..6], &date[6..8])
            } else {
                date.clone()
            };
            cmd.args(["-metadata", &format!("date={}", formatted_date)]);
        }

        cmd.args(["-metadata", &format!("comment=YouTube: {}", metadata.id)]);
        cmd.args(["-c:a", "copy"]);
        cmd.arg("-y").arg(output);

        let status = cmd.status().await?;

        if !status.success() {
            return Err(MetadataError::FfmpegFailed(status.code()));
        }

        Ok(())
    }
}

/// Sanitize filename for filesystem
pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("Normal Title"), "Normal Title");
        assert_eq!(sanitize_filename("Title/With:Special*Chars"), "Title_With_Special_Chars");
        assert_eq!(sanitize_filename("  Spaces  "), "Spaces");
    }
}
