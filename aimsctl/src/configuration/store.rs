use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use crate::configuration::config::AimsctlConfig;

pub(crate) struct ConfigStore {
    path: PathBuf,
}

impl ConfigStore {
    pub(crate) fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn exists(&self) -> bool {
        self.path.is_file()
    }

    pub(crate) fn load(&self) -> Result<AimsctlConfig, ConfigStoreError> {
        let contents = fs::read_to_string(&self.path).map_err(ConfigStoreError::Read)?;

        toml::from_str(&contents).map_err(ConfigStoreError::Deserialize)
    }

    #[allow(unused)]
    pub(crate) fn save(&self, config: &AimsctlConfig) -> Result<(), ConfigStoreError> {
        let contents = toml::to_string_pretty(config).map_err(ConfigStoreError::Serialize)?;

        if let Some(parent) = self.path().parent() {
            fs::create_dir_all(parent).map_err(ConfigStoreError::CreateDirectory)?;
        }

        fs::write(&self.path, contents).map_err(ConfigStoreError::Write)
    }

    pub(crate) fn create(&self, config: &AimsctlConfig) -> Result<(), ConfigStoreError> {
        let contents = toml::to_string_pretty(config).map_err(ConfigStoreError::Serialize)?;

        if let Some(parent) = self.path().parent() {
            fs::create_dir_all(parent).map_err(ConfigStoreError::CreateDirectory)?;
        }

        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(self.path())
            .map_err(|error| {
                if error.kind() == std::io::ErrorKind::AlreadyExists {
                    ConfigStoreError::AlreadyExists
                } else {
                    ConfigStoreError::Write(error)
                }
            })?;

        if let Err(write_error) = file.write_all(contents.as_bytes()) {
            let _ = fs::remove_file(&self.path);

            return Err(ConfigStoreError::Write(write_error));
        }

        Ok(())
    }

    pub(crate) fn remove(&self) -> Result<(), ConfigStoreError> {
        fs::remove_file(&self.path).map_err(ConfigStoreError::Remove)
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ConfigStoreError {
    #[error("failed to read aimsctl configuration")]
    Read(#[source] std::io::Error),
    #[error("failed to deserialize aimsctl configuration")]
    Deserialize(#[source] toml::de::Error),
    #[error("failed to serialize aimsctl configuration")]
    Serialize(#[source] toml::ser::Error),
    #[error("failed to create aimsctl configuration directory")]
    CreateDirectory(#[source] std::io::Error),
    #[error("failed to write aimsctl configuration")]
    Write(#[source] std::io::Error),
    #[error("failed to remove aimsctl configuration")]
    Remove(#[source] std::io::Error),
    #[error("aimsctl configuration already exists")]
    AlreadyExists,
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::configuration::{
        AimsctlConfigSection, DockerConfig, HttpConfig, InstallationConfig,
    };

    fn test_config() -> AimsctlConfig {
        AimsctlConfig {
            installation: InstallationConfig {
                root: PathBuf::from("/opt/aims"),
            },
            aimsctl: AimsctlConfigSection {
                install_path: PathBuf::from("/usr/local/bin/aimsctl"),
            },
            docker: DockerConfig {
                compose_project: "aims-prod".to_owned(),
                postgres_volume: "aims_prod_postgres_data".to_owned(),
            },
            http: HttpConfig { port: 80 },
        }
    }

    #[test]
    fn configuration_can_be_saved_and_loaded() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("aimsctl.toml");

        let store = ConfigStore::new(&path);
        let config = test_config();

        store.save(&config).unwrap();

        let loaded = store.load().unwrap();

        assert_eq!(loaded, config);
    }

    #[test]
    fn save_creates_missing_parent_directories() {
        let temp_dir = tempfile::tempdir().unwrap();

        let path = temp_dir
            .path()
            .join("etc")
            .join("aims")
            .join("aimsctl.toml");

        let store = ConfigStore::new(&path);

        assert!(!path.parent().unwrap().exists());

        store.save(&test_config()).unwrap();

        assert!(path.is_file());
    }

    #[test]
    fn exists_reports_existing_configuration() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("aimsctl.toml");

        let store = ConfigStore::new(&path);

        assert!(!store.exists());

        store.save(&test_config()).unwrap();

        assert!(store.exists());
    }

    #[test]
    fn missing_configuration_cannot_be_loaded() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("missing.toml");

        let store = ConfigStore::new(path);

        let result = store.load();

        assert!(matches!(result, Err(ConfigStoreError::Read(_))));
    }

    #[test]
    fn invalid_configuration_cannot_be_loaded() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("aimsctl.toml");

        fs::write(
            &path,
            r#"
            [installation
            root = "/opt/aims"
            "#,
        )
        .unwrap();

        let store = ConfigStore::new(path);

        let result = store.load();

        assert!(matches!(result, Err(ConfigStoreError::Deserialize(_))));
    }

    #[test]
    fn configuration_can_be_created() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("aimsctl.toml");

        let store = ConfigStore::new(&path);
        let config = test_config();

        store.create(&config).unwrap();

        let loaded = store.load().unwrap();

        assert_eq!(loaded, config);
    }

    #[test]
    fn create_does_not_overwrite_existing_configuration() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("aimsctl.toml");

        let store = ConfigStore::new(&path);

        let mut existing = test_config();
        existing.http.port = 8080;

        store.save(&existing).unwrap();

        let mut replacement = test_config();
        replacement.http.port = 12345;

        let result = store.create(&replacement);

        assert!(matches!(result, Err(ConfigStoreError::AlreadyExists)));

        let loaded = store.load().unwrap();

        assert_eq!(loaded, existing);
    }
}
