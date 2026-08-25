use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::modules::devices::domain::Device;

#[derive(Debug, Deserialize)]
pub struct RegisterDeviceRequest {
    pub name: String,
    pub kind: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterDeviceResponse {
    pub id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct DeviceResponse {
    pub id: Uuid,
    pub name: String,
    pub kind: String,
}

impl From<Device> for DeviceResponse {
    fn from(value: Device) -> Self {
        Self {
            id: value.id().into_uuid(),
            name: value.name().as_str().to_owned(),
            kind: value.kind().as_str().to_owned(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RenameDeviceRequest {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct DeviceCredentialResponse {
    pub token: String,
}
