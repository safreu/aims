use core::fmt;

use crate::shared::auth::TokenValue;

#[derive(Clone, PartialEq, Eq)]
pub struct DeviceToken(String);

impl DeviceToken {
    pub fn from_string(value: String) -> Result<Self, DeviceTokenError> {
        if value.trim().is_empty() {
            return Err(DeviceTokenError::Empty);
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

impl TokenValue for DeviceToken {
    type Error = DeviceTokenError;

    fn from_string(value: String) -> Result<Self, Self::Error> {
        DeviceToken::from_string(value)
    }

    fn as_str(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Debug for DeviceToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DeviceToken([REDACTED])")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DeviceTokenError {
    #[error("Device token cannot be empty")]
    Empty,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_string_preserves_value() {
        let token = "This is a session token";
        let device_token =
            DeviceToken::from_string(token.to_string()).expect("Test token should be valid");

        assert_eq!(device_token.as_str(), token)
    }

    #[test]
    fn into_string_returns_inner_string() {
        let token = "This is a device token";
        let device_token =
            DeviceToken::from_string(token.to_string()).expect("Test token should be valid");

        assert_eq!(device_token.into_string(), token)
    }

    #[test]
    fn empty_token_should_be_rejected() {
        let token = "    ";
        let device_token = DeviceToken::from_string(token.to_string());

        assert_eq!(device_token, Err(DeviceTokenError::Empty))
    }
}
