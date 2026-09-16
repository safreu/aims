mod config;
mod prompt;
mod store;
mod temporary;

pub(crate) use config::{
    AimsctlConfig, AimsctlConfigSection, DockerConfig, HttpConfig, InstallationConfig,
};

pub(crate) use store::{ConfigStore, ConfigStoreError};

pub(crate) use prompt::{ConfigPromptError, ConfigPrompter, ConsoleConfigPrompter};

pub(crate) use temporary::{TemporaryConfig, TemporaryConfigError};

pub(crate) const AIMSCTL_CONFIG_PATH: &str = "/etc/aims/aimsctl.toml";
