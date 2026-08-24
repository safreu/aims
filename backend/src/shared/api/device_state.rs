use std::sync::Arc;

use crate::modules::devices::application::{
    AuthenticateDeviceService, IssueDeviceCredentialService, ListDevicesService,
    RegisterDeviceService, RenameDeviceService, RevokeDeviceService, RotateDeviceCredentialService,
};

#[derive(Clone)]
pub struct DeviceState {
    pub register_device: Arc<RegisterDeviceService>,
    pub rename_device: Arc<RenameDeviceService>,
    pub revoke_device: Arc<RevokeDeviceService>,
    pub list_devices: Arc<ListDevicesService>,
    pub issue_device_credential: Arc<IssueDeviceCredentialService>,
    pub rotate_device_credential: Arc<RotateDeviceCredentialService>,
    pub authenticate_device: Arc<AuthenticateDeviceService>,
}
