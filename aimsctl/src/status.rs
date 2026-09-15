use crate::{
    docker::{ContainerRuntime, DockerComposeError, ServiceStatus},
    health::{HealthCheckError, HealthChecker},
    installation::{Installation, InstallationError},
    version::Version,
};

#[derive(Debug, PartialEq, Eq)]
pub struct AimsStatus {
    pub version: Version,
    pub services: Vec<ServiceStatus>,
    pub healthy: bool,
}

pub struct StatusChecker<R, H>
where
    R: ContainerRuntime,
    H: HealthChecker,
{
    installation: Installation,
    runtime: R,
    health_checker: H,
}

impl<R, H> StatusChecker<R, H>
where
    R: ContainerRuntime,
    H: HealthChecker,
{
    pub fn new(installation: Installation, runtime: R, health_checker: H) -> Self {
        Self {
            installation,
            runtime,
            health_checker,
        }
    }

    pub fn status(&self) -> Result<AimsStatus, StatusError> {
        let version = self.installation.current_version()?;
        let services = self.runtime.service_statuses(&self.installation)?;
        let healthy = self.health_checker.is_healthy(&self.installation)?;

        Ok(AimsStatus {
            version,
            services,
            healthy,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StatusError {
    #[error(transparent)]
    Installation(#[from] InstallationError),
    #[error(transparent)]
    Docker(#[from] DockerComposeError),
    #[error(transparent)]
    HealthCheck(#[from] HealthCheckError),
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        docker::{DockerComposeError, ServiceStatus},
        environment::{EnvironmentConfig, build_environment},
        health::HealthCheckError,
    };

    use std::cell::RefCell;

    struct FakeRuntime {
        statuses: RefCell<Option<Result<Vec<ServiceStatus>, DockerComposeError>>>,
    }

    impl FakeRuntime {
        fn with_statuses(statuses: Vec<ServiceStatus>) -> Self {
            Self {
                statuses: RefCell::new(Some(Ok(statuses))),
            }
        }

        fn failing() -> Self {
            Self {
                statuses: RefCell::new(Some(Err(DockerComposeError::CommandFailed {
                    exit_code: Some(1),
                    stdout: String::new(),
                    stderr: "docker failed".to_owned(),
                }))),
            }
        }
    }

    impl ContainerRuntime for FakeRuntime {
        fn pull(
            &self,
            _installation: &Installation,
            _version: &Version,
        ) -> Result<(), DockerComposeError> {
            Ok(())
        }

        fn apply(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
            Ok(())
        }

        fn start(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
            Ok(())
        }

        fn stop(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
            Ok(())
        }

        fn down(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
            Ok(())
        }

        fn service_statuses(
            &self,
            _installation: &Installation,
        ) -> Result<Vec<ServiceStatus>, DockerComposeError> {
            self.statuses
                .borrow_mut()
                .take()
                .expect("service_statuses called more than once")
        }
    }

    struct FakeHealthChecker {
        healthy: bool,
    }

    impl HealthChecker for FakeHealthChecker {
        fn is_healthy(&self, _installation: &Installation) -> Result<bool, HealthCheckError> {
            Ok(self.healthy)
        }

        fn wait_until_healthy(&self, _installation: &Installation) -> Result<(), HealthCheckError> {
            Ok(())
        }
    }

    fn create_installation() -> (tempfile::TempDir, Installation, Version) {
        let temp_dir = tempfile::tempdir().unwrap();
        let installation_root = temp_dir.path().join("aims");

        let installation = Installation::new(
            installation_root,
            "http://127.0.0.1:8080/api/v1/health".to_owned(),
        );

        installation.create().unwrap();

        let version = Version::parse("0.1.3").unwrap();

        let environment = build_environment(&EnvironmentConfig {
            version: &version,
            database_password: "password",
            compose_project: "test",
            http_port: 8080,
            postgres_volume: "test-volume",
        });

        installation.write_environment(&environment).unwrap();

        (temp_dir, installation, version)
    }

    #[test]
    fn status_contains_version_services_and_health() {
        let (_temp_dir, installation, version) = create_installation();

        let services = vec![
            ServiceStatus {
                name: "postgres".to_owned(),
                state: "running".to_owned(),
            },
            ServiceStatus {
                name: "backend".to_owned(),
                state: "running".to_owned(),
            },
            ServiceStatus {
                name: "frontend".to_owned(),
                state: "running".to_owned(),
            },
        ];

        let checker = StatusChecker::new(
            installation,
            FakeRuntime::with_statuses(services.clone()),
            FakeHealthChecker { healthy: true },
        );

        let status = checker.status().unwrap();

        assert_eq!(status.version, version);
        assert_eq!(status.services, services);
        assert!(status.healthy);
    }

    #[test]
    fn unhealthy_installation_is_reported() {
        let (_temp_dir, installation, version) = create_installation();

        let services = vec![
            ServiceStatus {
                name: "postgres".to_owned(),
                state: "running".to_owned(),
            },
            ServiceStatus {
                name: "backend".to_owned(),
                state: "running".to_owned(),
            },
            ServiceStatus {
                name: "frontend".to_owned(),
                state: "running".to_owned(),
            },
        ];

        let checker = StatusChecker::new(
            installation,
            FakeRuntime::with_statuses(services.clone()),
            FakeHealthChecker { healthy: false },
        );

        let status = checker.status().unwrap();

        assert_eq!(status.version, version);
        assert_eq!(status.services, services);
        assert!(!status.healthy);
    }

    #[test]
    fn docker_error_is_returned() {
        let (_temp_dir, installation, _version) = create_installation();

        let checker = StatusChecker::new(
            installation,
            FakeRuntime::failing(),
            FakeHealthChecker { healthy: true },
        );

        let result = checker.status();

        assert!(matches!(result, Err(StatusError::Docker(_))));
    }
}
