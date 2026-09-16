use std::path::Path;

use crate::commands::init::{InitApplier, InitApplyError};

pub(crate) struct ProcessInitApplier;

impl InitApplier for ProcessInitApplier {
    fn apply(&self, config_path: &Path, destination_path: &Path) -> Result<(), InitApplyError> {
        let executable = std::env::current_exe().map_err(InitApplyError::CurrentExecutable)?;

        let status = std::process::Command::new(executable)
            .arg("apply-init")
            .arg("--config-path")
            .arg(config_path)
            .arg("--destination-path")
            .arg(destination_path)
            .status()
            .map_err(InitApplyError::Process)?;

        if !status.success() {
            return Err(InitApplyError::Failed);
        }

        Ok(())
    }
}
