use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::{
    docker::DockerCompose, health::HttpHealthChecker, installation::Installation,
    installer::Installer, manager::Manager, progress::ConsoleReporter, status::StatusChecker,
    uninstaller::Uninstaller, updater::Updater, version::Version,
};

mod docker;
mod environment;
mod health;
mod installation;
mod installer;
mod manager;
mod progress;
mod status;
mod uninstaller;
mod updater;
mod version;

const AIMSCTL_INSTALL_PATH: &str = "/usr/local/bin/aimsctl";

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

        /// URL used to check whether Aims is healthy.
        #[arg(long, default_value = "http://127.0.0.1:8080/api/v1/health")]
        health_url: String,

        /// Docker Compose project name.
        #[arg(long, default_value = "aims-prod")]
        compose_project: String,

        /// HTTP port used to expose Aims.
        #[arg(long, default_value_t = 8080)]
        http_port: u16,

        /// Docker volume used for PostgreSQL data.
        #[arg(long, default_value = "aims_prod_postgres_data")]
        postgres_volume: String,
    },

    /// Update an existing Aims installation.
    Update {
        /// Version of Aims to update to, for example 0.1.3.
        version: String,

        /// Directory containing the Aims installation.
        #[arg(long, default_value = "/opt/aims")]
        installation_root: PathBuf,

        /// URL used to check whether Aims is healthy after the update.
        #[arg(long, default_value = "http://127.0.0.1:8080/api/v1/health")]
        health_url: String,
    },

    /// Show the aimsctl version.
    Version,

    /// Start existing Aims services.
    Start {
        /// Directory containing the Aims installation.
        #[arg(long, default_value = "/opt/aims")]
        installation_root: PathBuf,

        /// URL used by the Aims installation.
        #[arg(long, default_value = "http://127.0.0.1:8080/api/v1/health")]
        health_url: String,
    },

    /// Show the current Aims installation status.
    Status {
        /// Directory containing the Aims installation.
        #[arg(long, default_value = "/opt/aims")]
        installation_root: PathBuf,

        /// URL used to check whether Aims is healthy.
        #[arg(long, default_value = "http://127.0.0.1:8080/api/v1/health")]
        health_url: String,
    },

    /// Stop Aims services.
    Stop {
        /// Directory containing the Aims installation.
        #[arg(long, default_value = "/opt/aims")]
        installation_root: PathBuf,

        /// URL used by the Aims installation.
        #[arg(long, default_value = "http://127.0.0.1:8080/api/v1/health")]
        health_url: String,
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

        /// URL used by the Aims installation.
        #[arg(long, default_value = "http://127.0.0.1:8080/api/v1/health")]
        health_url: String,
    },
}
fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Install {
            installation_root,
            health_url,
            compose_project,
            http_port,
            postgres_volume,
        } => install(
            installation_root,
            health_url,
            compose_project,
            http_port,
            postgres_volume,
        ),

        Commands::Update {
            version,
            installation_root,
            health_url,
        } => update(&version, installation_root, health_url),

        Commands::Version => {
            println!("{}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }

        Commands::Start {
            installation_root,
            health_url,
        } => start(installation_root, health_url),
        Commands::Status {
            installation_root,
            health_url,
        } => status(installation_root, health_url),
        Commands::Stop {
            installation_root,
            health_url,
        } => stop(installation_root, health_url),

        Commands::Uninstall {
            installation_root,
            health_url,
        } => uninstall(installation_root, health_url),
    };

    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn install(
    installation_root: PathBuf,
    health_url: String,
    compose_project: String,
    http_port: u16,
    postgres_volume: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let version = Version::parse(env!("CARGO_PKG_VERSION"))?;

    let installation = Installation::new(installation_root, health_url);

    let installer = Installer::new(
        installation,
        DockerCompose,
        HttpHealthChecker,
        ConsoleReporter,
        compose_project,
        http_port,
        postgres_volume,
    );

    installer.install(&version)?;

    Ok(())
}

fn update(
    version: &str,
    installation_root: PathBuf,
    health_url: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let version = Version::parse(version)?;

    let installation = Installation::new(installation_root, health_url);

    let updater = Updater::new(
        installation,
        DockerCompose,
        HttpHealthChecker,
        ConsoleReporter,
    );

    updater.update(&version)?;

    Ok(())
}

fn start(installation_root: PathBuf, health_url: String) -> Result<(), Box<dyn std::error::Error>> {
    let installation = Installation::new(installation_root, health_url);

    let manager = Manager::new(installation, DockerCompose, ConsoleReporter);

    manager.start()?;

    Ok(())
}

fn stop(installation_root: PathBuf, health_url: String) -> Result<(), Box<dyn std::error::Error>> {
    let installation = Installation::new(installation_root, health_url);

    let manager = Manager::new(installation, DockerCompose, ConsoleReporter);

    manager.stop()?;

    Ok(())
}

fn uninstall(
    installation_root: PathBuf,
    health_url: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let installation = Installation::new(installation_root, health_url);

    let uninstaller = Uninstaller::new(installation, DockerCompose, ConsoleReporter);

    uninstaller.uninstall()?;

    std::fs::remove_file(AIMSCTL_INSTALL_PATH)?;

    Ok(())
}

fn status(
    installation_root: PathBuf,
    health_url: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let installation = Installation::new(installation_root, health_url);

    let checker = StatusChecker::new(installation, DockerCompose, HttpHealthChecker);

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
