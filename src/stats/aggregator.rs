use std::collections::HashMap;

use crate::config::Args;
use crate::stats::types::{Repo, Stats};

/// Apply CLI filters to a Stats struct and compute language totals.
pub fn aggregate(mut stats: Stats, args: &Args) -> Stats {
    stats.repos.retain(|r| passes_filters(r, args));

    // Sort repos by stars descending, then commits descending.
    stats.repos.sort_by(|a, b| {
        b.stars
            .cmp(&a.stars)
            .then_with(|| b.commits.cmp(&a.commits))
    });

    // Deduplicate by owner/name.
    let mut seen = std::collections::HashSet::new();
    stats
        .repos
        .retain(|r| seen.insert(format!("{}/{}", r.owner, r.name)));

    stats.language_totals = compute_language_totals(&stats.repos);
    stats
}

fn passes_filters(repo: &Repo, args: &Args) -> bool {
    if args.exclude_private && repo.is_private {
        return false;
    }
    if args.exclude_archived && repo.is_archived {
        return false;
    }
    if let Some(min) = args.min_stars
        && repo.stars < min
    {
        return false;
    }
    if let Some(min) = args.min_commits
        && repo.commits < min
    {
        return false;
    }
    if let Some(patterns) = &args.exclude_repos {
        for pat in patterns.split(',') {
            let pat = pat.trim();
            if glob_match(pat, &format!("{}/{}", repo.owner, repo.name)) {
                return false;
            }
        }
    }
    if let Some(langs) = &args.exclude_langs {
        let excluded: Vec<&str> = langs.split(',').map(|l| l.trim()).collect();
        if let Some(primary) = repo.primary_language()
            && excluded
                .iter()
                .any(|e| e.eq_ignore_ascii_case(&primary.name))
        {
            return false;
        }
    }
    true
}

/// Simple glob matching: supports `*` wildcard only.
fn glob_match(pattern: &str, text: &str) -> bool {
    if !pattern.contains('*') {
        return pattern == text;
    }
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 2 {
        let (prefix, suffix) = (parts[0], parts[1]);
        text.starts_with(prefix) && text.ends_with(suffix)
    } else {
        pattern == text
    }
}

fn compute_language_totals(repos: &[Repo]) -> HashMap<String, u64> {
    let mut totals = HashMap::new();
    for repo in repos {
        for lang in &repo.languages {
            *totals.entry(lang.name.clone()).or_insert(0) += lang.size;
        }
    }
    totals
}
