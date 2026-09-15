use crate::{
    installation::{Installation, InstallationError},
    version::Version,
};

pub fn migrate(
    installation: &Installation,
    previous_version: &Version,
    target_version: &Version,
) -> Result<(), MigrationError> {
    let port_migration_version = Version::parse("0.1.5")?;

    if previous_version < &port_migration_version && target_version >= &port_migration_version {
        migrate_http_port(installation)?;
    }

    Ok(())
}

fn migrate_http_port(installation: &Installation) -> Result<(), MigrationError> {
    let current_port = installation.environment_value("AIMS_HTTP_PORT")?;

    if current_port == "8080" {
        installation.set_environment_value("AIMS_HTTP_PORT", "80")?;
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum MigrationError {
    #[error("installation migration failed")]
    Installation(#[from] InstallationError),

    #[error("invalid migration version")]
    Version(#[from] crate::version::VersionParseError),
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        manager::Manager,
        test_support::{FakeRuntime, TestProgressReporter},
    };

    fn test_manager(runtime: FakeRuntime) -> Manager<FakeRuntime, TestProgressReporter> {
        let installation = Installation::new("/tmp/aims-test");

        Manager::new(installation, runtime, TestProgressReporter)
    }

    #[test]
    fn start_starts_existing_containers() {
        let runtime = FakeRuntime::new();

        let manager = test_manager(runtime.clone());

        manager.start().unwrap();

        assert_eq!(runtime.calls(), ["start"]);
    }

    #[test]
    fn stop_stops_existing_containers() {
        let runtime = FakeRuntime::new();

        let manager = test_manager(runtime.clone());

        manager.stop().unwrap();

        assert_eq!(runtime.calls(), ["stop"]);
    }
}
