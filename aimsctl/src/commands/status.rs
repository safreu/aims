use crate::{
    installation::{
        Version, {Installation, InstallationError},
    },
    runtime::{
        docker::{ContainerRuntime, DockerComposeError, ServiceStatus},
        health::{HealthCheckError, HealthChecker},
    },
};

#[derive(Debug, PartialEq, Eq)]
pub struct AimsStatus {
    pub version: Version,
    pub services: Vec<ServiceStatus>,
    pub healthy: bool,
}

pub struct Status<R, H>
where
    R: ContainerRuntime,
    H: HealthChecker,
{
    installation: Installation,
    runtime: R,
    health_checker: H,
}

impl<R, H> Status<R, H>
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
        runtime::docker::ServiceStatus,
        test_support::{self, FakeHealthChecker, FakeRuntime, docker_failure},
    };

    #[test]
    fn status_contains_version_services_and_health() {
        let test_installation = test_support::installation("0.1.3");

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

        let runtime = FakeRuntime::new();
        runtime.push_service_status_result(Ok(services.clone()));

        let checker = Status::new(
            test_installation.installation,
            runtime,
            FakeHealthChecker::healthy(),
        );

        let status = checker.status().unwrap();

        assert_eq!(status.version, Version::parse("0.1.3").unwrap());
        assert_eq!(status.services, services);
        assert!(status.healthy);
    }

    #[test]
    fn unhealthy_installation_is_reported() {
        let test_installation = test_support::installation("0.1.3");

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

        let runtime = FakeRuntime::new();
        runtime.push_service_status_result(Ok(services.clone()));

        let checker = Status::new(
            test_installation.installation,
            runtime,
            FakeHealthChecker::unhealthy(),
        );

        let status = checker.status().unwrap();

        assert_eq!(status.version, Version::parse("0.1.3").unwrap());
        assert_eq!(status.services, services);
        assert!(!status.healthy);
    }

    #[test]
    fn docker_error_is_returned() {
        let test_installation = test_support::installation("0.1.3");

        let runtime = FakeRuntime::new();
        runtime.push_service_status_result(Err(docker_failure()));

        let checker = Status::new(
            test_installation.installation,
            runtime,
            FakeHealthChecker::healthy(),
        );

        let result = checker.status();

        assert!(matches!(result, Err(StatusError::Docker(_))));
    }
}
