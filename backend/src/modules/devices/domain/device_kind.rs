use core::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceKind {
    Scanner,
    Display,
    Other,
}

impl DeviceKind {
    pub fn parse(value: &str) -> Result<Self, DeviceKindError> {
        value.parse()
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Scanner => "scanner",
            Self::Display => "display",
            Self::Other => "other",
        }
    }
}

impl fmt::Display for DeviceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for DeviceKind {
    type Err = DeviceKindError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "scanner" => Ok(Self::Scanner),
            "display" => Ok(Self::Display),
            "other" => Ok(Self::Other),
            _ => Err(DeviceKindError::Invalid),
        }
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum DeviceKindError {
    #[error("The given household kind is invalid")]
    Invalid,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scanner_can_be_parsed() {
        assert_eq!(DeviceKind::parse("scanner"), Ok(DeviceKind::Scanner))
    }

    #[test]
    fn display_can_be_parsed() {
        assert_eq!(DeviceKind::parse("display"), Ok(DeviceKind::Display))
    }

    #[test]
    fn other_can_be_parsed() {
        assert_eq!(DeviceKind::parse("other"), Ok(DeviceKind::Other))
    }

    #[test]
    fn unknown_kind_is_rejected() {
        assert_eq!(DeviceKind::parse("unknown"), Err(DeviceKindError::Invalid))
    }

    #[test]
    fn as_str_returns_database_representation() {
        assert_eq!(DeviceKind::Scanner.as_str(), "scanner");
        assert_eq!(DeviceKind::Display.as_str(), "display");
        assert_eq!(DeviceKind::Other.as_str(), "other");
    }
}
