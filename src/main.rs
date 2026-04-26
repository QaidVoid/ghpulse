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
        if let Some(dir) = &args.theme_dir
            && let Ok(entries) = std::fs::read_dir(dir)
        {
            for entry in entries.flatten() {
                if let Some(name) = entry.path().file_stem() {
                    println!("{}", name.to_string_lossy());
                }
            }
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
    let theme_names = if args.theme == "all" {
        vec![
            "nebula".to_string(),
            "nebula-light".to_string(),
            "terminal".to_string(),
            "radar".to_string(),
            "heatmap".to_string(),
            "fingerprint".to_string(),
        ]
    } else {
        vec![args.theme.clone()]
    };

    for name in &theme_names {
        let theme = svg::theme::load(name, args.theme_dir.as_deref())?;
        let ctx = render::context::RenderContext::new(&stats, &theme);
        let svg_str = render::render(&ctx, &theme)?;

        match args.format.as_str() {
            "png" => {
                #[cfg(feature = "png")]
                {
                    output::write_png(&args.output, name, &svg_str)?;
                }
                #[cfg(not(feature = "png"))]
                {
                    anyhow::bail!(
                        "PNG export requires the 'png' feature (build with --features png)"
                    );
                }
            }
            "both" => {
                output::write_svg(&args.output, name, &svg_str)?;
                #[cfg(feature = "png")]
                {
                    output::write_png(&args.output, name, &svg_str)?;
                }
            }
            _ => {
                output::write_svg(&args.output, name, &svg_str)?;
            }
        }
    }

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
