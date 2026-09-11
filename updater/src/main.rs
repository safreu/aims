mod docker;
mod health;
mod installation;
mod updater;
mod version;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::{
    docker::DockerCompose, health::HttpHealthChecker, installation::Installation, updater::Updater,
    version::Version,
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
    Update {
        version: String,

        #[arg(long, default_value = "/opt/aims")]
        installation_root: PathBuf,

        #[arg(long, default_value = "http://127.0.0.1:8080/api/v1/health")]
        health_url: String,
    },
}
fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Update {
            version,
            installation_root,
            health_url,
        } => {
            let version = match Version::parse(&version) {
                Ok(version) => version,
                Err(error) => {
                    eprintln!("Invalid version: {error}");
                    std::process::exit(1);
                }
            };

            let installation = Installation::new(installation_root, health_url);
            let updater = Updater::new(installation, DockerCompose, HttpHealthChecker);

            if let Err(error) = updater.update(&version) {
                eprintln!("Update failed: {error}");
                std::process::exit(1);
            };

            println!("Aims updated to version {version}")
        }
    }
}
