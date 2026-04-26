use std::collections::HashMap;

use anyhow::Result;

use crate::github::client::Client;
use crate::github::graphql::{self, PeriodTotals, RepoInfo, RepoNode};
use crate::stats::types::{ContributionYear, Language, Repo, Stats};

const REPO_LIMIT: usize = 100;

/// Collect all GitHub stats using the API client.
pub fn collect(client: &Client, _max_retries: u32) -> Result<Stats> {
    tracing::info!("fetching user info");
    let user = graphql::fetch_user(client)?;
    let login = user.login.clone();
    tracing::info!("authenticated as {login}");

    let mut repo_map: HashMap<String, Repo> = HashMap::new();

    tracing::info!("fetching contribution years");
    let years = graphql::fetch_contribution_years(client)?;
    tracing::info!("found {} contribution years: {years:?}", years.len());

    let mut contribution_years: Vec<ContributionYear> = Vec::with_capacity(years.len());

    for &year in &years {
        let mut totals = PeriodTotals::default();
        walk_period(client, year, 0, 12, &mut totals, &mut repo_map, &login)?;
        tracing::info!(
            "year {year}: {} commits, {} issues, {} PRs, {} reviews, {} repos contributed to",
            totals.commits,
            totals.issues,
            totals.pull_requests,
            totals.reviews,
            totals.repos,
        );
        contribution_years.push(ContributionYear {
            year,
            total_count: totals.commits,
            repos: totals.repos,
            issues: totals.issues,
            commits: totals.commits,
            pull_requests: totals.pull_requests,
            reviews: totals.reviews,
        });
    }

    contribution_years.sort_by_key(|y| std::cmp::Reverse(y.year));

    for repo in repo_map.values_mut() {
        if repo.is_private {
            continue;
        }
        match crate::github::rest::fetch_traffic(client, &repo.owner, &repo.name, 0) {
            Ok(Some(views)) => {
                tracing::debug!("{}/{}: {views} views", repo.owner, repo.name);
                repo.views = Some(views);
            }
            Ok(None) => {}
            Err(e) => tracing::debug!("traffic fetch failed for {}/{}: {e}", repo.owner, repo.name),
        }
    }

    let all_repos: Vec<Repo> = repo_map.into_values().collect();
    tracing::info!("collected {} unique repos", all_repos.len());

    Ok(Stats {
        user,
        contribution_years,
        repos: all_repos,
        language_totals: HashMap::new(),
    })
}

/// Walk a `[start_month, start_month+months)` slice of a year, subdividing
/// the window when the API returns >=100 repos (the per-query cap).
fn walk_period(
    client: &Client,
    year: i32,
    start_month: u32,
    months: u32,
    totals: &mut PeriodTotals,
    repo_map: &mut HashMap<String, Repo>,
    login: &str,
) -> Result<()> {
    let from_month = start_month + 1;
    let end_year = year + ((start_month + months) / 12) as i32;
    let end_month = (start_month + months) % 12 + 1;
    let from = format!("{year}-{from_month:02}-01T00:00:00Z");
    let to = format!("{end_year}-{end_month:02}-01T00:00:00Z");

    tracing::debug!("fetching contributions for {from}..{to}");
    let result = graphql::fetch_period_contributions(client, &from, &to)?;

    if result.repos.len() >= REPO_LIMIT {
        for &factor in &[2u32, 3u32] {
            if months.is_multiple_of(factor) {
                let chunk = months / factor;
                for i in 0..factor {
                    walk_period(
                        client,
                        year,
                        start_month + chunk * i,
                        chunk,
                        totals,
                        repo_map,
                        login,
                    )?;
                }
                return Ok(());
            }
        }
        tracing::warn!(
            "more than {REPO_LIMIT} repos for {year}/{from_month:02} ({months}mo) \
             — data may be truncated"
        );
    }

    totals.repos += result.totals.repos;
    totals.issues += result.totals.issues;
    totals.commits += result.totals.commits;
    totals.pull_requests += result.totals.pull_requests;
    totals.reviews += result.totals.reviews;

    for node in result.repos {
        ingest_repo(node, repo_map, login);
    }

    Ok(())
}

/// Merge a contribution-period repo node into the dedup map.
fn ingest_repo(node: RepoNode, repo_map: &mut HashMap<String, Repo>, login: &str) {
    let key = format!("{}/{}", node.repository.owner.login, node.repository.name);
    let commits = node.contributions.total_count;
    let owner_like = node.repository.is_owner_like() || node.repository.owner.login == login;

    repo_map
        .entry(key)
        .and_modify(|r| {
            r.commits += commits;
            if owner_like {
                r.is_owner = true;
            }
        })
        .or_insert_with(|| {
            let mut repo = info_to_repo(node.repository, owner_like);
            repo.commits = commits;
            repo
        });
}

/// Build a `Repo` from a GraphQL `RepoInfo`, tagging ownership.
fn info_to_repo(info: RepoInfo, is_owner: bool) -> Repo {
    Repo {
        name: info.name,
        owner: info.owner.login,
        stars: info.stargazer_count,
        forks: info.fork_count,
        is_private: info.is_private,
        is_archived: info.is_archived,
        is_owner,
        languages: info
            .languages
            .edges
            .into_iter()
            .map(|e| Language {
                name: e.node.name,
                size: e.size,
                color: e.node.color,
            })
            .collect(),
        commits: 0,
        views: None,
    }
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
