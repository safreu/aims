use cosca::{Auth, Backend};
use std::ffi::OsString;

use crate::privilege::PrivilegeError;

pub(crate) trait PrivilegeChecker {
    fn is_root(&self) -> bool;
}

pub(crate) trait PrivilegeEscalator {
    fn escalate(&self, arguments: &[OsString]) -> Result<bool, PrivilegeError>;
}

pub struct SystemPrivilegeChecker;

impl PrivilegeChecker for SystemPrivilegeChecker {
    fn is_root(&self) -> bool {
        cosca::elevation::is_elevated()
    }
}

pub struct CoscaPrivilegeEscalator;

impl PrivilegeEscalator for CoscaPrivilegeEscalator {
    fn escalate(&self, arguments: &[OsString]) -> Result<bool, PrivilegeError> {
        let current_exe = std::env::current_exe().map_err(PrivilegeError::CurrentExecutable)?;

        let mut argv = Vec::with_capacity(arguments.len() + 1);

        argv.push(current_exe.into_os_string());
        argv.extend_from_slice(arguments);

        let status = cosca::run(argv)
            .elevate()
            .elevation_backend(Backend::Auto)
            .elevation_auth(Auth::Interactive)
            .status()
            .map_err(PrivilegeError::Escalation)?;

        Ok(status.success())
    }
}
