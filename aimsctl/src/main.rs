use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::{
    commands::{
        ApplyInit, Init, Install, ProcessInitApplier, Status, SystemAimsInstaller, Uninstall,
        Update,
    },
    configuration::{
        AIMSCTL_CONFIG_PATH, AimsctlConfig, ConfigStore, ConfigStoreError, ConsoleConfigPrompter,
    },
    executable::{ExecutableInstaller, ExecutableManager, SelfReplaceTargetBinaryInstaller},
    installation::{Installation, Version},
    manager::Manager,
    privilege::{
        CoscaPrivilegeEscalator, Elevation, SystemPrivilegeChecker, current_arguments, ensure_root,
    },
    progress::ConsoleReporter,
    runtime::{docker::DockerCompose, health::HttpHealthChecker},
    self_update::{GithubReleaseBinaryProvider, ProcessTargetBinaryRunner, SelfUpdater},
};

mod commands;
mod configuration;
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
    /// Configure and install Aims.
    Init {
        /// Path where aimsctl configuration is stored.
        #[arg(long, default_value = AIMSCTL_CONFIG_PATH, hide = true)]
        config_path: PathBuf,
    },

    /// Complete initialization with elevated privileges.
    #[command(hide = true)]
    ApplyInit {
        /// Path containing the aimsctl configuration.
        #[arg(long)]
        config_path: PathBuf,

        #[arg(long)]
        destination_path: PathBuf,
    },

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

        /// Override the configured Aims installation directory.
        #[arg(long, hide = true)]
        installation_root: Option<PathBuf>,

        /// Override the aimsctl configuration path.
        #[arg(long, default_value = AIMSCTL_CONFIG_PATH, hide = true)]
        config_path: PathBuf,
    },

    /// Show the aimsctl version.
    Version,

    /// Start existing Aims services.
    Start {
        /// Override the configured Aims installation directory.
        #[arg(long, hide = true)]
        installation_root: Option<PathBuf>,

        /// Override the aimsctl configuration path.
        #[arg(long, default_value = AIMSCTL_CONFIG_PATH, hide = true)]
        config_path: PathBuf,
    },

    /// Show the current Aims installation status.
    Status {
        /// Override the configured Aims installation directory.
        #[arg(long, hide = true)]
        installation_root: Option<PathBuf>,

        /// Override the aimsctl configuration path.
        #[arg(long, default_value = AIMSCTL_CONFIG_PATH, hide = true)]
        config_path: PathBuf,
    },

    /// Stop Aims services.
    Stop {
        /// Override the configured Aims installation directory.
        #[arg(long, hide = true)]
        installation_root: Option<PathBuf>,

        /// Override the aimsctl configuration path.
        #[arg(long, default_value = AIMSCTL_CONFIG_PATH, hide = true)]
        config_path: PathBuf,
    },

    /// Stop and remove Aims while preserving database data.
    ///
    /// Removes the Aims containers and installation files. The PostgreSQL
    /// Docker volume is preserved so that Aims can be installed again
    /// without losing the existing database.
    Uninstall {
        /// Override the configured Aims installation directory.
        #[arg(long, hide = true)]
        installation_root: Option<PathBuf>,

        /// Override the configured aimsctl executable path.
        #[arg(long, hide = true)]
        aimsctl_install_path: Option<PathBuf>,

        /// Override the aimsctl configuration path.
        #[arg(long, default_value = AIMSCTL_CONFIG_PATH, hide = true)]
        config_path: PathBuf,
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
        !matches!(self, Commands::Version | Commands::Init { .. })
    }
}

