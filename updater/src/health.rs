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

        for _ in 0..attempts {
            match reqwest::blocking::get(installation.health_url()) {
                Ok(respone) if respone.status().is_success() => {
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
}
