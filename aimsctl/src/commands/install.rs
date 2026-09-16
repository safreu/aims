use crate::{
    installation::{
        EnvironmentConfig, Installation, InstallationError, Version, build_environment,
        generate_database_password,
    },
    progress::ProgressReporter,
    runtime::{
        docker::{ContainerRuntime, DockerComposeError, RuntimeAvailabilityError},
        health::{HealthCheckError, HealthChecker},
    },
};

pub struct Install<D, H, P>
where
    D: ContainerRuntime,
    H: HealthChecker,
    P: ProgressReporter,
{
    installation: Installation,
    docker: D,
    health_checker: H,
    progress: P,
    compose_project: String,
    http_port: u16,
    postgres_volume: String,
}

impl<D, H, P> Install<D, H, P>
where
    D: ContainerRuntime,
    H: HealthChecker,
    P: ProgressReporter,
{
    pub fn new(
        installation: Installation,
        docker: D,
        health_checker: H,
        progress: P,
        compose_project: String,
        http_port: u16,
        postgres_volume: String,
    ) -> Self {
        Self {
            installation,
            docker,
            health_checker,
            progress,
            compose_project,
            http_port,
            postgres_volume,
        }
    }

    pub fn install(&self, version: &Version) -> Result<(), InstallError> {
        const TOTAL_STEPS: usize = 5;

        self.docker.check_available()?;

        self.progress.header(&format!("Aims {version} installer"));

        self.progress.step(1, TOTAL_STEPS, "Preparing installation");

        self.installation.create()?;
        self.installation.write_compose_file()?;

        self.progress.detail("Installation directory created");

        self.progress.step(2, TOTAL_STEPS, "Writing configuration");

        let database_password = generate_database_password();

        let environment_config = EnvironmentConfig {
            version,
            database_password: &database_password,
            compose_project: &self.compose_project,
            http_port: self.http_port,
            postgres_volume: &self.postgres_volume,
        };

        let environment = build_environment(&environment_config);

        self.installation.write_environment(&environment)?;

        self.progress.detail("Configuration written");

        self.progress
            .step(3, TOTAL_STEPS, &format!("Downloading Aims {version}"));

        if let Err(error) = self.docker.pull(&self.installation, version) {
            let installation_error = InstallError::Docker(error);
            return Err(self.cleanup_after_failure(installation_error, self.cleanup_files()));
        }

        self.progress.detail("Docker images downloaded");

        self.progress.step(4, TOTAL_STEPS, "Starting services");

        if let Err(error) = self.docker.apply(&self.installation) {
            let installation_error = InstallError::Docker(error);
            return Err(self
                .cleanup_after_failure(installation_error, self.cleanup_running_installations()));
        }

        self.progress.detail("Services started");

        self.progress.step(5, TOTAL_STEPS, "Checking health");

        if let Err(error) = self.health_checker.wait_until_healthy(&self.installation) {
            let installation_error = InstallError::Health(error);

            return Err(self
                .cleanup_after_failure(installation_error, self.cleanup_running_installations()));
        }

        self.progress.detail("Aims is healthy");

        self.progress
            .success(&format!("Aims {version} installed successfully"));

        Ok(())
    }

    fn cleanup_files(&self) -> Result<(), InstallError> {
        self.installation.remove()?;
        Ok(())
    }

    fn cleanup_running_installations(&self) -> Result<(), InstallError> {
        self.docker.down(&self.installation)?;
        self.installation.remove()?;
        Ok(())
    }

    fn cleanup_after_failure(
        &self,
        installation_error: InstallError,
        cleanup_result: Result<(), InstallError>,
    ) -> InstallError {
        match cleanup_result {
            Ok(()) => installation_error,
            Err(cleanup_error) => InstallError::CleanupFailed {
                installation_error: Box::new(installation_error),
                cleanup_error: Box::new(cleanup_error),
            },
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InstallError {
    #[error(transparent)]
    Installation(#[from] InstallationError),
    #[error(transparent)]
    Docker(#[from] DockerComposeError),
    #[error(transparent)]
    Health(#[from] HealthCheckError),
    #[error("installation failed: {installation_error}; cleanup also failed: {cleanup_error}")]
    CleanupFailed {
        installation_error: Box<InstallError>,
        cleanup_error: Box<InstallError>,
    },
    #[error("container runtime is not available")]
    RuntimeUnavailable(#[from] RuntimeAvailabilityError),
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_support::{
        FakeHealthChecker, FakeRuntime, TestProgressReporter, docker_failure,
    };

    fn test_installer(
        installation: Installation,
        runtime: FakeRuntime,
        health_checker: FakeHealthChecker,
    ) -> Install<FakeRuntime, FakeHealthChecker, TestProgressReporter> {
        Install::new(
            installation,
            runtime,
            health_checker,
            TestProgressReporter,
            "aims-installer-test".to_string(),
            18081,
            "aims_installer_test_postgres_data".to_string(),
        )
    }

    #[test]
    fn installation_runs_all_steps() {
        let temp_dir = tempfile::tempdir().unwrap();
        let installation_root = temp_dir.path().join("aims");

        let installation = Installation::new(&installation_root);
        let runtime = FakeRuntime::new();

        let installer = test_installer(installation, runtime.clone(), FakeHealthChecker::healthy());

        let version = Version::parse("0.1.1").unwrap();

        installer.install(&version).unwrap();

        assert!(installation_root.is_dir());
        assert!(installation_root.join("compose.prod.yml").is_file());
        assert!(installation_root.join(".env.prod").is_file());

        assert_eq!(runtime.calls(), ["pull:0.1.1", "apply"]);
    }

    #[test]
    fn pull_failure_is_returned_and_files_are_cleaned_up() {
        let temp_dir = tempfile::tempdir().unwrap();
        let installation_root = temp_dir.path().join("aims");

        let installation = Installation::new(&installation_root);
        let runtime = FakeRuntime::new();

        runtime.push_pull_result(Err(docker_failure()));

        let installer = test_installer(installation, runtime.clone(), FakeHealthChecker::healthy());

        let version = Version::parse("0.1.1").unwrap();

        let result = installer.install(&version);

        assert!(matches!(result, Err(InstallError::Docker(_))));

        assert_eq!(runtime.calls(), ["pull:0.1.1"]);
        assert!(!installation_root.exists());
    }

    #[test]
    fn apply_failure_is_returned_and_running_installation_is_cleaned_up() {
        let temp_dir = tempfile::tempdir().unwrap();
        let installation_root = temp_dir.path().join("aims");

        let installation = Installation::new(&installation_root);
        let runtime = FakeRuntime::new();

        runtime.push_apply_result(Err(docker_failure()));

        let installer = test_installer(installation, runtime.clone(), FakeHealthChecker::healthy());

        let version = Version::parse("0.1.1").unwrap();

        let result = installer.install(&version);

        assert!(matches!(result, Err(InstallError::Docker(_))));

        assert_eq!(runtime.calls(), ["pull:0.1.1", "apply", "down"]);

        assert!(!installation_root.exists());
    }

    #[test]
    fn health_failure_is_returned_and_running_installation_is_cleaned_up() {
        let temp_dir = tempfile::tempdir().unwrap();
        let installation_root = temp_dir.path().join("aims");

        let installation = Installation::new(&installation_root);
        let runtime = FakeRuntime::new();

        let installer = test_installer(installation, runtime.clone(), FakeHealthChecker::failing());

        let version = Version::parse("0.1.1").unwrap();

        let result = installer.install(&version);

        assert!(matches!(result, Err(InstallError::Health(_))));

        assert_eq!(runtime.calls(), ["pull:0.1.1", "apply", "down"]);

        assert!(!installation_root.exists());
    }

    #[test]
    fn cleanup_failure_preserves_installation_failure() {
        let temp_dir = tempfile::tempdir().unwrap();
        let installation_root = temp_dir.path().join("aims");

        let installation = Installation::new(&installation_root);
        let runtime = FakeRuntime::new();

        runtime.push_down_result(Err(docker_failure()));

        let installer = test_installer(installation, runtime.clone(), FakeHealthChecker::failing());

        let version = Version::parse("0.1.1").unwrap();

        let result = installer.install(&version);

        assert!(matches!(
            result,
            Err(InstallError::CleanupFailed {
                installation_error,
                cleanup_error,
            })
                if matches!(
                    *installation_error,
                    InstallError::Health(_)
                )
                    && matches!(
                        *cleanup_error,
                        InstallError::Docker(_)
                    )
        ));

        assert_eq!(runtime.calls(), ["pull:0.1.1", "apply", "down"]);
    }

    #[test]
    fn installation_writes_configured_environment_values() {
        let temp_dir = tempfile::tempdir().unwrap();
        let installation_root = temp_dir.path().join("aims");

        let installation = Installation::new(&installation_root);
        let runtime = FakeRuntime::new();

        let installer = test_installer(installation, runtime, FakeHealthChecker::healthy());

        let version = Version::parse("0.1.1").unwrap();

        installer.install(&version).unwrap();

        let env = std::fs::read_to_string(installation_root.join(".env.prod")).unwrap();

        assert!(env.contains("AIMS_VERSION=0.1.1"));
        assert!(env.contains("AIMS_COMPOSE_PROJECT=aims-installer-test"));
        assert!(env.contains("AIMS_HTTP_PORT=18081"));
        assert!(env.contains("AIMS_POSTGRES_VOLUME=aims_installer_test_postgres_data"));
    }

    #[test]
    fn unavailable_container_runtime_fails_before_installation_starts() {
        let temp_dir = tempfile::tempdir().unwrap();
        let installation_root = temp_dir.path().join("aims");

        let installation = Installation::new(&installation_root);

        let runtime = FakeRuntime::new();

        runtime.push_availability_result(Err(RuntimeAvailabilityError::Docker(
            DockerComposeError::CommandFailedToStart(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "docker not found",
            )),
        )));

        let installer = test_installer(installation, runtime.clone(), FakeHealthChecker::healthy());

        let version = Version::parse("0.1.1").unwrap();

        let result = installer.install(&version);

        assert!(matches!(
            result,
            Err(InstallError::RuntimeUnavailable(
                RuntimeAvailabilityError::Docker(_)
            ))
        ));

        assert!(!installation_root.exists());

        assert!(runtime.calls().is_empty());
    }
}
