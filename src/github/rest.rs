use anyhow::Result;
use serde::Deserialize;

use crate::github::client::Client;

/// Traffic views for a repo (requires push access).
#[derive(Debug, Deserialize)]
pub struct TrafficViews {
    pub count: u64,
    #[expect(dead_code)]
    pub uniques: u64,
}

/// Fetch traffic/view count for a repo. Requires push access.
pub fn fetch_traffic(client: &Client, owner: &str, repo: &str, max_retries: u32) -> Result<u64> {
    let path = format!("/repos/{owner}/{repo}/traffic/views");
    let resp = client.get_with_retry(&path, max_retries)?;
    let views: TrafficViews = serde_json::from_reader(resp.into_body().as_reader())?;
    Ok(views.count)
}

/// Contributor stats (lines added/removed). Notoriously flaky.
#[derive(Debug, Deserialize)]
pub struct ContributorStats {
    pub author: Option<Author>,
    pub weeks: Vec<WeekStats>,
}

#[derive(Debug, Deserialize)]
pub struct Author {
    pub login: String,
}

#[derive(Debug, Deserialize)]
pub struct WeekStats {
    pub a: u64,
    pub d: u64,
    #[expect(dead_code)]
    pub c: u64,
}

/// Fetch lines added/removed for a specific author in a repo.
#[expect(dead_code)]
pub fn fetch_contributor_stats(
    client: &Client,
    owner: &str,
    repo: &str,
    author: &str,
    max_retries: u32,
) -> Result<(u64, u64)> {
    let path = format!("/repos/{owner}/{repo}/stats/contributors");
    let resp = client.get_with_retry(&path, max_retries)?;
    let stats: Vec<ContributorStats> = serde_json::from_reader(resp.into_body().as_reader())?;

    for cs in &stats {
        if let Some(ref a) = cs.author
            && a.login == author
        {
            let added: u64 = cs.weeks.iter().map(|w| w.a).sum();
            let removed: u64 = cs.weeks.iter().map(|w| w.d).sum();
            return Ok((added, removed));
        }
    }

    Ok((0, 0))
}

/// Email addresses for the authenticated user.
#[derive(Debug, Deserialize)]
#[expect(dead_code)]
pub struct UserEmail {
    pub email: String,
    pub primary: bool,
}

/// Fetch email addresses for the authenticated user.
#[expect(dead_code)]
pub fn fetch_emails(client: &Client, max_retries: u32) -> Result<Vec<UserEmail>> {
    let resp = client.get_with_retry("/user/emails", max_retries)?;
    let emails: Vec<UserEmail> = serde_json::from_reader(resp.into_body().as_reader())?;
    Ok(emails)
}
