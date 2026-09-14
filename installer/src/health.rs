use std::{thread, time::Duration};

use crate::installation::Installation;

pub trait HealthChecker {
    fn wait_until_healthy(&self, installation: &Installation) -> Result<(), HealthCheckError>;
}

pub struct HttpHealthChecker;

impl HealthChecker for HttpHealthChecker {
    fn wait_until_healthy(&self, installation: &Installation) -> Result<(), HealthCheckError> {
        let attempts = 30;
        let delay = Duration::from_secs(2);
        let request_timeout = Duration::from_secs(5);

        let client = reqwest::blocking::Client::builder()
            .timeout(request_timeout)
            .build()
            .map_err(|_| HealthCheckError::Client)?;

        for _ in 0..attempts {
            match client.get(installation.health_url()).send() {
                Ok(response) if response.status().is_success() => {
                    return Ok(());
                }
                _ => {
                    thread::sleep(delay);
                }
            }
        }

        Err(HealthCheckError::TimedOut)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HealthCheckError {
    #[error("Aims did not become healthy in time")]
    TimedOut,
    #[error("failed to create HTTP client")]
    Client,
}
