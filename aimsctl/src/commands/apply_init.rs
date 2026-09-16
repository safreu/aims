use crate::{
    commands::{Install, install::InstallError},
    configuration::{AimsctlConfig, ConfigStore, ConfigStoreError},
    executable::{ExecutableInstaller, ExecutableManagerError},
    installation::{Installation, Version, VersionParseError},
    progress::ConsoleReporter,
    runtime::{docker::DockerCompose, health::HttpHealthChecker},
};

pub(crate) trait AimsInstaller {
    fn install(&self, config: &AimsctlConfig) -> Result<(), AimsInstallError>;
}

pub(crate) struct SystemAimsInstaller;

impl AimsInstaller for SystemAimsInstaller {
    fn install(&self, config: &AimsctlConfig) -> Result<(), AimsInstallError> {
        let version = Version::parse(env!("CARGO_PKG_VERSION"))?;

        let installation = Installation::new(config.installation.root.clone());

        let installer = Install::new(
            installation,
            DockerCompose,
            HttpHealthChecker,
            ConsoleReporter,
            config.docker.compose_project.clone(),
            config.http.port,
            config.docker.postgres_volume.clone(),
        );

        installer.install(&version)?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum AimsInstallError {
    #[error(transparent)]
    Install(#[from] InstallError),

    #[error("invalid aimsctl version")]
    Version(#[from] VersionParseError),
}

pub(crate) struct ApplyInit<E, I>
where
    E: ExecutableInstaller,
    I: AimsInstaller,
{
    config_store: ConfigStore,
    executable_installer: E,
    aims_installer: I,
}

impl<E, I> ApplyInit<E, I>
where
    E: ExecutableInstaller,
    I: AimsInstaller,
{
    pub(crate) fn new(
        config_store: ConfigStore,
        executable_installer: E,
        aims_installer: I,
    ) -> Self {
        Self {
            config_store,
            executable_installer,
            aims_installer,
        }
    }

    pub(crate) fn apply(&self, config: &AimsctlConfig) -> Result<(), ApplyInitError> {
        self.config_store.create(config)?;

        if let Err(error) = self.executable_installer.install() {
            let initialization_error = ApplyInitError::Executable(error);

            if let Err(cleanup_error) = self.cleanup_config() {
                return Err(ApplyInitError::CleanupFailed {
                    initialization_error: Box::new(initialization_error),
                    cleanup_error: Box::new(cleanup_error),
                });
            }

            return Err(initialization_error);
        }

        if let Err(error) = self.aims_installer.install(config) {
            let initialization_error = ApplyInitError::Aims(error);

            if let Err(cleanup_error) = self.cleanup_executable_and_config() {
                return Err(ApplyInitError::CleanupFailed {
                    initialization_error: Box::new(initialization_error),
                    cleanup_error: Box::new(cleanup_error),
                });
            }

            return Err(initialization_error);
        }

        Ok(())
    }

    fn cleanup_config(&self) -> Result<(), ApplyInitCleanupError> {
        self.config_store
            .remove()
            .map_err(ApplyInitCleanupError::ConfigStore)
    }

    fn cleanup_executable_and_config(&self) -> Result<(), ApplyInitCleanupError> {
        let executable_result = self.executable_installer.uninstall();
        let config_result = self.config_store.remove();

        match (executable_result, config_result) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(executable_error), Ok(())) => {
                Err(ApplyInitCleanupError::Executable(executable_error))
            }
            (Ok(()), Err(config_error)) => Err(ApplyInitCleanupError::ConfigStore(config_error)),
            (Err(executable_error), Err(config_error)) => {
                Err(ApplyInitCleanupError::ExecutableAndConfig {
                    executable_error,
                    config_error,
                })
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ApplyInitError {
    #[error("failed to store aimsctl configuration")]
    ConfigStore(#[from] ConfigStoreError),

    #[error("failed to install aimsctl executable")]
    Executable(#[from] ExecutableManagerError),

    #[error("failed to install Aims")]
    Aims(#[from] AimsInstallError),

    #[error("initialization failed: {initialization_error}; cleanup also failed: {cleanup_error}")]
    CleanupFailed {
        initialization_error: Box<ApplyInitError>,
        cleanup_error: Box<ApplyInitCleanupError>,
    },
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ApplyInitCleanupError {
    #[error("failed to remove aimsctl executable")]
    Executable(ExecutableManagerError),
    #[error("failed to remove aimsctl configuration")]
    ConfigStore(ConfigStoreError),
    #[error("failed to remove aimsctl executable and configuration")]
    ExecutableAndConfig {
        executable_error: ExecutableManagerError,
        config_error: ConfigStoreError,
    },
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, path::PathBuf, rc::Rc};

    use super::*;

    #[derive(Clone)]
    struct FakeExecutableInstaller {
        calls: Rc<RefCell<Vec<&'static str>>>,
        install_should_fail: bool,
        uninstall_should_fail: bool,
    }

    impl FakeExecutableInstaller {
        fn succeeding() -> Self {
            Self {
                calls: Rc::new(RefCell::new(Vec::new())),
                install_should_fail: false,
                uninstall_should_fail: false,
            }
        }

        fn failing_install() -> Self {
            Self {
                calls: Rc::new(RefCell::new(Vec::new())),
                install_should_fail: true,
                uninstall_should_fail: false,
            }
        }

        fn calls(&self) -> Vec<&'static str> {
            self.calls.borrow().clone()
        }

        fn failing_uninstall() -> Self {
            Self {
                calls: Rc::new(RefCell::new(Vec::new())),
                install_should_fail: false,
                uninstall_should_fail: true,
            }
        }
    }

    impl ExecutableInstaller for FakeExecutableInstaller {
        fn install(&self) -> Result<(), ExecutableManagerError> {
            self.calls.borrow_mut().push("install");

            if self.install_should_fail {
                return Err(ExecutableManagerError::Copy(std::io::Error::other(
                    "test install failure",
                )));
            }

            Ok(())
        }

        fn uninstall(&self) -> Result<(), ExecutableManagerError> {
            self.calls.borrow_mut().push("uninstall");

            if self.uninstall_should_fail {
                return Err(ExecutableManagerError::Remove(std::io::Error::other(
                    "test uninstall failure",
                )));
            }

            Ok(())
        }
    }

    #[derive(Clone)]
    struct FakeAimsInstaller {
        calls: Rc<RefCell<Vec<AimsctlConfig>>>,
        should_fail: bool,
    }

    impl FakeAimsInstaller {
        fn succeeding() -> Self {
            Self {
                calls: Rc::new(RefCell::new(Vec::new())),
                should_fail: false,
            }
        }

        fn failing() -> Self {
            Self {
                calls: Rc::new(RefCell::new(Vec::new())),
                should_fail: true,
            }
        }

        fn calls(&self) -> Vec<AimsctlConfig> {
            self.calls.borrow().clone()
        }
    }

    impl AimsInstaller for FakeAimsInstaller {
        fn install(&self, config: &AimsctlConfig) -> Result<(), AimsInstallError> {
            self.calls.borrow_mut().push(config.clone());

            if self.should_fail {
                return Err(AimsInstallError::Install(InstallError::Test));
            }

            Ok(())
        }
    }

    #[test]
    fn successful_initialization_persists_config_and_installs_everything() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("aimsctl.toml");

        let mut config = AimsctlConfig::default();
        config.installation.root = PathBuf::from("/srv/aims");
        config.http.port = 8080;

        let executable = FakeExecutableInstaller::succeeding();
        let aims = FakeAimsInstaller::succeeding();

        let apply_init = ApplyInit::new(
            ConfigStore::new(&config_path),
            executable.clone(),
            aims.clone(),
        );

        apply_init.apply(&config).unwrap();

        let stored = ConfigStore::new(&config_path).load().unwrap();

        assert_eq!(stored, config);
        assert_eq!(executable.calls(), vec!["install"]);
        assert_eq!(aims.calls(), vec![config]);
    }

    #[test]
    fn executable_failure_removes_configuration() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("aimsctl.toml");

        let config = AimsctlConfig::default();

        let executable = FakeExecutableInstaller::failing_install();
        let aims = FakeAimsInstaller::succeeding();

        let apply_init = ApplyInit::new(
            ConfigStore::new(&config_path),
            executable.clone(),
            aims.clone(),
        );

        let result = apply_init.apply(&config);

        assert!(matches!(result, Err(ApplyInitError::Executable(_))));
        assert!(!config_path.exists());

        assert_eq!(executable.calls(), vec!["install"]);
        assert!(aims.calls().is_empty());
    }

    #[test]
    fn aims_failure_removes_executable_and_configuration() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("aimsctl.toml");

        let config = AimsctlConfig::default();

        let executable = FakeExecutableInstaller::succeeding();
        let aims = FakeAimsInstaller::failing();

        let apply_init = ApplyInit::new(
            ConfigStore::new(&config_path),
            executable.clone(),
            aims.clone(),
        );

        let result = apply_init.apply(&config);

        assert!(matches!(result, Err(ApplyInitError::Aims(_))));
        assert!(!config_path.exists());

        assert_eq!(executable.calls(), vec!["install", "uninstall"]);
        assert_eq!(aims.calls(), vec![config]);
    }

    #[test]
    fn aims_failure_still_removes_configuration_when_executable_cleanup_fails() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("aimsctl.toml");

        let config = AimsctlConfig::default();

        let executable = FakeExecutableInstaller::failing_uninstall();
        let aims = FakeAimsInstaller::failing();

        let apply_init = ApplyInit::new(
            ConfigStore::new(&config_path),
            executable.clone(),
            aims.clone(),
        );

        let result = apply_init.apply(&config);

        assert!(matches!(
            result,
            Err(ApplyInitError::CleanupFailed {
                initialization_error,
                cleanup_error,
            }) if
                matches!(*initialization_error, ApplyInitError::Aims(_))
                && matches!(*cleanup_error, ApplyInitCleanupError::Executable(_))
        ));

        assert!(!config_path.exists());

        assert_eq!(executable.calls(), vec!["install", "uninstall"]);
        assert_eq!(aims.calls(), vec![config]);
    }
}
