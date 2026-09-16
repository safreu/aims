use std::{thread, time::Duration};

use crate::installation::{Installation, InstallationError};

pub trait HealthChecker {
    fn wait_until_healthy(&self, installation: &Installation) -> Result<(), HealthCheckError>;

    fn is_healthy(&self, installation: &Installation) -> Result<bool, HealthCheckError>;
}

pub struct HttpHealthChecker;

impl HealthChecker for HttpHealthChecker {
    fn wait_until_healthy(&self, installation: &Installation) -> Result<(), HealthCheckError> {
        let attempts = 30;
        let delay = Duration::from_secs(2);

        for _ in 0..attempts {
            if self.is_healthy(installation)? {
                return Ok(());
            };

            thread::sleep(delay);
        }

        Err(HealthCheckError::TimedOut)
    }

    fn is_healthy(&self, installation: &Installation) -> Result<bool, HealthCheckError> {
        let request_timeout = Duration::from_secs(5);

        let client = reqwest::blocking::Client::builder()
            .timeout(request_timeout)
            .build()
            .map_err(|_| HealthCheckError::Client)?;

        let health_url = installation.health_url()?;

        match client.get(health_url).send() {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HealthCheckError {
    #[error("Aims did not become healthy in time")]
    TimedOut,
    #[error("failed to create HTTP client")]
    Client,
    #[error("failed to determine Aims health URL")]
    Installation(#[from] InstallationError),
}
