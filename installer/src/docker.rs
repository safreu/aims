use std::process::Command;

use crate::installation::Installation;

pub trait ContainerRuntime {
    fn pull(&self, installation: &Installation) -> Result<(), DockerComposeError>;

    fn up(&self, installation: &Installation) -> Result<(), DockerComposeError>;

    fn down(&self, installation: &Installation) -> Result<(), DockerComposeError>;
}

pub struct DockerCompose;

impl ContainerRuntime for DockerCompose {
    fn pull(&self, installation: &Installation) -> Result<(), DockerComposeError> {
        run_compose(installation, &["pull"])
    }

    fn up(&self, installation: &Installation) -> Result<(), DockerComposeError> {
        run_compose(installation, &["up", "-d"])
    }

    fn down(&self, installation: &Installation) -> Result<(), DockerComposeError> {
        run_compose(installation, &["down"])
    }
}

fn run_compose(installation: &Installation, args: &[&str]) -> Result<(), DockerComposeError> {
    let output = Command::new("docker")
        .arg("compose")
        .arg("--env-file")
        .arg(installation.env_file())
        .arg("-f")
        .arg(installation.compose_file())
        .args(args)
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
