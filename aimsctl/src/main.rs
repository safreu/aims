use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::{
    commands::{Install, Status, Uninstall, Update},
    executable::{AIMSCTL_INSTALL_PATH, ExecutableManager, SelfReplaceTargetBinaryInstaller},
    installation::{Installation, Version},
    manager::Manager,
    privilege::{Elevation, SudoEscalator, SystemPrivilegeChecker, ensure_root},
    progress::ConsoleReporter,
    runtime::{docker::DockerCompose, health::HttpHealthChecker},
    self_update::{GithubReleaseBinaryProvider, ProcessTargetBinaryRunner, SelfUpdater},
};

mod commands;
mod executable;
mod installation;
mod privilege;
mod runtime;
mod self_update;

mod manager;
mod progress;

#[cfg(test)]
mod test_support;

#[derive(Parser)]
#[command(name = "aimsctl")]
#[command(about = "Management tool for Aims installations")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install Aims and start all services
    Install {
        /// Directory where Aims configuration files are installed.
        #[arg(long, default_value = "/opt/aims")]
        installation_root: PathBuf,

        /// Docker Compose project name.
        #[arg(long, default_value = "aims-prod")]
        compose_project: String,

        /// HTTP port used to expose Aims.
        #[arg(long, default_value_t = 80)]
        http_port: u16,

        /// Docker volume used for PostgreSQL data.
        #[arg(long, default_value = "aims_prod_postgres_data")]
        postgres_volume: String,

        ///Destination where aimsctl installs itself.
        #[arg(long, default_value = "/usr/local/bin/aimsctl", hide = true)]
        aimsctl_install_path: PathBuf,
    },

    /// Update an existing Aims installation.
    Update {
        /// Version of Aims to update to, for example 0.1.3.
        version: String,

        /// Directory containing the Aims installation.
        #[arg(long, default_value = "/opt/aims")]
        installation_root: PathBuf,
    },

    /// Show the aimsctl version.
    Version,

    /// Start existing Aims services.
    Start {
        /// Directory containing the Aims installation.
        #[arg(long, default_value = "/opt/aims")]
        installation_root: PathBuf,
    },

    /// Show the current Aims installation status.
    Status {
        /// Directory containing the Aims installation.
        #[arg(long, default_value = "/opt/aims")]
        installation_root: PathBuf,
    },

    /// Stop Aims services.
    Stop {
        /// Directory containing the Aims installation.
        #[arg(long, default_value = "/opt/aims")]
        installation_root: PathBuf,
    },

    /// Stop and remove Aims while preserving database data.
    ///
    /// Removes the Aims containers and installation files. The PostgreSQL
    /// Docker volume is preserved so that Aims can be installed again
    /// without losing the existing database.
    Uninstall {
        /// Directory containing the Aims installation.
        #[arg(long, default_value = "/opt/aims")]
        installation_root: PathBuf,
    },

    /// Apply an update using the target aimsctl binary.
    #[command(hide = true)]
    ApplyUpdate {
        /// Version of Aims to update to.
        version: String,

        /// Directory containing the Aims installation.
        #[arg(long)]
        installation_root: PathBuf,
    },
}

impl Commands {
    fn requires_root(&self) -> bool {
        matches!(
            self,
            Self::Install { .. }
                | Self::Update { .. }
                | Self::Start { .. }
                | Self::Stop { .. }
                | Self::Uninstall { .. }
                | Self::ApplyUpdate { .. }
        )
    }
}

