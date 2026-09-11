use std::process::Command;

use crate::{installation::Installation, version::Version};

pub trait ContainerRuntime {
    fn pull(
        &self,
        installation: &Installation,
        version: &Version,
    ) -> Result<(), DockerComposeError>;

    fn apply(&self, installation: &Installation) -> Result<(), DockerComposeError>;
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
}

fn run_command(command: &mut Command) -> Result<(), DockerComposeError> {
    let output = command
        .output()
        .map_err(DockerComposeError::CommandFailedToStart)?;

    if !output.status.success() {
        return Err(DockerComposeError::CommandFailed {
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).trim().to_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        });
    };

    Ok(())
}
#[derive(Debug, thiserror::Error)]
pub enum DockerComposeError {
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
