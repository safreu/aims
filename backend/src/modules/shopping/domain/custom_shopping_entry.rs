use chrono::{DateTime, Utc};

use crate::modules::{
    households::domain::HouseholdId,
    inventory::domain::InventoryPriority,
    shopping::domain::{CustomShoppingEntryTitle, custom_shopping_entry_id::CustomShoppingEntryId},
};

const MAX_NOTE_LENGTH: usize = 50;

#[derive(Debug, PartialEq, Eq)]
pub struct CustomShoppingEntry {
    id: CustomShoppingEntryId,
    household_id: HouseholdId,
    title: CustomShoppingEntryTitle,
    quantity: u32,
    priority: InventoryPriority,
    note: Option<String>,
    checked: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl CustomShoppingEntry {
    pub fn new(
        id: CustomShoppingEntryId,
        household_id: HouseholdId,
        title: CustomShoppingEntryTitle,
        quantity: u32,
        priority: InventoryPriority,
        note: Option<String>,
        now: DateTime<Utc>,
    ) -> Result<Self, CustomShoppingEntryError> {
        if quantity == 0 {
            return Err(CustomShoppingEntryError::InvalidQuantity);
        }

        let note = Self::validate_note(note)?;

        Ok(Self {
            id,
            household_id,
            title,
            quantity,
            priority,
            note,
            checked: false,
            created_at: now,
            updated_at: now,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_persisted(
        id: CustomShoppingEntryId,
        household_id: HouseholdId,
        title: CustomShoppingEntryTitle,
        quantity: u32,
        priority: InventoryPriority,
        note: Option<String>,
        checked: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, CustomShoppingEntryError> {
        if quantity == 0 {
            return Err(CustomShoppingEntryError::InvalidQuantity);
        }

        let note = Self::validate_note(note)?;

        if created_at > updated_at {
            return Err(CustomShoppingEntryError::InvalidTimestamps);
        }

        Ok(Self {
            id,
            household_id,
            title,
            quantity,
            priority,
            note,
            checked,
            created_at,
            updated_at,
        })
    }

    pub fn id(&self) -> CustomShoppingEntryId {
        self.id
    }

    pub fn household_id(&self) -> HouseholdId {
        self.household_id
    }

    pub fn title(&self) -> &CustomShoppingEntryTitle {
        &self.title
    }

    pub fn quantity(&self) -> u32 {
        self.quantity
    }

    pub fn priority(&self) -> InventoryPriority {
        self.priority
    }

    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }

    pub fn checked(&self) -> bool {
        self.checked
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn rename(
        &mut self,
        title: CustomShoppingEntryTitle,
        now: DateTime<Utc>,
    ) -> Result<(), CustomShoppingEntryError> {
        self.validate_update_time(now)?;

        self.title = title;
        self.updated_at = now;

        Ok(())
    }

    pub fn set_quantity(
        &mut self,
        quantity: u32,
        now: DateTime<Utc>,
    ) -> Result<(), CustomShoppingEntryError> {
        self.validate_update_time(now)?;
        if quantity == 0 {
            return Err(CustomShoppingEntryError::InvalidQuantity);
        }

        self.quantity = quantity;
        self.updated_at = now;

        Ok(())
    }

    pub fn set_priority(
        &mut self,
        priority: InventoryPriority,
        now: DateTime<Utc>,
    ) -> Result<(), CustomShoppingEntryError> {
        self.validate_update_time(now)?;

        self.priority = priority;
        self.updated_at = now;

        Ok(())
    }

    pub fn set_note(
        &mut self,
        note: Option<String>,
        now: DateTime<Utc>,
    ) -> Result<(), CustomShoppingEntryError> {
        self.validate_update_time(now)?;

        self.note = Self::validate_note(note)?;
        self.updated_at = now;

        Ok(())
    }

    pub fn check(&mut self, now: DateTime<Utc>) -> Result<(), CustomShoppingEntryError> {
        self.validate_update_time(now)?;

        self.checked = true;
        self.updated_at = now;

        Ok(())
    }

    pub fn uncheck(&mut self, now: DateTime<Utc>) -> Result<(), CustomShoppingEntryError> {
        self.validate_update_time(now)?;

        self.checked = false;
        self.updated_at = now;

        Ok(())
    }

    fn validate_update_time(&self, now: DateTime<Utc>) -> Result<(), CustomShoppingEntryError> {
        if now < self.updated_at {
            return Err(CustomShoppingEntryError::InvalidTimestamps);
        }
        Ok(())
    }

    fn validate_note(note: Option<String>) -> Result<Option<String>, CustomShoppingEntryError> {
        let note = note
            .map(|note| note.trim().to_owned())
            .filter(|note| !note.is_empty());

        if let Some(note) = &note
            && note.chars().count() > MAX_NOTE_LENGTH
        {
            return Err(CustomShoppingEntryError::InvalidNoteLength);
        }

        Ok(note)
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum CustomShoppingEntryError {
    #[error("Shopping quantity must be greater than zero")]
    InvalidQuantity,
    #[error("Shopping note must not exceed 50 characters")]
    InvalidNoteLength,
    #[error("Updated time cannot be before creation time")]
    InvalidTimestamps,
}
