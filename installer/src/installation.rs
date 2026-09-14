use std::{
    fs::{self, File},
    io::Write,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct Installation {
    root: PathBuf,
    health_url: String,
}

const COMPOSE_FILE_CONTENT: &str = include_str!("../../compose.prod.yml");

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

    pub fn remove(&self) -> Result<(), InstallationError> {
        std::fs::remove_dir_all(&self.root).map_err(InstallationError::RemoveDirectory)
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installation_directory_can_be_created() {
        let temp_dir = tempfile::tempdir().unwrap();
        let installation_root = temp_dir.path().join("aims");

        let installation = Installation::new(&installation_root, "");

        installation.create().unwrap();

        assert!(installation_root.is_dir());
    }

    #[test]
    fn existing_installation_directory_is_rejected() {
        let temp_dir = tempfile::tempdir().unwrap();
        let installation_root = temp_dir.path().join("aims");

        std::fs::create_dir(&installation_root).unwrap();

        let installation = Installation::new(&installation_root, "");

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

        let installation = Installation::new(&installation_root, "");

        installation.create().unwrap();
        installation.write_compose_file().unwrap();

        let content = std::fs::read_to_string(installation.compose_file()).unwrap();

        assert_eq!(content, COMPOSE_FILE_CONTENT);
    }

    #[test]
    fn environment_file_can_be_written() {
        let temp_dir = tempfile::tempdir().unwrap();
        let installation_root = temp_dir.path().join("aims");

        let installation = Installation::new(&installation_root, "");
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

        let installation = Installation::new(&installation_root, "");
        installation.create().unwrap();

        installation
            .write_environment("AIMS_VERSION=0.1.1\n")
            .unwrap();

        let metadata = std::fs::metadata(installation.env_file()).unwrap();

        assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
    }
}
