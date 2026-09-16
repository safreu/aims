use std::process::Command;

use crate::installation::{Installation, Version};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceStatus {
    pub name: String,
    pub state: String,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct DockerComposeService {
    service: String,
    state: String,
}

pub trait ContainerRuntime {
    fn pull(
        &self,
        installation: &Installation,
        version: &Version,
    ) -> Result<(), DockerComposeError>;

    fn apply(&self, installation: &Installation) -> Result<(), DockerComposeError>;

    fn start(&self, installation: &Installation) -> Result<(), DockerComposeError>;

    fn stop(&self, installation: &Installation) -> Result<(), DockerComposeError>;

    fn down(&self, installation: &Installation) -> Result<(), DockerComposeError>;

    fn service_statuses(
        &self,
        installation: &Installation,
    ) -> Result<Vec<ServiceStatus>, DockerComposeError>;

    fn check_available(&self) -> Result<(), RuntimeAvailabilityError>;
}

pub struct DockerCompose;

impl ContainerRuntime for DockerCompose {
    fn pull(
        &self,
        installation: &Installation,
        version: &Version,
    ) -> Result<(), DockerComposeError> {
        let mut command = Command::new("docker");

        command
            .arg("compose")
            .arg("--env-file")
            .arg(installation.env_file())
            .arg("-f")
            .arg(installation.compose_file())
            .arg("pull")
            .arg("backend")
            .arg("frontend")
            .env("AIMS_VERSION", version.to_string());

        run_command(&mut command)
    }

    fn apply(&self, installation: &Installation) -> Result<(), DockerComposeError> {
        let mut command = Command::new("docker");

        command
            .arg("compose")
            .arg("--env-file")
            .arg(installation.env_file())
            .arg("-f")
            .arg(installation.compose_file())
            .arg("up")
            .arg("-d")
            .arg("--remove-orphans");

        run_command(&mut command)
    }

    fn start(&self, installation: &Installation) -> Result<(), DockerComposeError> {
        let mut command = Command::new("docker");

        command
            .arg("compose")
            .arg("--env-file")
            .arg(installation.env_file())
            .arg("-f")
            .arg(installation.compose_file())
            .arg("start");

        run_command(&mut command)
    }

    fn stop(&self, installation: &Installation) -> Result<(), DockerComposeError> {
        let mut command = Command::new("docker");

        command
            .arg("compose")
            .arg("--env-file")
            .arg(installation.env_file())
            .arg("-f")
            .arg(installation.compose_file())
            .arg("stop");

        run_command(&mut command)
    }
    fn down(&self, installation: &Installation) -> Result<(), DockerComposeError> {
        let mut command = Command::new("docker");

        command
            .arg("compose")
            .arg("--env-file")
            .arg(installation.env_file())
            .arg("-f")
            .arg(installation.compose_file())
            .arg("down");

        run_command(&mut command)
    }

    fn service_statuses(
        &self,
        installation: &Installation,
    ) -> Result<Vec<ServiceStatus>, DockerComposeError> {
        let mut command = Command::new("docker");

        command
            .arg("compose")
            .arg("--env-file")
            .arg(installation.env_file())
            .arg("-f")
            .arg(installation.compose_file())
            .arg("ps")
            .arg("--all")
            .arg("--format")
            .arg("json");

        let stdout = run_command_with_output(&mut command)?;

        parse_service_statuses(&stdout)
    }

    fn check_available(&self) -> Result<(), RuntimeAvailabilityError> {
        let mut docker = Command::new("docker");
        docker.arg("--version");

        run_command(&mut docker).map_err(RuntimeAvailabilityError::Docker)?;

        let mut compose = Command::new("docker");
        compose.arg("compose").arg("version");

        run_command(&mut compose).map_err(RuntimeAvailabilityError::Compose)?;

        Ok(())
    }
}

fn run_command(command: &mut Command) -> Result<(), DockerComposeError> {
    run_command_with_output(command)?;
    Ok(())
}

fn run_command_with_output(command: &mut Command) -> Result<String, DockerComposeError> {
    let output = command
        .output()
        .map_err(DockerComposeError::CommandFailedToStart)?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();

    if !output.status.success() {
        return Err(DockerComposeError::CommandFailed {
            exit_code: output.status.code(),
            stdout,
            stderr,
        });
    };

    Ok(stdout)
}

fn parse_service_statuses(output: &str) -> Result<Vec<ServiceStatus>, DockerComposeError> {
    output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let service: DockerComposeService =
                serde_json::from_str(line).map_err(DockerComposeError::InvalidOutput)?;

            Ok(ServiceStatus {
                name: service.service,
                state: service.state,
            })
        })
        .collect()
}

#[derive(Debug, thiserror::Error)]
pub enum DockerComposeError {
    #[error("failed to parse Docker Compose output")]
    InvalidOutput(#[source] serde_json::Error),
    #[error("failed to start Docker Compose")]
    CommandFailedToStart(#[source] std::io::Error),
    #[error(
        "Docker Compose failed with exit code {exit_code:?}\nstdout: {stdout}\nstderr: {stderr}"
    )]
    CommandFailed {
        exit_code: Option<i32>,
        stdout: String,
        stderr: String,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeAvailabilityError {
    #[error("Docker is not available")]
    Docker(#[source] DockerComposeError),

    #[error("Docker Compose is not available")]
    Compose(#[source] DockerComposeError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_statuses_can_be_parsed() {
        let output = r#"
        {"Service":"postgres","State":"running"}
        {"Service":"backend","State":"restarting"}
        {"Service":"frontend","State":"running"}
        "#;
        let statuses = parse_service_statuses(output).unwrap();

        assert_eq!(
            statuses,
            vec![
                ServiceStatus {
                    name: "postgres".to_owned(),
                    state: "running".to_owned(),
                },
                ServiceStatus {
                    name: "backend".to_owned(),
                    state: "restarting".to_owned(),
                },
                ServiceStatus {
                    name: "frontend".to_owned(),
                    state: "running".to_owned(),
                },
            ]
        );
    }

    #[test]
    fn empty_service_statuses_can_be_parsed() {
        let statuses = parse_service_statuses("").unwrap();

        assert!(statuses.is_empty());
    }

    #[test]
    fn invalid_service_status_output_is_rejected() {
        let result = parse_service_statuses("not valid json");

        assert!(matches!(result, Err(DockerComposeError::InvalidOutput(_))));
    }

    #[test]
    fn service_status_output_with_missing_fields_is_rejected() {
        let output = r#"
        {"Service":"backend"}
        "#;
        let result = parse_service_statuses(output);

        assert!(matches!(result, Err(DockerComposeError::InvalidOutput(_))));
    }
}
