use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use crate::{
    docker::{ContainerRuntime, DockerComposeError, ServiceStatus},
    environment::{EnvironmentConfig, build_environment},
    health::{HealthCheckError, HealthChecker},
    installation::Installation,
    progress::ProgressReporter,
    version::Version,
};

pub struct TestInstallation {
    pub _temp_dir: tempfile::TempDir,
    pub installation: Installation,
}

pub fn installation(version: &str) -> TestInstallation {
    installation_with_port(version, 8080)
}

pub fn installation_with_port(version: &str, http_port: u16) -> TestInstallation {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path().join("aims");

    let installation = Installation::new(&root);
    installation.create().unwrap();

    let version = Version::parse(version).unwrap();

    let environment = build_environment(&EnvironmentConfig {
        version: &version,
        database_password: "test-password",
        compose_project: "aims-test",
        http_port,
        postgres_volume: "aims_test_postgres_data",
    });

    installation.write_environment(&environment).unwrap();

    TestInstallation {
        _temp_dir: temp_dir,
        installation,
    }
}

pub struct TestProgressReporter;

impl ProgressReporter for TestProgressReporter {
    fn header(&self, _message: &str) {}

    fn step(&self, _current: usize, _total: usize, _message: &str) {}

    fn detail(&self, _message: &str) {}

    fn phase(&self, _name: &str, _message: &str) {}

    fn success(&self, _message: &str) {}
}

#[derive(Clone)]
pub struct FakeRuntime {
    calls: Rc<RefCell<Vec<String>>>,
    pull_results: Rc<RefCell<VecDeque<Result<(), DockerComposeError>>>>,
    apply_results: Rc<RefCell<VecDeque<Result<(), DockerComposeError>>>>,
    start_results: Rc<RefCell<VecDeque<Result<(), DockerComposeError>>>>,
    stop_results: Rc<RefCell<VecDeque<Result<(), DockerComposeError>>>>,
    down_results: Rc<RefCell<VecDeque<Result<(), DockerComposeError>>>>,
    service_status_results: Rc<RefCell<VecDeque<Result<Vec<ServiceStatus>, DockerComposeError>>>>,
}

#[allow(unused)]
impl FakeRuntime {
    pub fn new() -> Self {
        Self {
            calls: Rc::new(RefCell::new(Vec::new())),
            pull_results: Rc::new(RefCell::new(VecDeque::new())),
            apply_results: Rc::new(RefCell::new(VecDeque::new())),
            start_results: Rc::new(RefCell::new(VecDeque::new())),
            stop_results: Rc::new(RefCell::new(VecDeque::new())),
            down_results: Rc::new(RefCell::new(VecDeque::new())),
            service_status_results: Rc::new(RefCell::new(VecDeque::new())),
        }
    }

    pub fn calls(&self) -> Vec<String> {
        self.calls.borrow().clone()
    }

    pub fn push_pull_result(&self, result: Result<(), DockerComposeError>) {
        self.pull_results.borrow_mut().push_back(result);
    }

    pub fn push_apply_result(&self, result: Result<(), DockerComposeError>) {
        self.apply_results.borrow_mut().push_back(result);
    }

    pub fn push_start_result(&self, result: Result<(), DockerComposeError>) {
        self.start_results.borrow_mut().push_back(result);
    }

    pub fn push_stop_result(&self, result: Result<(), DockerComposeError>) {
        self.stop_results.borrow_mut().push_back(result);
    }

    pub fn push_down_result(&self, result: Result<(), DockerComposeError>) {
        self.down_results.borrow_mut().push_back(result);
    }

    pub fn push_service_status_result(
        &self,
        result: Result<Vec<ServiceStatus>, DockerComposeError>,
    ) {
        self.service_status_results.borrow_mut().push_back(result);
    }
}

impl Default for FakeRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl ContainerRuntime for FakeRuntime {
    fn pull(
        &self,
        _installation: &Installation,
        version: &Version,
    ) -> Result<(), DockerComposeError> {
        self.calls.borrow_mut().push(format!("pull:{version}"));

        self.pull_results.borrow_mut().pop_front().unwrap_or(Ok(()))
    }

    fn apply(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
        self.calls.borrow_mut().push("apply".to_string());

        self.apply_results
            .borrow_mut()
            .pop_front()
            .unwrap_or(Ok(()))
    }

    fn start(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
        self.calls.borrow_mut().push("start".to_string());

        self.start_results
            .borrow_mut()
            .pop_front()
            .unwrap_or(Ok(()))
    }

    fn stop(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
        self.calls.borrow_mut().push("stop".to_string());

        self.stop_results.borrow_mut().pop_front().unwrap_or(Ok(()))
    }

    fn down(&self, _installation: &Installation) -> Result<(), DockerComposeError> {
        self.calls.borrow_mut().push("down".to_string());

        self.down_results.borrow_mut().pop_front().unwrap_or(Ok(()))
    }

    fn service_statuses(
        &self,
        _installation: &Installation,
    ) -> Result<Vec<ServiceStatus>, DockerComposeError> {
        self.calls.borrow_mut().push("service_statuses".to_string());

        self.service_status_results
            .borrow_mut()
            .pop_front()
            .unwrap_or(Ok(Vec::new()))
    }
}

pub fn docker_failure() -> DockerComposeError {
    DockerComposeError::CommandFailed {
        exit_code: Some(1),
        stdout: "test stdout".to_string(),
        stderr: "test failure".to_string(),
    }
}

pub struct FakeHealthChecker {
    wait_result: Result<(), HealthCheckError>,
    healthy: bool,
}

impl FakeHealthChecker {
    pub fn healthy() -> Self {
        Self {
            wait_result: Ok(()),
            healthy: true,
        }
    }

    pub fn unhealthy() -> Self {
        Self {
            wait_result: Ok(()),
            healthy: false,
        }
    }

    pub fn failing() -> Self {
        Self {
            wait_result: Err(HealthCheckError::TimedOut),
            healthy: false,
        }
    }
}

impl HealthChecker for FakeHealthChecker {
    fn wait_until_healthy(&self, _installation: &Installation) -> Result<(), HealthCheckError> {
        match &self.wait_result {
            Ok(()) => Ok(()),
            Err(_) => Err(HealthCheckError::TimedOut),
        }
    }

    fn is_healthy(&self, _installation: &Installation) -> Result<bool, HealthCheckError> {
        Ok(self.healthy)
    }
}
