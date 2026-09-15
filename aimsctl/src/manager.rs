use crate::{
    docker::{ContainerRuntime, DockerComposeError},
    installation::Installation,
    progress::ProgressReporter,
};

pub struct Manager<R, P>
where
    R: ContainerRuntime,
    P: ProgressReporter,
{
    installation: Installation,
    runtime: R,
    progress: P,
}

impl<R, P> Manager<R, P>
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

    pub fn start(&self) -> Result<(), ManagerError> {
        self.progress.header("Starting Aims");
        self.runtime.start(&self.installation)?;
        self.progress.success("Aims started successfully");
        Ok(())
    }

    pub fn stop(&self) -> Result<(), ManagerError> {
        self.progress.header("Stopping Aims");
        self.runtime.stop(&self.installation)?;
        self.progress.success("Aims stopped successfully");
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ManagerError {
    #[error("Docker compose operation failed")]
    Docker(#[from] DockerComposeError),
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{cell::RefCell, rc::Rc};

    use crate::{docker::ServiceStatus, test_support::TestProgressReporter, version::Version};

    struct FakeRuntime {
        calls: Rc<RefCell<Vec<String>>>,
    }

    impl ContainerRuntime for FakeRuntime {
        fn pull(
            &self,
            _installation: &Installation,
            _version: &Version,
        ) -> Result<(), DockerComposeError> {
            panic!("pull must not be called by manager");
        }

        fn apply(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
            panic!("apply must not be called by manager");
        }

        fn start(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
            self.calls.borrow_mut().push("start".to_string());
            Ok(())
        }

        fn stop(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
            self.calls.borrow_mut().push("stop".to_string());
            Ok(())
        }

        fn down(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
            panic!("down must not be called by manager");
        }

        fn service_statuses(
            &self,
            _installation: &Installation,
        ) -> Result<Vec<ServiceStatus>, DockerComposeError> {
            Ok(Vec::new())
        }
    }

    fn test_manager(calls: Rc<RefCell<Vec<String>>>) -> Manager<FakeRuntime, TestProgressReporter> {
        let installation = Installation::new("/tmp/aims-test");

        Manager::new(installation, FakeRuntime { calls }, TestProgressReporter)
    }

    #[test]
    fn start_starts_existing_containers() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let manager = test_manager(calls.clone());

        manager.start().unwrap();

        assert_eq!(calls.borrow().as_slice(), ["start"]);
    }

    #[test]
    fn stop_stops_existing_containers() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let manager = test_manager(calls.clone());

        manager.stop().unwrap();

        assert_eq!(calls.borrow().as_slice(), ["stop"]);
    }
}
