use crate::modules::{households::domain::HouseholdId, inventory::domain::InventoryItemId};

const MAX_NOTE_LENGTH: usize = 50;

pub struct InventoryShoppingState {
    household_id: HouseholdId,
    item_id: InventoryItemId,
    quantity_override: Option<u32>,
    note: Option<String>,
    checked: bool,
    dismissed: bool,
}

impl InventoryShoppingState {
    pub fn new(household_id: HouseholdId, item_id: InventoryItemId) -> Self {
        Self {
            household_id,
            item_id,
            quantity_override: None,
            note: None,
            checked: false,
            dismissed: false,
        }
    }

    pub fn from_persisted(
        household_id: HouseholdId,
        item_id: InventoryItemId,
        quantity_override: Option<u32>,
        note: Option<String>,
        checked: bool,
        dismissed: bool,
    ) -> Result<Self, InventoryShoppingStateError> {
        if quantity_override == Some(0) {
            return Err(InventoryShoppingStateError::InvalidQuantity);
        }

        let note = note
            .map(|note| note.trim().to_owned())
            .filter(|note| !note.is_empty());

        if let Some(note) = &note
            && note.chars().count() > MAX_NOTE_LENGTH
        {
            return Err(InventoryShoppingStateError::NoteTooLong);
        }

        Ok(Self {
            household_id,
            item_id,
            quantity_override,
            note,
            checked,
            dismissed,
        })
    }

    pub fn household_id(&self) -> HouseholdId {
        self.household_id
    }

    pub fn item_id(&self) -> InventoryItemId {
        self.item_id
    }

    pub fn quantity_override(&self) -> Option<u32> {
        self.quantity_override
    }

    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }

    pub fn checked(&self) -> bool {
        self.checked
    }

    pub fn dismissed(&self) -> bool {
        self.dismissed
    }

    pub fn check(&mut self) {
        self.checked = true
    }

    pub fn uncheck(&mut self) {
        self.checked = false
    }

    pub fn dismiss(&mut self) {
        self.dismissed = true
    }

    pub fn set_quantity_override(
        &mut self,
        quantity: u32,
    ) -> Result<(), InventoryShoppingStateError> {
        if quantity == 0 {
            return Err(InventoryShoppingStateError::InvalidQuantity);
        }

        self.quantity_override = Some(quantity);

        Ok(())
    }

    pub fn clear_quantity_override(&mut self) {
        self.quantity_override = None
    }

    pub fn set_note(&mut self, note: Option<String>) -> Result<(), InventoryShoppingStateError> {
        let note = note
            .map(|note| note.trim().to_owned())
            .filter(|note| !note.is_empty());

        if let Some(note) = &note
            && note.chars().count() > MAX_NOTE_LENGTH
        {
            return Err(InventoryShoppingStateError::NoteTooLong);
        }

        self.note = note;

        Ok(())
    }

    pub fn reset_after_inventory_change(&mut self) {
        self.quantity_override = None;
        self.checked = false;
        self.dismissed = false;
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum InventoryShoppingStateError {
    #[error("Shopping note must not exceed 50 characters")]
    NoteTooLong,
    #[error("Shopping quantity must be greater than zero")]
    InvalidQuantity,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_state_has_default_values() {
        let state = InventoryShoppingState::new(HouseholdId::new(), InventoryItemId::new());

        assert!(state.quantity_override().is_none());
        assert!(state.note().is_none());
        assert!(!state.checked());
        assert!(!state.dismissed());
    }

    #[test]
    fn state_can_be_checked_and_unchecked() {
        let mut state = InventoryShoppingState::new(HouseholdId::new(), InventoryItemId::new());

        state.check();
        assert!(state.checked());

        state.uncheck();
        assert!(!state.checked());
    }

    #[test]
    fn state_can_be_dismissed() {
        let mut state = InventoryShoppingState::new(HouseholdId::new(), InventoryItemId::new());

        state.dismiss();

        assert!(state.dismissed());
    }

    #[test]
    fn positive_quantity_override_is_accepted() {
        let mut state = InventoryShoppingState::new(HouseholdId::new(), InventoryItemId::new());

        state
            .set_quantity_override(5)
            .expect("Setting quantity should be valid");

        assert_eq!(state.quantity_override(), Some(5));
    }

    #[test]
    fn zero_quantity_override_is_rejected() {
        let mut state = InventoryShoppingState::new(HouseholdId::new(), InventoryItemId::new());

        let result = state.set_quantity_override(0);

        assert_eq!(result, Err(InventoryShoppingStateError::InvalidQuantity))
    }

    #[test]
    fn quantity_override_can_be_cleared() {
        let mut state = InventoryShoppingState::new(HouseholdId::new(), InventoryItemId::new());

        state
            .set_quantity_override(5)
            .expect("Setting quantity should be valid");

        state.clear_quantity_override();

        assert!(state.quantity_override().is_none())
    }

    #[test]
    fn empty_note_is_stored_as_none() {
        let mut state = InventoryShoppingState::new(HouseholdId::new(), InventoryItemId::new());

        state
            .set_note(Some("      ".to_owned()))
            .expect("Setting note should succeed");

        assert!(state.note().is_none())
    }

    #[test]
    fn note_can_be_set() {
        let mut state = InventoryShoppingState::new(HouseholdId::new(), InventoryItemId::new());

        state
            .set_note(Some("this is a note".to_owned()))
            .expect("Setting note should succeed");

        assert_eq!(state.note(), Some("this is a note"))
    }

    #[test]
    fn inventory_change_resets_temporary_state() {
        let mut state = InventoryShoppingState::new(HouseholdId::new(), InventoryItemId::new());

        state.check();
        state.dismiss();
        state
            .set_quantity_override(5)
            .expect("Setting quantity should succeed");

        state.reset_after_inventory_change();

        assert!(state.quantity_override().is_none());
        assert!(!state.checked());
        assert!(!state.dismissed());
    }

    #[test]
    fn note_is_trimmed() {
        let mut state = InventoryShoppingState::new(HouseholdId::new(), InventoryItemId::new());

        state
            .set_note(Some("    this is a note    ".to_owned()))
            .expect("Setting note should succeed");

        assert_eq!(state.note(), Some("this is a note"))
    }

    #[test]
    fn note_longer_than_50_characters_is_rejected() {
        let mut state = InventoryShoppingState::new(HouseholdId::new(), InventoryItemId::new());

        let result = state.set_note(Some("a".repeat(MAX_NOTE_LENGTH + 1).to_owned()));

        assert_eq!(result, Err(InventoryShoppingStateError::NoteTooLong))
    }

    #[test]
    fn note_with_50_characters_is_accepted() {
        let mut state = InventoryShoppingState::new(HouseholdId::new(), InventoryItemId::new());

        state
            .set_note(Some("a".repeat(MAX_NOTE_LENGTH).to_owned()))
            .expect("Setting note should succeed");

        assert!(state.note().is_some())
    }
}
