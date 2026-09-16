use std::{path::Path, process::Command};

use crate::installation::{Installation, Version};

pub(crate) trait TargetBinaryRunner {
    fn run(
        &self,
        binary: &Path,
        version: &Version,
        installation: &Installation,
    ) -> Result<(), TargetBinaryRunnerError>;
}

pub struct ProcessTargetBinaryRunner;

impl TargetBinaryRunner for ProcessTargetBinaryRunner {
    fn run(
        &self,
        binary: &Path,
        version: &Version,
        installation: &Installation,
    ) -> Result<(), TargetBinaryRunnerError> {
        let status = Command::new(binary)
            .arg("apply-update")
            .arg(version.to_string())
            .arg("--installation-root")
            .arg(installation.root())
            .status()
            .map_err(TargetBinaryRunnerError::ProcessStart)?;

        if !status.success() {
            return Err(TargetBinaryRunnerError::ProcessFailed(status));
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum TargetBinaryRunnerError {
    #[error("failed to start target aimsctl")]
    ProcessStart(#[source] std::io::Error),

    #[error("target aimsctl exited unsuccessfully with status {0}")]
    ProcessFailed(std::process::ExitStatus),
}
