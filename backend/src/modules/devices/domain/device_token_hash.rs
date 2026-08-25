use core::fmt;

use crate::shared::auth::TokenHashValue;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct DeviceTokenHash(String);

impl DeviceTokenHash {
    pub fn from_encoded(value: &str) -> Result<Self, DeviceTokenHashError> {
        let value = value.trim();

        if value.is_empty() {
            return Err(DeviceTokenHashError::Empty);
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

impl TokenHashValue for DeviceTokenHash {
    type Error = DeviceTokenHashError;

    fn from_encoded(value: &str) -> Result<Self, Self::Error> {
        DeviceTokenHash::from_encoded(value)
    }

    fn as_str(&self) -> &str {
        self.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DeviceTokenHashError {
    #[error("Device token hash cannot be empty")]
    Empty,
}

impl fmt::Debug for DeviceTokenHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DeviceTokenHash([REDACTED])")
    }
}

impl fmt::Display for DeviceTokenHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DeviceTokenHash([REDACTED])")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_empty_hash_is_accepted() {
        let hash = DeviceTokenHash::from_encoded("hashed_device_token");

        assert!(hash.is_ok());
    }

    #[test]
    fn empty_hash_is_rejected() {
        let result = DeviceTokenHash::from_encoded("");

        assert_eq!(result, Err(DeviceTokenHashError::Empty));
    }

    #[test]
    fn whitespace_only_hash_is_rejected() {
        let result = DeviceTokenHash::from_encoded("      ");

        assert_eq!(result, Err(DeviceTokenHashError::Empty));
    }

    #[test]
    fn debug_output_is_redacted() {
        let hash =
            DeviceTokenHash::from_encoded("hashed_device_token").expect("Hash should be valid");

        assert_eq!(format!("{hash:?}"), "DeviceTokenHash([REDACTED])")
    }
}
