use anyhow::Result;

use crate::github::client::Client;
use crate::stats::types::User;

/// Fetch the authenticated user's info.
pub fn fetch_user(client: &Client) -> Result<User> {
    let query = r#"{
        viewer {
            login
            name
            avatarUrl
        }
    }"#;

    let body = format!("{{\"query\": {}}}", serde_json::to_string(query)?);
    let data = client.graphql_query(&body)?;

    let viewer = &data["data"]["viewer"];

    Ok(User {
        login: viewer["login"].as_str().unwrap_or_default().to_string(),
        name: viewer["name"].as_str().map(String::from),
        avatar_url: viewer["avatarUrl"].as_str().unwrap_or_default().to_string(),
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
    let data = client.graphql_query(&body)?;

    let years = data["data"]["viewer"]["contributionsCollection"]["contributionYears"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_i64().map(|y| y as i32))
                .collect()
        })
        .unwrap_or_default();

    Ok(years)
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PeriodTotals {
    pub repos: u32,
    pub issues: u32,
    pub commits: u32,
    pub pull_requests: u32,
    pub reviews: u32,
}

pub struct PeriodContributions {
    pub totals: PeriodTotals,
    pub repos: Vec<RepoNode>,
}

/// Fetch contributions and contributed-to repos for a half-open date range
/// `[from, to)`. Walks `commitContributionsByRepository` so org repos and
/// forks the user contributed to surface even when `viewer.repositories`
/// can't see them (fine-grained PAT scoping, etc.).
pub fn fetch_period_contributions(
    client: &Client,
    from: &str,
    to: &str,
) -> Result<PeriodContributions> {
    let query = format!(
        r#"{{
        viewer {{
            contributionsCollection(from: "{from}", to: "{to}") {{
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
                        viewerPermission
                        languages(first: 50, orderBy: {{field: SIZE, direction: DESC}}) {{
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
    let data = client.graphql_query(&body)?;

    if let Some(errs) = data.get("errors")
        && !errs.is_null()
    {
        anyhow::bail!("GraphQL error for period {from}..{to}: {errs}");
    }

    let collection = &data["data"]["viewer"]["contributionsCollection"];

    let totals = PeriodTotals {
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

    let repos: Vec<RepoNode> = collection["commitContributionsByRepository"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|entry| serde_json::from_value(entry.clone()).ok())
                .collect()
        })
        .unwrap_or_default();

    Ok(PeriodContributions { totals, repos })
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoNode {
    pub repository: RepoInfo,
    pub contributions: ContributionCount,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoInfo {
    pub name: String,
    pub owner: OwnerInfo,
    pub stargazer_count: u32,
    pub fork_count: u32,
    pub is_private: bool,
    pub is_archived: bool,
    pub viewer_permission: Option<String>,
    pub languages: LanguagesConnection,
}

impl RepoInfo {
    pub fn is_owner_like(&self) -> bool {
        matches!(self.viewer_permission.as_deref(), Some("ADMIN"))
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct OwnerInfo {
    pub login: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct LanguagesConnection {
    pub edges: Vec<LanguageEdge>,
}

#[derive(Debug, serde::Deserialize)]
pub struct LanguageEdge {
    pub size: u64,
    pub node: LanguageNode,
}

#[derive(Debug, serde::Deserialize)]
pub struct LanguageNode {
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContributionCount {
    pub total_count: u32,
}
