use crate::{
    docker::{ContainerRuntime, DockerComposeError},
    health::{HealthCheckError, HealthChecker},
    installation::{Installation, InstallationError},
    progress::ProgressReporter,
    version::Version,
};

pub struct Updater<R, H, P>
where
    R: ContainerRuntime,
    H: HealthChecker,
    P: ProgressReporter,
{
    installation: Installation,
    runtime: R,
    health_checker: H,
    progress: P,
}

impl<R, H, P> Updater<R, H, P>
where
    R: ContainerRuntime,
    H: HealthChecker,
    P: ProgressReporter,
{
    pub fn new(installation: Installation, runtime: R, health_checker: H, progress: P) -> Self {
        Self {
            installation,
            runtime,
            health_checker,
            progress,
        }
    }

    pub fn update(&self, version: &Version) -> Result<(), UpdateError> {
        const TOTAL_STEPS: usize = 3;

        let previous_version = self.installation.current_version()?;

        self.progress
            .header(&format!("Aims updater\n{previous_version} -> {version}"));

        if version == &previous_version {
            return Err(UpdateError::AlreadyInstalled(version.clone()));
        }

        if version < &previous_version {
            return Err(UpdateError::DowngradeNotAllowed {
                current: previous_version,
                requested: version.clone(),
            });
        }

        self.progress
            .step(1, TOTAL_STEPS, &format!("Downloading Aims {version}"));

        self.runtime.pull(&self.installation, version)?;

        self.progress.detail("Docker images downloaded");

        self.progress.step(2, TOTAL_STEPS, "Applying update");

        self.installation.set_version(version)?;

        if let Err(error) = self.runtime.apply(&self.installation) {
            return Err(self.recover(&previous_version, error.into()));
        }

        self.progress.detail("Updated services started");

        self.progress.step(3, TOTAL_STEPS, "Checking health");

        if let Err(error) = self.health_checker.wait_until_healthy(&self.installation) {
            return Err(self.recover(&previous_version, error.into()));
        }

        self.progress.detail("Aims is healthy");

        self.progress.success(&format!(
            "Aims updated successfully\n{previous_version} -> {version}"
        ));

        Ok(())
    }

    fn restore_version(&self, version: &Version) -> Result<(), UpdateError> {
        self.installation.set_version(version)?;
        self.runtime.apply(&self.installation)?;

        Ok(())
    }

    fn recover(&self, previous_version: &Version, original_error: UpdateError) -> UpdateError {
        self.progress
            .phase("Recovery", &format!("Restoring Aims {previous_version}"));
        match self.restore_version(previous_version) {
            Ok(()) => original_error,
            Err(recovery_error) => {
                self.progress.detail("Failed to restore previous version");
                UpdateError::RecoveryFailed {
                    update: Box::new(original_error),
                    recovery: Box::new(recovery_error),
                }
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateError {
    #[error("Aims version {0} is already installed")]
    AlreadyInstalled(Version),
    #[error(
        "downgrade is not allowed: installed version is {current}, request version is {requested}"
    )]
    DowngradeNotAllowed {
        current: Version,
        requested: Version,
    },
    #[error("Docker Compose operation failed")]
    Docker(#[from] DockerComposeError),
    #[error("failed to update installation state")]
    Installation(#[from] InstallationError),
    #[error("health check failed")]
    HealthCheck(#[from] HealthCheckError),
    #[error("update failed and restoring the previous version also failed")]
    RecoveryFailed {
        update: Box<UpdateError>,
        recovery: Box<UpdateError>,
    },
}

#[cfg(test)]
mod tests {

    use crate::docker::ServiceStatus;

    use super::*;

    use std::{cell::RefCell, rc::Rc};

    fn docker_failure() -> DockerComposeError {
        DockerComposeError::CommandFailed {
            exit_code: Some(1),
            stdout: "test stdout".to_string(),
            stderr: "test failure".to_string(),
        }
    }

    struct TestProgressReporter;

    impl ProgressReporter for TestProgressReporter {
        fn header(&self, _message: &str) {}
        fn step(&self, _current: usize, _total: usize, _message: &str) {}
        fn detail(&self, _message: &str) {}
        fn phase(&self, _name: &str, _message: &str) {}
        fn success(&self, _message: &str) {}
    }

    fn test_updater<R, H>(
        installation: Installation,
        runtime: R,
        health_checker: H,
    ) -> Updater<R, H, TestProgressReporter>
    where
        R: ContainerRuntime,
        H: HealthChecker,
    {
        Updater::new(installation, runtime, health_checker, TestProgressReporter)
    }

    struct FakeRuntime {
        calls: Rc<RefCell<Vec<String>>>,
        pull_result: Result<(), DockerComposeError>,
        apply_results: Rc<RefCell<Vec<Result<(), DockerComposeError>>>>,
    }

    impl FakeRuntime {
        fn succeeding(calls: Rc<RefCell<Vec<String>>>) -> Self {
            Self {
                calls,
                pull_result: Ok(()),
                apply_results: Rc::new(RefCell::new(vec![Ok(())])),
            }
        }

        fn pull_failing(calls: Rc<RefCell<Vec<String>>>) -> Self {
            Self {
                calls,
                pull_result: Err(docker_failure()),
                apply_results: Rc::new(RefCell::new(Vec::new())),
            }
        }

        fn apply_failing_then_recovering(calls: Rc<RefCell<Vec<String>>>) -> Self {
            Self {
                calls,
                pull_result: Ok(()),
                apply_results: Rc::new(RefCell::new(vec![Err(docker_failure()), Ok(())])),
            }
        }
    }

    impl ContainerRuntime for FakeRuntime {
        fn pull(
            &self,
            _installation: &Installation,
            version: &Version,
        ) -> Result<(), DockerComposeError> {
            self.calls.borrow_mut().push(format!("pull:{version}"));

            match &self.pull_result {
                Ok(()) => Ok(()),
                Err(_) => Err(docker_failure()),
            }
        }

        fn apply(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
            self.calls.borrow_mut().push("apply".to_string());

            self.apply_results.borrow_mut().remove(0)
        }

        fn start(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
            panic!("start must not be called by updater");
        }

        fn stop(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
            panic!("stop must not be called by updater");
        }

        fn down(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
            panic!("down must not be called by updater");
        }

        fn service_statuses(
            &self,
            _installation: &Installation,
        ) -> Result<Vec<ServiceStatus>, DockerComposeError> {
            Ok(Vec::new())
        }
    }

    struct FakeHealthChecker {
        calls: Rc<RefCell<Vec<String>>>,
        result: Result<(), HealthCheckError>,
    }

    impl FakeHealthChecker {
        fn succeeding(calls: Rc<RefCell<Vec<String>>>) -> Self {
            Self {
                calls,
                result: Ok(()),
            }
        }

        fn failing(calls: Rc<RefCell<Vec<String>>>) -> Self {
            Self {
                calls,
                result: Err(HealthCheckError::TimedOut),
            }
        }
    }

    impl HealthChecker for FakeHealthChecker {
        fn wait_until_healthy(&self, _installation: &Installation) -> Result<(), HealthCheckError> {
            self.calls.borrow_mut().push("health".to_string());

            match &self.result {
                Ok(()) => Ok(()),
                Err(_) => Err(HealthCheckError::TimedOut),
            }
        }

        fn is_healthy(&self, _installation: &Installation) -> Result<bool, HealthCheckError> {
            Ok(true)
        }
    }

    fn create_installation(version: &str) -> (tempfile::TempDir, Installation) {
        let temp_dir = tempfile::tempdir().unwrap();

        std::fs::write(
            temp_dir.path().join(".env.prod"),
            format!("AIMS_VERSION={version}\n"),
        )
        .unwrap();

        let installation = Installation::new(temp_dir.path(), "http://127.0.0.1:8080/api/health");

        (temp_dir, installation)
    }

    #[test]
    fn newer_version_can_be_installed() {
        let (_temp_dir, installation) = create_installation("0.1.0");

        let calls = Rc::new(RefCell::new(Vec::new()));

        let updater = test_updater(
            installation.clone(),
            FakeRuntime::succeeding(calls.clone()),
            FakeHealthChecker::succeeding(calls.clone()),
        );

        let version = Version::parse("0.2.0").unwrap();

        let result = updater.update(&version);

        assert!(result.is_ok());

        assert_eq!(installation.current_version().unwrap(), version);
    }

    #[test]
    fn already_installed_version_is_rejected() {
        let (_temp_dir, installation) = create_installation("0.1.0");

        let calls = Rc::new(RefCell::new(Vec::new()));

        let updater = test_updater(
            installation.clone(),
            FakeRuntime::succeeding(calls.clone()),
            FakeHealthChecker::succeeding(calls.clone()),
        );

        let version = Version::parse("0.1.0").unwrap();

        let result = updater.update(&version);

        assert!(matches!(result, Err(UpdateError::AlreadyInstalled(_))));
    }

    #[test]
    fn downgrade_is_rejected() {
        let (_temp_dir, installation) = create_installation("0.2.0");

        let calls = Rc::new(RefCell::new(Vec::new()));

        let updater = test_updater(
            installation.clone(),
            FakeRuntime::succeeding(calls.clone()),
            FakeHealthChecker::succeeding(calls.clone()),
        );

        let version = Version::parse("0.1.0").unwrap();

        let result = updater.update(&version);

        assert!(matches!(
            result,
            Err(UpdateError::DowngradeNotAllowed { .. })
        ));
    }

    #[test]
    fn update_runs_steps_in_order() {
        let (_temp_dir, installation) = create_installation("0.1.0");

        let calls = Rc::new(RefCell::new(Vec::new()));

        let runtime = FakeRuntime::succeeding(calls.clone());

        let health_checker = FakeHealthChecker {
            calls: calls.clone(),
            result: Ok(()),
        };

        let updater = test_updater(installation.clone(), runtime, health_checker);

        let version = Version::parse("0.2.0").unwrap();

        updater.update(&version).unwrap();

        assert_eq!(
            calls.borrow().as_slice(),
            ["pull:0.2.0", "apply", "health",]
        );
    }

    #[test]
    fn pull_failure_keeps_previous_version() {
        let (_temp_dir, installation) = create_installation("0.1.0");

        let calls = Rc::new(RefCell::new(Vec::new()));

        let updater = test_updater(
            installation.clone(),
            FakeRuntime::pull_failing(calls.clone()),
            FakeHealthChecker::succeeding(calls.clone()),
        );

        let version = Version::parse("0.2.0").unwrap();

        let result = updater.update(&version);

        assert!(matches!(
            result,
            Err(UpdateError::Docker(
                DockerComposeError::CommandFailed { .. }
            ))
        ));

        assert_eq!(
            installation.current_version().unwrap(),
            Version::parse("0.1.0").unwrap()
        );

        assert_eq!(calls.borrow().as_slice(), ["pull:0.2.0"]);
    }

    #[test]
    fn apply_failure_restores_previous_version() {
        let (_temp_dir, installation) = create_installation("0.1.0");

        let calls = Rc::new(RefCell::new(Vec::new()));

        let updater = test_updater(
            installation.clone(),
            FakeRuntime::apply_failing_then_recovering(calls.clone()),
            FakeHealthChecker::succeeding(calls.clone()),
        );

        let version = Version::parse("0.2.0").unwrap();

        let result = updater.update(&version);

        assert!(matches!(
            result,
            Err(UpdateError::Docker(
                DockerComposeError::CommandFailed { .. }
            ))
        ));

        assert_eq!(
            installation.current_version().unwrap(),
            Version::parse("0.1.0").unwrap()
        );

        assert_eq!(calls.borrow().as_slice(), ["pull:0.2.0", "apply", "apply",]);
    }

    #[test]
    fn health_failure_restores_previous_version() {
        let (_temp_dir, installation) = create_installation("0.1.0");

        let calls = Rc::new(RefCell::new(Vec::new()));

        let runtime = FakeRuntime {
            calls: calls.clone(),
            pull_result: Ok(()),
            apply_results: Rc::new(RefCell::new(vec![Ok(()), Ok(())])),
        };

        let updater = test_updater(
            installation.clone(),
            runtime,
            FakeHealthChecker::failing(calls.clone()),
        );

        let version = Version::parse("0.2.0").unwrap();

        let result = updater.update(&version);

        assert!(matches!(
            result,
            Err(UpdateError::HealthCheck(HealthCheckError::TimedOut))
        ));

        assert_eq!(
            installation.current_version().unwrap(),
            Version::parse("0.1.0").unwrap()
        );

        assert_eq!(
            calls.borrow().as_slice(),
            ["pull:0.2.0", "apply", "health", "apply",]
        );
    }

    #[test]
    fn recovery_failure_returns_recovery_failed_error() {
        let (_temp_dir, installation) = create_installation("0.1.0");

        let calls = Rc::new(RefCell::new(Vec::new()));

        let runtime = FakeRuntime {
            calls: calls.clone(),
            pull_result: Ok(()),
            apply_results: Rc::new(RefCell::new(vec![
                Err(docker_failure()),
                Err(docker_failure()),
            ])),
        };

        let updater = test_updater(
            installation.clone(),
            runtime,
            FakeHealthChecker::succeeding(calls.clone()),
        );

        let version = Version::parse("0.2.0").unwrap();

        let result = updater.update(&version);

        assert!(matches!(result, Err(UpdateError::RecoveryFailed { .. })));

        assert_eq!(
            installation.current_version().unwrap(),
            Version::parse("0.1.0").unwrap()
        );

        assert_eq!(calls.borrow().as_slice(), ["pull:0.2.0", "apply", "apply",]);
    }
}
