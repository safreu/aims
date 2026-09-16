mod manager;
mod replacer;

pub(crate) const AIMSCTL_INSTALL_PATH: &str = "/usr/local/bin/aimsctl";

pub(crate) use replacer::{
    SelfReplaceTargetBinaryInstaller, TargetBinaryInstaller, TargetBinaryInstallerError,
};

pub(crate) use manager::ExecutableManager;
