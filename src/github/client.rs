use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use ureq::http::Response;
use ureq::http::header::{AUTHORIZATION, USER_AGENT};

const API_BASE: &str = "https://api.github.com";
const USER_AGENT_VALUE: &str = concat!("ghpulse/", env!("CARGO_PKG_VERSION"));

/// Rate-limit state parsed from response headers.
#[derive(Debug, Default)]
struct RateLimit {
    remaining: u64,
    reset: u64,
}

/// GitHub API HTTP client with auth and rate-limit awareness.
pub struct Client {
    agent: ureq::Agent,
    token: String,
    rate_limit: Mutex<RateLimit>,
}

impl Client {
    /// Create a new client with the given GitHub personal access token.
    pub fn new(token: String) -> Self {
        let agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .https_only(true)
            .build()
            .into();

        Self {
            agent,
            token,
            rate_limit: Mutex::new(RateLimit::default()),
        }
    }

    fn auth_header_value(&self) -> String {
        format!("Bearer {}", self.token)
    }

    /// Send an authenticated GraphQL POST request and parse the response body
    /// as JSON. Retries 5xx and 429 responses with exponential backoff so
    /// transient gateway errors (502 Bad Gateway is common from GitHub) don't
    /// abort a long collection run.
    pub fn graphql_query(&self, body: &str) -> Result<serde_json::Value> {
        const MAX_RETRIES: u32 = 5;
        let mut attempt: u32 = 0;

        loop {
            self.rate_limit_check();

            let resp = self
                .agent
                .post(&format!("{API_BASE}/graphql"))
                .header(AUTHORIZATION, self.auth_header_value())
                .header(USER_AGENT, USER_AGENT_VALUE)
                .content_type("application/json")
                .send(body)?;

            self.update_rate_limit(&resp);

            let status = resp.status();
            let code = status.as_u16();
            let text = resp
                .into_body()
                .read_to_string()
                .context("failed to read GraphQL response body")?;

            let retriable = code == 429 || (500..=599).contains(&code);
            if retriable && attempt < MAX_RETRIES {
                attempt += 1;
                let backoff = Duration::from_secs(2u64.pow(attempt));
                tracing::warn!(
                    "GraphQL HTTP {status}, retry {attempt}/{MAX_RETRIES} in {backoff:?}"
                );
                std::thread::sleep(backoff);
                continue;
            }

            if !status.is_success() {
                let snippet: String = text.chars().take(500).collect();
                anyhow::bail!("GraphQL HTTP {status}: {snippet}");
            }

            return serde_json::from_str(&text).with_context(|| {
                let snippet: String = text.chars().take(500).collect();
                format!("invalid JSON from GraphQL (status {status}): {snippet}")
            });
        }
    }

    /// Send an authenticated REST GET request with retry logic.
    pub fn get_with_retry(&self, path: &str, max_retries: u32) -> Result<Response<ureq::Body>> {
        let url = format!("{API_BASE}{path}");
        let mut attempt = 0;

        loop {
            self.rate_limit_check();

            let resp = self
                .agent
                .get(&url)
                .header(AUTHORIZATION, self.auth_header_value())
                .header(USER_AGENT, USER_AGENT_VALUE)
                .call()?;

            self.update_rate_limit(&resp);

            let status = resp.status();
            if status.is_success() {
                return Ok(resp);
            }

            if (status.as_u16() == 403 || status.as_u16() == 202) && attempt < max_retries {
                attempt += 1;
                let backoff = Duration::from_secs(2u64.pow(attempt));
                tracing::warn!(path, attempt, "retrying in {backoff:?}");
                std::thread::sleep(backoff);
                continue;
            }

            let body = resp.into_body().read_to_string().unwrap_or_default();
            anyhow::bail!("HTTP {} for {}: {body}", status, path);
        }
    }

    /// Wait if we are at the rate limit.
    fn rate_limit_check(&self) {
        let rl = self.rate_limit.lock().unwrap();
        if rl.remaining == 0 {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            if rl.reset > now {
                let wait = Duration::from_secs(rl.reset - now + 1);
                tracing::warn!("rate limited, sleeping {wait:?}");
                drop(rl);
                std::thread::sleep(wait);
            }
        }
    }

    /// Update rate-limit state from response headers.
    fn update_rate_limit(&self, resp: &Response<ureq::Body>) {
        let mut rl = self.rate_limit.lock().unwrap();
        if let Some(val) = resp.headers().get("x-ratelimit-remaining") {
            rl.remaining = val.to_str().unwrap().parse().unwrap_or(rl.remaining);
        }
        if let Some(val) = resp.headers().get("x-ratelimit-reset") {
            rl.reset = val.to_str().unwrap().parse().unwrap_or(rl.reset);
        }
    }
}
