use std::{io::Write, path::Path};

use crate::configuration::AimsctlConfig;

pub(crate) struct TemporaryConfig {
    file: tempfile::NamedTempFile,
}

impl TemporaryConfig {
    pub(crate) fn create(config: &AimsctlConfig) -> Result<Self, TemporaryConfigError> {
        let contents = toml::to_string_pretty(config)?;

        let mut file = tempfile::NamedTempFile::new()?;

        file.write_all(contents.as_bytes())?;

        Ok(Self { file })
    }

    pub(crate) fn path(&self) -> &Path {
        self.file.path()
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum TemporaryConfigError {
    #[error("failed to serialize temporary configuration")]
    Serialize(#[from] toml::ser::Error),

    #[error("failed to create or write temporary configuration")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temporary_config_contains_serialized_configuration() {
        let config = AimsctlConfig::default();

        let temporary = TemporaryConfig::create(&config).unwrap();

        let contents = std::fs::read_to_string(temporary.path()).unwrap();
        let loaded: AimsctlConfig = toml::from_str(&contents).unwrap();

        assert_eq!(loaded, config);
    }

    #[test]
    fn temporary_config_is_removed_when_dropped() {
        let config = AimsctlConfig::default();

        let path = {
            let temporary = TemporaryConfig::create(&config).unwrap();
            let path = temporary.path().to_path_buf();

            assert!(path.exists());

            path
        };

        assert!(!path.exists());
    }
}
