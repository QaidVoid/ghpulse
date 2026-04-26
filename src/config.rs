use clap::Parser;

/// Your GitHub activity, visualized as generative SVG art.
#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Args {
    /// GitHub personal access token.
    #[arg(short, long, env = "ACCESS_TOKEN", hide_env_values = true)]
    pub token: Option<String>,

    /// Visualization theme.
    #[arg(long, default_value = "nebula")]
    pub theme: String,

    /// Output format.
    #[arg(long, default_value = "svg")]
    pub format: String,

    /// Output directory.
    #[arg(short, long, default_value = ".")]
    pub output: String,

    /// Comma-separated glob patterns of repos to exclude.
    #[arg(long)]
    pub exclude_repos: Option<String>,

    /// Comma-separated language names to exclude.
    #[arg(long)]
    pub exclude_langs: Option<String>,

    /// Skip private repositories.
    #[arg(long)]
    pub exclude_private: bool,

    /// Skip archived repositories.
    #[arg(long)]
    pub exclude_archived: bool,

    /// Minimum stars to include a repo.
    #[arg(long)]
    pub min_stars: Option<u32>,

    /// Minimum commits to include a repo.
    #[arg(long)]
    pub min_commits: Option<u32>,

    /// Save raw stats to JSON file.
    #[arg(long)]
    pub dump_json: Option<String>,

    /// Render from cached JSON file (no API calls).
    #[arg(long)]
    pub from_json: Option<String>,

    /// List available themes.
    #[arg(long)]
    pub list_themes: bool,

    /// List detected languages (for exclude-langs).
    #[arg(long)]
    pub list_langs: bool,

    /// Load additional themes from directory.
    #[arg(long)]
    pub theme_dir: Option<String>,

    /// Max retries for flaky API endpoints.
    #[arg(long, default_value_t = 10)]
    pub max_retries: u32,

    /// Verbose output.
    #[arg(long)]
    pub verbose: bool,

    /// Debug output.
    #[arg(long)]
    pub debug: bool,
}
