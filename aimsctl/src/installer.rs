use crate::{
    docker::ContainerRuntime, environment::EnvironmentConfig, health::HealthChecker,
    installation::Installation, progress::ProgressReporter, version::Version,
};

pub struct Installer<D, H, P>
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

impl<D, H, P> Installer<D, H, P>
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

    pub fn install(&self, version: &Version) -> Result<(), InstallerError> {
        const TOTAL_STEPS: usize = 5;

        self.progress.header(&format!("Aims {version} installer"));

        self.progress.step(1, TOTAL_STEPS, "Preparing installation");

        self.installation.create()?;
        self.installation.write_compose_file()?;

        self.progress.detail("Installation directory created");

        self.progress.step(2, TOTAL_STEPS, "Writing configuration");

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

        self.progress.detail("Configuration written");

        self.progress
            .step(3, TOTAL_STEPS, &format!("Downloading Aims {version}"));

        if let Err(error) = self.docker.pull(&self.installation, version) {
            let installation_error = InstallerError::Docker(error);
            return Err(self.cleanup_after_failure(installation_error, self.cleanup_files()));
        }

        self.progress.detail("Docker images downloaded");

        self.progress.step(4, TOTAL_STEPS, "Starting services");

        if let Err(error) = self.docker.apply(&self.installation) {
            let installation_error = InstallerError::Docker(error);
            return Err(self
                .cleanup_after_failure(installation_error, self.cleanup_running_installations()));
        }

        self.progress.detail("Services started");

        self.progress.step(5, TOTAL_STEPS, "Checking health");

        if let Err(error) = self.health_checker.wait_until_healthy(&self.installation) {
            let installation_error = InstallerError::Health(error);

            return Err(self
                .cleanup_after_failure(installation_error, self.cleanup_running_installations()));
        }

        self.progress.detail("Aims is healthy");

        self.progress
            .success(&format!("Aims {version} installed successfully"));

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
        docker::{DockerComposeError, ServiceStatus},
        health::{HealthCheckError, HealthChecker},
    };
    struct TestProgressReporter;

    impl ProgressReporter for TestProgressReporter {
        fn header(&self, _message: &str) {}
        fn step(&self, _current: usize, _total: usize, _message: &str) {}
        fn detail(&self, _message: &str) {}
        fn phase(&self, _name: &str, _message: &str) {}
        fn success(&self, _message: &str) {}
    }

    fn test_installer<D, H>(
        installation: Installation,
        docker: D,
        health_checker: H,
    ) -> Installer<D, H, TestProgressReporter>
    where
        D: ContainerRuntime,
        H: HealthChecker,
    {
        Installer::new(
            installation,
            docker,
            health_checker,
            TestProgressReporter,
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
        fn pull(
            &self,
            _installation: &Installation,
            _version: &Version,
        ) -> Result<(), DockerComposeError> {
            self.calls.borrow_mut().push("pull");
            Ok(())
        }

        fn apply(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
            self.calls.borrow_mut().push("apply");
            Ok(())
        }

        fn start(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
            panic!("start must not be called by installer");
        }

        fn stop(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
            panic!("stop must not be called by installer");
        }

        fn down(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
            self.calls.borrow_mut().push("down");
            Ok(())
        }

        fn service_statuses(
            &self,
            _installation: &Installation,
        ) -> Result<Vec<ServiceStatus>, DockerComposeError> {
            Ok(Vec::new())
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

        fn is_healthy(&self, _installation: &Installation) -> Result<bool, HealthCheckError> {
            Ok(true)
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

        assert_eq!(calls.borrow().as_slice(), ["pull", "apply", "health"],);
    }

    #[test]
    fn pull_failure_is_returned_and_files_are_cleaned_up() {
        struct FailingDocker {
            calls: Rc<RefCell<Vec<&'static str>>>,
        }

        impl ContainerRuntime for FailingDocker {
            fn pull(
                &self,
                _installation: &Installation,
                _version: &Version,
            ) -> Result<(), DockerComposeError> {
                self.calls.borrow_mut().push("pull");

                Err(DockerComposeError::CommandFailed {
                    exit_code: Some(1),
                    stdout: String::new(),
                    stderr: "pull failed".to_string(),
                })
            }

            fn apply(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
                panic!("apply must not be called after pull failure");
            }

            fn start(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
                panic!("start must not be called by installer");
            }

            fn stop(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
                panic!("stop must not be called by installer");
            }

            fn down(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
                self.calls.borrow_mut().push("down");
                Ok(())
            }

            fn service_statuses(
                &self,
                _installation: &Installation,
            ) -> Result<Vec<ServiceStatus>, DockerComposeError> {
                Ok(Vec::new())
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

            fn is_healthy(&self, _installation: &Installation) -> Result<bool, HealthCheckError> {
                Ok(true)
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
    fn apply_failure_is_returned_and_running_installation_is_cleaned_up() {
        struct FailingDocker {
            calls: Rc<RefCell<Vec<&'static str>>>,
        }

        impl ContainerRuntime for FailingDocker {
            fn pull(
                &self,
                _installation: &Installation,
                _version: &Version,
            ) -> Result<(), DockerComposeError> {
                self.calls.borrow_mut().push("pull");
                Ok(())
            }

            fn apply(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
                self.calls.borrow_mut().push("apply");

                Err(DockerComposeError::CommandFailed {
                    exit_code: Some(1),
                    stdout: String::new(),
                    stderr: "apply failed".to_string(),
                })
            }

            fn start(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
                panic!("start must not be called by installer");
            }

            fn stop(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
                panic!("stop must not be called by installer");
            }

            fn down(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
                self.calls.borrow_mut().push("down");
                Ok(())
            }

            fn service_statuses(
                &self,
                _installation: &Installation,
            ) -> Result<Vec<ServiceStatus>, DockerComposeError> {
                Ok(Vec::new())
            }
        }

        struct FakeHealthChecker;

        impl HealthChecker for FakeHealthChecker {
            fn wait_until_healthy(
                &self,
                _installation: &Installation,
            ) -> Result<(), HealthCheckError> {
                panic!("health check must not run after apply failure");
            }

            fn is_healthy(&self, _installation: &Installation) -> Result<bool, HealthCheckError> {
                Ok(true)
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
        assert_eq!(calls.borrow().as_slice(), ["pull", "apply", "down"],);
        assert!(!installation_root.exists());
    }

    #[test]
    fn health_failure_is_returned_and_running_installation_is_cleaned_up() {
        struct FakeDocker {
            calls: Rc<RefCell<Vec<&'static str>>>,
        }

        impl ContainerRuntime for FakeDocker {
            fn pull(
                &self,
                _installation: &Installation,
                _version: &Version,
            ) -> Result<(), DockerComposeError> {
                self.calls.borrow_mut().push("pull");
                Ok(())
            }

            fn apply(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
                self.calls.borrow_mut().push("apply");
                Ok(())
            }

            fn start(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
                panic!("start must not be called by installer");
            }

            fn stop(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
                panic!("stop must not be called by installer");
            }

            fn down(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
                self.calls.borrow_mut().push("down");
                Ok(())
            }

            fn service_statuses(
                &self,
                _installation: &Installation,
            ) -> Result<Vec<ServiceStatus>, DockerComposeError> {
                Ok(Vec::new())
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

            fn is_healthy(&self, _installation: &Installation) -> Result<bool, HealthCheckError> {
                Ok(true)
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
        assert_eq!(
            calls.borrow().as_slice(),
            ["pull", "apply", "health", "down"],
        );
        assert!(!installation_root.exists());
    }

    #[test]
    fn cleanup_failure_preserves_installation_failure() {
        struct FailingDocker;

        impl ContainerRuntime for FailingDocker {
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
                panic!("start must not be called by installer");
            }

            fn stop(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
                panic!("stop must not be called by installer");
            }

            fn down(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
                Err(DockerComposeError::CommandFailed {
                    exit_code: Some(1),
                    stdout: String::new(),
                    stderr: "down failed".to_string(),
                })
            }

            fn service_statuses(
                &self,
                _installation: &Installation,
            ) -> Result<Vec<ServiceStatus>, DockerComposeError> {
                Ok(Vec::new())
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

            fn is_healthy(&self, _installation: &Installation) -> Result<bool, HealthCheckError> {
                Ok(true)
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
                panic!("start must not be called by installer");
            }

            fn stop(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
                panic!("stop must not be called by installer");
            }

            fn down(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
                Ok(())
            }

            fn service_statuses(
                &self,
                _installation: &Installation,
            ) -> Result<Vec<ServiceStatus>, DockerComposeError> {
                Ok(Vec::new())
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

            fn is_healthy(&self, _installation: &Installation) -> Result<bool, HealthCheckError> {
                Ok(true)
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
