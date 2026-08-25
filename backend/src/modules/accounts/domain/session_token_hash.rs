use core::fmt;

use crate::shared::auth::TokenHashValue;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SessionTokenHash(String);

impl SessionTokenHash {
    pub fn from_encoded(value: &str) -> Result<Self, SessionTokenHashError> {
        let value = value.trim();

        if value.is_empty() {
            return Err(SessionTokenHashError::Empty);
        }

        Ok(Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl TokenHashValue for SessionTokenHash {
    type Error = SessionTokenHashError;

    fn from_encoded(value: &str) -> Result<Self, Self::Error> {
        SessionTokenHash::from_encoded(value)
    }

    fn as_str(&self) -> &str {
        self.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SessionTokenHashError {
    #[error("Session token hash cannot be empty")]
    Empty,
}

impl fmt::Debug for SessionTokenHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SessionTokenHash([REDACTED])")
    }
}

impl fmt::Display for SessionTokenHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SessionTokenHash([REDACTED])")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_empty_hash_is_accepted() {
        let hash = SessionTokenHash::from_encoded("hashed_session_token");

        assert!(hash.is_ok());
    }

    #[test]
    fn empty_hash_is_rejected() {
        let result = SessionTokenHash::from_encoded("");

        assert_eq!(result, Err(SessionTokenHashError::Empty));
    }

    #[test]
    fn whitespace_only_hash_is_rejected() {
        let result = SessionTokenHash::from_encoded("      ");

        assert_eq!(result, Err(SessionTokenHashError::Empty));
    }

    #[test]
    fn debug_output_is_redacted() {
        let hash =
            SessionTokenHash::from_encoded("hashed_session_token").expect("Hash should be valid");

        assert_eq!(format!("{hash:?}"), "SessionTokenHash([REDACTED])")
    }
}
