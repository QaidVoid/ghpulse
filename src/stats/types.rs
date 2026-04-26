use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// A programming language with size info from GitHub.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Language {
    pub name: String,
    pub size: u64,
    pub color: Option<String>,
}

/// A repository with relevant stats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repo {
    pub name: String,
    pub owner: String,
    pub stars: u32,
    pub forks: u32,
    pub is_private: bool,
    pub is_archived: bool,
    #[serde(default)]
    pub is_owner: bool,
    pub languages: Vec<Language>,
    pub commits: u32,
    pub views: Option<u64>,
}

impl Repo {
    /// The primary language by size.
    pub fn primary_language(&self) -> Option<&Language> {
        self.languages.iter().max_by_key(|l| l.size)
    }
}

/// Per-year contribution breakdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionYear {
    pub year: i32,
    pub total_count: u32,
    pub repos: u32,
    pub issues: u32,
    pub commits: u32,
    pub pull_requests: u32,
    pub reviews: u32,
}

/// Aggregated stats for a GitHub user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    pub user: User,
    pub contribution_years: Vec<ContributionYear>,
    pub repos: Vec<Repo>,
    pub language_totals: HashMap<String, u64>,
}

/// Basic GitHub user info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub login: String,
    pub name: Option<String>,
    pub avatar_url: String,
}
