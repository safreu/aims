use crate::{
    docker::ContainerRuntime, environment::EnvironmentConfig, health::HealthChecker,
    installation::Installation, version::Version,
};

pub struct Installer<D, H> {
    installation: Installation,
    docker: D,
    health_checker: H,
    compose_project: String,
    http_port: u16,
    postgres_volume: String,
}

impl<D, H> Installer<D, H>
where
    D: ContainerRuntime,
    H: HealthChecker,
{
    pub fn new(
        installation: Installation,
        docker: D,
        health_checker: H,
        compose_project: String,
        http_port: u16,
        postgres_volume: String,
    ) -> Self {
        Self {
            installation,
            docker,
            health_checker,
            compose_project,
            http_port,
            postgres_volume,
        }
    }

    pub fn install(&self, version: &Version) -> Result<(), InstallerError> {
        self.installation.create()?;
        self.installation.write_compose_file()?;

        let database_password = crate::environment::generate_database_password();

        let environment_config = EnvironmentConfig {
            version,
            database_password: &database_password,
            compose_project: &self.compose_project,
            http_port: self.http_port,
            postgres_volume: &self.postgres_volume,
        };

        let environment = crate::environment::build_environment(&environment_config);

        self.installation.write_environment(&environment)?;

        if let Err(error) = self.docker.pull(&self.installation) {
            let installation_error = InstallerError::Docker(error);
            return Err(self.cleanup_after_failure(installation_error, self.cleanup_files()));
        }

        if let Err(error) = self.docker.up(&self.installation) {
            let installation_error = InstallerError::Docker(error);
            return Err(self
                .cleanup_after_failure(installation_error, self.cleanup_running_installations()));
        }

        if let Err(error) = self.health_checker.wait_until_healthy(&self.installation) {
            let installation_error = InstallerError::Health(error);

            return Err(self
                .cleanup_after_failure(installation_error, self.cleanup_running_installations()));
        }

        Ok(())
    }

    fn cleanup_files(&self) -> Result<(), InstallerError> {
        self.installation.remove()?;
        Ok(())
    }

    fn cleanup_running_installations(&self) -> Result<(), InstallerError> {
        self.docker.down(&self.installation)?;
        self.installation.remove()?;
        Ok(())
    }

    fn cleanup_after_failure(
        &self,
        installation_error: InstallerError,
        cleanup_result: Result<(), InstallerError>,
    ) -> InstallerError {
        match cleanup_result {
            Ok(()) => installation_error,
            Err(cleanup_error) => InstallerError::CleanupFailed {
                installation_error: Box::new(installation_error),
                cleanup_error: Box::new(cleanup_error),
            },
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InstallerError {
    #[error(transparent)]
    Installation(#[from] crate::installation::InstallationError),
    #[error(transparent)]
    Docker(#[from] crate::docker::DockerComposeError),
    #[error(transparent)]
    Health(#[from] crate::health::HealthCheckError),
    #[error("installation failed: {installation_error}; cleanup also failed: {cleanup_error}")]
    CleanupFailed {
        installation_error: Box<InstallerError>,
        cleanup_error: Box<InstallerError>,
    },
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use super::*;
    use crate::{
        docker::DockerComposeError,
        health::{HealthCheckError, HealthChecker},
    };

    fn test_installer<D, H>(
        installation: Installation,
        docker: D,
        health_checker: H,
    ) -> Installer<D, H>
    where
        D: ContainerRuntime,
        H: HealthChecker,
    {
        Installer::new(
            installation,
            docker,
            health_checker,
            "aims-installer-test".to_string(),
            18081,
            "aims_installer_test_postgres_data".to_string(),
        )
    }

    #[derive(Clone)]
    struct FakeDocker {
        calls: Rc<RefCell<Vec<&'static str>>>,
    }

    impl ContainerRuntime for FakeDocker {
        fn pull(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
            self.calls.borrow_mut().push("pull");
            Ok(())
        }

        fn up(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
            self.calls.borrow_mut().push("up");
            Ok(())
        }

        fn down(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
            self.calls.borrow_mut().push("down");
            Ok(())
        }
    }

    #[derive(Clone)]
    struct FakeHealthChecker {
        calls: Rc<RefCell<Vec<&'static str>>>,
    }

    impl HealthChecker for FakeHealthChecker {
        fn wait_until_healthy(&self, _installation: &Installation) -> Result<(), HealthCheckError> {
            self.calls.borrow_mut().push("health");
            Ok(())
        }
    }

    #[test]
    fn installation_runs_all_steps() {
        let temp_dir = tempfile::tempdir().unwrap();
        let installation_root = temp_dir.path().join("aims");

        let calls = Rc::new(RefCell::new(Vec::new()));

        let installation =
            Installation::new(&installation_root, "http://127.0.0.1:18081/api/v1/health");

        let installer = test_installer(
            installation,
            FakeDocker {
                calls: Rc::clone(&calls),
            },
            FakeHealthChecker {
                calls: Rc::clone(&calls),
            },
        );

        let version = Version::parse("0.1.1").unwrap();

        installer.install(&version).unwrap();

        assert!(installation_root.is_dir());
        assert!(installation_root.join("compose.prod.yml").is_file());
        assert!(installation_root.join(".env.prod").is_file());

        assert_eq!(calls.borrow().as_slice(), ["pull", "up", "health"],);
    }

    #[test]
    fn pull_failure_is_returned_and_files_are_cleaned_up() {
        struct FailingDocker {
            calls: Rc<RefCell<Vec<&'static str>>>,
        }

        impl ContainerRuntime for FailingDocker {
            fn pull(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
                self.calls.borrow_mut().push("pull");

                Err(DockerComposeError::CommandFailed {
                    exit_code: Some(1),
                    stdout: String::new(),
                    stderr: "pull failed".to_string(),
                })
            }

            fn up(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
                panic!("up must not be called after pull failure");
            }

            fn down(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
                self.calls.borrow_mut().push("down");
                Ok(())
            }
        }

        struct FakeHealthChecker;

        impl HealthChecker for FakeHealthChecker {
            fn wait_until_healthy(
                &self,
                _installation: &Installation,
            ) -> Result<(), HealthCheckError> {
                panic!("health check must not run after pull failure");
            }
        }

        let temp_dir = tempfile::tempdir().unwrap();
        let installation_root = temp_dir.path().join("aims");

        let calls = Rc::new(RefCell::new(Vec::new()));

        let installation =
            Installation::new(&installation_root, "http://127.0.0.1:18081/api/v1/health");

        let installer = test_installer(
            installation,
            FailingDocker {
                calls: Rc::clone(&calls),
            },
            FakeHealthChecker,
        );

        let version = Version::parse("0.1.1").unwrap();

        let result = installer.install(&version);

        assert!(matches!(result, Err(InstallerError::Docker(_))));

        assert_eq!(calls.borrow().as_slice(), ["pull"]);

        assert!(!installation_root.exists());
    }

    #[test]
    fn up_failure_is_returned_and_running_installation_is_cleaned_up() {
        struct FailingDocker {
            calls: Rc<RefCell<Vec<&'static str>>>,
        }

        impl ContainerRuntime for FailingDocker {
            fn pull(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
                self.calls.borrow_mut().push("pull");
                Ok(())
            }

            fn up(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
                self.calls.borrow_mut().push("up");

                Err(DockerComposeError::CommandFailed {
                    exit_code: Some(1),
                    stdout: String::new(),
                    stderr: "up failed".to_string(),
                })
            }

            fn down(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
                self.calls.borrow_mut().push("down");
                Ok(())
            }
        }

        struct FakeHealthChecker;

        impl HealthChecker for FakeHealthChecker {
            fn wait_until_healthy(
                &self,
                _installation: &Installation,
            ) -> Result<(), HealthCheckError> {
                panic!("health check must not run after up failure");
            }
        }

        let temp_dir = tempfile::tempdir().unwrap();
        let installation_root = temp_dir.path().join("aims");

        let calls = Rc::new(RefCell::new(Vec::new()));

        let installation =
            Installation::new(&installation_root, "http://127.0.0.1:18081/api/v1/health");

        let installer = test_installer(
            installation,
            FailingDocker {
                calls: Rc::clone(&calls),
            },
            FakeHealthChecker,
        );

        let version = Version::parse("0.1.1").unwrap();

        let result = installer.install(&version);

        assert!(matches!(result, Err(InstallerError::Docker(_))));

        assert_eq!(calls.borrow().as_slice(), ["pull", "up", "down"],);

        assert!(!installation_root.exists());
    }

    #[test]
    fn health_failure_is_returned_and_running_installation_is_cleaned_up() {
        struct FakeDocker {
            calls: Rc<RefCell<Vec<&'static str>>>,
        }

        impl ContainerRuntime for FakeDocker {
            fn pull(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
                self.calls.borrow_mut().push("pull");
                Ok(())
            }

            fn up(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
                self.calls.borrow_mut().push("up");
                Ok(())
            }

            fn down(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
                self.calls.borrow_mut().push("down");
                Ok(())
            }
        }

        struct FailingHealthChecker {
            calls: Rc<RefCell<Vec<&'static str>>>,
        }

        impl HealthChecker for FailingHealthChecker {
            fn wait_until_healthy(
                &self,
                _installation: &Installation,
            ) -> Result<(), HealthCheckError> {
                self.calls.borrow_mut().push("health");
                Err(HealthCheckError::TimedOut)
            }
        }

        let temp_dir = tempfile::tempdir().unwrap();
        let installation_root = temp_dir.path().join("aims");

        let calls = Rc::new(RefCell::new(Vec::new()));

        let installation =
            Installation::new(&installation_root, "http://127.0.0.1:18081/api/v1/health");

        let installer = test_installer(
            installation,
            FakeDocker {
                calls: Rc::clone(&calls),
            },
            FailingHealthChecker {
                calls: Rc::clone(&calls),
            },
        );

        let version = Version::parse("0.1.1").unwrap();

        let result = installer.install(&version);

        assert!(matches!(result, Err(InstallerError::Health(_))));

        assert_eq!(calls.borrow().as_slice(), ["pull", "up", "health", "down"],);

        assert!(!installation_root.exists());
    }

    #[test]
    fn cleanup_failure_preserves_installation_failure() {
        struct FailingDocker;

        impl ContainerRuntime for FailingDocker {
            fn pull(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
                Ok(())
            }

            fn up(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
                Ok(())
            }

            fn down(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
                Err(DockerComposeError::CommandFailed {
                    exit_code: Some(1),
                    stdout: String::new(),
                    stderr: "down failed".to_string(),
                })
            }
        }

        struct FailingHealthChecker;

        impl HealthChecker for FailingHealthChecker {
            fn wait_until_healthy(
                &self,
                _installation: &Installation,
            ) -> Result<(), HealthCheckError> {
                Err(HealthCheckError::TimedOut)
            }
        }

        let temp_dir = tempfile::tempdir().unwrap();
        let installation_root = temp_dir.path().join("aims");

        let installation =
            Installation::new(&installation_root, "http://127.0.0.1:18081/api/v1/health");

        let installer = test_installer(installation, FailingDocker, FailingHealthChecker);

        let version = Version::parse("0.1.1").unwrap();

        let result = installer.install(&version);

        assert!(matches!(
            result,
            Err(InstallerError::CleanupFailed {
                installation_error,
                cleanup_error,
            })
                if matches!(*installation_error, InstallerError::Health(_))
                    && matches!(*cleanup_error, InstallerError::Docker(_))
        ));
    }

    #[test]
    fn installation_writes_configured_environment_values() {
        let temp_dir = tempfile::tempdir().unwrap();
        let installation_root = temp_dir.path().join("aims");

        let installation =
            Installation::new(&installation_root, "http://127.0.0.1:18081/api/v1/health");

        struct FakeDocker;

        impl ContainerRuntime for FakeDocker {
            fn pull(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
                Ok(())
            }

            fn up(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
                Ok(())
            }

            fn down(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
                Ok(())
            }
        }

        struct FakeHealthChecker;

        impl HealthChecker for FakeHealthChecker {
            fn wait_until_healthy(
                &self,
                _installation: &Installation,
            ) -> Result<(), HealthCheckError> {
                Ok(())
            }
        }

        let installer = test_installer(installation, FakeDocker, FakeHealthChecker);

        let version = Version::parse("0.1.1").unwrap();

        installer.install(&version).unwrap();

        let env = std::fs::read_to_string(installation_root.join(".env.prod")).unwrap();

        assert!(env.contains("AIMS_VERSION=0.1.1"));
        assert!(env.contains("AIMS_COMPOSE_PROJECT=aims-installer-test"));
        assert!(env.contains("AIMS_HTTP_PORT=18081"));
        assert!(env.contains("AIMS_POSTGRES_VOLUME=aims_installer_test_postgres_data"));
    }
}
