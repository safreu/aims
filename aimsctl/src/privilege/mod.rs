use std::ffi::OsString;

use crate::privilege::process::{PrivilegeChecker, PrivilegeEscalator};

mod process;
pub use process::{CoscaPrivilegeEscalator, SystemPrivilegeChecker};

pub fn current_arguments() -> Vec<OsString> {
    std::env::args_os().skip(1).collect()
}

pub fn ensure_root<C, E>(
    checker: &C,
    escalator: &E,
    arguments: &[OsString],
) -> Result<Elevation, PrivilegeError>
where
    C: PrivilegeChecker,
    E: PrivilegeEscalator,
{
    if checker.is_root() {
        return Ok(Elevation::AlreadyRoot);
    }

    let success = escalator.escalate(arguments)?;

    if success {
        Ok(Elevation::Reexecuted)
    } else {
        Err(PrivilegeError::EscalationFailed)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Elevation {
    AlreadyRoot,
    Reexecuted,
}

#[derive(Debug, thiserror::Error)]
pub enum PrivilegeError {
    #[error("failed to determine current aimsctl executable")]
    CurrentExecutable(#[source] std::io::Error),
    #[error("failed to execute elevated aimsctl")]
    Escalation(#[source] cosca::error::Error),
    #[error("elevated aimsctl exited unsuccessfully")]
    EscalationFailed,
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, ffi::OsString, rc::Rc};

    use super::*;

    struct FakePrivilegeChecker {
        root: bool,
    }

    impl PrivilegeChecker for FakePrivilegeChecker {
        fn is_root(&self) -> bool {
            self.root
        }
    }

    struct FakePrivilegeEscalator {
        calls: Rc<RefCell<Vec<Vec<OsString>>>>,
        succeeds: bool,
    }

    impl FakePrivilegeEscalator {
        fn succeeding() -> Self {
            Self {
                calls: Rc::new(RefCell::new(Vec::new())),
                succeeds: true,
            }
        }

        fn failing() -> Self {
            Self {
                calls: Rc::new(RefCell::new(Vec::new())),
                succeeds: false,
            }
        }

        fn calls(&self) -> Vec<Vec<OsString>> {
            self.calls.borrow().clone()
        }
    }

    impl PrivilegeEscalator for FakePrivilegeEscalator {
        fn escalate(&self, arguments: &[OsString]) -> Result<bool, PrivilegeError> {
            self.calls.borrow_mut().push(arguments.to_vec());

            Ok(self.succeeds)
        }
    }

    #[test]
    fn already_root_does_not_escalate() {
        let checker = FakePrivilegeChecker { root: true };
        let escalator = FakePrivilegeEscalator::succeeding();

        let arguments = vec![
            OsString::from("apply-init"),
            OsString::from("--config-path"),
            OsString::from("/tmp/aimsctl.toml"),
        ];

        let result = ensure_root(&checker, &escalator, &arguments).unwrap();

        assert_eq!(result, Elevation::AlreadyRoot);
        assert!(escalator.calls().is_empty());
    }

    #[test]
    fn non_root_executes_requested_arguments_with_elevated_privileges() {
        let checker = FakePrivilegeChecker { root: false };
        let escalator = FakePrivilegeEscalator::succeeding();

        let arguments = vec![
            OsString::from("apply-init"),
            OsString::from("--config-path"),
            OsString::from("/tmp/aimsctl.toml"),
        ];

        let result = ensure_root(&checker, &escalator, &arguments).unwrap();

        assert_eq!(result, Elevation::Reexecuted);
        assert_eq!(escalator.calls(), vec![arguments]);
    }

    #[test]
    fn failed_elevation_is_returned() {
        let checker = FakePrivilegeChecker { root: false };
        let escalator = FakePrivilegeEscalator::failing();

        let arguments = vec![
            OsString::from("apply-init"),
            OsString::from("--config-path"),
            OsString::from("/tmp/aimsctl.toml"),
        ];

        let result = ensure_root(&checker, &escalator, &arguments);

        assert!(matches!(result, Err(PrivilegeError::EscalationFailed)));

        assert_eq!(escalator.calls(), vec![arguments]);
    }
}
