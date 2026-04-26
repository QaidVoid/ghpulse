use anyhow::Result;
use serde::Deserialize;

use crate::github::client::Client;
use crate::stats::types::{ContributionYear, Language, Repo, User};

/// GraphQL response wrapper for user info query.
#[derive(Deserialize)]
struct UserResponse {
    data: UserData,
}

#[derive(Deserialize)]
struct UserData {
    viewer: UserNode,
}

#[derive(Deserialize)]
struct UserNode {
    login: String,
    name: Option<String>,
    avatar_url: String,
    contributions_collection: ContributionsCollection,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct ContributionsCollection {
    contribution_years: Vec<i32>,
    contribution_calendar: ContributionCalendar,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct ContributionCalendar {
    total_contributions: u32,
}

/// Fetch the authenticated user's info and contribution years.
pub fn fetch_user(client: &Client) -> Result<User> {
    let query = r#"{
        viewer {
            login
            name
            avatarUrl
            contributionsCollection {
                contributionYears
                contributionCalendar {
                    totalContributions
                }
            }
        }
    }"#;

    let body = format!("{{\"query\": {}}}", serde_json::to_string(query)?);
    let resp = client.graphql_post(&body)?;
    let data: UserResponse = serde_json::from_reader(resp.into_body().as_reader())?;

    Ok(User {
        login: data.data.viewer.login,
        name: data.data.viewer.name,
        avatar_url: data.data.viewer.avatar_url,
    })
}

/// Fetch contribution years for the authenticated user.
pub fn fetch_contribution_years(client: &Client) -> Result<Vec<i32>> {
    let query = r#"{
        viewer {
            contributionsCollection {
                contributionYears
            }
        }
    }"#;

    let body = format!("{{\"query\": {}}}", serde_json::to_string(query)?);
    let resp = client.graphql_post(&body)?;
    let data: UserResponse = serde_json::from_reader(resp.into_body().as_reader())?;

    Ok(data.data.viewer.contributions_collection.contribution_years)
}

/// Fetch per-year contribution stats and commit-contributed repos.
pub fn fetch_year_contributions(
    client: &Client,
    _login: &str,
    year: i32,
) -> Result<(ContributionYear, Vec<RepoNode>)> {
    let from = format!("{year}-01-01T00:00:00Z");
    let to = format!("{}-01-01T00:00:00Z", year + 1);

    let query = format!(
        r#"{{
        viewer {{
            contributionsCollection(from: "{from}" to: "{to}") {{
                totalCommitContributions
                totalIssueContributions
                totalPullRequestContributions
                totalPullRequestReviewContributions
                totalRepositoryContributions
                commitContributionsByRepository(maxRepositories: 100) {{
                    repository {{
                        name
                        owner {{ login }}
                        stargazerCount
                        forkCount
                        isPrivate
                        isArchived
                        languages(first: 10, orderBy: {{field: SIZE, direction: DESC}}) {{
                            edges {{
                                size
                                node {{
                                    name
                                    color
                                }}
                            }}
                        }}
                    }}
                    contributions(first: 1) {{
                        totalCount
                    }}
                }}
            }}
        }}
    }}"#
    );

    let body = format!("{{\"query\": {}}}", serde_json::to_string(&query)?);
    let resp = client.graphql_post(&body)?;
    let data: serde_json::Value = serde_json::from_reader(resp.into_body().as_reader())?;

    let collection = &data["data"]["viewer"]["contributionsCollection"];

    let year_stats = ContributionYear {
        year,
        total_count: collection["totalCommitContributions"].as_u64().unwrap_or(0) as u32,
        repos: collection["totalRepositoryContributions"]
            .as_u64()
            .unwrap_or(0) as u32,
        issues: collection["totalIssueContributions"].as_u64().unwrap_or(0) as u32,
        commits: collection["totalCommitContributions"].as_u64().unwrap_or(0) as u32,
        pull_requests: collection["totalPullRequestContributions"]
            .as_u64()
            .unwrap_or(0) as u32,
        reviews: collection["totalPullRequestReviewContributions"]
            .as_u64()
            .unwrap_or(0) as u32,
    };

    let repos_raw = collection["commitContributionsByRepository"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    let repos: Vec<RepoNode> = repos_raw
        .iter()
        .filter_map(|entry| serde_json::from_value(entry.clone()).ok())
        .collect();

    Ok((year_stats, repos))
}

/// Intermediate repo node from GraphQL response.
#[derive(Debug, Deserialize)]
pub struct RepoNode {
    pub repository: RepoInfo,
    pub contributions: ContributionCount,
}

#[derive(Debug, Deserialize)]
pub struct RepoInfo {
    pub name: String,
    pub owner: OwnerInfo,
    pub stargazer_count: u32,
    pub fork_count: u32,
    pub is_private: bool,
    pub is_archived: bool,
    pub languages: LanguagesConnection,
}

#[derive(Debug, Deserialize)]
pub struct OwnerInfo {
    pub login: String,
}

#[derive(Debug, Deserialize)]
pub struct LanguagesConnection {
    pub edges: Vec<LanguageEdge>,
}

#[derive(Debug, Deserialize)]
pub struct LanguageEdge {
    pub size: u64,
    pub node: LanguageNode,
}

#[derive(Debug, Deserialize)]
pub struct LanguageNode {
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ContributionCount {
    pub total_count: u32,
}

/// Convert a GraphQL RepoNode into a stats Repo.
impl RepoNode {
    pub fn into_repo(self) -> Repo {
        Repo {
            name: self.repository.name,
            owner: self.repository.owner.login,
            stars: self.repository.stargazer_count,
            forks: self.repository.fork_count,
            is_private: self.repository.is_private,
            is_archived: self.repository.is_archived,
            languages: self
                .repository
                .languages
                .edges
                .into_iter()
                .map(|e| Language {
                    name: e.node.name,
                    size: e.size,
                    color: e.node.color,
                })
                .collect(),
            commits: self.contributions.total_count,
            views: None,
        }
    }
}
