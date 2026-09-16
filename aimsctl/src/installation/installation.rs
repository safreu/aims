use std::{
    fs::{self, File},
    io::Write,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

use crate::installation::version::{Version, VersionParseError};

#[derive(Debug, Clone)]
pub struct Installation {
    root: PathBuf,
}

#[derive(Debug, Clone)]
pub struct EnvironmentSnapshot {
    content: String,
}

const COMPOSE_FILE_CONTENT: &str = include_str!("../../../compose.prod.yml");

impl Installation {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
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

    pub fn health_url(&self) -> Result<String, InstallationError> {
        let port = self.environment_value("AIMS_HTTP_PORT")?;

        Ok(format!("http://127.0.0.1:{port}/api/v1/health"))
    }

    pub fn create(&self) -> Result<(), InstallationError> {
        if self.root().exists() {
            return Err(InstallationError::AlreadyExists(self.root.clone()));
        }

        std::fs::create_dir_all(&self.root).map_err(InstallationError::CreateDirectory)
    }

    pub fn write_compose_file(&self) -> Result<(), InstallationError> {
        std::fs::write(self.compose_file(), COMPOSE_FILE_CONTENT)
            .map_err(InstallationError::WriteComposeFile)
    }

    pub fn write_environment(&self, content: &str) -> Result<(), InstallationError> {
        self.write_environment_file(content)
    }

    pub fn remove(&self) -> Result<(), InstallationError> {
        std::fs::remove_dir_all(&self.root).map_err(InstallationError::RemoveDirectory)
    }

    pub fn current_version(&self) -> Result<Version, InstallationError> {
        let content = fs::read_to_string(self.env_file()).map_err(InstallationError::ReadEnv)?;

        let value = content
            .lines()
            .find_map(|line| line.strip_prefix("AIMS_VERSION="))
            .ok_or(InstallationError::MissingEnvironmentVariable(
                "AIMS_VERSION".to_string(),
            ))?;

        Version::parse(value).map_err(InstallationError::InvalidVersion)
    }

    pub fn set_version(&self, version: &Version) -> Result<(), InstallationError> {
        self.set_environment_value("AIMS_VERSION", &version.to_string())
    }

    fn write_environment_file(&self, content: &str) -> Result<(), InstallationError> {
        let path = self.env_file();
        let temp_path = path.with_extension("prod.tmp");

        let mut temp_file = File::create(&temp_path).map_err(InstallationError::WriteEnv)?;

        temp_file
            .write_all(content.as_bytes())
            .map_err(InstallationError::WriteEnv)?;

        temp_file.sync_all().map_err(InstallationError::WriteEnv)?;

        fs::set_permissions(&temp_path, fs::Permissions::from_mode(0o600))
            .map_err(InstallationError::SetPermissions)?;

        fs::rename(&temp_path, &path).map_err(InstallationError::WriteEnv)?;

        let directory = File::open(&self.root).map_err(InstallationError::WriteEnv)?;

        directory.sync_all().map_err(InstallationError::WriteEnv)?;

        Ok(())
    }

    pub fn environment_snapshot(&self) -> Result<EnvironmentSnapshot, InstallationError> {
        let content = fs::read_to_string(self.env_file()).map_err(InstallationError::ReadEnv)?;

        Ok(EnvironmentSnapshot { content })
    }

    pub fn restore_environment(
        &self,
        snapshot: &EnvironmentSnapshot,
    ) -> Result<(), InstallationError> {
        self.write_environment_file(&snapshot.content)
    }
    pub(crate) fn environment_value(&self, key: &str) -> Result<String, InstallationError> {
        let content = fs::read_to_string(self.env_file()).map_err(InstallationError::ReadEnv)?;

        let prefix = format!("{key}=");

        content
            .lines()
            .find_map(|line| line.strip_prefix(&prefix))
            .map(str::to_owned)
            .ok_or_else(|| InstallationError::MissingEnvironmentVariable(key.to_owned()))
    }

    pub(crate) fn set_environment_value(
        &self,
        key: &str,
        value: &str,
    ) -> Result<(), InstallationError> {
        let content = fs::read_to_string(self.env_file()).map_err(InstallationError::ReadEnv)?;

        let prefix = format!("{key}=");
        let mut found = false;

        let updated = content
            .lines()
            .map(|line| {
                if line.starts_with(&prefix) {
                    found = true;
                    format!("{prefix}{value}")
                } else {
                    line.to_owned()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        if !found {
            return Err(InstallationError::MissingEnvironmentVariable(
                key.to_owned(),
            ));
        }

        self.write_environment_file(&format!("{updated}\n"))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InstallationError {
    #[error("installation already exists at {0}")]
    AlreadyExists(PathBuf),
    #[error("failed to create installation directory")]
    CreateDirectory(#[source] std::io::Error),
    #[error("failed to remove installation directory")]
    RemoveDirectory(#[source] std::io::Error),
    #[error("failed to write compose file")]
    WriteComposeFile(#[source] std::io::Error),
    #[error("failed to write environment file")]
    WriteEnv(#[source] std::io::Error),
    #[error("failed to set environment file permissions")]
    SetPermissions(#[source] std::io::Error),
    #[error("failed to read environment file")]
    ReadEnv(#[source] std::io::Error),
    #[error("environment variable {0} is missing")]
    MissingEnvironmentVariable(String),
    #[error("AIMS_VERSION in environment file is invalid")]
    InvalidVersion(#[source] VersionParseError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installation_directory_can_be_created() {
        let temp_dir = tempfile::tempdir().unwrap();
        let installation_root = temp_dir.path().join("aims");

        let installation = Installation::new(&installation_root);

        installation.create().unwrap();

        assert!(installation_root.is_dir());
    }

    #[test]
    fn existing_installation_directory_is_rejected() {
        let temp_dir = tempfile::tempdir().unwrap();
        let installation_root = temp_dir.path().join("aims");

        std::fs::create_dir(&installation_root).unwrap();

        let installation = Installation::new(&installation_root);

        let result = installation.create();

        assert!(matches!(
            result,
            Err(InstallationError::AlreadyExists(path))
                if path == installation_root
        ));
    }

    #[test]
    fn compose_file_can_be_written() {
        let temp_dir = tempfile::tempdir().unwrap();
        let installation_root = temp_dir.path().join("aims");

        let installation = Installation::new(&installation_root);

        installation.create().unwrap();
        installation.write_compose_file().unwrap();

        let content = std::fs::read_to_string(installation.compose_file()).unwrap();

        assert_eq!(content, COMPOSE_FILE_CONTENT);
    }

    #[test]
    fn environment_file_can_be_written() {
        let temp_dir = tempfile::tempdir().unwrap();
        let installation_root = temp_dir.path().join("aims");

        let installation = Installation::new(&installation_root);
        installation.create().unwrap();

        installation
            .write_environment("AIMS_VERSION=0.1.1\n")
            .unwrap();

        let content = std::fs::read_to_string(installation.env_file()).unwrap();

        assert_eq!(content, "AIMS_VERSION=0.1.1\n");
    }

    #[test]
    fn environment_file_has_restricted_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = tempfile::tempdir().unwrap();
        let installation_root = temp_dir.path().join("aims");

        let installation = Installation::new(&installation_root);
        installation.create().unwrap();

        installation
            .write_environment("AIMS_VERSION=0.1.1\n")
            .unwrap();

        let metadata = std::fs::metadata(installation.env_file()).unwrap();

        assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
    }

    #[test]
    fn version_can_be_updated() {
        let temp_dir = tempfile::tempdir().unwrap();

        let env_file = temp_dir.path().join(".env.prod");

        std::fs::write(&env_file, "AIMS_VERSION=0.1.0\nAPP_PORT=3000\n").unwrap();

        let installation = Installation::new(temp_dir.path());
        let version = Version::parse("0.2.0").unwrap();

        installation.set_version(&version).unwrap();

        let content = std::fs::read_to_string(env_file).unwrap();

        assert_eq!(content, "AIMS_VERSION=0.2.0\nAPP_PORT=3000\n");
    }

    #[test]
    fn missing_version_is_rejected() {
        let temp_dir = tempfile::tempdir().unwrap();

        std::fs::write(temp_dir.path().join(".env.prod"), "APP_PORT=3000\n").unwrap();

        let installation = Installation::new(temp_dir.path());
        let version = Version::parse("0.2.0").unwrap();

        let result = installation.set_version(&version);

        assert!(matches!(
            result,
            Err(InstallationError::MissingEnvironmentVariable(key))
                if key == "AIMS_VERSION"
        ));
    }

    #[test]
    fn current_version_can_be_read() {
        let temp_dir = tempfile::tempdir().unwrap();

        std::fs::write(
            temp_dir.path().join(".env.prod"),
            "AIMS_VERSION=0.1.0\nAPP_PORT=3000\n",
        )
        .unwrap();

        let installation = Installation::new(temp_dir.path());

        let version = installation.current_version().unwrap();

        assert_eq!(version, Version::parse("0.1.0").unwrap());
    }

    #[test]
    fn updated_environment_file_has_restricted_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = tempfile::tempdir().unwrap();
        let installation_root = temp_dir.path().join("aims");

        let installation = Installation::new(&installation_root);
        installation.create().unwrap();

        installation
            .write_environment("AIMS_VERSION=0.1.1\n")
            .unwrap();

        let version = Version::parse("0.1.2").unwrap();

        installation.set_version(&version).unwrap();

        let metadata = std::fs::metadata(installation.env_file()).unwrap();

        assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
    }

    #[test]
    fn environment_value_can_be_read() {
        let temp_dir = tempfile::tempdir().unwrap();

        std::fs::write(
            temp_dir.path().join(".env.prod"),
            "AIMS_VERSION=0.1.4\nAIMS_HTTP_PORT=8080\n",
        )
        .unwrap();

        let installation = Installation::new(temp_dir.path());

        let value = installation.environment_value("AIMS_HTTP_PORT").unwrap();

        assert_eq!(value, "8080");
    }

    #[test]
    fn environment_value_can_be_updated() {
        let temp_dir = tempfile::tempdir().unwrap();

        std::fs::write(
            temp_dir.path().join(".env.prod"),
            "AIMS_VERSION=0.1.4\nAIMS_HTTP_PORT=8080\n",
        )
        .unwrap();

        let installation = Installation::new(temp_dir.path());

        installation
            .set_environment_value("AIMS_HTTP_PORT", "80")
            .unwrap();

        let content = std::fs::read_to_string(installation.env_file()).unwrap();

        assert_eq!(content, "AIMS_VERSION=0.1.4\nAIMS_HTTP_PORT=80\n");
    }

    #[test]
    fn environment_snapshot_can_be_restored() {
        let temp_dir = tempfile::tempdir().unwrap();

        let original = "AIMS_VERSION=0.1.4\nAIMS_HTTP_PORT=8080\nDATABASE_USER=aims\n";

        std::fs::write(temp_dir.path().join(".env.prod"), original).unwrap();

        let installation = Installation::new(temp_dir.path());

        let snapshot = installation.environment_snapshot().unwrap();

        installation
            .set_environment_value("AIMS_VERSION", "0.1.5")
            .unwrap();

        installation
            .set_environment_value("AIMS_HTTP_PORT", "80")
            .unwrap();

        installation.restore_environment(&snapshot).unwrap();

        let content = std::fs::read_to_string(installation.env_file()).unwrap();

        assert_eq!(content, original);
    }

    #[test]
    fn health_url_is_derived_from_environment() {
        let temp_dir = tempfile::tempdir().unwrap();

        std::fs::write(temp_dir.path().join(".env.prod"), "AIMS_HTTP_PORT=80\n").unwrap();

        let installation = Installation::new(temp_dir.path());

        assert_eq!(
            installation.health_url().unwrap(),
            "http://127.0.0.1:80/api/v1/health"
        );
    }

    #[test]
    fn missing_http_port_is_rejected_when_building_health_url() {
        let temp_dir = tempfile::tempdir().unwrap();

        std::fs::write(temp_dir.path().join(".env.prod"), "AIMS_VERSION=0.1.5\n").unwrap();

        let installation = Installation::new(temp_dir.path());

        let result = installation.health_url();

        assert!(matches!(
            result,
            Err(InstallationError::MissingEnvironmentVariable(key))
                if key == "AIMS_HTTP_PORT"
        ));
    }
}
