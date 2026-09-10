use std::process::ExitCode;

use backend::{app::Application, config::AppConfig, shared::observability::init_tracing};

#[tokio::main]
async fn main() -> ExitCode {
    let _ = dotenvy::dotenv();

    init_tracing();
    if let Err(error) = run().await {
        tracing::error!(error = ?error, "application startup failed");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::from_env()?;
    let application = Application::build(config).await?;

    application.run().await?;

    Ok(())
}
