mod config;

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

fn run(args: Args) -> Result<()> {
    init_tracing(&args);

    tracing::info!("ghpulse starting");

    // TODO: Phase 1 — data collection
    // TODO: Phase 2 — rendering
    // TODO: Phase 3 — output

    Ok(())
}

fn main() {
    let args = Args::parse();

    if let Err(e) = run(args) {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}
