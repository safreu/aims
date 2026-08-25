use core::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QrActionKind {
    Increase,
    Decrease,
}

impl QrActionKind {
    pub fn parse(value: &str) -> Result<Self, QrActionKindError> {
        value.parse()
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Increase => "increase",
            Self::Decrease => "decrease",
        }
    }
}

impl fmt::Display for QrActionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for QrActionKind {
    type Err = QrActionKindError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "increase" => Ok(Self::Increase),
            "decrease" => Ok(Self::Decrease),
            _ => Err(QrActionKindError::Invalid),
        }
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum QrActionKindError {
    #[error("The given QR action kind is invalid")]
    Invalid,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn increase_can_be_parsed() {
        assert_eq!(QrActionKind::parse("increase"), Ok(QrActionKind::Increase))
    }

    #[test]
    fn decrease_can_be_parsed() {
        assert_eq!(QrActionKind::parse("decrease"), Ok(QrActionKind::Decrease))
    }

    #[test]
    fn unknown_kind_is_rejected() {
        assert_eq!(
            QrActionKind::parse("unknown"),
            Err(QrActionKindError::Invalid)
        )
    }

    #[test]
    fn as_str_returns_database_representation() {
        assert_eq!(QrActionKind::Increase.as_str(), "increase");
        assert_eq!(QrActionKind::Decrease.as_str(), "decrease");
    }
}
