mod installer;
mod prepared_binary;
mod provider;
mod runner;
mod updater;

pub use installer::SelfReplaceTargetBinaryInstaller;
pub use provider::GithubReleaseBinaryProvider;
pub use runner::ProcessTargetBinaryRunner;
pub use updater::SelfUpdater;
