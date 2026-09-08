use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum UsageError {
    #[error("Codex was not detected at {0}")]
    CodexNotDetected(PathBuf),

    #[error("Codex is not logged in")]
    NotLoggedIn,

    #[error("unable to read Codex authentication")]
    AuthRead,

    #[error("Codex authentication data has an unsupported format")]
    AuthFormat,

    #[error("Codex session expired")]
    SessionExpired,

    #[error("usage request returned HTTP {0}")]
    HttpStatus(u16),

    #[error("usage request failed: {0}")]
    Network(String),

    #[error("usage response has an unsupported format")]
    ResponseFormat,

    #[error("no usable Codex quota entry was found in local logs")]
    LogDataUnavailable,

    #[error("unable to read Codex logs")]
    LogRead,

    #[error("unable to refresh usage; API: {api}; logs: {logs}")]
    ProvidersFailed { api: String, logs: String },
}

impl UsageError {
    pub fn user_message(&self) -> String {
        match self {
            Self::CodexNotDetected(_) => "Codex not detected".into(),
            Self::NotLoggedIn | Self::AuthRead | Self::AuthFormat => {
                "Codex is not logged in".into()
            }
            Self::SessionExpired => "Codex session expired".into(),
            Self::ProvidersFailed { .. }
            | Self::HttpStatus(_)
            | Self::Network(_)
            | Self::ResponseFormat
            | Self::LogDataUnavailable
            | Self::LogRead => "Unable to refresh Codex usage".into(),
        }
    }
}
