use core::fmt;

use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QrActionId(Uuid);

impl QrActionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    pub fn into_uuid(self) -> Uuid {
        self.0
    }
}

impl fmt::Display for QrActionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Default for QrActionId {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_different_ids() {
        let first = QrActionId::new();
        let second = QrActionId::new();

        assert_ne!(first, second);
    }

    #[test]
    fn from_uuid_preservers_uuid() {
        let uuid = Uuid::new_v4();

        let user_id = QrActionId::from_uuid(uuid);

        assert_eq!(user_id.as_uuid(), &uuid);
    }

    #[test]
    fn into_uuid_returns_the_inner_uuid() {
        let uuid = Uuid::new_v4();
        let user_id = QrActionId::from_uuid(uuid);

        let result = user_id.into_uuid();

        assert_eq!(result, uuid);
    }
}
