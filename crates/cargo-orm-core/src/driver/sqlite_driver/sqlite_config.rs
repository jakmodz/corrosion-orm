use crate::driver::connection_config::{ConnectionConfig, ConnectionConfigError};

pub struct SqliteConfig {
    pub url: String,
    pub max_connections: usize,
    pub min_connections: usize,
    pub connection_timeout_ms: u64,
}

impl ConnectionConfig for SqliteConfig {
    fn url(&self) -> &str {
        &self.url
    }
    fn max_connections(&self) -> usize {
        self.max_connections
    }
    fn min_connections(&self) -> usize {
        self.min_connections
    }
    fn connection_timeout_ms(&self) -> u64 {
        self.connection_timeout_ms
    }

    fn from_env() -> Result<Self, ConnectionConfigError>
    where
        Self: Sized,
    {
        let url = std::env::var("DATABASE_URL").unwrap_or(String::new());
        if url.is_empty() {
            return Err(ConnectionConfigError::EnvFileNotFound);
        }
        let max_connections = std::env::var("MAX_CONNECTIONS")
            .unwrap_or("1".to_string())
            .parse()
            .unwrap_or(1);
        let min_connections = std::env::var("MIN_CONNECTIONS")
            .unwrap_or("0".to_string())
            .parse()
            .unwrap_or(0);
        let connection_timeout_ms = std::env::var("CONNECTION_TIMEOUT_MS")
            .unwrap_or("3000".to_string())
            .parse()
            .unwrap_or(3000);
        Ok(Self {
            url,
            max_connections,
            min_connections,
            connection_timeout_ms,
        })
    }
}
impl Default for SqliteConfig {
    fn default() -> Self {
        Self {
            url: String::from("sqlite::memory:"),
            max_connections: 1,
            min_connections: 0,
            connection_timeout_ms: 3000,
        }
    }
}
