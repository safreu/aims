use crate::{
    modules::inventory::application::{
        ArchiveInventoryItemError, CreateCategoryError, CreateInventoryItemError,
        DecreaseInventoryStockError, DeleteCategoryError, GetInventoryItemError,
        IncreaseInventoryStockError, ListCategoriesError, ListInventoryItemsError,
        ListInventoryStockHistoryError, RestoreInventoryItemError, SetInventoryStockError,
        UpdateCategoryError, UpdateInventoryItemError,
    },
    shared::api::ApiError,
};

impl From<CreateInventoryItemError> for ApiError {
    fn from(error: CreateInventoryItemError) -> Self {
        match error {
            CreateInventoryItemError::HouseholdAccess(error) => error.into(),
            CreateInventoryItemError::CategoryNotFound => {
                ApiError::not_found("category_not_found", "The category was not found")
            }
            CreateInventoryItemError::Internal(_) => ApiError::internal_error(),
            CreateInventoryItemError::InvalidName => ApiError::bad_request(
                "invalid_inventory_item_name",
                "The inventory item name is invalid",
            ),
            CreateInventoryItemError::ItemAlreadyExists => ApiError::conflict(
                "inventory_item_already_exists",
                "An active inventory item with this name already exists",
            ),
        }
    }
}

impl From<CreateCategoryError> for ApiError {
    fn from(error: CreateCategoryError) -> Self {
        match error {
            CreateCategoryError::HouseholdAccess(error) => error.into(),
            CreateCategoryError::CategoryAlreadyExists => ApiError::conflict(
                "category_already_exists",
                "A category with this name already exists",
            ),
            CreateCategoryError::InvalidName => {
                ApiError::bad_request("invalid_category_name", "The category name is invalid")
            }
            CreateCategoryError::Internal(_) => ApiError::internal_error(),
        }
    }
}

impl From<UpdateCategoryError> for ApiError {
    fn from(error: UpdateCategoryError) -> Self {
        match error {
            UpdateCategoryError::HouseholdAccess(error) => error.into(),
            UpdateCategoryError::CategoryNotFound => {
                ApiError::not_found("category_not_found", "No category was found")
            }
            UpdateCategoryError::InvalidName => {
                ApiError::bad_request("invalid_category_name", "The category name is invalid")
            }
            UpdateCategoryError::Internal(_) => ApiError::internal_error(),
        }
    }
}

impl From<ListCategoriesError> for ApiError {
    fn from(error: ListCategoriesError) -> Self {
        match error {
            ListCategoriesError::HouseholdAccess(error) => error.into(),
            ListCategoriesError::Internal(_) => ApiError::internal_error(),
        }
    }
}

impl From<DeleteCategoryError> for ApiError {
    fn from(error: DeleteCategoryError) -> Self {
        match error {
            DeleteCategoryError::HouseholdAccess(error) => error.into(),
            DeleteCategoryError::CategoryNotFound => {
                ApiError::not_found("category_not_found", "The category was not found")
            }
            DeleteCategoryError::Internal(_) => ApiError::internal_error(),
        }
    }
}

impl From<ListInventoryItemsError> for ApiError {
    fn from(error: ListInventoryItemsError) -> Self {
        match error {
            ListInventoryItemsError::HouseholdAccess(error) => error.into(),
            ListInventoryItemsError::Internal(_) => ApiError::internal_error(),
        }
    }
}

impl From<GetInventoryItemError> for ApiError {
    fn from(error: GetInventoryItemError) -> Self {
        match error {
            GetInventoryItemError::HouseholdAccess(error) => error.into(),
            GetInventoryItemError::ItemNotFound => ApiError::not_found(
                "inventory_item_not_found",
                "The inventory item was not found",
            ),
            GetInventoryItemError::Internal(_) => ApiError::internal_error(),
        }
    }
}

impl From<UpdateInventoryItemError> for ApiError {
    fn from(value: UpdateInventoryItemError) -> Self {
        match value {
            UpdateInventoryItemError::HouseholdAccess(error) => error.into(),
            UpdateInventoryItemError::InvalidName => {
                ApiError::bad_request("invalid_category_name", "The category name is invalid")
            }
            UpdateInventoryItemError::ItemNotFound => ApiError::not_found(
                "inventory_item_not_found",
                "The inventory item was not found",
            ),
            UpdateInventoryItemError::CategoryNotFound => {
                ApiError::not_found("category_not_found", "The category was not found")
            }
            UpdateInventoryItemError::ItemAlreadyExists => ApiError::conflict(
                "inventory_item_already_exists",
                "An active inventory item with this name already exists",
            ),
            UpdateInventoryItemError::InvalidPriority => {
                ApiError::bad_request("invalid_priority", "The priority is invalid")
            }
            UpdateInventoryItemError::NoChanges => ApiError::bad_request(
                "no_changes",
                "At least one field must be provided for an update",
            ),
            UpdateInventoryItemError::ItemArchived => ApiError::conflict(
                "item_archived",
                "Archived inventory items cannot be modified",
            ),
            UpdateInventoryItemError::Internal(_) => ApiError::internal_error(),
        }
    }
}

