use crate::{
    docker::{ContainerRuntime, DockerComposeError},
    health::{HealthCheckError, HealthChecker},
    installation::{EnvironmentSnapshot, Installation, InstallationError},
    migration::{self, MigrationError},
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

        let environment_snapshot = self.installation.environment_snapshot()?;

        self.progress
            .step(1, TOTAL_STEPS, &format!("Downloading Aims {version}"));

        self.runtime.pull(&self.installation, version)?;

        self.progress.detail("Docker images downloaded");

        self.progress.step(2, TOTAL_STEPS, "Applying update");

        if let Err(error) = self.apply_update(&previous_version, version) {
            return Err(self.recover(&environment_snapshot, error));
        }

        self.progress.detail("Updated services started");

        self.progress.step(3, TOTAL_STEPS, "Checking health");

        if let Err(error) = self.health_checker.wait_until_healthy(&self.installation) {
            return Err(self.recover(&environment_snapshot, error.into()));
        }

        self.progress.detail("Aims is healthy");

        self.progress.success(&format!(
            "Aims updated successfully\n{previous_version} -> {version}"
        ));

        Ok(())
    }

    fn recover(&self, snapshot: &EnvironmentSnapshot, original_error: UpdateError) -> UpdateError {
        self.progress
            .phase("Recovery", "Restoring previous Aims installation");

        if let Err(error) = self.installation.restore_environment(snapshot) {
            self.progress
                .detail("Failed to restore previous environment");

            return UpdateError::RecoveryFailed {
                update: Box::new(original_error),
                recovery: Box::new(error.into()),
            };
        }

        match self.runtime.apply(&self.installation) {
            Ok(()) => original_error,
            Err(error) => {
                self.progress.detail("Failed to restore previous version");
                UpdateError::RecoveryFailed {
                    update: Box::new(original_error),
                    recovery: Box::new(error.into()),
                }
            }
        }
    }

    fn apply_update(
        &self,
        previous_version: &Version,
        version: &Version,
    ) -> Result<(), UpdateError> {
        migration::migrate(&self.installation, previous_version, version)?;

        self.installation.set_version(version)?;

        self.runtime.apply(&self.installation)?;

        Ok(())
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
    #[error("installation migration failed")]
    Migration(#[from] MigrationError),
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_support::{
        self, FakeHealthChecker, FakeRuntime, TestProgressReporter, docker_failure,
    };

    fn test_updater(
        installation: Installation,
        runtime: FakeRuntime,
        health_checker: FakeHealthChecker,
    ) -> Updater<FakeRuntime, FakeHealthChecker, TestProgressReporter> {
        Updater::new(installation, runtime, health_checker, TestProgressReporter)
    }

    #[test]
    fn newer_version_can_be_installed() {
        let test_installation = test_support::installation("0.1.0");
        let installation = test_installation.installation.clone();

        let runtime = FakeRuntime::new();

        let updater = test_updater(installation.clone(), runtime, FakeHealthChecker::healthy());

        let version = Version::parse("0.2.0").unwrap();

        let result = updater.update(&version);

        assert!(result.is_ok());
        assert_eq!(installation.current_version().unwrap(), version);
    }

    #[test]
    fn already_installed_version_is_rejected() {
        let test_installation = test_support::installation("0.1.0");

        let runtime = FakeRuntime::new();

        let updater = test_updater(
            test_installation.installation.clone(),
            runtime,
            FakeHealthChecker::healthy(),
        );

        let version = Version::parse("0.1.0").unwrap();

        let result = updater.update(&version);

        assert!(matches!(result, Err(UpdateError::AlreadyInstalled(_))));
    }

    #[test]
    fn downgrade_is_rejected() {
        let test_installation = test_support::installation("0.2.0");

        let runtime = FakeRuntime::new();

        let updater = test_updater(
            test_installation.installation.clone(),
            runtime,
            FakeHealthChecker::healthy(),
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
        let test_installation = test_support::installation("0.1.0");

        let runtime = FakeRuntime::new();

        let updater = test_updater(
            test_installation.installation.clone(),
            runtime.clone(),
            FakeHealthChecker::healthy(),
        );

        let version = Version::parse("0.2.0").unwrap();

        updater.update(&version).unwrap();

        assert_eq!(runtime.calls(), ["pull:0.2.0", "apply"]);
    }

    #[test]
    fn pull_failure_keeps_previous_version() {
        let test_installation = test_support::installation("0.1.0");
        let installation = test_installation.installation.clone();

        let runtime = FakeRuntime::new();
        runtime.push_pull_result(Err(docker_failure()));

        let updater = test_updater(
            installation.clone(),
            runtime.clone(),
            FakeHealthChecker::healthy(),
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

        assert_eq!(runtime.calls(), ["pull:0.2.0"]);
    }

    #[test]
    fn apply_failure_restores_previous_version() {
        let test_installation = test_support::installation("0.1.0");
        let installation = test_installation.installation.clone();

        let runtime = FakeRuntime::new();
        runtime.push_apply_result(Err(docker_failure()));
        runtime.push_apply_result(Ok(()));

        let updater = test_updater(
            installation.clone(),
            runtime.clone(),
            FakeHealthChecker::healthy(),
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

        assert_eq!(runtime.calls(), ["pull:0.2.0", "apply", "apply"]);
    }

    #[test]
    fn health_failure_restores_previous_version() {
        let test_installation = test_support::installation("0.1.0");
        let installation = test_installation.installation.clone();

        let runtime = FakeRuntime::new();
        runtime.push_apply_result(Ok(()));
        runtime.push_apply_result(Ok(()));

        let updater = test_updater(
            installation.clone(),
            runtime.clone(),
            FakeHealthChecker::failing(),
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

        assert_eq!(runtime.calls(), ["pull:0.2.0", "apply", "apply"]);
    }

    #[test]
    fn recovery_failure_returns_recovery_failed_error() {
        let test_installation = test_support::installation("0.1.0");
        let installation = test_installation.installation.clone();

        let runtime = FakeRuntime::new();
        runtime.push_apply_result(Err(docker_failure()));
        runtime.push_apply_result(Err(docker_failure()));

        let updater = test_updater(
            installation.clone(),
            runtime.clone(),
            FakeHealthChecker::healthy(),
        );

        let version = Version::parse("0.2.0").unwrap();

        let result = updater.update(&version);

        assert!(matches!(result, Err(UpdateError::RecoveryFailed { .. })));

        assert_eq!(
            installation.current_version().unwrap(),
            Version::parse("0.1.0").unwrap()
        );

        assert_eq!(runtime.calls(), ["pull:0.2.0", "apply", "apply"]);
    }

    #[test]
    fn update_migrates_default_http_port() {
        let test_installation = test_support::installation_with_port("0.1.4", 8080);

        let installation = test_installation.installation.clone();

        let runtime = FakeRuntime::new();

        let updater = test_updater(installation.clone(), runtime, FakeHealthChecker::healthy());

        let version = Version::parse("0.1.5").unwrap();

        updater.update(&version).unwrap();

        assert_eq!(
            installation.environment_value("AIMS_HTTP_PORT").unwrap(),
            "80"
        );

        assert_eq!(
            installation.current_version().unwrap(),
            Version::parse("0.1.5").unwrap()
        );
    }

    #[test]
    fn failed_update_restores_http_port_migration() {
        let test_installation = test_support::installation_with_port("0.1.4", 8080);

        let installation = test_installation.installation.clone();

        let runtime = FakeRuntime::new();
        runtime.push_apply_result(Err(docker_failure()));
        runtime.push_apply_result(Ok(()));

        let updater = test_updater(
            installation.clone(),
            runtime.clone(),
            FakeHealthChecker::healthy(),
        );

        let version = Version::parse("0.1.5").unwrap();

        let result = updater.update(&version);

        assert!(result.is_err());

        assert_eq!(
            installation.current_version().unwrap(),
            Version::parse("0.1.4").unwrap()
        );

        assert_eq!(
            installation.environment_value("AIMS_HTTP_PORT").unwrap(),
            "8080"
        );

        assert_eq!(runtime.calls(), ["pull:0.1.5", "apply", "apply"]);
    }
}
