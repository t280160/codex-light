use std::path::{Path, PathBuf};

use async_trait::async_trait;
use regex::Regex;
use rusqlite::{Connection, OpenFlags};

use super::{remaining_percent, unix_now, CodexUsage, UsageError, UsageProvider};

const STALE_AFTER_SECONDS: i64 = 30 * 60;
const QUERY: &str = r#"
    select ts, replace(feedback_log_body, char(10), ' ')
    from logs
    where (feedback_log_body like '%x-codex-primary-used-percent%'
        or feedback_log_body like '%x-codex-secondary-used-percent%')
      and feedback_log_body like '%Request completed method=POST url=https://chatgpt.com/backend-api/codex/responses%'
      and feedback_log_body like '%headers=%'
      and feedback_log_body like '%:responses.stream_request{%'
      and feedback_log_body like '%:endpoint_session.stream_encoded_json_with{%'
      and feedback_log_body not like '%ToolCall:%'
      and feedback_log_body not like '%tool_name="exec_command"%'
    order by ts desc, ts_nanos desc
    limit 200
"#;

pub(crate) struct LogUsageProvider {
    candidates: [PathBuf; 2],
}

impl LogUsageProvider {
    pub(crate) fn new(codex_home: &Path) -> Self {
        Self {
            candidates: [
                codex_home.join("logs_2.sqlite"),
                codex_home.join("sqlite").join("logs_2.sqlite"),
            ],
        }
    }

    fn read_candidate(path: &Path) -> Result<Option<CodexUsage>, UsageError> {
        if !path.is_file() {
            return Ok(None);
        }

        let connection = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|_| UsageError::LogRead)?;
        connection
            .busy_timeout(std::time::Duration::from_millis(1_500))
            .map_err(|_| UsageError::LogRead)?;

        let mut statement = connection.prepare(QUERY).map_err(|_| UsageError::LogRead)?;
        let rows = statement
            .query_map([], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|_| UsageError::LogRead)?;

        for row in rows {
            let (observed_at, body) = row.map_err(|_| UsageError::LogRead)?;
            if let Some(usage) = usage_from_log_entry(observed_at, &body) {
                return Ok(Some(usage));
            }
        }
        Ok(None)
    }
}

#[async_trait]
impl UsageProvider for LogUsageProvider {
    async fn get_usage(&self) -> Result<CodexUsage, UsageError> {
        let mut read_failed = false;
        for candidate in &self.candidates {
            match Self::read_candidate(candidate) {
                Ok(Some(usage)) => return Ok(usage),
                Ok(None) => {}
                Err(_) => read_failed = true,
            }
        }

        if read_failed {
            Err(UsageError::LogRead)
        } else {
            Err(UsageError::LogDataUnavailable)
        }
    }
}

fn usage_from_log_entry(observed_at: i64, body: &str) -> Option<CodexUsage> {
    let headers = codex_headers(body);
    let primary_used = parse_percent(headers.get("x-codex-primary-used-percent")?)?;
    let secondary_used = parse_percent(headers.get("x-codex-secondary-used-percent")?)?;
    let primary_reset = headers
        .get("x-codex-primary-reset-after-seconds")
        .and_then(|value| value.parse::<f64>().ok());
    let secondary_reset = headers
        .get("x-codex-secondary-reset-after-seconds")
        .and_then(|value| value.parse::<f64>().ok());
    let now = unix_now();
    let stale = now.saturating_sub(observed_at) > STALE_AFTER_SECONDS;

    Some(CodexUsage {
        five_hour_remaining: remaining_percent(primary_used),
        weekly_remaining: remaining_percent(secondary_used),
        five_hour_reset_at: reset_from_log(observed_at, primary_reset, stale),
        weekly_reset_at: reset_from_log(observed_at, secondary_reset, stale),
        updated_at: observed_at,
        source: "logs".into(),
        stale,
    })
}

fn codex_headers(body: &str) -> std::collections::HashMap<String, String> {
    let Ok(pattern) = Regex::new(r#""(x-codex-[^"]+)"\s*:\s*"([^"]*)""#) else {
        return Default::default();
    };
    pattern
        .captures_iter(body)
        .filter_map(|capture| {
            Some((
                capture.get(1)?.as_str().to_owned(),
                capture.get(2)?.as_str().to_owned(),
            ))
        })
        .collect()
}

fn parse_percent(value: &str) -> Option<f64> {
    value.parse::<f64>().ok().filter(|value| value.is_finite())
}

fn reset_from_log(observed_at: i64, reset_after: Option<f64>, stale: bool) -> Option<i64> {
    if stale {
        return None;
    }
    reset_after
        .filter(|value| value.is_finite() && *value >= 0.0)
        .map(|value| observed_at.saturating_add(value.round() as i64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_codex_headers_without_other_content() {
        let body = r#"headers={"x-codex-primary-used-percent":"27.4","authorization":"secret","x-codex-secondary-used-percent":"62","x-codex-primary-reset-after-seconds":"900"}"#;
        let headers = codex_headers(body);

        assert_eq!(
            headers
                .get("x-codex-primary-used-percent")
                .map(String::as_str),
            Some("27.4")
        );
        assert_eq!(
            headers
                .get("x-codex-secondary-used-percent")
                .map(String::as_str),
            Some("62")
        );
        assert!(!headers.contains_key("authorization"));
    }

    #[test]
    fn stale_log_reset_is_not_presented_as_current() {
        assert_eq!(reset_from_log(1_000, Some(300.0), true), None);
        assert_eq!(reset_from_log(1_000, Some(300.0), false), Some(1_300));
    }
}
