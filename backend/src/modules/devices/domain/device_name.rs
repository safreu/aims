#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceName(String);

impl DeviceName {
    pub fn parse(value: &str) -> Result<Self, DeviceNameError> {
        let value = value.trim();
        if value.is_empty() {
            return Err(DeviceNameError::Empty);
        };

        if value.chars().count() > 50 {
            return Err(DeviceNameError::TooLong);
        }

        Ok(Self(value.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DeviceNameError {
    #[error("Display name cannot be empty")]
    Empty,
    #[error("Display name cannot be longer than 50 characters")]
    TooLong,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surrounding_whitespace_is_removed() {
        let name = "    device_name      ";
        let result = DeviceName::parse(name).expect("Device name should be parseable");

        assert_eq!(result.as_str(), "device_name")
    }

    #[test]
    fn empty_device_name_is_rejected() {
        let name = " ";
        let result = DeviceName::parse(name);

        assert_eq!(result, Err(DeviceNameError::Empty))
    }

    #[test]
    fn too_long_device_name_is_rejected() {
        let name = &"a".repeat(101);
        let result = DeviceName::parse(name);

        assert_eq!(result, Err(DeviceNameError::TooLong))
    }

    #[test]
    fn valid_device_name_is_accepted() {
        let name = "this is a valid device name";
        let result = DeviceName::parse(name);

        assert!(result.is_ok())
    }
}
