use anyhow::Result;
use std::path::Path;
use ytaudio_core::config::Config;

pub async fn run(config_path: Option<&Path>) -> Result<()> {
    let config = Config::load(config_path)?;

    println!("ytaudio configuration\n");

    println!("[paths]");
    if let Some(ref p) = config.paths.yt_dlp {
        println!("  yt_dlp = {:?}", p);
    } else {
        println!("  yt_dlp = (auto-detect)");
    }
    if let Some(ref p) = config.paths.ffmpeg {
        println!("  ffmpeg = {:?}", p);
    } else {
        println!("  ffmpeg = (auto-detect)");
    }
    if let Some(ref p) = config.paths.python {
        println!("  python = {:?}", p);
    } else {
        println!("  python = (auto-detect)");
    }

    println!("\n[output]");
    println!("  default_format = {:?}", config.output.default_format);
    println!("  default_directory = {:?}", config.output.default_directory);

    println!("\n[upscale]");
    println!("  default_quality = {:?}", config.upscale.default_quality);

    println!("\n[upscale.audiosr]");
    println!("  ddim_steps = {}", config.upscale.audiosr.ddim_steps);
    println!("  guidance_scale = {}", config.upscale.audiosr.guidance_scale);
    println!("  model = {:?}", config.upscale.audiosr.model);

    println!("\n[normalize]");
    println!("  enabled = {}", config.normalize.enabled);
    println!("  target_lufs = {}", config.normalize.target_lufs);
    println!("  true_peak = {}", config.normalize.true_peak);
    println!("  lra = {}", config.normalize.lra);

    println!("\n[batch]");
    println!("  max_parallel = {}", config.batch.max_parallel);
    println!("  continue_on_error = {}", config.batch.continue_on_error);

    println!("\n[temp]");
    println!("  cleanup = {}", config.temp.cleanup);
    if let Some(ref d) = config.temp.directory {
        println!("  directory = {:?}", d);
    } else {
        println!("  directory = (system temp)");
    }

    // Show config file locations
    println!("\nConfig file locations (in priority order):");
    if let Some(p) = config_path {
        println!("  1. {} (specified)", p.display());
    }
    if let Some(config_dir) = dirs::config_dir() {
        println!("  2. {}/ytaudio/config.toml", config_dir.display());
    }
    println!("  3. Environment variables (YTAUDIO_*)");

    Ok(())
}
