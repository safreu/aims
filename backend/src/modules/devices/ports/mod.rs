mod device_repository;
pub use device_repository::{DeviceRepository, DeviceRepositoryError};

mod device_credential_repository;
pub use device_credential_repository::{
    DeviceCredentialRepository, DeviceCredentialRepositoryError,
};
