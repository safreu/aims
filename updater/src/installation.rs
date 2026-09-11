use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::version::Version;

#[derive(Debug, Clone)]
pub struct Installation {
    root: PathBuf,
    health_url: String,
}

impl Installation {
    pub fn new(root: impl Into<PathBuf>, health_url: impl Into<String>) -> Self {
        Self {
            root: root.into(),
            health_url: health_url.into(),
        }
    }

    pub fn env_file(&self) -> PathBuf {
        self.root.join(".env.prod")
    }

    pub fn compose_file(&self) -> PathBuf {
        self.root.join("compose.prod.yml")
    }

    #[allow(unused)]
    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn health_url(&self) -> &str {
        &self.health_url
    }

    pub fn current_version(&self) -> Result<Version, InstallationError> {
        let content = fs::read_to_string(self.env_file()).map_err(InstallationError::ReadEnv)?;

        let value = content
            .lines()
            .find_map(|line| line.strip_prefix("AIMS_VERSION="))
            .ok_or(InstallationError::MissingVersion)?;

        Version::parse(value).map_err(InstallationError::InvalidVersion)
    }

    pub fn set_version(&self, version: &Version) -> Result<(), InstallationError> {
        let path = self.env_file();

        let content = fs::read_to_string(&path).map_err(InstallationError::ReadEnv)?;

        let mut found = false;

        let updated = content
            .lines()
            .map(|line| {
                if line.starts_with("AIMS_VERSION=") {
                    found = true;
                    format!("AIMS_VERSION={version}")
                } else {
                    line.to_owned()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        if !found {
            return Err(InstallationError::MissingVersion);
        }

        fs::write(path, format!("{updated}\n")).map_err(InstallationError::WriteEnv)?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InstallationError {
    #[error("failed to read environment file")]
    ReadEnv(#[source] std::io::Error),
    #[error("failed to write environment file")]
    WriteEnv(#[source] std::io::Error),
    #[error("AIMS_VERSION is missing from environment file")]
    MissingVersion,
    #[error("AIMS_VERSION in environment file is invalid")]
    InvalidVersion(#[source] crate::version::VersionParseError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_file_is_inside_installation_root() {
        let installation = Installation::new("/opt/aims", "");

        assert_eq!(
            installation.env_file(),
            PathBuf::from("/opt/aims/.env.prod")
        )
    }

    #[test]
    fn compose_file_is_inside_installation_root() {
        let installation = Installation::new("/opt/aims", "");

        assert_eq!(
            installation.compose_file(),
            PathBuf::from("/opt/aims/compose.prod.yml")
        )
    }

    #[test]
    fn version_can_be_updated() {
        let temp_dir = tempfile::tempdir().unwrap();

        let env_file = temp_dir.path().join(".env.prod");

        std::fs::write(&env_file, "AIMS_VERSION=0.1.0\nAPP_PORT=3000\n").unwrap();

        let installation = Installation::new(temp_dir.path(), "");
        let version = Version::parse("0.2.0").unwrap();

        installation.set_version(&version).unwrap();

        let content = std::fs::read_to_string(env_file).unwrap();

        assert_eq!(content, "AIMS_VERSION=0.2.0\nAPP_PORT=3000\n");
    }

    #[test]
    fn missing_version_is_rejected() {
        let temp_dir = tempfile::tempdir().unwrap();

        std::fs::write(temp_dir.path().join(".env.prod"), "APP_PORT=3000\n").unwrap();

        let installation = Installation::new(temp_dir.path(), "");
        let version = Version::parse("0.2.0").unwrap();

        let result = installation.set_version(&version);

        assert!(matches!(result, Err(InstallationError::MissingVersion)));
    }

    #[test]
    fn current_version_can_be_read() {
        let temp_dir = tempfile::tempdir().unwrap();

        std::fs::write(
            temp_dir.path().join(".env.prod"),
            "AIMS_VERSION=0.1.0\nAPP_PORT=3000\n",
        )
        .unwrap();

        let installation = Installation::new(temp_dir.path(), "");

        let version = installation.current_version().unwrap();

        assert_eq!(version, Version::parse("0.1.0").unwrap());
    }
}