fn main() {
    let cli = Cli::parse();

    if cli.command.requires_root() {
        match ensure_root(&SystemPrivilegeChecker, &SudoEscalator) {
            Ok(Elevation::AlreadyRoot) => {}
            Ok(Elevation::Reexecuted) => return,
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(1);
            }
        }
    }

    let result = match cli.command {
        Commands::Install {
            installation_root,
            compose_project,
            http_port,
            postgres_volume,
            aimsctl_install_path,
        } => install(
            installation_root,
            compose_project,
            http_port,
            postgres_volume,
            aimsctl_install_path,
        ),

        Commands::Update {
            version,
            installation_root,
        } => update(&version, installation_root),

        Commands::Version => {
            println!("{}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }

        Commands::Start { installation_root } => start(installation_root),
        Commands::Status { installation_root } => status(installation_root),
        Commands::Stop { installation_root } => stop(installation_root),

        Commands::Uninstall { installation_root } => uninstall(installation_root),
        Commands::ApplyUpdate {
            version,
            installation_root,
        } => apply_update(&version, installation_root),
    };

    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn install(
    installation_root: PathBuf,
    compose_project: String,
    http_port: u16,
    postgres_volume: String,
    aimsctl_install_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let version = Version::parse(env!("CARGO_PKG_VERSION"))?;

    let installation = Installation::new(installation_root);

    installation.ensure_not_exists()?;

    let installer = Install::new(
        installation,
        DockerCompose,
        HttpHealthChecker,
        ConsoleReporter,
        compose_project,
        http_port,
        postgres_volume,
    );

    let executable_manager = ExecutableManager::new(aimsctl_install_path);

    executable_manager.install()?;

    if let Err(error) = installer.install(&version) {
        let _ = executable_manager.uninstall();
        return Err(error.into());
    }

    Ok(())
}

fn update(version: &str, installation_root: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let version = Version::parse(version)?;

    let installation = Installation::new(installation_root);

    let updater = SelfUpdater::new(
        installation,
        GithubReleaseBinaryProvider,
        ProcessTargetBinaryRunner,
        SelfReplaceTargetBinaryInstaller,
    );

    updater.update(&version)?;

    Ok(())
}

fn start(installation_root: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let installation = Installation::new(installation_root);

    let manager = Manager::new(installation, DockerCompose, ConsoleReporter);

    manager.start()?;

    Ok(())
}

fn stop(installation_root: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let installation = Installation::new(installation_root);

    let manager = Manager::new(installation, DockerCompose, ConsoleReporter);

    manager.stop()?;

    Ok(())
}

fn uninstall(installation_root: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let installation = Installation::new(installation_root);

    let uninstaller = Uninstall::new(installation, DockerCompose, ConsoleReporter);

    uninstaller.uninstall()?;

    let executable_manager = ExecutableManager::new(AIMSCTL_INSTALL_PATH);

    executable_manager.uninstall()?;

    Ok(())
}

fn status(installation_root: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let installation = Installation::new(installation_root);

    let checker = Status::new(installation, DockerCompose, HttpHealthChecker);

    let status = checker.status()?;

    println!("Aims status\n");
    println!("Version: {}", status.version);
    println!();
    println!("{:<15}Status", "Service");

    for service in status.services {
        println!("{:<15}{}", service.name, service.state);
    }

    println!();
    println!(
        "Aims health: {}",
        if status.healthy {
            "healthy"
        } else {
            "unhealthy"
        }
    );

    Ok(())
}

fn apply_update(
    version: &str,
    installation_root: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let version = Version::parse(version)?;

    let installation = Installation::new(installation_root);

    let updater = Update::new(
        installation,
        DockerCompose,
        HttpHealthChecker,
        ConsoleReporter,
    );

    updater.update(&version)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn mutating_commands_require_root() {
        let commands = [
            Commands::Install {
                installation_root: PathBuf::from("/opt/aims"),
                compose_project: "aims-prod".to_string(),
                http_port: 80,
                postgres_volume: "aims_prod_postgres_data".to_string(),
                aimsctl_install_path: PathBuf::from("/usr/local/bin/aimsctl"),
            },
            Commands::Update {
                version: "0.1.5".to_string(),
                installation_root: PathBuf::from("/opt/aims"),
            },
            Commands::Start {
                installation_root: PathBuf::from("/opt/aims"),
            },
            Commands::Stop {
                installation_root: PathBuf::from("/opt/aims"),
            },
            Commands::Uninstall {
                installation_root: PathBuf::from("/opt/aims"),
            },
            Commands::ApplyUpdate {
                version: "0.1.5".to_string(),
                installation_root: PathBuf::from("/opt/aims"),
            },
        ];

        for command in commands {
            assert!(command.requires_root());
        }
    }

    #[test]
    fn read_only_commands_do_not_require_root() {
        let commands = [
            Commands::Version,
            Commands::Status {
                installation_root: PathBuf::from("/opt/aims"),
            },
        ];

        for command in commands {
            assert!(!command.requires_root());
        }
    }

    #[test]
    fn install_accepts_hidden_aimsctl_install_path() {
        let cli = Cli::try_parse_from([
            "aimsctl",
            "install",
            "--aimsctl-install-path",
            "/tmp/aims-e2e-bin/aimsctl",
        ])
        .unwrap();

        assert!(matches!(
            cli.command,
            Commands::Install {
                aimsctl_install_path,
                ..
            } if aimsctl_install_path == Path::new("/tmp/aims-e2e-bin/aimsctl")
        ));
    }

    #[test]
    fn install_uses_default_aimsctl_install_path() {
        let cli = Cli::try_parse_from(["aimsctl", "install"]).unwrap();

        assert!(matches!(
            cli.command,
            Commands::Install {
                aimsctl_install_path,
                ..
            } if aimsctl_install_path == Path::new("/usr/local/bin/aimsctl")
        ));
    }
}
