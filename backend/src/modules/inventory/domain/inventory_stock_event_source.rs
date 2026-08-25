use core::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventoryStockEventSource {
    Manual,
    Qr,
    Barcode,
    System,
}

impl InventoryStockEventSource {
    pub fn parse(value: &str) -> Result<Self, InventoryStockEventSourceError> {
        value.parse()
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::Qr => "qr",
            Self::Barcode => "barcode",
            Self::System => "system",
        }
    }
}

impl fmt::Display for InventoryStockEventSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for InventoryStockEventSource {
    type Err = InventoryStockEventSourceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "manual" => Ok(Self::Manual),
            "qr" => Ok(Self::Qr),
            "barcode" => Ok(Self::Barcode),
            "system" => Ok(Self::System),
            _ => Err(InventoryStockEventSourceError::Invalid),
        }
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum InventoryStockEventSourceError {
    #[error("The given household kind is invalid")]
    Invalid,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manual_can_be_parsed() {
        assert_eq!(
            InventoryStockEventSource::parse("manual"),
            Ok(InventoryStockEventSource::Manual)
        )
    }

    #[test]
    fn qr_can_be_parsed() {
        assert_eq!(
            InventoryStockEventSource::parse("qr"),
            Ok(InventoryStockEventSource::Qr)
        )
    }

    #[test]
    fn barcode_can_be_parsed() {
        assert_eq!(
            InventoryStockEventSource::parse("barcode"),
            Ok(InventoryStockEventSource::Barcode)
        )
    }

    #[test]
    fn system_can_be_parsed() {
        assert_eq!(
            InventoryStockEventSource::parse("system"),
            Ok(InventoryStockEventSource::System)
        )
    }

    #[test]
    fn unknown_kind_is_rejected() {
        assert_eq!(
            InventoryStockEventSource::parse("unknown"),
            Err(InventoryStockEventSourceError::Invalid)
        )
    }

    #[test]
    fn as_str_returns_database_representation() {
        assert_eq!(InventoryStockEventSource::Manual.as_str(), "manual");
        assert_eq!(InventoryStockEventSource::Qr.as_str(), "qr");
        assert_eq!(InventoryStockEventSource::Barcode.as_str(), "barcode");
        assert_eq!(InventoryStockEventSource::System.as_str(), "system");
    }
}
