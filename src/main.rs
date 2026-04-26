mod config;
mod github;
mod stats;

use anyhow::Result;
use clap::Parser;
use config::Args;

fn init_tracing(args: &Args) {
    use tracing_subscriber::EnvFilter;

    let level = if args.debug {
        "ghpulse=debug"
    } else if args.verbose {
        "ghpulse=info"
    } else {
        "ghpulse=warn"
    };

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_new(level).unwrap_or_default())
        .init();
}

fn run(_args: Args) -> Result<()> {
    // TODO: data collection, rendering, output
    Ok(())
}

fn main() {
    let args = Args::parse();

    init_tracing(&args);

    if let Err(e) = run(args) {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}
