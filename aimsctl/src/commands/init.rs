use std::path::Path;

use crate::configuration::{
    AimsctlConfig, ConfigPromptError, ConfigPrompter, ConfigStore, TemporaryConfig,
    TemporaryConfigError,
};

pub(crate) trait InitApplier {
    fn apply(&self, config_path: &Path, destination_path: &Path) -> Result<(), InitApplyError>;
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum InitApplyError {
    #[error("failed to determine current aimsctl executable")]
    CurrentExecutable(#[source] std::io::Error),

    #[error("failed to execute privileged initialization")]
    Process(#[source] std::io::Error),

    #[error("privileged initialization failed")]
    Failed,
}

pub(crate) struct Init<P, A>
where
    P: ConfigPrompter,
    A: InitApplier,
{
    config_store: ConfigStore,
    prompter: P,
    applier: A,
}

impl<P, A> Init<P, A>
where
    P: ConfigPrompter,
    A: InitApplier,
{
    pub(crate) fn new(config_store: ConfigStore, prompter: P, applier: A) -> Self {
        Self {
            config_store,
            prompter,
            applier,
        }
    }

    pub(crate) fn init(&self) -> Result<(), InitError> {
        if self.config_store.exists() {
            return Err(InitError::AlreadyInitialized);
        }

        let defaults = AimsctlConfig::default();

        let config = self.prompter.prompt(&defaults)?;

        let temporary_config = TemporaryConfig::create(&config)?;

        self.applier
            .apply(temporary_config.path(), self.config_store.path())?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum InitError {
    #[error("aimsctl is already initialized")]
    AlreadyInitialized,

    #[error("failed to collect aimsctl configuration")]
    Prompt(#[from] ConfigPromptError),

    #[error("failed to create temporary aimsctl configuration")]
    TemporaryConfig(#[from] TemporaryConfigError),

    #[error("failed to apply aimsctl initialization")]
    Apply(#[from] InitApplyError),
}

#[cfg(test)]
mod tests {
    use std::{
        cell::RefCell,
        path::{Path, PathBuf},
        rc::Rc,
    };

    use super::*;

    #[derive(Clone)]
    struct FakeConfigPrompter {
        config: AimsctlConfig,
    }

    impl FakeConfigPrompter {
        fn new(config: AimsctlConfig) -> Self {
            Self { config }
        }
    }

    impl ConfigPrompter for FakeConfigPrompter {
        fn prompt(&self, _defaults: &AimsctlConfig) -> Result<AimsctlConfig, ConfigPromptError> {
            Ok(self.config.clone())
        }
    }

    struct FailingConfigPrompter;

    impl ConfigPrompter for FailingConfigPrompter {
        fn prompt(&self, _defaults: &AimsctlConfig) -> Result<AimsctlConfig, ConfigPromptError> {
            Err(ConfigPromptError::Test)
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct ApplyCall {
        config: AimsctlConfig,
        destination_path: PathBuf,
    }

    #[derive(Clone)]
    struct FakeInitApplier {
        calls: Rc<RefCell<Vec<ApplyCall>>>,
        should_fail: bool,
    }

    impl FakeInitApplier {
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

        fn calls(&self) -> Vec<ApplyCall> {
            self.calls.borrow().clone()
        }
    }

    impl InitApplier for FakeInitApplier {
        fn apply(&self, config_path: &Path, destination_path: &Path) -> Result<(), InitApplyError> {
            let contents = std::fs::read_to_string(config_path).unwrap();
            let config: AimsctlConfig = toml::from_str(&contents).unwrap();

            self.calls.borrow_mut().push(ApplyCall {
                config,
                destination_path: destination_path.to_path_buf(),
            });

            if self.should_fail {
                return Err(InitApplyError::Failed);
            }

            Ok(())
        }
    }

    #[test]
    fn init_applies_prompted_configuration() {
        let temp_dir = tempfile::tempdir().unwrap();
        let destination_path = temp_dir.path().join("aimsctl.toml");

        let mut config = AimsctlConfig::default();
        config.installation.root = PathBuf::from("/srv/aims");
        config.aimsctl.install_path = PathBuf::from("/custom/bin/aimsctl");
        config.docker.compose_project = "custom-project".to_owned();
        config.docker.postgres_volume = "custom-volume".to_owned();
        config.http.port = 12345;

        let applier = FakeInitApplier::succeeding();

        let init = Init::new(
            ConfigStore::new(&destination_path),
            FakeConfigPrompter::new(config.clone()),
            applier.clone(),
        );

        init.init().unwrap();

        assert_eq!(
            applier.calls(),
            vec![ApplyCall {
                config,
                destination_path,
            }]
        );
    }

    #[test]
    fn init_does_not_write_permanent_configuration() {
        let temp_dir = tempfile::tempdir().unwrap();
        let destination_path = temp_dir.path().join("aimsctl.toml");

        let applier = FakeInitApplier::succeeding();

        let init = Init::new(
            ConfigStore::new(&destination_path),
            FakeConfigPrompter::new(AimsctlConfig::default()),
            applier,
        );

        init.init().unwrap();

        assert!(!destination_path.exists());
    }

    #[test]
    fn existing_configuration_prevents_initialization() {
        let temp_dir = tempfile::tempdir().unwrap();
        let destination_path = temp_dir.path().join("aimsctl.toml");

        let store = ConfigStore::new(&destination_path);

        let mut existing = AimsctlConfig::default();
        existing.http.port = 8080;

        store.save(&existing).unwrap();

        let applier = FakeInitApplier::succeeding();

        let init = Init::new(
            ConfigStore::new(&destination_path),
            FakeConfigPrompter::new(AimsctlConfig::default()),
            applier.clone(),
        );

        let result = init.init();

        assert!(matches!(result, Err(InitError::AlreadyInitialized)));

        let stored = ConfigStore::new(&destination_path).load().unwrap();

        assert_eq!(stored, existing);
        assert!(applier.calls().is_empty());
    }

    #[test]
    fn prompt_failure_does_not_apply_initialization() {
        let temp_dir = tempfile::tempdir().unwrap();
        let destination_path = temp_dir.path().join("aimsctl.toml");

        let applier = FakeInitApplier::succeeding();

        let init = Init::new(
            ConfigStore::new(&destination_path),
            FailingConfigPrompter,
            applier.clone(),
        );

        let result = init.init();

        assert!(matches!(result, Err(InitError::Prompt(_))));
        assert!(!destination_path.exists());
        assert!(applier.calls().is_empty());
    }

    #[test]
    fn apply_failure_is_returned() {
        let temp_dir = tempfile::tempdir().unwrap();
        let destination_path = temp_dir.path().join("aimsctl.toml");

        let config = AimsctlConfig::default();
        let applier = FakeInitApplier::failing();

        let init = Init::new(
            ConfigStore::new(&destination_path),
            FakeConfigPrompter::new(config.clone()),
            applier.clone(),
        );

        let result = init.init();

        assert!(matches!(result, Err(InitError::Apply(_))));

        assert_eq!(
            applier.calls(),
            vec![ApplyCall {
                config,
                destination_path: destination_path.clone(),
            }]
        );

        assert!(!destination_path.exists());
    }
}
