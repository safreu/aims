use core::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventoryStockEventKind {
    Increase,
    Decrease,
    Set,
}

impl InventoryStockEventKind {
    pub fn parse(value: &str) -> Result<Self, InventoryStockEventKindError> {
        value.parse()
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Increase => "increase",
            Self::Decrease => "decrease",
            Self::Set => "set",
        }
    }
}

impl fmt::Display for InventoryStockEventKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for InventoryStockEventKind {
    type Err = InventoryStockEventKindError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "increase" => Ok(Self::Increase),
            "decrease" => Ok(Self::Decrease),
            "set" => Ok(Self::Set),
            _ => Err(InventoryStockEventKindError::Invalid),
        }
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum InventoryStockEventKindError {
    #[error("The given household kind is invalid")]
    Invalid,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn increase_can_be_parsed() {
        assert_eq!(
            InventoryStockEventKind::parse("increase"),
            Ok(InventoryStockEventKind::Increase)
        )
    }

    #[test]
    fn decrease_can_be_parsed() {
        assert_eq!(
            InventoryStockEventKind::parse("decrease"),
            Ok(InventoryStockEventKind::Decrease)
        )
    }

    #[test]
    fn set_can_be_parsed() {
        assert_eq!(
            InventoryStockEventKind::parse("set"),
            Ok(InventoryStockEventKind::Set)
        )
    }

    #[test]
    fn unknown_kind_is_rejected() {
        assert_eq!(
            InventoryStockEventKind::parse("unknown"),
            Err(InventoryStockEventKindError::Invalid)
        )
    }

    #[test]
    fn as_str_returns_database_representation() {
        assert_eq!(InventoryStockEventKind::Increase.as_str(), "increase");
        assert_eq!(InventoryStockEventKind::Decrease.as_str(), "decrease");
        assert_eq!(InventoryStockEventKind::Set.as_str(), "set");
    }
}
