use std::fmt::{self};

use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InventoryStockEventId(Uuid);

impl InventoryStockEventId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(id: Uuid) -> Self {
        Self(id)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    pub fn into_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for InventoryStockEventId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for InventoryStockEventId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_different_ids() {
        let first = InventoryStockEventId::new();
        let second = InventoryStockEventId::new();

        assert_ne!(first, second);
    }

    #[test]
    fn from_uuid_preservers_uuid() {
        let uuid = Uuid::new_v4();

        let user_id = InventoryStockEventId::from_uuid(uuid);

        assert_eq!(user_id.as_uuid(), &uuid);
    }

    #[test]
    fn into_uuid_returns_the_inner_uuid() {
        let uuid = Uuid::new_v4();
        let user_id = InventoryStockEventId::from_uuid(uuid);

        let result = user_id.into_uuid();

        assert_eq!(result, uuid);
    }
}
