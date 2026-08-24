use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};

use crate::{
    modules::{
        devices::{
            application::AuthenticateDeviceCommand,
            domain::{DeviceId, DeviceToken},
        },
        households::domain::HouseholdId,
    },
    shared::api::{ApiError, AppState},
};

#[derive(Debug, Clone, Copy)]
pub struct CurrentDevice {
    device_id: DeviceId,
    household_id: HouseholdId,
}

impl CurrentDevice {
    pub fn device_id(&self) -> DeviceId {
        self.device_id
    }

    pub fn household_id(&self) -> HouseholdId {
        self.household_id
    }
}

impl FromRequestParts<AppState> for CurrentDevice {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let authorization = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| {
                ApiError::unauthorized(
                    "device_authentication_required",
                    "Device authentication is required",
                )
            })?;

        let token = authorization.strip_prefix("Bearer ").ok_or_else(|| {
            ApiError::unauthorized(
                "invalid_device_credentials",
                "Device credentials are invalid",
            )
        })?;

        let token = DeviceToken::from_string(token.to_owned()).map_err(|_| {
            ApiError::unauthorized(
                "invalid_device_credentials",
                "Device credentials are invalid",
            )
        })?;

        let authenticated = state
            .device
            .authenticate_device
            .execute(AuthenticateDeviceCommand { token })
            .await
            .map_err(ApiError::from)?;

        Ok(Self {
            device_id: authenticated.device_id,
            household_id: authenticated.household_id,
        })
    }
}
