use std::process::{Command, ExitStatus};

use crate::privilege::{PrivilegeError, is_root};

pub(crate) trait PrivilegeEscalator {
    fn escalate(&self) -> Result<ExitStatus, PrivilegeError>;
}

pub struct SudoEscalator;

impl PrivilegeEscalator for SudoEscalator {
    fn escalate(&self) -> Result<ExitStatus, PrivilegeError> {
        let current_exe = std::env::current_exe().map_err(PrivilegeError::CurrentExecutable)?;

        let arguments = std::env::args_os().skip(1);

        Command::new("sudo")
            .arg(current_exe)
            .args(arguments)
            .status()
            .map_err(PrivilegeError::EscalationFailedToStart)
    }
}

pub(crate) trait PrivilegeChecker {
    fn is_root(&self) -> bool;
}

pub struct SystemPrivilegeChecker;

impl PrivilegeChecker for SystemPrivilegeChecker {
    fn is_root(&self) -> bool {
        is_root()
    }
}
