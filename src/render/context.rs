use std::collections::HashMap;

use crate::stats::types::{ContributionYear, Stats};
use crate::svg::theme::Theme;

/// Normalized data prepared for rendering.
#[allow(dead_code)]
pub struct RenderContext {
    pub user_login: String,
    pub user_name: String,
    pub total_repos: usize,
    pub total_stars: u32,
    pub total_commits: u32,
    pub total_forks: u32,
    pub repos: Vec<RepoContext>,
    pub language_totals: HashMap<String, u64>,
    pub contribution_years: Vec<ContributionYear>,
    pub top_languages: Vec<LanguageContext>,
}

/// Normalized repo data for rendering.
#[allow(dead_code)]
pub struct RepoContext {
    pub name: String,
    pub full_name: String,
    pub stars: u32,
    pub forks: u32,
    pub commits: u32,
    pub views: u64,
    pub primary_language: Option<String>,
    pub primary_language_color: Option<String>,
    pub languages: Vec<LanguageContext>,
}

/// Normalized language data for rendering.
#[allow(dead_code)]
pub struct LanguageContext {
    pub name: String,
    pub size: u64,
    pub color: String,
    pub percentage: f64,
}

impl RenderContext {
    pub fn new(stats: &Stats, _theme: &Theme) -> Self {
        let total_stars: u32 = stats.repos.iter().map(|r| r.stars).sum();
        let total_commits: u32 = stats.repos.iter().map(|r| r.commits).sum();
        let total_forks: u32 = stats.repos.iter().map(|r| r.forks).sum();

        let total_lang_size: u64 = stats.language_totals.values().sum();

        let top_languages: Vec<LanguageContext> = stats
            .language_totals
            .iter()
            .map(|(name, &size)| {
                let color = stats
                    .repos
                    .iter()
                    .flat_map(|r| &r.languages)
                    .find(|l| l.name == *name)
                    .and_then(|l| l.color.clone())
                    .unwrap_or_else(|| "#8b949e".to_string());

                let percentage = if total_lang_size > 0 {
                    (size as f64 / total_lang_size as f64) * 100.0
                } else {
                    0.0
                };

                LanguageContext {
                    name: name.clone(),
                    size,
                    color,
                    percentage,
                }
            })
            .collect();

        let mut top_languages = top_languages;
        top_languages.sort_by_key(|l| std::cmp::Reverse(l.size));

        let repos: Vec<RepoContext> = stats
            .repos
            .iter()
            .map(|r| RepoContext {
                name: r.name.clone(),
                full_name: format!("{}/{}", r.owner, r.name),
                stars: r.stars,
                forks: r.forks,
                commits: r.commits,
                views: r.views.unwrap_or(0),
                primary_language: r.primary_language().map(|l| l.name.clone()),
                primary_language_color: r.primary_language().and_then(|l| l.color.clone()),
                languages: r
                    .languages
                    .iter()
                    .map(|l| LanguageContext {
                        name: l.name.clone(),
                        size: l.size,
                        color: l.color.clone().unwrap_or_else(|| "#8b949e".to_string()),
                        percentage: 0.0,
                    })
                    .collect(),
            })
            .collect();

        Self {
            user_login: stats.user.login.clone(),
            user_name: stats
                .user
                .name
                .clone()
                .unwrap_or_else(|| stats.user.login.clone()),
            total_repos: stats.repos.len(),
            total_stars,
            total_commits,
            total_forks,
            repos,
            language_totals: stats.language_totals.clone(),
            contribution_years: stats.contribution_years.clone(),
            top_languages,
        }
    }
}
