use rand::{RngExt, distr::Alphanumeric};

use crate::version::Version;

const ENV_TEMPLATE: &str = include_str!("../../deployment/raspberry-pi/env.template");

pub struct EnvironmentConfig<'a> {
    pub version: &'a Version,
    pub database_password: &'a str,
    pub compose_project: &'a str,
    pub http_port: u16,
    pub postgres_volume: &'a str,
}

pub fn generate_database_password() -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(48)
        .map(char::from)
        .collect()
}

pub fn build_environment(config: &EnvironmentConfig<'_>) -> String {
    ENV_TEMPLATE
        .replace("{{AIMS_VERSION}}", &config.version.to_string())
        .replace("{{DATABASE_PASSWORD}}", config.database_password)
        .replace("{{AIMS_COMPOSE_PROJECT}}", config.compose_project)
        .replace("{{AIMS_HTTP_PORT}}", &config.http_port.to_string())
        .replace("{{AIMS_POSTGRES_VOLUME}}", config.postgres_volume)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn database_password_is_generated() {
        let password = generate_database_password();

        assert_eq!(password.len(), 48);
        assert!(
            password
                .chars()
                .all(|character| character.is_ascii_alphanumeric())
        );
    }

    #[test]
    fn environment_contains_configured_values() {
        let version = Version::parse("0.1.1").unwrap();

        let config = EnvironmentConfig {
            version: &version,
            database_password: "test-password",
            compose_project: "aims-installer-test",
            http_port: 18081,
            postgres_volume: "aims_installer_test_postgres_data",
        };

        let environment = build_environment(&config);

        assert!(environment.contains("AIMS_VERSION=0.1.1"));
        assert!(environment.contains("DATABASE_PASSWORD=test-password"));
        assert!(environment.contains("AIMS_COMPOSE_PROJECT=aims-installer-test"));
        assert!(environment.contains("AIMS_HTTP_PORT=18081"));
        assert!(environment.contains("AIMS_POSTGRES_VOLUME=aims_installer_test_postgres_data"));
    }
}
