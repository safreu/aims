use std::{
    env,
    net::{IpAddr, SocketAddr},
};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub session: SessionConfig,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub address: SocketAddr,
}

#[derive(Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub name: String,
    pub user: String,
    pub password: String,
    pub max_connections: u32,
}

impl std::fmt::Debug for DatabaseConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DatabaseConfig")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("name", &self.name)
            .field("user", &self.user)
            .field("password", &"[REDACTED]")
            .field("max_connections", &self.max_connections)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub lifetime_days: i64,
    pub cookie_name: String,
    pub cookie_secure: bool,
}

#[derive(Debug, Clone)]
pub struct SessionCookieConfig {
    pub name: String,
    pub secure: bool,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let app_host = required_variable("APP_HOST")?
            .parse::<IpAddr>()
            .map_err(ConfigError::InvalidHost)?;

        let app_port = required_variable("APP_PORT")?
            .parse::<u16>()
            .map_err(ConfigError::InvalidAppPort)?;

        let database_host = required_variable("DATABASE_HOST")?;

        let database_port = required_variable("DATABASE_PORT")?
            .parse::<u16>()
            .map_err(ConfigError::InvalidDatabasePort)?;

        let database_name = required_variable("DATABASE_NAME")?;

        let database_user = required_variable("DATABASE_USER")?;

        let database_password = required_variable("DATABASE_PASSWORD")?;

        let max_connections = required_variable("DATABASE_MAX_CONNECTIONS")?
            .parse::<u32>()
            .map_err(ConfigError::InvalidMaxConnections)?;

        let session_lifetime_days = required_variable("SESSION_LIFETIME_DAYS")?
            .parse::<i64>()
            .map_err(ConfigError::InvalidSessionLifetime)?;

        if session_lifetime_days <= 0 {
            return Err(ConfigError::NonPositiveSessionLifetime);
        }

        let session_cookie_name = required_variable("SESSION_COOKIE_NAME")?;

        if session_cookie_name.trim().is_empty() {
            return Err(ConfigError::EmptySessionCookieName);
        }

        let session_cookie_secure = required_variable("SESSION_COOKIE_SECURE")?
            .parse::<bool>()
            .map_err(ConfigError::InvalidSessionCookieSecure)?;

        Ok(Self {
            server: ServerConfig {
                address: SocketAddr::new(app_host, app_port),
            },
            database: DatabaseConfig {
                host: database_host,
                port: database_port,
                name: database_name,
                user: database_user,
                password: database_password,
                max_connections,
            },
            session: SessionConfig {
                lifetime_days: session_lifetime_days,
                cookie_name: session_cookie_name,
                cookie_secure: session_cookie_secure,
            },
        })
    }
}

fn required_variable(name: &'static str) -> Result<String, ConfigError> {
    env::var(name).map_err(|_| ConfigError::MissingVariable(name))
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("required environment variable `{0}` is missing")]
    MissingVariable(&'static str),

    #[error("APP_HOST is invalid")]
    InvalidHost(#[source] std::net::AddrParseError),

    #[error("APP_PORT is invalid")]
    InvalidAppPort(#[source] std::num::ParseIntError),

    #[error("DATABASE_PORT is invalid")]
    InvalidDatabasePort(#[source] std::num::ParseIntError),

    #[error("DATABASE_MAX_CONNECTIONS is invalid")]
    InvalidMaxConnections(#[source] std::num::ParseIntError),

    #[error("SESSION_LIFETIME_DAYS is invalid")]
    InvalidSessionLifetime(#[source] std::num::ParseIntError),

    #[error("SESSION_LIFETIME_DAYS must be greater than zero")]
    NonPositiveSessionLifetime,

    #[error("SESSION_COOKIE_SECURE is invalid")]
    InvalidSessionCookieSecure(#[source] std::str::ParseBoolError),

    #[error("SESSION_COOKIE_NAME must not be empty")]
    EmptySessionCookieName,
}
