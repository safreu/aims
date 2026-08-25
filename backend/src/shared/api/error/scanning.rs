use crate::{
    modules::scanning::application::{
        CreateQrActionError, ExecuteQrActionError, ListQrActionsError, RevokeQrActionError,
    },
    shared::api::ApiError,
};

impl From<CreateQrActionError> for ApiError {
    fn from(value: CreateQrActionError) -> Self {
        match value {
            CreateQrActionError::ItemNotFound => ApiError::not_found(
                "inventory_item_not_found",
                "The inventory item was not found",
            ),
            CreateQrActionError::ItemArchived => {
                ApiError::conflict("inventory_item_archived", "The inventory items is archived")
            }
            CreateQrActionError::InvalidAmount => ApiError::bad_request(
                "invalid_qr_action_amount",
                "The QR action amount is invalid",
            ),
            CreateQrActionError::InvalidKind => {
                ApiError::bad_request("invalid_qr_action_kind", "The QR action kind is invalid")
            }
            CreateQrActionError::HouseholdAccess(error) => error.into(),
            CreateQrActionError::Internal(_) => ApiError::internal_error(),
        }
    }
}

impl From<ListQrActionsError> for ApiError {
    fn from(value: ListQrActionsError) -> Self {
        match value {
            ListQrActionsError::HouseholdAccess(error) => error.into(),
            ListQrActionsError::Internal(_) => ApiError::internal_error(),
        }
    }
}

impl From<RevokeQrActionError> for ApiError {
    fn from(value: RevokeQrActionError) -> Self {
        match value {
            RevokeQrActionError::QrActionNotFound => {
                ApiError::not_found("qr_action_not_found", "The QR action was not found")
            }
            RevokeQrActionError::AlreadyRevoked => ApiError::conflict(
                "qr_action_already_revoked",
                "The QR action has already been revoked",
            ),
            RevokeQrActionError::HouseholdAccess(error) => error.into(),
            RevokeQrActionError::Internal(_) => ApiError::internal_error(),
        }
    }
}

impl From<ExecuteQrActionError> for ApiError {
    fn from(value: ExecuteQrActionError) -> Self {
        match value {
            ExecuteQrActionError::InsufficientStock => ApiError::conflict(
                "insufficient_stock",
                "Inventory stock is insufficient for this decrease",
            ),
            ExecuteQrActionError::ItemArchived => ApiError::conflict(
                "item_archived",
                "Archived inventory items cannot be modified",
            ),
            ExecuteQrActionError::ItemNotFound => ApiError::not_found(
                "inventory_item_not_found",
                "The inventory item was not found",
            ),
            ExecuteQrActionError::QrActionNotFound => {
                ApiError::not_found("qr_action_not_found", "The QR action was not found")
            }
            ExecuteQrActionError::QrActionRevoked => {
                ApiError::conflict("qr_action_revoked", "The QR action has been revoked")
            }
            ExecuteQrActionError::StockOverflow => ApiError::conflict(
                "stock_overflow",
                "Inventory stock cannot be increased further",
            ),
            ExecuteQrActionError::Internal(_) => ApiError::internal_error(),
        }
    }
}
