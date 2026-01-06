use anyhow::{Context, Result};
use futures::stream::{self, StreamExt};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::Semaphore;

use crate::args::{ExtractOptions, OutputFormat as CliFormat, UpscaleQuality as CliQuality};
use ytaudio_core::{
    config::Config,
    pipeline::{Pipeline, PipelineConfig, OutputFormat, UpscaleQuality},
};

pub async fn run(
    input: &Path,
    parallel: usize,
    options: &ExtractOptions,
    config_path: Option<&Path>,
) -> Result<()> {
    let config = Config::load(config_path)?;

    // Read URLs from file
    let content = fs::read_to_string(input)
        .await
        .context("Failed to read input file")?;

    let urls: Vec<String> = content
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(String::from)
        .collect();

    if urls.is_empty() {
        println!("No URLs found in input file");
        return Ok(());
    }

    let total_urls = urls.len();
    println!("Processing {} URLs with {} parallel workers\n", total_urls, parallel);

    let semaphore = Arc::new(Semaphore::new(parallel));
    let multi = MultiProgress::new();

    let spinner_style = ProgressStyle::with_template("{spinner:.cyan} {msg}")
        .unwrap()
        .tick_chars("=>-");

    let output_dir = options
        .output
        .clone()
        .unwrap_or_else(|| config.output.default_directory.clone());

    let results: Vec<_> = stream::iter(urls.iter().enumerate())
        .map(|(idx, url)| {
            let sem = semaphore.clone();
            let opts = options.clone();
            let config = config.clone();
            let output_dir = output_dir.clone();
            let pb = multi.add(ProgressBar::new_spinner());
            pb.set_style(spinner_style.clone());
            let url = url.clone();

            async move {
                let _permit = sem.acquire().await.unwrap();
                pb.set_message(format!("[{}/{}] {}", idx + 1, total_urls, truncate(&url, 50)));
                pb.enable_steady_tick(std::time::Duration::from_millis(100));

                // Convert CLI types to pipeline types
                let format = match opts.format {
                    CliFormat::Flac => OutputFormat::Flac,
                    CliFormat::Wav => OutputFormat::Wav,
                    CliFormat::Mp3 => OutputFormat::Mp3,
                    CliFormat::Aac => OutputFormat::Aac,
                    CliFormat::Opus => OutputFormat::Opus,
                };

                let upscale_quality = match opts.quality {
                    CliQuality::Best => UpscaleQuality::Best,
                    CliQuality::Fast => UpscaleQuality::Fast,
                };

                let pipeline_config = PipelineConfig {
                    url: url.clone(),
                    output_dir,
                    format,
                    enhance: opts.enhance,
                    upscale_quality,
                    normalize: opts.normalize,
                    target_lufs: opts.lufs,
                    keep_temp: opts.keep_temp,
                    paths: config.paths.clone(),
                };

                // Create a dummy channel (batch mode doesn't show per-item progress)
                let (tx, mut rx) = tokio::sync::mpsc::channel(1);
                tokio::spawn(async move {
                    while rx.recv().await.is_some() {}
                });

                let pipeline = Pipeline::new(pipeline_config, tx);
                let result = pipeline.run().await;

                match &result {
                    Ok(path) => {
                        pb.finish_with_message(format!(
                            "[{}/{}] Done: {}",
                            idx + 1,
                            total_urls,
                            path.file_name().unwrap_or_default().to_string_lossy()
                        ));
                    }
                    Err(e) => {
                        pb.finish_with_message(format!(
                            "[{}/{}] Failed: {}",
                            idx + 1,
                            total_urls,
                            e
                        ));
                    }
                }

                (url, result)
            }
        })
        .buffer_unordered(parallel)
        .collect()
        .await;

    // Summary
    let succeeded: Vec<_> = results.iter().filter(|(_, r)| r.is_ok()).collect();
    let failed: Vec<_> = results.iter().filter(|(_, r)| r.is_err()).collect();

    println!("\n=== Batch Complete ===");
    println!("Succeeded: {}", succeeded.len());
    println!("Failed: {}", failed.len());

    if !failed.is_empty() {
        println!("\nFailed URLs:");
        for (url, result) in &failed {
            if let Err(e) = result {
                println!("  {} - {}", url, e);
            }
        }
    }

    Ok(())
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
