mod manager;
mod replacer;

pub(crate) use replacer::{
    SelfReplaceTargetBinaryInstaller, TargetBinaryInstaller, TargetBinaryInstallerError,
};

pub(crate) use manager::{ExecutableInstaller, ExecutableManager, ExecutableManagerError};
