use crate::{
    modules::devices::application::{
        ListDevicesError, RegisterDeviceError, RenameDeviceError, RevokeDeviceError,
    },
    shared::api::ApiError,
};

impl From<RegisterDeviceError> for ApiError {
    fn from(value: RegisterDeviceError) -> Self {
        match value {
            RegisterDeviceError::Forbidden => ApiError::forbidden(
                "household_access_forbidden",
                "You do not have permissions to access this household",
            ),
            RegisterDeviceError::HouseholdNotFound => {
                ApiError::not_found("household_not_found", "The household was not found")
            }
            RegisterDeviceError::Internal(_) => ApiError::internal_error(),
            RegisterDeviceError::InvalidName => {
                ApiError::bad_request("invalid_device_name", "The device name is invalid")
            }
            RegisterDeviceError::InvalidKind => {
                ApiError::bad_request("invalid_device_kind", "The device kind is invalid")
            }
        }
    }
}

impl From<RenameDeviceError> for ApiError {
    fn from(value: RenameDeviceError) -> Self {
        match value {
            RenameDeviceError::Forbidden => ApiError::forbidden(
                "household_access_forbidden",
                "You do not have permissions to access this household",
            ),
            RenameDeviceError::HouseholdNotFound => {
                ApiError::not_found("household_not_found", "The household was not found")
            }
            RenameDeviceError::DeviceNotFound => {
                ApiError::not_found("device_not_found", "The device was not found")
            }
            RenameDeviceError::Internal(_) => ApiError::internal_error(),
            RenameDeviceError::InvalidName => {
                ApiError::bad_request("invalid_device_name", "The device name is invalid")
            }
            RenameDeviceError::DeviceRevoked => {
                ApiError::conflict("device_revoked", "Revoked devices cannot be modified")
            }
        }
    }
}

impl From<RevokeDeviceError> for ApiError {
    fn from(value: RevokeDeviceError) -> Self {
        match value {
            RevokeDeviceError::Forbidden => ApiError::forbidden(
                "household_access_forbidden",
                "You do not have permissions to access this household",
            ),
            RevokeDeviceError::HouseholdNotFound => {
                ApiError::not_found("household_not_found", "The household was not found")
            }
            RevokeDeviceError::DeviceNotFound => {
                ApiError::not_found("device_not_found", "The device was not found")
            }
            RevokeDeviceError::Internal(_) => ApiError::internal_error(),
            RevokeDeviceError::AlreadyRevoked => {
                ApiError::conflict("device_already_revoked", "The device is already revoked")
            }
        }
    }
}

impl From<ListDevicesError> for ApiError {
    fn from(value: ListDevicesError) -> Self {
        match value {
            ListDevicesError::Forbidden => ApiError::forbidden(
                "household_access_forbidden",
                "You do not have permissions to access this household",
            ),
            ListDevicesError::HouseholdNotFound => {
                ApiError::not_found("household_not_found", "The household was not found")
            }
            ListDevicesError::Internal(_) => ApiError::internal_error(),
        }
    }
}
