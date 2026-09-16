use std::path::PathBuf;

use dialoguer::Input;

use crate::configuration::AimsctlConfig;

pub(crate) trait ConfigPrompter {
    fn prompt(&self, defaults: &AimsctlConfig) -> Result<AimsctlConfig, ConfigPromptError>;
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ConfigPromptError {
    #[error("failed to read configuration")]
    Input(#[source] dialoguer::Error),

    #[cfg(test)]
    #[error("test prompt failure")]
    Test,
}

pub(crate) struct ConsoleConfigPrompter;

impl ConfigPrompter for ConsoleConfigPrompter {
    fn prompt(&self, defaults: &AimsctlConfig) -> Result<AimsctlConfig, ConfigPromptError> {
        let installation_root: String = Input::new()
            .with_prompt("Installation root")
            .default(defaults.installation.root.display().to_string())
            .interact_text()
            .map_err(ConfigPromptError::Input)?;

        let aimsctl_install_path: String = Input::new()
            .with_prompt("aimsctl install path")
            .default(defaults.aimsctl.install_path.display().to_string())
            .interact_text()
            .map_err(ConfigPromptError::Input)?;

        let compose_project: String = Input::new()
            .with_prompt("Docker compose project")
            .default(defaults.docker.compose_project.clone())
            .interact_text()
            .map_err(ConfigPromptError::Input)?;

        let postgres_volume: String = Input::new()
            .with_prompt("PostgreSQL volume")
            .default(defaults.docker.postgres_volume.clone())
            .interact_text()
            .map_err(ConfigPromptError::Input)?;

        let http_port: u16 = Input::new()
            .with_prompt("HTTP port")
            .default(defaults.http.port)
            .interact_text()
            .map_err(ConfigPromptError::Input)?;

        Ok(AimsctlConfig {
            installation: super::InstallationConfig {
                root: PathBuf::from(installation_root),
            },
            aimsctl: super::AimsctlConfigSection {
                install_path: PathBuf::from(aimsctl_install_path),
            },
            docker: super::DockerConfig {
                compose_project,
                postgres_volume,
            },
            http: super::HttpConfig { port: http_port },
        })
    }
}
