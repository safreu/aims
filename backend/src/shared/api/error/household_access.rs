use crate::{modules::households::ports::HouseholdAccessError, shared::api::ApiError};

impl From<HouseholdAccessError> for ApiError {
    fn from(value: HouseholdAccessError) -> Self {
        match value {
            HouseholdAccessError::Forbidden => ApiError::forbidden(
                "household_access_forbidden",
                "You do not have permission to access this household",
            ),
            HouseholdAccessError::HouseholdNotFound => {
                ApiError::not_found("household_not_found", "The household was not found")
            }
            HouseholdAccessError::Internal(_) => ApiError::internal_error(),
        }
    }
}
