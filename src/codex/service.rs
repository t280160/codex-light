use std::path::PathBuf;

use super::{
    api_provider::ApiUsageProvider, auth::read_auth, log_provider::LogUsageProvider, CodexUsage,
    UsageError, UsageProvider,
};

pub struct UsageService {
    codex_home: PathBuf,
}

impl UsageService {
    pub fn new(codex_home: PathBuf) -> Self {
        Self { codex_home }
    }

    pub async fn get_usage(&self) -> Result<CodexUsage, UsageError> {
        let api_result = match read_auth(&self.codex_home) {
            Ok(auth) => match ApiUsageProvider::new(auth) {
                Ok(provider) => provider.get_usage().await,
                Err(error) => Err(error),
            },
            Err(error) => Err(error),
        };

        let api_error = match api_result {
            Ok(usage) => return Ok(usage),
            Err(error) => error,
        };
        let log_result = LogUsageProvider::new(&self.codex_home).get_usage().await;
        match log_result {
            Ok(usage) => Ok(usage),
            Err(_)
                if matches!(
                    &api_error,
                    UsageError::NotLoggedIn | UsageError::AuthRead | UsageError::AuthFormat
                ) =>
            {
                Err(api_error)
            }
            Err(_) if matches!(&api_error, UsageError::SessionExpired) => Err(api_error),
            Err(log_error) => Err(UsageError::ProvidersFailed {
                api: api_error.to_string(),
                logs: log_error.to_string(),
            }),
        }
    }
}
