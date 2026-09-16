use crate::{
    executable::{TargetBinaryInstaller, TargetBinaryInstallerError},
    installation::{
        Version, {Installation, InstallationError},
    },
};

use super::{
    provider::{TargetBinaryProvider, TargetBinaryProviderError},
    runner::{TargetBinaryRunner, TargetBinaryRunnerError},
};

pub struct SelfUpdater<P, R, I>
where
    P: TargetBinaryProvider,
    R: TargetBinaryRunner,
    I: TargetBinaryInstaller,
{
    installation: Installation,
    provider: P,
    runner: R,
    installer: I,
}

impl<P, R, I> SelfUpdater<P, R, I>
where
    P: TargetBinaryProvider,
    R: TargetBinaryRunner,
    I: TargetBinaryInstaller,
{
    pub fn new(installation: Installation, provider: P, runner: R, installer: I) -> Self {
        Self {
            installation,
            provider,
            runner,
            installer,
        }
    }

    pub fn update(&self, version: &Version) -> Result<(), SelfUpdateError> {
        let current_version = self.installation.current_version()?;

        if version == &current_version {
            return Err(SelfUpdateError::AlreadyInstalled(version.clone()));
        }

        if version < &current_version {
            return Err(SelfUpdateError::DowngradeNotAllowed {
                current: current_version,
                requested: version.clone(),
            });
        }

        let binary = self.provider.prepare(version)?;

        self.runner
            .run(binary.path(), version, &self.installation)?;

        self.installer.install(binary.path())?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SelfUpdateError {
    #[error("Aims version {0} is already installed")]
    AlreadyInstalled(Version),
    #[error(
        "downgrade is not allowed: installed version is {current}, requested version is {requested}"
    )]
    DowngradeNotAllowed {
        current: Version,
        requested: Version,
    },
    #[error("failed to read installation state")]
    Installation(#[from] InstallationError),
    #[error("failed to prepare target aimsctl binary")]
    Provider(#[from] TargetBinaryProviderError),
    #[error("failed to run target aimsctl binary")]
    Runner(#[from] TargetBinaryRunnerError),
    #[error("failed to install target aimsctl binary")]
    Installer(#[from] TargetBinaryInstallerError),
}

#[cfg(test)]
mod tests {
    use std::{
        cell::RefCell,
        path::{Path, PathBuf},
        rc::Rc,
    };

    use super::*;

    use crate::{
        self_update::{
            prepared_binary::PreparedBinary,
            provider::{TargetBinaryProvider, TargetBinaryProviderError},
            runner::{TargetBinaryRunner, TargetBinaryRunnerError},
        },
        test_support,
    };

    #[derive(Clone)]
    struct FakeTargetBinaryProvider {
        calls: Rc<RefCell<Vec<String>>>,
        result: Rc<RefCell<Result<PathBuf, ()>>>,
    }

    impl FakeTargetBinaryProvider {
        fn succeeding(binary: impl Into<PathBuf>) -> Self {
            Self {
                calls: Rc::new(RefCell::new(Vec::new())),
                result: Rc::new(RefCell::new(Ok(binary.into()))),
            }
        }

        fn failing() -> Self {
            Self {
                calls: Rc::new(RefCell::new(Vec::new())),
                result: Rc::new(RefCell::new(Err(()))),
            }
        }

        fn calls(&self) -> Vec<String> {
            self.calls.borrow().clone()
        }
    }

    impl TargetBinaryProvider for FakeTargetBinaryProvider {
        fn prepare(&self, version: &Version) -> Result<PreparedBinary, TargetBinaryProviderError> {
            self.calls.borrow_mut().push(format!("prepare:{version}"));

            match &*self.result.borrow() {
                Ok(path) => Ok(PreparedBinary::for_test(path.clone())),
                Err(()) => Err(TargetBinaryProviderError::UnsupportedArchitecture(
                    "test".to_string(),
                )),
            }
        }
    }

    #[derive(Clone)]
    struct FakeTargetBinaryRunner {
        calls: Rc<RefCell<Vec<(PathBuf, String)>>>,
        succeeds: bool,
    }

    impl FakeTargetBinaryRunner {
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

        fn calls(&self) -> Vec<(PathBuf, String)> {
            self.calls.borrow().clone()
        }
    }

    impl TargetBinaryRunner for FakeTargetBinaryRunner {
        fn run(
            &self,
            binary: &Path,
            version: &Version,
            _installation: &Installation,
        ) -> Result<(), TargetBinaryRunnerError> {
            self.calls
                .borrow_mut()
                .push((binary.to_path_buf(), version.to_string()));

            if !self.succeeds {
                return Err(TargetBinaryRunnerError::ProcessFailed(
                    std::process::ExitStatus::default(),
                ));
            }

            Ok(())
        }
    }

    #[derive(Clone)]
    struct FakeTargetBinaryInstaller {
        calls: Rc<RefCell<Vec<PathBuf>>>,
    }

    impl FakeTargetBinaryInstaller {
        fn new() -> Self {
            Self {
                calls: Rc::new(RefCell::new(Vec::new())),
            }
        }

        fn calls(&self) -> Vec<PathBuf> {
            self.calls.borrow().clone()
        }
    }

    impl TargetBinaryInstaller for FakeTargetBinaryInstaller {
        fn install(&self, binary: &Path) -> Result<(), TargetBinaryInstallerError> {
            self.calls.borrow_mut().push(binary.to_path_buf());

            Ok(())
        }
    }

    #[test]
    fn newer_version_prepares_runs_and_installs_target_binary() {
        let test_installation = test_support::installation("0.1.4");

        let provider = FakeTargetBinaryProvider::succeeding("/tmp/aimsctl-0.1.5");

        let runner = FakeTargetBinaryRunner::succeeding();
        let installer = FakeTargetBinaryInstaller::new();

        let updater = SelfUpdater::new(
            test_installation.installation,
            provider.clone(),
            runner.clone(),
            installer.clone(),
        );

        let version = Version::parse("0.1.5").unwrap();

        updater.update(&version).unwrap();

        assert_eq!(provider.calls(), ["prepare:0.1.5"]);

        assert_eq!(
            runner.calls(),
            vec![(PathBuf::from("/tmp/aimsctl-0.1.5"), "0.1.5".to_string(),)]
        );

        assert_eq!(installer.calls(), [PathBuf::from("/tmp/aimsctl-0.1.5")]);
    }

    #[test]
    fn same_version_is_rejected_before_preparing_binary() {
        let test_installation = test_support::installation("0.1.4");

        let provider = FakeTargetBinaryProvider::succeeding("/tmp/aimsctl-0.1.4");

        let runner = FakeTargetBinaryRunner::succeeding();
        let installer = FakeTargetBinaryInstaller::new();

        let updater = SelfUpdater::new(
            test_installation.installation,
            provider.clone(),
            runner.clone(),
            installer.clone(),
        );

        let version = Version::parse("0.1.4").unwrap();

        let result = updater.update(&version);

        assert!(matches!(result, Err(SelfUpdateError::AlreadyInstalled(_))));

        assert!(provider.calls().is_empty());
        assert!(runner.calls().is_empty());
        assert!(installer.calls().is_empty());
    }

    #[test]
    fn downgrade_is_rejected_before_preparing_binary() {
        let test_installation = test_support::installation("0.1.4");

        let provider = FakeTargetBinaryProvider::succeeding("/tmp/aimsctl-0.1.3");

        let runner = FakeTargetBinaryRunner::succeeding();
        let installer = FakeTargetBinaryInstaller::new();

        let updater = SelfUpdater::new(
            test_installation.installation,
            provider.clone(),
            runner.clone(),
            installer.clone(),
        );

        let version = Version::parse("0.1.3").unwrap();

        let result = updater.update(&version);

        assert!(matches!(
            result,
            Err(SelfUpdateError::DowngradeNotAllowed { .. })
        ));

        assert!(provider.calls().is_empty());
        assert!(runner.calls().is_empty());
        assert!(installer.calls().is_empty());
    }

    #[test]
    fn prepare_failure_does_not_run_or_install_target_binary() {
        let test_installation = test_support::installation("0.1.4");

        let provider = FakeTargetBinaryProvider::failing();
        let runner = FakeTargetBinaryRunner::succeeding();
        let installer = FakeTargetBinaryInstaller::new();

        let updater = SelfUpdater::new(
            test_installation.installation,
            provider.clone(),
            runner.clone(),
            installer.clone(),
        );

        let version = Version::parse("0.1.5").unwrap();

        let result = updater.update(&version);

        assert!(matches!(result, Err(SelfUpdateError::Provider(_))));

        assert_eq!(provider.calls(), ["prepare:0.1.5"]);
        assert!(runner.calls().is_empty());
        assert!(installer.calls().is_empty());
    }

    #[test]
    fn runner_failure_does_not_install_target_binary() {
        let test_installation = test_support::installation("0.1.4");

        let provider = FakeTargetBinaryProvider::succeeding("/tmp/aimsctl-0.1.5");

        let runner = FakeTargetBinaryRunner::failing();
        let installer = FakeTargetBinaryInstaller::new();

        let updater = SelfUpdater::new(
            test_installation.installation,
            provider.clone(),
            runner.clone(),
            installer.clone(),
        );

        let version = Version::parse("0.1.5").unwrap();

        let result = updater.update(&version);

        assert!(matches!(result, Err(SelfUpdateError::Runner(_))));

        assert_eq!(provider.calls(), ["prepare:0.1.5"]);

        assert_eq!(
            runner.calls(),
            vec![(PathBuf::from("/tmp/aimsctl-0.1.5"), "0.1.5".to_string(),)]
        );

        assert!(installer.calls().is_empty());
    }
}
