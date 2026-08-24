use core::fmt;

use crate::shared::auth::TokenValue;

#[derive(Clone, PartialEq, Eq)]
pub struct SessionToken(String);

impl SessionToken {
    pub fn from_string(value: String) -> Result<Self, SessionTokenError> {
        if value.trim().is_empty() {
            return Err(SessionTokenError::Empty);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl TokenValue for SessionToken {
    type Error = SessionTokenError;

    fn from_string(value: String) -> Result<Self, Self::Error> {
        SessionToken::from_string(value)
    }

    fn as_str(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Debug for SessionToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SessionToken([REDACTED])")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SessionTokenError {
    #[error("Session token cannot be empty")]
    Empty,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_string_preserves_value() {
        let token = "This is a session token";
        let session_token =
            SessionToken::from_string(token.to_string()).expect("Test token should be valid");

        assert_eq!(session_token.as_str(), token)
    }

    #[test]
    fn into_string_returns_inner_string() {
        let token = "This is a session token";
        let session_token =
            SessionToken::from_string(token.to_string()).expect("Test token should be valid");

        assert_eq!(session_token.into_string(), token)
    }

    #[test]
    fn empty_token_should_be_rejected() {
        let token = "    ";
        let session_token = SessionToken::from_string(token.to_string());

        assert_eq!(session_token, Err(SessionTokenError::Empty))
    }
}
