use crate::{
    installation::{Installation, InstallationError},
    progress::ProgressReporter,
    runtime::docker::{ContainerRuntime, DockerComposeError},
};

pub struct Uninstall<R, P>
where
    R: ContainerRuntime,
    P: ProgressReporter,
{
    installation: Installation,
    runtime: R,
    progress: P,
}

impl<R, P> Uninstall<R, P>
where
    R: ContainerRuntime,
    P: ProgressReporter,
{
    pub fn new(installation: Installation, runtime: R, progress: P) -> Self {
        Self {
            installation,
            runtime,
            progress,
        }
    }

    pub fn uninstall(&self) -> Result<(), UninstallError> {
        const TOTAL_STEPS: usize = 2;

        self.progress.header("Aims uninstaller");

        self.progress
            .step(1, TOTAL_STEPS, "Stopping and removing services");

        self.runtime.down_with_volumes(&self.installation)?;

        self.progress.detail("Services and database data removed");

        self.progress
            .step(2, TOTAL_STEPS, "Removing installation files");

        self.installation.remove()?;

        self.progress.detail("Installation files removed");

        self.progress.success("Aims uninstalled successfully");

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UninstallError {
    #[error("Docker compose operation failed")]
    Docker(#[from] DockerComposeError),
    #[error("failed to remove Aims installation")]
    Installation(#[from] InstallationError),
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_support::{FakeRuntime, TestProgressReporter, docker_failure};

    #[test]
    fn uninstall_removes_services_database_data_and_installation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let installation_root = temp_dir.path().join("aims");

        std::fs::create_dir_all(&installation_root).unwrap();

        let installation = Installation::new(&installation_root);
        let runtime = FakeRuntime::new();

        let uninstaller = Uninstall::new(installation, runtime.clone(), TestProgressReporter);

        uninstaller.uninstall().unwrap();

        assert_eq!(runtime.calls(), ["down_with_volumes"]);
        assert!(!installation_root.exists());
    }

    #[test]
    fn docker_failure_keeps_installation_files() {
        let temp_dir = tempfile::tempdir().unwrap();
        let installation_root = temp_dir.path().join("aims");

        std::fs::create_dir_all(&installation_root).unwrap();

        let installation = Installation::new(&installation_root);
        let runtime = FakeRuntime::new();

        runtime.push_down_with_volumes_result(Err(docker_failure()));

        let uninstaller = Uninstall::new(installation, runtime.clone(), TestProgressReporter);

        let result = uninstaller.uninstall();

        assert!(matches!(result, Err(UninstallError::Docker(_))));

        assert_eq!(runtime.calls(), ["down_with_volumes"]);
        assert!(installation_root.exists());
    }
}
