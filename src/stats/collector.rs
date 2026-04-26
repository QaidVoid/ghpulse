use anyhow::Result;
use tracing;

use crate::github::client::Client;
use crate::github::graphql;
use crate::stats::types::{ContributionYear, Repo, Stats};

/// Collect all GitHub stats using the API client.
pub fn collect(client: &Client, max_retries: u32) -> Result<Stats> {
    tracing::info!("fetching user info");
    let user = graphql::fetch_user(client)?;
    let login = user.login.clone();
    tracing::info!("authenticated as {login}");

    tracing::info!("fetching contribution years");
    let years = graphql::fetch_contribution_years(client)?;
    tracing::info!("found {} contribution years", years.len());

    let mut all_repos: Vec<Repo> = Vec::new();
    let mut contribution_years: Vec<ContributionYear> = Vec::new();

    // Collect data for the most recent 3 years to stay within rate limits.
    for &year in years.iter().take(3) {
        tracing::info!("fetching contributions for {year}");
        match graphql::fetch_year_contributions(client, &login, year) {
            Ok((year_stats, repo_nodes)) => {
                tracing::info!(
                    "year {year}: {} commits across {} repos",
                    year_stats.commits,
                    repo_nodes.len()
                );
                contribution_years.push(year_stats);

                for node in repo_nodes {
                    let mut repo = node.into_repo();

                    // Try to fetch traffic data (requires push access).
                    if !repo.is_private {
                        match crate::github::rest::fetch_traffic(
                            client,
                            &repo.owner,
                            &repo.name,
                            max_retries,
                        ) {
                            Ok(views) => {
                                tracing::debug!("{}: {} views", repo.name, views);
                                repo.views = Some(views);
                            }
                            Err(e) => {
                                tracing::debug!("{}: traffic unavailable ({e})", repo.name);
                            }
                        }
                    }

                    all_repos.push(repo);
                }
            }
            Err(e) => {
                tracing::warn!("failed to fetch contributions for {year}: {e}");
            }
        }
    }

    contribution_years.sort_by_key(|y| std::cmp::Reverse(y.year));

    Ok(Stats {
        user,
        contribution_years,
        repos: all_repos,
        language_totals: std::collections::HashMap::new(),
    })
}

/// Load stats from a cached JSON file.
pub fn from_json(path: &str) -> Result<Stats> {
    let data = std::fs::read_to_string(path)?;
    let stats: Stats = serde_json::from_str(&data)?;
    Ok(stats)
}

/// Dump stats to a JSON file.
pub fn dump_json(stats: &Stats, path: &str) -> Result<()> {
    let data = serde_json::to_string_pretty(stats)?;
    std::fs::write(path, data)?;
    Ok(())
}
