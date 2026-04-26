use anyhow::Result;

use crate::github::client::Client;
use crate::stats::types::Stats;

/// Collect all GitHub stats using the API client.
#[allow(dead_code)]
pub fn collect(_client: &Client, _max_retries: u32) -> Result<Stats> {
    // TODO: GraphQL user info, contribution years, repos
    // TODO: REST traffic/views, contributor stats
    anyhow::bail!("data collection not yet implemented")
}

/// Load stats from a cached JSON file.
#[allow(dead_code)]
pub fn from_json(path: &str) -> Result<Stats> {
    let data = std::fs::read_to_string(path)?;
    let stats: Stats = serde_json::from_str(&data)?;
    Ok(stats)
}

/// Dump stats to a JSON file.
#[allow(dead_code)]
pub fn dump_json(stats: &Stats, path: &str) -> Result<()> {
    let data = serde_json::to_string_pretty(stats)?;
    std::fs::write(path, data)?;
    Ok(())
}
