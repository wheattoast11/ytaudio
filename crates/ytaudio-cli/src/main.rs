mod args;
mod commands;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use args::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging based on verbosity
    let filter = match cli.verbose {
        0 => "ytaudio=info",
        1 => "ytaudio=debug",
        2 => "ytaudio=trace",
        _ => "trace",
    };

    tracing_subscriber::registry()
        .with(fmt::layer().with_target(false).without_time())
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(filter)))
        .init();

    // Handle commands
    match cli.command {
        Some(Commands::Extract { url, options }) => {
            commands::extract::run(&url, &options, cli.config.as_deref()).await
        }
        Some(Commands::Batch {
            input,
            parallel,
            options,
        }) => commands::batch::run(&input, parallel, &options, cli.config.as_deref()).await,
        Some(Commands::Doctor) => commands::doctor::run().await,
        Some(Commands::UpdateModels) => commands::update_models::run().await,
        Some(Commands::Config) => commands::config::run(cli.config.as_deref()).await,
        None => {
            // If URL provided directly, treat as extract command
            if let Some(url) = cli.url {
                let options = args::ExtractOptions {
                    enhance: cli.enhance,
                    format: cli.format,
                    normalize: cli.normalize,
                    lufs: cli.lufs,
                    quality: cli.quality,
                    output: Some(cli.output),
                    keep_temp: false,
                };
                commands::extract::run(&url, &options, cli.config.as_deref()).await
            } else {
                // No URL, print help
                use clap::CommandFactory;
                Cli::command().print_help()?;
                println!();
                Ok(())
            }
        }
    }
}
