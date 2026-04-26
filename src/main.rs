mod config;
mod github;
mod output;
mod render;
mod stats;
mod svg;

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
    // Load stats from cache or collect from GitHub API.
    let raw_stats = if let Some(path) = &args.from_json {
        stats::collector::from_json(path)?
    } else {
        let token = args.token.as_deref().ok_or_else(|| {
            anyhow::anyhow!("GitHub token required (use --token or ACCESS_TOKEN env)")
        })?;

        let client = github::client::Client::new(token.to_string());
        let stats = stats::collector::collect(&client, args.max_retries)?;

        if let Some(path) = &args.dump_json {
            stats::collector::dump_json(&stats, path)?;
            tracing::info!("saved stats to {path}");
        }

        stats
    };

    // Always run aggregation (filtering, sorting, language totals).
    let stats = stats::aggregator::aggregate(raw_stats, &args);

    // List modes.
    if args.list_themes {
        let themes = [
            "nebula",
            "nebula-light",
            "terminal",
            "radar",
            "heatmap",
            "fingerprint",
        ];
        for t in &themes {
            println!("{t}");
        }
        return Ok(());
    }

    if args.list_langs {
        let mut langs: Vec<_> = stats.language_totals.keys().collect();
        langs.sort();
        for l in langs {
            println!("{l}");
        }
        return Ok(());
    }

    // Render.
    let theme = svg::theme::builtin(&args.theme)?;
    let ctx = render::context::RenderContext::new(&stats, &theme);
    let svg_str = render::render(&ctx, &theme)?;

    // Output.
    output::write_svg(&args.output, &args.theme, &svg_str)?;

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