impl From<ArchiveInventoryItemError> for ApiError {
    fn from(error: ArchiveInventoryItemError) -> Self {
        match error {
            ArchiveInventoryItemError::ItemNotFound => ApiError::not_found(
                "inventory_item_not_found",
                "The inventory item was not found",
            ),
            ArchiveInventoryItemError::HouseholdAccess(error) => error.into(),
            ArchiveInventoryItemError::Internal(_) => ApiError::internal_error(),
            ArchiveInventoryItemError::AlreadyArchived => ApiError::conflict(
                "inventory_item_already_archived",
                "The inventory item is already archived",
            ),
        }
    }
}

impl From<RestoreInventoryItemError> for ApiError {
    fn from(value: RestoreInventoryItemError) -> Self {
        match value {
            RestoreInventoryItemError::HouseholdAccess(error) => error.into(),
            RestoreInventoryItemError::ItemNotFound => ApiError::not_found(
                "inventory_item_not_found",
                "The inventory item was not found",
            ),
            RestoreInventoryItemError::Internal(_) => ApiError::internal_error(),
            RestoreInventoryItemError::ItemAlreadyExists => ApiError::conflict(
                "inventory_item_already_exists",
                "An active inventory item with this name already exists",
            ),
            RestoreInventoryItemError::NotArchived => ApiError::conflict(
                "inventory_item_not_archived",
                "The inventory item is not archived",
            ),
        }
    }
}

impl From<IncreaseInventoryStockError> for ApiError {
    fn from(error: IncreaseInventoryStockError) -> Self {
        match error {
            IncreaseInventoryStockError::HouseholdAccess(error) => error.into(),
            IncreaseInventoryStockError::ItemNotFound => ApiError::not_found(
                "inventory_item_not_found",
                "The inventory item was not found",
            ),
            IncreaseInventoryStockError::Internal(_) => ApiError::internal_error(),
            IncreaseInventoryStockError::ItemArchived => ApiError::conflict(
                "item_archived",
                "Archived inventory items cannot be modified",
            ),
            IncreaseInventoryStockError::InvalidAmount => ApiError::bad_request(
                "invalid_amount",
                "Increase amount must be greater than zero",
            ),
            IncreaseInventoryStockError::StockOverflow => ApiError::conflict(
                "stock_overflow",
                "Inventory stock cannot be increased further",
            ),
        }
    }
}

impl From<DecreaseInventoryStockError> for ApiError {
    fn from(error: DecreaseInventoryStockError) -> Self {
        match error {
            DecreaseInventoryStockError::HouseholdAccess(error) => error.into(),
            DecreaseInventoryStockError::ItemNotFound => ApiError::not_found(
                "inventory_item_not_found",
                "The inventory item was not found",
            ),
            DecreaseInventoryStockError::Internal(_) => ApiError::internal_error(),
            DecreaseInventoryStockError::ItemArchived => ApiError::conflict(
                "item_archived",
                "Archived inventory items cannot be modified",
            ),
            DecreaseInventoryStockError::InvalidAmount => ApiError::bad_request(
                "invalid_amount",
                "Decrease amount must be greater than zero",
            ),
            DecreaseInventoryStockError::InsufficientStock => ApiError::conflict(
                "insufficient_stock",
                "Inventory stock is insufficient for this decrease",
            ),
        }
    }
}

impl From<SetInventoryStockError> for ApiError {
    fn from(error: SetInventoryStockError) -> Self {
        match error {
            SetInventoryStockError::HouseholdAccess(error) => error.into(),
            SetInventoryStockError::ItemNotFound => ApiError::not_found(
                "inventory_item_not_found",
                "The inventory item was not found",
            ),
            SetInventoryStockError::Internal(_) => ApiError::internal_error(),
            SetInventoryStockError::ItemArchived => ApiError::conflict(
                "item_archived",
                "Archived inventory items cannot be modified",
            ),
        }
    }
}

impl From<ListInventoryStockHistoryError> for ApiError {
    fn from(value: ListInventoryStockHistoryError) -> Self {
        match value {
            ListInventoryStockHistoryError::HouseholdAccess(error) => error.into(),
            ListInventoryStockHistoryError::Internal(_) => ApiError::internal_error(),
            ListInventoryStockHistoryError::ItemNotFound => ApiError::not_found(
                "inventory_item_not_found",
                "The inventory item was not found",
            ),
        }
    }
}
