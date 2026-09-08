use std::time::Duration;

use async_trait::async_trait;
use reqwest::StatusCode;
use serde::Deserialize;

use super::{auth::CodexAuth, remaining_percent, unix_now, CodexUsage, UsageError, UsageProvider};

const USAGE_URL: &str = "https://chatgpt.com/backend-api/wham/usage";

pub(crate) struct ApiUsageProvider {
    auth: CodexAuth,
    client: reqwest::Client,
}

impl ApiUsageProvider {
    pub(crate) fn new(auth: CodexAuth) -> Result<Self, UsageError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("CodexLight/0.1")
            .build()
            .map_err(|error| UsageError::Network(error.to_string()))?;
        Ok(Self { auth, client })
    }
}

#[derive(Debug, Deserialize)]
struct UsageResponse {
    rate_limit: RateLimit,
}

#[derive(Debug, Deserialize)]
struct RateLimit {
    primary_window: UsageWindow,
    secondary_window: UsageWindow,
}

#[derive(Debug, Deserialize)]
struct UsageWindow {
    used_percent: f64,
    #[serde(default)]
    reset_at: Option<f64>,
    #[serde(default)]
    reset_after_seconds: Option<f64>,
}

#[async_trait]
impl UsageProvider for ApiUsageProvider {
    async fn get_usage(&self) -> Result<CodexUsage, UsageError> {
        let mut request = self
            .client
            .get(USAGE_URL)
            .bearer_auth(&self.auth.access_token)
            .header(reqwest::header::ACCEPT, "application/json");

        if !self.auth.account_id.trim().is_empty() {
            request = request.header("ChatGPT-Account-Id", self.auth.account_id.as_str());
        }

        let response = request
            .send()
            .await
            .map_err(|error| UsageError::Network(error.to_string()))?;
        let status = response.status();
        if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
            return Err(UsageError::SessionExpired);
        }
        if !status.is_success() {
            return Err(UsageError::HttpStatus(status.as_u16()));
        }

        let body: UsageResponse = response
            .json()
            .await
            .map_err(|_| UsageError::ResponseFormat)?;
        let now = unix_now();

        Ok(CodexUsage {
            five_hour_remaining: remaining_percent(body.rate_limit.primary_window.used_percent),
            weekly_remaining: remaining_percent(body.rate_limit.secondary_window.used_percent),
            five_hour_reset_at: reset_at(&body.rate_limit.primary_window, now),
            weekly_reset_at: reset_at(&body.rate_limit.secondary_window, now),
            updated_at: now,
            source: "api".into(),
            stale: false,
        })
    }
}

fn reset_at(window: &UsageWindow, now: i64) -> Option<i64> {
    window
        .reset_at
        .filter(|value| value.is_finite() && *value > 0.0)
        .map(|value| value.round() as i64)
        .or_else(|| {
            window
                .reset_after_seconds
                .filter(|value| value.is_finite() && *value >= 0.0)
                .map(|value| now.saturating_add(value.round() as i64))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absolute_reset_timestamp_takes_precedence() {
        let window = UsageWindow {
            used_percent: 0.0,
            reset_at: Some(1_000.0),
            reset_after_seconds: Some(50.0),
        };
        assert_eq!(reset_at(&window, 100), Some(1_000));
    }

    #[test]
    fn reset_after_is_used_when_timestamp_is_missing() {
        let window = UsageWindow {
            used_percent: 0.0,
            reset_at: None,
            reset_after_seconds: Some(50.0),
        };
        assert_eq!(reset_at(&window, 100), Some(150));
    }
}
