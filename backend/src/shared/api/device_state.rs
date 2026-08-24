use std::sync::Arc;

use crate::modules::devices::application::{
    ListDevicesService, RegisterDeviceService, RenameDeviceService, RevokeDeviceService,
};

#[derive(Clone)]
pub struct DeviceState {
    pub register_device: Arc<RegisterDeviceService>,
    pub rename_device: Arc<RenameDeviceService>,
    pub revoke_device: Arc<RevokeDeviceService>,
    pub list_devices: Arc<ListDevicesService>,
}
