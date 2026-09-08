use std::{fs, path::Path};

use serde::Deserialize;

use super::UsageError;

#[derive(Debug, Deserialize)]
struct CodexAuthFile {
    tokens: CodexTokens,
}

#[derive(Debug, Deserialize)]
struct CodexTokens {
    access_token: String,
    #[serde(default)]
    account_id: String,
}

#[derive(Clone)]
pub(crate) struct CodexAuth {
    pub(crate) access_token: String,
    pub(crate) account_id: String,
}

pub(crate) fn read_auth(codex_home: &Path) -> Result<CodexAuth, UsageError> {
    let path = codex_home.join("auth.json");
    if !path.is_file() {
        return Err(UsageError::NotLoggedIn);
    }

    let bytes = fs::read(path).map_err(|_| UsageError::AuthRead)?;
    let file: CodexAuthFile = serde_json::from_slice(&bytes).map_err(|_| UsageError::AuthFormat)?;

    if file.tokens.access_token.trim().is_empty() {
        return Err(UsageError::NotLoggedIn);
    }

    Ok(CodexAuth {
        access_token: file.tokens.access_token,
        account_id: file.tokens.account_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_supported_auth_shape_without_exposing_extra_fields() {
        let file: CodexAuthFile = serde_json::from_str(
            r#"{"tokens":{"access_token":"secret","account_id":"account","refresh_token":"ignored"}}"#,
        )
        .expect("valid auth fixture");

        assert_eq!(file.tokens.access_token, "secret");
        assert_eq!(file.tokens.account_id, "account");
    }
}