fn main() {
    let cli = Cli::parse();

    if cli.command.requires_root() {
        let arguments = current_arguments();
        match ensure_root(
            &SystemPrivilegeChecker,
            &CoscaPrivilegeEscalator,
            &arguments,
        ) {
            Ok(Elevation::AlreadyRoot) => {}
            Ok(Elevation::Reexecuted) => return,
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(1);
            }
        }
    }

    let result = match cli.command {
        Commands::Init { config_path } => init(config_path),
        Commands::ApplyInit {
            config_path,
            destination_path,
        } => apply_init(config_path, destination_path),

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
            config_path,
        } => update(&version, installation_root, config_path),

        Commands::Version => {
            println!("{}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }

        Commands::Start {
            installation_root,
            config_path,
        } => start(installation_root, config_path),
        Commands::Status {
            installation_root,
            config_path,
        } => status(installation_root, config_path),
        Commands::Stop {
            installation_root,
            config_path,
        } => stop(installation_root, config_path),

        Commands::Uninstall {
            installation_root,
            aimsctl_install_path,
            config_path,
        } => uninstall(installation_root, aimsctl_install_path, config_path),
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

fn resolve_installation_root(
    override_path: Option<PathBuf>,
    config_path: PathBuf,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Some(path) = override_path {
        return Ok(path);
    }

    let config = load_config(config_path)?;

    Ok(config.installation.root)
}

fn load_config(config_path: PathBuf) -> Result<AimsctlConfig, ConfigStoreError> {
    ConfigStore::new(config_path).load()
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

fn update(
    version: &str,
    installation_root: Option<PathBuf>,
    config_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let version = Version::parse(version)?;

    let installation_root = resolve_installation_root(installation_root, config_path)?;
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

fn start(
    installation_root: Option<PathBuf>,
    config_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let installation_root = resolve_installation_root(installation_root, config_path)?;
    let installation = Installation::new(installation_root);

    let manager = Manager::new(installation, DockerCompose, ConsoleReporter);

    manager.start()?;

    Ok(())
}

fn stop(
    installation_root: Option<PathBuf>,
    config_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let installation_root = resolve_installation_root(installation_root, config_path)?;

    let installation = Installation::new(installation_root);

    let manager = Manager::new(installation, DockerCompose, ConsoleReporter);

    manager.stop()?;

    Ok(())
}

fn uninstall(
    installation_root: Option<PathBuf>,
    aimsctl_install_path: Option<PathBuf>,
    config_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let config_store = ConfigStore::new(config_path);
    let config = config_store.load()?;

    let installation_root = installation_root.unwrap_or(config.installation.root);

    let aimsctl_install_path = aimsctl_install_path.unwrap_or(config.aimsctl.install_path);

    let installation = Installation::new(installation_root);

    let uninstaller = Uninstall::new(installation, DockerCompose, ConsoleReporter);

    uninstaller.uninstall()?;

    let executable_manager = ExecutableManager::new(aimsctl_install_path);

    executable_manager.uninstall()?;

    config_store.remove()?;

    Ok(())
}

fn status(
    installation_root: Option<PathBuf>,
    config_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let installation_root = resolve_installation_root(installation_root, config_path)?;

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

fn init(config_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let store = ConfigStore::new(config_path);

    let init = Init::new(store, ConsoleConfigPrompter, ProcessInitApplier);

    init.init()?;

    Ok(())
}

fn apply_init(
    config_path: PathBuf,
    destination_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let source_store = ConfigStore::new(config_path);
    let destination_store = ConfigStore::new(destination_path);
    let config = source_store.load()?;

    let executable_manager = ExecutableManager::new(config.aimsctl.install_path.clone());

    let apply_init = ApplyInit::new(destination_store, executable_manager, SystemAimsInstaller);

    apply_init.apply(&config)?;

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
                installation_root: None,
                config_path: PathBuf::from(AIMSCTL_CONFIG_PATH),
            },
            Commands::Start {
                installation_root: None,
                config_path: PathBuf::from(AIMSCTL_CONFIG_PATH),
            },
            Commands::Stop {
                installation_root: None,
                config_path: PathBuf::from(AIMSCTL_CONFIG_PATH),
            },
            Commands::Uninstall {
                installation_root: None,
                aimsctl_install_path: None,
                config_path: PathBuf::from(AIMSCTL_CONFIG_PATH),
            },
            Commands::ApplyUpdate {
                version: "0.1.5".to_string(),
                installation_root: PathBuf::from("/opt/aims"),
            },
            Commands::ApplyInit {
                config_path: PathBuf::from("/tmp/aimsctl.toml"),
                destination_path: PathBuf::from("/etc/aims/aimsctl.toml"),
            },
            Commands::Status {
                installation_root: None,
                config_path: PathBuf::from(AIMSCTL_CONFIG_PATH),
            },
        ];

        for command in commands {
            assert!(command.requires_root());
        }
    }

    #[test]
    fn read_only_commands_do_not_require_root() {
        let commands = [Commands::Version];

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

    #[test]
    fn init_uses_default_config_path() {
        let cli = Cli::try_parse_from(["aimsctl", "init"]).unwrap();

        assert!(matches!(
            cli.command,
            Commands::Init {
                config_path,
            } if config_path == Path::new(AIMSCTL_CONFIG_PATH)
        ));
    }

    #[test]
    fn init_accepts_custom_config_path() {
        let cli = Cli::try_parse_from([
            "aimsctl",
            "init",
            "--config-path",
            "/tmp/aims-e2e-config/aimsctl.toml",
        ])
        .unwrap();

        assert!(matches!(
            cli.command,
            Commands::Init {
                config_path,
            } if config_path == Path::new("/tmp/aims-e2e-config/aimsctl.toml")
        ));
    }

    #[test]
    fn status_has_no_overrides_by_default() {
        let cli = Cli::try_parse_from(["aimsctl", "status"]).unwrap();

        assert!(matches!(
            cli.command,
            Commands::Status {
                installation_root: None,
                config_path,
            } if config_path == Path::new(AIMSCTL_CONFIG_PATH)
        ));
    }

    #[test]
    fn status_accepts_installation_root_override() {
        let cli =
            Cli::try_parse_from(["aimsctl", "status", "--installation-root", "/tmp/aims-e2e"])
                .unwrap();

        assert!(matches!(
            cli.command,
            Commands::Status {
                installation_root: Some(path),
                ..
            } if path == Path::new("/tmp/aims-e2e")
        ));
    }

    #[test]
    fn status_accepts_config_path_override() {
        let cli = Cli::try_parse_from([
            "aimsctl",
            "status",
            "--config-path",
            "/tmp/aims-e2e-config/aimsctl.toml",
        ])
        .unwrap();

        assert!(matches!(
            cli.command,
            Commands::Status {
                installation_root: None,
                config_path,
            } if config_path == Path::new("/tmp/aims-e2e-config/aimsctl.toml")
        ));
    }

    #[test]
    fn start_has_no_overrides_by_default() {
        let cli = Cli::try_parse_from(["aimsctl", "start"]).unwrap();

        assert!(matches!(
            cli.command,
            Commands::Start {
                installation_root: None,
                config_path,
            } if config_path == Path::new(AIMSCTL_CONFIG_PATH)
        ));
    }

    #[test]
    fn start_accepts_installation_root_override() {
        let cli = Cli::try_parse_from(["aimsctl", "start", "--installation-root", "/tmp/aims-e2e"])
            .unwrap();

        assert!(matches!(
            cli.command,
            Commands::Start {
                installation_root: Some(path),
                ..
            } if path == Path::new("/tmp/aims-e2e")
        ));
    }

    #[test]
    fn start_accepts_config_path_override() {
        let cli = Cli::try_parse_from([
            "aimsctl",
            "start",
            "--config-path",
            "/tmp/aims-e2e-config/aimsctl.toml",
        ])
        .unwrap();

        assert!(matches!(
            cli.command,
            Commands::Start {
                installation_root: None,
                config_path,
            } if config_path == Path::new("/tmp/aims-e2e-config/aimsctl.toml")
        ));
    }

    #[test]
    fn stop_has_no_overrides_by_default() {
        let cli = Cli::try_parse_from(["aimsctl", "stop"]).unwrap();

        assert!(matches!(
            cli.command,
            Commands::Stop {
                installation_root: None,
                config_path,
            } if config_path == Path::new(AIMSCTL_CONFIG_PATH)
        ));
    }

    #[test]
    fn stop_accepts_installation_root_override() {
        let cli = Cli::try_parse_from(["aimsctl", "stop", "--installation-root", "/tmp/aims-e2e"])
            .unwrap();

        assert!(matches!(
            cli.command,
            Commands::Stop {
                installation_root: Some(path),
                ..
            } if path == Path::new("/tmp/aims-e2e")
        ));
    }

    #[test]
    fn stop_accepts_config_path_override() {
        let cli = Cli::try_parse_from([
            "aimsctl",
            "stop",
            "--config-path",
            "/tmp/aims-e2e-config/aimsctl.toml",
        ])
        .unwrap();

        assert!(matches!(
            cli.command,
            Commands::Stop {
                installation_root: None,
                config_path,
            } if config_path == Path::new("/tmp/aims-e2e-config/aimsctl.toml")
        ));
    }

    #[test]
    fn update_has_no_overrides_by_default() {
        let cli = Cli::try_parse_from(["aimsctl", "update", "0.1.5"]).unwrap();

        assert!(matches!(
            cli.command,
            Commands::Update {
                version,
                installation_root: None,
                config_path,
            } if
                version == "0.1.5"
                && config_path == Path::new(AIMSCTL_CONFIG_PATH)
        ));
    }

    #[test]
    fn update_accepts_installation_root_override() {
        let cli = Cli::try_parse_from([
            "aimsctl",
            "update",
            "0.1.5",
            "--installation-root",
            "/tmp/aims-e2e",
        ])
        .unwrap();

        assert!(matches!(
            cli.command,
            Commands::Update {
                installation_root: Some(path),
                ..
            } if path == Path::new("/tmp/aims-e2e")
        ));
    }

    #[test]
    fn update_accepts_config_path_override() {
        let cli = Cli::try_parse_from([
            "aimsctl",
            "update",
            "0.1.5",
            "--config-path",
            "/tmp/aims-e2e-config/aimsctl.toml",
        ])
        .unwrap();

        assert!(matches!(
            cli.command,
            Commands::Update {
                installation_root: None,
                config_path,
                ..
            } if config_path == Path::new("/tmp/aims-e2e-config/aimsctl.toml")
        ));
    }

    #[test]
    fn uninstall_has_no_overrides_by_default() {
        let cli = Cli::try_parse_from(["aimsctl", "uninstall"]).unwrap();

        assert!(matches!(
            cli.command,
            Commands::Uninstall {
                installation_root: None,
                aimsctl_install_path: None,
                config_path,
            } if config_path == Path::new(AIMSCTL_CONFIG_PATH)
        ));
    }

    #[test]
    fn uninstall_accepts_e2e_overrides() {
        let cli = Cli::try_parse_from([
            "aimsctl",
            "uninstall",
            "--installation-root",
            "/tmp/aims-e2e",
            "--aimsctl-install-path",
            "/tmp/aims-e2e-bin/aimsctl",
            "--config-path",
            "/tmp/aims-e2e-config/aimsctl.toml",
        ])
        .unwrap();

        assert!(matches!(
            cli.command,
            Commands::Uninstall {
                installation_root: Some(installation_root),
                aimsctl_install_path: Some(aimsctl_install_path),
                config_path,
            } if
                installation_root == Path::new("/tmp/aims-e2e")
                && aimsctl_install_path == Path::new("/tmp/aims-e2e-bin/aimsctl")
                && config_path == Path::new("/tmp/aims-e2e-config/aimsctl.toml")
        ));
    }
}
