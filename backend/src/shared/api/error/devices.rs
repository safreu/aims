use crate::{
    modules::devices::application::{
        AuthenticateDeviceError, IssueDeviceCredentialError, ListDevicesError, RegisterDeviceError,
        RenameDeviceError, RevokeDeviceError, RotateDeviceCredentialError,
    },
    shared::api::ApiError,
};

impl From<RegisterDeviceError> for ApiError {
    fn from(value: RegisterDeviceError) -> Self {
        match value {
            RegisterDeviceError::HouseholdAccess(error) => error.into(),
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
            RenameDeviceError::HouseholdAccess(error) => error.into(),
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
            RevokeDeviceError::HouseholdAccess(error) => error.into(),
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
            ListDevicesError::HouseholdAccess(error) => error.into(),
            ListDevicesError::Internal(_) => ApiError::internal_error(),
        }
    }
}

impl From<AuthenticateDeviceError> for ApiError {
    fn from(value: AuthenticateDeviceError) -> Self {
        match value {
            AuthenticateDeviceError::InvalidCredentials => ApiError::unauthorized(
                "invalid_device_credentials",
                "Device credentials are invalid",
            ),
            AuthenticateDeviceError::Internal(_) => ApiError::internal_error(),
        }
    }
}

impl From<IssueDeviceCredentialError> for ApiError {
    fn from(value: IssueDeviceCredentialError) -> Self {
        match value {
            IssueDeviceCredentialError::HouseholdAccess(error) => error.into(),
            IssueDeviceCredentialError::DeviceNotFound => {
                ApiError::not_found("device_not_found", "The device was not found")
            }
            IssueDeviceCredentialError::TokenGenerationFailed
            | IssueDeviceCredentialError::Internal(_) => ApiError::internal_error(),
            IssueDeviceCredentialError::DeviceRevoked => {
                ApiError::conflict("device_revoked", "The device has been revoked")
            }
            IssueDeviceCredentialError::ActiveCredentialAlreadyExists => ApiError::conflict(
                "device_credential_already_exists",
                "The device already has an active credential",
            ),
        }
    }
}

impl From<RotateDeviceCredentialError> for ApiError {
    fn from(value: RotateDeviceCredentialError) -> Self {
        match value {
            RotateDeviceCredentialError::HouseholdAccess(error) => error.into(),
            RotateDeviceCredentialError::DeviceNotFound => {
                ApiError::not_found("device_not_found", "The device was not found")
            }
            RotateDeviceCredentialError::TokenGenerationFailed
            | RotateDeviceCredentialError::Internal(_) => ApiError::internal_error(),
            RotateDeviceCredentialError::DeviceRevoked => {
                ApiError::conflict("device_revoked", "The device has been revoked")
            }
            RotateDeviceCredentialError::CredentialNotFound => ApiError::conflict(
                "device_has_no_active_credential",
                "The device has no active credential to rotate",
            ),
        }
    }
}
