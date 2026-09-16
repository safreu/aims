use std::path::PathBuf;

use serde::{Deserialize, Serialize};

const DEFAULT_INSTALLATION_ROOT: &str = "/opt/aims";
const DEFAULT_AIMSCTL_INSTALL_PATH: &str = "/usr/local/bin/aimsctl";
const DEFAULT_COMPOSE_PROJECT: &str = "aims-prod";
const DEFAULT_POSTGRES_VOLUME: &str = "aims_prod_postgres_data";
const DEFAULT_HTTP_PORT: u16 = 80;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct AimsctlConfig {
    pub installation: InstallationConfig,
    pub aimsctl: AimsctlConfigSection,
    pub docker: DockerConfig,
    pub http: HttpConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct InstallationConfig {
    pub root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct AimsctlConfigSection {
    pub install_path: PathBuf,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DockerConfig {
    pub compose_project: String,
    pub postgres_volume: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct HttpConfig {
    pub port: u16,
}

impl Default for AimsctlConfig {
    fn default() -> Self {
        Self {
            installation: InstallationConfig {
                root: PathBuf::from(DEFAULT_INSTALLATION_ROOT),
            },
            aimsctl: AimsctlConfigSection {
                install_path: PathBuf::from(DEFAULT_AIMSCTL_INSTALL_PATH),
            },
            docker: DockerConfig {
                compose_project: DEFAULT_COMPOSE_PROJECT.to_owned(),
                postgres_volume: DEFAULT_POSTGRES_VOLUME.to_owned(),
            },
            http: HttpConfig {
                port: DEFAULT_HTTP_PORT,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configuration_can_be_serialized_and_deserialized() {
        let config = AimsctlConfig::default();

        let serialized = toml::to_string_pretty(&config).unwrap();

        let deserialized: AimsctlConfig = toml::from_str(&serialized).unwrap();

        assert_eq!(deserialized, config);
    }

    #[test]
    fn default_configuration_uses_standard_paths_and_values() {
        let config = AimsctlConfig::default();

        assert_eq!(config.installation.root, PathBuf::from("/opt/aims"));

        assert_eq!(
            config.aimsctl.install_path,
            PathBuf::from("/usr/local/bin/aimsctl")
        );

        assert_eq!(config.docker.compose_project, "aims-prod");

        assert_eq!(config.docker.postgres_volume, "aims_prod_postgres_data");

        assert_eq!(config.http.port, 80);
    }
}
