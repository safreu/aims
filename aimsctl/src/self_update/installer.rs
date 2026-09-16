use std::path::Path;

pub(crate) trait TargetBinaryInstaller {
    fn install(&self, binary: &Path) -> Result<(), TargetBinaryInstallerError>;
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum TargetBinaryInstallerError {
    #[error("failed to replace installed aimsctl binary")]
    Replace(#[source] std::io::Error),
}

pub struct SelfReplaceTargetBinaryInstaller;

impl TargetBinaryInstaller for SelfReplaceTargetBinaryInstaller {
    fn install(&self, binary: &Path) -> Result<(), TargetBinaryInstallerError> {
        self_replace::self_replace(binary).map_err(TargetBinaryInstallerError::Replace)?;

        Ok(())
    }
}
