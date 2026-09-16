use rustix::process::geteuid;

use crate::privilege::sudo::{PrivilegeChecker, PrivilegeEscalator};

mod sudo;
pub use sudo::{SudoEscalator, SystemPrivilegeChecker};

pub fn is_root() -> bool {
    geteuid().is_root()
}

pub fn ensure_root<C, E>(checker: &C, escalator: &E) -> Result<Elevation, PrivilegeError>
where
    C: PrivilegeChecker,
    E: PrivilegeEscalator,
{
    if checker.is_root() {
        return Ok(Elevation::AlreadyRoot);
    }

    let status = escalator.escalate()?;

    if status.success() {
        Ok(Elevation::Reexecuted)
    } else {
        Err(PrivilegeError::EscalationFailed(status))
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

    #[error("failed to start sudo")]
    EscalationFailedToStart(#[source] std::io::Error),

    #[error("elevated aimsctl exited unsuccessfully with status {0}")]
    EscalationFailed(std::process::ExitStatus),
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, os::unix::process::ExitStatusExt, process::ExitStatus, rc::Rc};

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
        calls: Rc<RefCell<usize>>,
        status: ExitStatus,
    }

    impl FakePrivilegeEscalator {
        fn succeeding() -> Self {
            Self {
                calls: Rc::new(RefCell::new(0)),
                status: ExitStatus::from_raw(0),
            }
        }

        fn failing() -> Self {
            Self {
                calls: Rc::new(RefCell::new(0)),
                status: ExitStatus::from_raw(1 << 8),
            }
        }

        fn call_count(&self) -> usize {
            *self.calls.borrow()
        }
    }

    impl PrivilegeEscalator for FakePrivilegeEscalator {
        fn escalate(&self) -> Result<ExitStatus, PrivilegeError> {
            *self.calls.borrow_mut() += 1;

            Ok(self.status)
        }
    }

    #[test]
    fn root_detection_matches_effective_user_id() {
        assert_eq!(is_root(), geteuid().is_root());
    }

    #[test]
    fn already_root_does_not_escalate() {
        let checker = FakePrivilegeChecker { root: true };
        let escalator = FakePrivilegeEscalator::succeeding();

        let result = ensure_root(&checker, &escalator).unwrap();

        assert_eq!(result, Elevation::AlreadyRoot);
        assert_eq!(escalator.call_count(), 0);
    }

    #[test]
    fn non_root_reexecutes_with_elevated_privileges() {
        let checker = FakePrivilegeChecker { root: false };
        let escalator = FakePrivilegeEscalator::succeeding();

        let result = ensure_root(&checker, &escalator).unwrap();

        assert_eq!(result, Elevation::Reexecuted);
        assert_eq!(escalator.call_count(), 1);
    }

    #[test]
    fn failed_elevation_is_returned() {
        let checker = FakePrivilegeChecker { root: false };
        let escalator = FakePrivilegeEscalator::failing();

        let result = ensure_root(&checker, &escalator);

        assert!(matches!(result, Err(PrivilegeError::EscalationFailed(_))));

        assert_eq!(escalator.call_count(), 1);
    }
}
