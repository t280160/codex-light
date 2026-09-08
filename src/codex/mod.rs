mod api_provider;
mod auth;
mod error;
mod log_provider;
mod model;
mod path;
mod service;

pub use error::UsageError;
#[allow(unused_imports)]
pub use model::{human_duration, remaining_percent, CodexUsage};
pub use path::resolve_codex_home;
pub use service::UsageService;

use async_trait::async_trait;

#[async_trait]
trait UsageProvider {
    async fn get_usage(&self) -> Result<CodexUsage, UsageError>;
}

pub(crate) fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or_default()
}
