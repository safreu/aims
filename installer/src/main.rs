mod docker;
mod environment;
mod health;
mod installation;
mod installer;
mod version;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::{
    docker::DockerCompose, health::HttpHealthChecker, installation::Installation,
    installer::Installer, version::Version,
};

#[derive(Parser)]
#[command(name = "aims-updater")]
#[command(about = "Updater for Aims installations")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Install {
        version: String,

        #[arg(long, default_value = "/opt/aims")]
        installation_root: PathBuf,

        #[arg(long, default_value = "http://127.0.0.1:8080/api/v1/health")]
        health_url: String,

        #[arg(long, default_value = "aims-prod")]
        compose_project: String,

        #[arg(long, default_value_t = 8080)]
        http_port: u16,

        #[arg(long, default_value = "aims_prod_postgres_data")]
        postgres_volume: String,
    },
}
fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Install {
            version,
            installation_root,
            health_url,
            compose_project,
            http_port,
            postgres_volume,
        } => {
            let version = match Version::parse(&version) {
                Ok(version) => version,
                Err(error) => {
                    eprintln!("Invalid version: {error}");
                    std::process::exit(1);
                }
            };

            let installation = Installation::new(installation_root, health_url);

            let installer = Installer::new(
                installation,
                DockerCompose,
                HttpHealthChecker,
                compose_project,
                http_port,
                postgres_volume,
            );

            if let Err(error) = installer.install(&version) {
                eprintln!("Installation failed: {error}");
                std::process::exit(1);
            }

            println!("Aims {version} installed successfully")
        }
    }
}
