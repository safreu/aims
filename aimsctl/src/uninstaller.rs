use crate::{
    docker::{ContainerRuntime, DockerComposeError},
    installation::{Installation, InstallationError},
    progress::ProgressReporter,
};

pub struct Uninstaller<R, P>
where
    R: ContainerRuntime,
    P: ProgressReporter,
{
    installation: Installation,
    runtime: R,
    progress: P,
}

impl<R, P> Uninstaller<R, P>
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

    pub fn uninstall(&self) -> Result<(), UninstallerError> {
        const TOTAL_STEPS: usize = 2;

        self.progress.header("Aims uninstaller");

        self.progress
            .step(1, TOTAL_STEPS, "Stopping and removing services");

        self.runtime.down(&self.installation)?;

        self.progress.detail("Services removed");

        self.progress
            .step(2, TOTAL_STEPS, "Stopping and removing services");

        self.installation.remove()?;

        self.progress.detail("Installation files removed");

        self.progress
            .success("Aims uninstalled successfully\nDatabase data has been preserved");

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UninstallerError {
    #[error("Docker compose operation failed")]
    Docker(#[from] DockerComposeError),
    #[error("failed to remove Aims installation")]
    Installation(#[from] InstallationError),
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{cell::RefCell, rc::Rc};

    use crate::{docker::ServiceStatus, version::Version};

    struct FakeRuntime {
        calls: Rc<RefCell<Vec<String>>>,
        down_result: Result<(), DockerComposeError>,
    }

    impl ContainerRuntime for FakeRuntime {
        fn pull(
            &self,
            _installation: &Installation,
            _version: &Version,
        ) -> Result<(), DockerComposeError> {
            panic!("pull must not be called by uninstaller");
        }

        fn apply(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
            panic!("apply must not be called by uninstaller");
        }

        fn start(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
            panic!("start must not be called by uninstaller");
        }

        fn stop(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
            panic!("stop must not be called by uninstaller");
        }

        fn down(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
            self.calls.borrow_mut().push("down".to_string());

            match &self.down_result {
                Ok(()) => Ok(()),
                Err(_) => Err(DockerComposeError::CommandFailed {
                    exit_code: Some(1),
                    stdout: "test stdout".to_string(),
                    stderr: "test failure".to_string(),
                }),
            }
        }

        fn service_statuses(
            &self,
            _installation: &Installation,
        ) -> Result<Vec<ServiceStatus>, DockerComposeError> {
            Ok(Vec::new())
        }
    }

    fn docker_failure() -> DockerComposeError {
        DockerComposeError::CommandFailed {
            exit_code: Some(1),
            stdout: "test stdout".to_string(),
            stderr: "test failure".to_string(),
        }
    }

    struct TestProgressReporter;

    impl ProgressReporter for TestProgressReporter {
        fn header(&self, _message: &str) {}
        fn step(&self, _current: usize, _total: usize, _message: &str) {}
        fn detail(&self, _message: &str) {}
        fn phase(&self, _name: &str, _message: &str) {}
        fn success(&self, _message: &str) {}
    }

    #[test]
    fn uninstall_stops_services_and_removes_installation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let installation_root = temp_dir.path().join("aims");

        std::fs::create_dir_all(&installation_root).unwrap();

        let installation =
            Installation::new(&installation_root, "http://127.0.0.1:8080/api/v1/health");

        let calls = Rc::new(RefCell::new(Vec::new()));

        let uninstaller = Uninstaller::new(
            installation,
            FakeRuntime {
                calls: calls.clone(),
                down_result: Ok(()),
            },
            TestProgressReporter,
        );

        uninstaller.uninstall().unwrap();

        assert_eq!(calls.borrow().as_slice(), ["down"]);
        assert!(!installation_root.exists());
    }

    #[test]
    fn docker_failure_keeps_installation_files() {
        let temp_dir = tempfile::tempdir().unwrap();
        let installation_root = temp_dir.path().join("aims");

        std::fs::create_dir_all(&installation_root).unwrap();

        let installation =
            Installation::new(&installation_root, "http://127.0.0.1:8080/api/v1/health");

        let calls = Rc::new(RefCell::new(Vec::new()));

        let uninstaller = Uninstaller::new(
            installation,
            FakeRuntime {
                calls: calls.clone(),
                down_result: Err(docker_failure()),
            },
            TestProgressReporter,
        );

        let result = uninstaller.uninstall();

        assert!(matches!(result, Err(UninstallerError::Docker(_))));
        assert_eq!(calls.borrow().as_slice(), ["down"]);
        assert!(installation_root.exists());
    }
}
