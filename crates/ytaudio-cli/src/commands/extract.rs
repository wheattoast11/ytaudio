use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use tokio::sync::mpsc;

use crate::args::{ExtractOptions, OutputFormat as CliFormat, UpscaleQuality as CliQuality};
use ytaudio_core::{
    config::Config,
    pipeline::{Pipeline, PipelineConfig, PipelineStage, OutputFormat, UpscaleQuality},
};

pub async fn run(url: &str, options: &ExtractOptions, config_path: Option<&Path>) -> Result<()> {
    let config = Config::load(config_path)?;

    let output_dir = options
        .output
        .clone()
        .unwrap_or_else(|| config.output.default_directory.clone());

    // Convert CLI types to pipeline types
    let format = match options.format {
        CliFormat::Flac => OutputFormat::Flac,
        CliFormat::Wav => OutputFormat::Wav,
        CliFormat::Mp3 => OutputFormat::Mp3,
        CliFormat::Aac => OutputFormat::Aac,
        CliFormat::Opus => OutputFormat::Opus,
    };

    let upscale_quality = match options.quality {
        CliQuality::Best => UpscaleQuality::Best,
        CliQuality::Fast => UpscaleQuality::Fast,
    };

    let pipeline_config = PipelineConfig {
        url: url.to_string(),
        output_dir,
        format,
        enhance: options.enhance,
        upscale_quality,
        normalize: options.normalize,
        target_lufs: options.lufs,
        keep_temp: options.keep_temp,
        paths: config.paths.clone(),
    };

    // Create progress channel
    let (tx, mut rx) = mpsc::channel(32);

    // Create progress bar
    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.cyan} [{elapsed_precise}] {bar:40.cyan/blue} {msg}",
        )?
        .progress_chars("=>-"),
    );

    // Spawn progress handler
    let progress_handle = tokio::spawn(async move {
        while let Some(stage) = rx.recv().await {
            match stage {
                PipelineStage::Downloading { progress, title } => {
                    pb.set_position((progress * 20.0) as u64);
                    pb.set_message(format!("Downloading: {}", truncate(&title, 40)));
                }
                PipelineStage::Decoding => {
                    pb.set_position(20);
                    pb.set_message("Decoding to WAV...");
                }
                PipelineStage::Upscaling { method, progress } => {
                    pb.set_position(20 + (progress * 40.0) as u64);
                    pb.set_message(format!("Upscaling ({})...", method));
                }
                PipelineStage::Normalizing { target_lufs } => {
                    pb.set_position(60);
                    pb.set_message(format!("Normalizing to {:.1} LUFS...", target_lufs));
                }
                PipelineStage::Encoding { format } => {
                    pb.set_position(75);
                    pb.set_message(format!("Encoding to {}...", format));
                }
                PipelineStage::EmbeddingMetadata => {
                    pb.set_position(90);
                    pb.set_message("Embedding metadata...");
                }
                PipelineStage::Complete { output, duration } => {
                    pb.set_position(100);
                    pb.finish_with_message(format!(
                        "Done: {} ({:.1}s)",
                        output.display(),
                        duration.as_secs_f32()
                    ));
                }
                PipelineStage::Failed { stage, error } => {
                    pb.abandon_with_message(format!("Failed at {}: {}", stage, error));
                }
            }
        }
    });

    // Run pipeline
    let pipeline = Pipeline::new(pipeline_config, tx);
    let result = pipeline.run().await;

    // Wait for progress handler
    progress_handle.await?;

    match result {
        Ok(output) => {
            println!("\nOutput: {}", output.display());
            Ok(())
        }
        Err(e) => {
            eprintln!("\nError: {}", e);
            Err(e.into())
        }
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
