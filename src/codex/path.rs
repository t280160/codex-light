use std::path::PathBuf;

use super::UsageError;

pub fn resolve_codex_home() -> Result<PathBuf, UsageError> {
    let path = std::env::var_os("CODEX_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|home| home.join(".codex")))
        .ok_or_else(|| UsageError::CodexNotDetected(PathBuf::from(".codex")))?;

    if path.is_dir() {
        Ok(path)
    } else {
        Err(UsageError::CodexNotDetected(path))
    }
}
