use crate::{
    modules::shopping::application::{
        CreateCustomShoppingEntryError, DeleteCustomShoppingEntryError, DismissShoppingItemError,
        ListShoppingError, SetCheckedError, SetCustomShoppingEntryCheckedError, SetNoteError,
        SetShoppingQuantityError, UpdateCustomShoppingEntryError,
    },
    shared::api::ApiError,
};

impl From<ListShoppingError> for ApiError {
    fn from(value: ListShoppingError) -> Self {
        match value {
            ListShoppingError::HouseholdAccess(error) => error.into(),
            ListShoppingError::Internal(_) => ApiError::internal_error(),
        }
    }
}

impl From<SetShoppingQuantityError> for ApiError {
    fn from(value: SetShoppingQuantityError) -> Self {
        match value {
            SetShoppingQuantityError::HouseholdAccess(error) => error.into(),
            SetShoppingQuantityError::Internal(_) => ApiError::internal_error(),
            SetShoppingQuantityError::ItemNotFound => ApiError::not_found(
                "inventory_item_not_found",
                "The inventory item was not found",
            ),
            SetShoppingQuantityError::InvalidQuantity => ApiError::bad_request(
                "invalid_shopping_list_amount",
                "The amount is invalid for the shopping list",
            ),
            SetShoppingQuantityError::ItemArchived => {
                ApiError::conflict("inventory_item_archived", "The inventory items is archived")
            }
        }
    }
}

impl From<SetNoteError> for ApiError {
    fn from(value: SetNoteError) -> Self {
        match value {
            SetNoteError::HouseholdAccess(error) => error.into(),
            SetNoteError::Internal(_) => ApiError::internal_error(),
            SetNoteError::ItemNotFound => ApiError::not_found(
                "inventory_item_not_found",
                "The inventory item was not found",
            ),
            SetNoteError::NoteTooLong => ApiError::bad_request(
                "invalid_shopping_list_note_length",
                "The length is invalid for the shopping list note",
            ),
            SetNoteError::ItemArchived => {
                ApiError::conflict("inventory_item_archived", "The inventory items is archived")
            }
        }
    }
}

impl From<SetCheckedError> for ApiError {
    fn from(value: SetCheckedError) -> Self {
        match value {
            SetCheckedError::HouseholdAccess(error) => error.into(),
            SetCheckedError::Internal(_) => ApiError::internal_error(),
            SetCheckedError::ItemNotFound => ApiError::not_found(
                "inventory_item_not_found",
                "The inventory item was not found",
            ),
            SetCheckedError::ItemArchived => {
                ApiError::conflict("inventory_item_archived", "The inventory items is archived")
            }
        }
    }
}

impl From<DismissShoppingItemError> for ApiError {
    fn from(value: DismissShoppingItemError) -> Self {
        match value {
            DismissShoppingItemError::HouseholdAccess(error) => error.into(),
            DismissShoppingItemError::Internal(_) => ApiError::internal_error(),
            DismissShoppingItemError::ItemNotFound => ApiError::not_found(
                "inventory_item_not_found",
                "The inventory item was not found",
            ),
            DismissShoppingItemError::ItemArchived => {
                ApiError::conflict("inventory_item_archived", "The inventory items is archived")
            }
        }
    }
}

impl From<CreateCustomShoppingEntryError> for ApiError {
    fn from(value: CreateCustomShoppingEntryError) -> Self {
        match value {
            CreateCustomShoppingEntryError::HouseholdAccess(error) => error.into(),
            CreateCustomShoppingEntryError::Internal(_) => ApiError::internal_error(),
            CreateCustomShoppingEntryError::InvalidQuantity => ApiError::bad_request(
                "invalid_shopping_list_amount",
                "The amount is invalid for the shopping list",
            ),
            CreateCustomShoppingEntryError::InvalidNote => ApiError::bad_request(
                "invalid_shopping_list_note_length",
                "The length is invalid for the shopping list note",
            ),
            CreateCustomShoppingEntryError::InvalidPriority => ApiError::bad_request(
                "invalid_shopping_list_priority",
                "The priority for the shopping list is invalid",
            ),
            CreateCustomShoppingEntryError::InvalidTitle => ApiError::bad_request(
                "invalid_shopping_list_title",
                "The title for the shopping list is invalid",
            ),
        }
    }
}

impl From<UpdateCustomShoppingEntryError> for ApiError {
    fn from(value: UpdateCustomShoppingEntryError) -> Self {
        match value {
            UpdateCustomShoppingEntryError::HouseholdAccess(error) => error.into(),
            UpdateCustomShoppingEntryError::Internal(_) => ApiError::internal_error(),
            UpdateCustomShoppingEntryError::InvalidQuantity => ApiError::bad_request(
                "invalid_shopping_list_amount",
                "The amount is invalid for the shopping list",
            ),
            UpdateCustomShoppingEntryError::InvalidNote => ApiError::bad_request(
                "invalid_shopping_list_note_length",
                "The length is invalid for the shopping list note",
            ),
            UpdateCustomShoppingEntryError::InvalidPriority => ApiError::bad_request(
                "invalid_shopping_list_priority",
                "The priority for the shopping list is invalid",
            ),
            UpdateCustomShoppingEntryError::InvalidTitle => ApiError::bad_request(
                "invalid_shopping_list_title",
                "The title for the shopping list is invalid",
            ),
            UpdateCustomShoppingEntryError::EntryNotFound => ApiError::not_found(
                "custom_shopping_list_entry_not_found",
                "The custom shopping list entry was not found",
            ),
        }
    }
}

impl From<SetCustomShoppingEntryCheckedError> for ApiError {
    fn from(value: SetCustomShoppingEntryCheckedError) -> Self {
        match value {
            SetCustomShoppingEntryCheckedError::HouseholdAccess(error) => error.into(),
            SetCustomShoppingEntryCheckedError::Internal(_) => ApiError::internal_error(),
            SetCustomShoppingEntryCheckedError::EntryNotFound => ApiError::not_found(
                "custom_shopping_list_entry_not_found",
                "The custom shopping list entry was not found",
            ),
        }
    }
}

impl From<DeleteCustomShoppingEntryError> for ApiError {
    fn from(value: DeleteCustomShoppingEntryError) -> Self {
        match value {
            DeleteCustomShoppingEntryError::HouseholdAccess(error) => error.into(),
            DeleteCustomShoppingEntryError::Internal(_) => ApiError::internal_error(),
            DeleteCustomShoppingEntryError::EntryNotFound => ApiError::not_found(
                "custom_shopping_list_entry_not_found",
                "The custom shopping list entry was not found",
            ),
        }
    }
}
