use std::{
    fs,
    path::{Path, PathBuf},
};

pub(crate) struct ExecutableManager {
    destination: PathBuf,
}

impl ExecutableManager {
    pub(crate) fn new(destination: impl Into<PathBuf>) -> Self {
        Self {
            destination: destination.into(),
        }
    }

    pub(crate) fn install(&self) -> Result<(), ExecutableManagerError> {
        let current_executable =
            std::env::current_exe().map_err(ExecutableManagerError::CurrentExecutable)?;

        self.install_from(&current_executable)
    }

    fn install_from(&self, source: &Path) -> Result<(), ExecutableManagerError> {
        if self.destination.exists() {
            return Err(ExecutableManagerError::AlreadyExists(
                self.destination.clone(),
            ));
        }

        if let Some(parent) = self.destination.parent() {
            std::fs::create_dir_all(parent)
                .map_err(ExecutableManagerError::CreateInstallDirectory)?;
        }

        fs::copy(source, &self.destination).map_err(ExecutableManagerError::Copy)?;

        Ok(())
    }

    pub(crate) fn uninstall(&self) -> Result<(), ExecutableManagerError> {
        std::fs::remove_file(&self.destination).map_err(ExecutableManagerError::Remove)?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ExecutableManagerError {
    #[error("failed to determine current aimsctl executable")]
    CurrentExecutable(#[source] std::io::Error),
    #[error("failed to install aimsctl executable")]
    Copy(#[source] std::io::Error),
    #[error("failed to remove installed aimsctl executable")]
    Remove(#[source] std::io::Error),
    #[error("aimsctl executable already exists at {0}")]
    AlreadyExists(PathBuf),
    #[error("failed to create aimsctl installation directory")]
    CreateInstallDirectory(#[source] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn executable_is_copied_to_destination() {
        let temp_dir = tempfile::tempdir().unwrap();

        let source = temp_dir.path().join("source-aimsctl");
        let destination = temp_dir.path().join("installed-aimsctl");

        fs::write(&source, b"test aimsctl binary").unwrap();

        let installer = ExecutableManager::new(&destination);

        installer.install_from(&source).unwrap();

        assert_eq!(fs::read(&destination).unwrap(), b"test aimsctl binary");
    }

    #[test]
    fn missing_source_is_rejected() {
        let temp_dir = tempfile::tempdir().unwrap();

        let source = temp_dir.path().join("missing-aimsctl");
        let destination = temp_dir.path().join("installed-aimsctl");

        let installer = ExecutableManager::new(&destination);

        let result = installer.install_from(&source);

        assert!(matches!(result, Err(ExecutableManagerError::Copy(_))));

        assert!(!destination.exists());
    }

    #[test]
    fn existing_destination_is_not_overwritten() {
        let temp_dir = tempfile::tempdir().unwrap();

        let source = temp_dir.path().join("source-aimsctl");
        let destination = temp_dir.path().join("installed-aimsctl");

        fs::write(&source, b"new aimsctl").unwrap();
        fs::write(&destination, b"existing aimsctl").unwrap();

        let manager = ExecutableManager::new(&destination);

        let result = manager.install_from(&source);

        assert!(matches!(
            result,
            Err(ExecutableManagerError::AlreadyExists(path))
                if path == destination
        ));

        assert_eq!(fs::read(&destination).unwrap(), b"existing aimsctl");
    }

    #[test]
    fn installed_executable_can_be_removed() {
        let temp_dir = tempfile::tempdir().unwrap();

        let destination = temp_dir.path().join("aimsctl");

        fs::write(&destination, b"aimsctl").unwrap();

        let manager = ExecutableManager::new(&destination);

        manager.uninstall().unwrap();

        assert!(!destination.exists());
    }

    #[test]
    fn install_creates_missing_parent_directory() {
        let temp_dir = tempfile::tempdir().unwrap();

        let source = temp_dir.path().join("source-aimsctl");
        std::fs::write(&source, b"test binary").unwrap();

        let destination = temp_dir.path().join("nested").join("bin").join("aimsctl");

        let parent = destination.parent().unwrap();

        assert!(!parent.exists());

        let manager = ExecutableManager::new(destination.clone());

        manager.install_from(&source).unwrap();

        assert!(parent.is_dir());
        assert!(destination.is_file());
        assert_eq!(std::fs::read(&destination).unwrap(), b"test binary");
    }
}
